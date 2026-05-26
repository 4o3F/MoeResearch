use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::error::{Error, JsonSnafu, Result};
use crate::net::NetworkClient;
use crate::schema::network::{Header, NetworkRequest};
use crate::schema::policy::Freshness;
use crate::schema::search::{SearchRequest, SearchResponse, SearchResult};
use crate::search::provider::SearchProvider;

pub struct GrokSearchProvider {
    network: Arc<dyn NetworkClient>,
    base_url: String,
    api_key: String,
    timeout_ms: Option<u64>,
    model: String,
    max_output_tokens: Option<u32>,
}

impl GrokSearchProvider {
    pub fn new(
        network: Arc<dyn NetworkClient>,
        base_url: String,
        api_key: String,
        timeout_ms: Option<u64>,
        model: String,
    ) -> Self {
        Self::with_max_output_tokens(network, base_url, api_key, timeout_ms, model, None)
    }

    /// Constructs the provider with the response-size cap supplied from
    /// configuration. `None` leaves the cap to the upstream provider default.
    /// Validation of this input is owned by `ProviderEndpoint::validate`; this
    /// constructor trusts its caller.
    #[must_use]
    pub fn with_max_output_tokens(
        network: Arc<dyn NetworkClient>,
        base_url: String,
        api_key: String,
        timeout_ms: Option<u64>,
        model: String,
        max_output_tokens: Option<u32>,
    ) -> Self {
        Self {
            network,
            base_url,
            api_key,
            timeout_ms,
            model,
            max_output_tokens,
        }
    }
}

#[async_trait]
impl SearchProvider for GrokSearchProvider {
    fn name(&self) -> &'static str {
        "grok"
    }

    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let max_results = request.max_results;
        let body = serde_json::to_value(GrokSearchRequest {
            model: self.model.clone(),
            input: vec![GrokSearchInputMessage {
                role: "user",
                content: search_prompt(&request),
            }],
            tools: vec![GrokSearchTool::WebSearch(GrokWebSearchTool {
                filters: grok_filters(&request),
            })],
            max_output_tokens: self.max_output_tokens,
            stream: false,
        })
        .context(JsonSnafu)?;

        let response = self
            .network
            .send(NetworkRequest {
                method: "POST".to_owned(),
                url: format!("{}/responses", self.base_url.trim_end_matches('/')),
                headers: vec![
                    Header {
                        name: "authorization".to_owned(),
                        value: format!("Bearer {}", self.api_key),
                    },
                    Header {
                        name: "content-type".to_owned(),
                        value: "application/json".to_owned(),
                    },
                ],
                body: Some(body),
                timeout_ms: self.timeout_ms,
            })
            .await?;

        if !(200..300).contains(&response.status) {
            return Err(Error::HttpStatus {
                status: response.status,
                message: "grok search provider returned non-success status".to_owned(),
                retryable: response.status == 429 || response.status >= 500,
            });
        }

        let provider_response: GrokSearchResponse =
            serde_json::from_value(response.body).context(JsonSnafu)?;

        Ok(SearchResponse {
            provider: self.name().to_owned(),
            results: map_grok_response(provider_response, max_results),
        })
    }
}

fn grok_filters(request: &SearchRequest) -> Option<GrokWebSearchFilters> {
    if request.include_domains.is_empty() {
        None
    } else {
        Some(GrokWebSearchFilters {
            allowed_domains: Some(request.include_domains.clone()),
        })
    }
}

fn search_prompt(request: &SearchRequest) -> String {
    let mut prompt = format!(
        "Search the web for: {}\nReturn concise sourced findings.\nMaximum results: {}",
        request.query, request.max_results
    );

    if let Some(language) = request.language.as_ref() {
        prompt.push_str("\nLanguage: ");
        prompt.push_str(language);
    }

    if let Some(region) = request.region.as_ref() {
        prompt.push_str("\nRegion: ");
        prompt.push_str(region);
    }

    if !request.exclude_domains.is_empty() {
        prompt.push_str("\nExclude domains: ");
        prompt.push_str(&request.exclude_domains.join(", "));
    }

    if let Some(window) = request
        .freshness
        .as_ref()
        .and_then(Freshness::describe_for_prompt)
    {
        prompt.push_str("\nFreshness: ");
        prompt.push_str(&window);
    }

    prompt
}

fn map_grok_response(response: GrokSearchResponse, max_results: usize) -> Vec<SearchResult> {
    let mut full_text = String::new();
    let mut citations = Vec::new();
    let mut fallback_sources: Vec<GrokSearchSource> = Vec::new();

    for output in response.output {
        match output {
            GrokSearchOutput::Message {
                content,
                search_sources,
            } => {
                for item in content {
                    match item {
                        GrokSearchContent::OutputText { text, annotations } => {
                            if !full_text.is_empty() {
                                full_text.push('\n');
                            }
                            full_text.push_str(&text);
                            citations.extend(annotations.into_iter().filter_map(|annotation| {
                                GrokSearchCitation::new(annotation, &text)
                            }));
                        }
                        GrokSearchContent::Other => {}
                    }
                }
                fallback_sources.extend(search_sources);
            }
            GrokSearchOutput::Reasoning {}
            | GrokSearchOutput::WebSearchCall {}
            | GrokSearchOutput::Other => {}
        }
    }

    let mut seen_urls = HashSet::new();
    let mut results = Vec::new();

    for citation in citations {
        if !seen_urls.insert(citation.url.clone()) {
            continue;
        }

        // Citation-derived rows get a per-source snippet and summary
        // anchored at the citation indices, so two evidence rows in the
        // same search no longer carry identical 1 KiB Markdown blobs.
        let snippet = citation_snippet(&citation.text, citation.start_index, citation.end_index);
        let summary =
            citation_local_summary(&citation.text, citation.start_index, citation.end_index);

        results.push(SearchResult {
            title: citation.title.unwrap_or_else(|| citation.url.clone()),
            url: Some(citation.url),
            snippet,
            summary: Some(summary),
            published_at: None,
        });

        if results.len() == max_results {
            break;
        }
    }

    // Grok also returns `search_sources` alongside `content`. These are URLs the
    // model consulted but did not surface as inline `url_citation` annotations
    // (e.g., supporting reddit/substack threads). Append them after the
    // citation-derived results so high-signal annotated sources still rank
    // first; dedupe by URL so we never double-list.
    //
    // search_sources entries have no positional anchor inside `full_text`,
    // so the message-level summary is the best we can attribute to them
    // without inventing content the model never asserted.
    for source in fallback_sources {
        if results.len() >= max_results {
            break;
        }
        if !seen_urls.insert(source.url.clone()) {
            continue;
        }

        let title = source.title.clone().unwrap_or_else(|| source.url.clone());
        let snippet = source.title.clone().unwrap_or_else(|| source.url.clone());
        let summary = if full_text.is_empty() {
            None
        } else {
            Some(full_text.clone())
        };
        results.push(SearchResult {
            title,
            url: Some(source.url),
            snippet,
            summary,
            published_at: None,
        });
    }

    if results.is_empty() && !full_text.is_empty() && max_results > 0 {
        results.push(SearchResult {
            title: "Grok web search result".to_owned(),
            url: None,
            snippet: full_text.clone(),
            summary: Some(full_text),
            published_at: None,
        });
    }

    results
}

/// Byte budget added to each side of a citation when extracting the
/// snippet shown in `SearchResult.snippet`. Calibrated so a typical
/// citation marker (`[[N]](url)`) lands inside a readable sentence.
const SNIPPET_PAD_BYTES: usize = 80;

/// Hard upper bound on `SearchResult.snippet`. Anything longer is
/// truncated to a char boundary with a trailing ellipsis so log
/// consumers always see a bounded line.
const SNIPPET_MAX_BYTES: usize = 240;

/// Larger byte budget for the per-source `SearchResult.summary`. The
/// summary is the model's main reading material so it warrants more
/// context than the snippet, but it still needs a ceiling — without
/// it every evidence row in a single search would carry the same
/// 1 KiB+ Markdown blob and waste a large fraction of Layer 2's
/// prompt budget.
const SUMMARY_PAD_BYTES: usize = 240;
const SUMMARY_MAX_BYTES: usize = 600;

/// Controls how aggressively `excerpt_around` walks away from the
/// citation when picking word boundaries.
///
/// `Tight` stops at the first whitespace just outside the citation
/// range, so the resulting snippet hugs the original indices — useful
/// when we just want to round mid-word cuts back to a clean boundary.
///
/// `Wide` extends to the *farthest* whitespace within `pad_bytes`, so
/// the resulting excerpt includes as much surrounding context as the
/// budget allows — used for summaries so two evidence rows from the
/// same `output_text` describe their own passage instead of sharing
/// one identical Markdown blob.
#[derive(Clone, Copy)]
enum ExpandStrategy {
    Tight,
    Wide,
}

/// Builds the per-source `snippet`: a readable sentence rounded outward
/// from the citation indices to the nearest word boundaries. Without
/// indices we fall back to a clamped excerpt of the whole text rather
/// than failing — Grok occasionally omits indices on some annotation
/// variants.
fn citation_snippet(text: &str, start: Option<usize>, end: Option<usize>) -> String {
    excerpt_around(
        text,
        start,
        end,
        SNIPPET_PAD_BYTES,
        SNIPPET_MAX_BYTES,
        ExpandStrategy::Tight,
    )
}

/// Builds the per-source `summary`: a longer excerpt around the citation
/// using the wide expansion strategy, so two evidence rows from the
/// same `output_text` describe their own passage rather than sharing
/// one identical Markdown blob.
fn citation_local_summary(text: &str, start: Option<usize>, end: Option<usize>) -> String {
    excerpt_around(
        text,
        start,
        end,
        SUMMARY_PAD_BYTES,
        SUMMARY_MAX_BYTES,
        ExpandStrategy::Wide,
    )
}

/// Extracts a UTF-8 safe excerpt around `[start, end)` padded by
/// `pad_bytes` on each side, snapped to whitespace per `strategy`,
/// then clamped to `max_bytes` with a trailing ellipsis when truncated.
///
/// Returns a clamped excerpt of the whole input when the indices are
/// missing, malformed, or do not land on UTF-8 char boundaries. The
/// returned string is always trimmed.
fn excerpt_around(
    text: &str,
    start: Option<usize>,
    end: Option<usize>,
    pad_bytes: usize,
    max_bytes: usize,
    strategy: ExpandStrategy,
) -> String {
    let trimmed_fallback = || clamp_to_max(text.trim(), max_bytes);
    let (Some(start), Some(end)) = (start, end) else {
        return trimmed_fallback();
    };
    if start >= end
        || end > text.len()
        || !text.is_char_boundary(start)
        || !text.is_char_boundary(end)
    {
        return trimmed_fallback();
    }

    let left = expand_left(text, start, pad_bytes, strategy);
    let right = expand_right(text, end, pad_bytes, strategy);
    clamp_to_max(text[left..right].trim(), max_bytes)
}

/// Walks left from `anchor` by at most `budget` bytes and returns the
/// byte offset where the excerpt should start so it begins at a clean
/// word boundary.
///
/// In `Tight` mode the search starts at `anchor` and walks backward,
/// returning at the first whitespace encountered — the smallest legal
/// expansion. In `Wide` mode the search starts at `anchor - budget`
/// and walks forward, returning at the first whitespace it finds —
/// the largest legal expansion within budget. Wide also short-circuits
/// to byte 0 when the budget reaches the start of text, because the
/// edge is a cleaner boundary than any interior whitespace. Both modes
/// fall back to the UTF-8 boundary at or below `anchor - budget` when
/// no whitespace exists in the search window.
fn expand_left(text: &str, anchor: usize, budget: usize, strategy: ExpandStrategy) -> usize {
    let lower = anchor.saturating_sub(budget);
    // Wide prefers maximum extension; start-of-text outranks any
    // interior whitespace within budget.
    if matches!(strategy, ExpandStrategy::Wide) && lower == 0 {
        return 0;
    }
    let bytes = text.as_bytes();
    let range: Box<dyn Iterator<Item = usize>> = match strategy {
        ExpandStrategy::Tight => Box::new((lower..anchor).rev()),
        ExpandStrategy::Wide => Box::new(lower..anchor),
    };
    for i in range {
        if bytes[i].is_ascii_whitespace() {
            return i + 1;
        }
    }
    // Edge of text counts as a word boundary.
    if lower == 0 {
        return 0;
    }
    let mut p = lower;
    while p > 0 && !text.is_char_boundary(p) {
        p -= 1;
    }
    p
}

/// Mirror of [`expand_left`] walking right; same strategy semantics.
fn expand_right(text: &str, anchor: usize, budget: usize, strategy: ExpandStrategy) -> usize {
    let upper = anchor.saturating_add(budget).min(text.len());
    // Wide prefers maximum extension; end-of-text outranks any
    // interior whitespace within budget.
    if matches!(strategy, ExpandStrategy::Wide) && upper == text.len() {
        return upper;
    }
    let bytes = text.as_bytes();
    let range: Box<dyn Iterator<Item = usize>> = match strategy {
        ExpandStrategy::Tight => Box::new(anchor..upper),
        ExpandStrategy::Wide => Box::new((anchor..upper).rev()),
    };
    for i in range {
        if bytes[i].is_ascii_whitespace() {
            return i;
        }
    }
    if upper == text.len() {
        return upper;
    }
    let mut p = upper;
    while p < text.len() && !text.is_char_boundary(p) {
        p += 1;
    }
    p
}

/// Clamps a string to at most `max_bytes` bytes at a UTF-8 boundary,
/// appending an ellipsis when truncation occurs.
fn clamp_to_max(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_owned();
    }
    let mut cut = max_bytes;
    while cut > 0 && !text.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}…", text[..cut].trim_end())
}

struct GrokSearchCitation {
    url: String,
    title: Option<String>,
    start_index: Option<usize>,
    end_index: Option<usize>,
    text: String,
}

impl GrokSearchCitation {
    fn new(annotation: GrokSearchAnnotation, text: &str) -> Option<Self> {
        match annotation {
            GrokSearchAnnotation::UrlCitation {
                url,
                title,
                start_index,
                end_index,
            } => Some(Self {
                url,
                title,
                start_index,
                end_index,
                text: text.to_owned(),
            }),
            GrokSearchAnnotation::Other => None,
        }
    }
}

#[derive(Serialize)]
struct GrokSearchRequest {
    model: String,
    input: Vec<GrokSearchInputMessage>,
    tools: Vec<GrokSearchTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    stream: bool,
}

#[derive(Serialize)]
struct GrokSearchInputMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum GrokSearchTool {
    #[serde(rename = "web_search")]
    WebSearch(GrokWebSearchTool),
}

#[derive(Serialize)]
struct GrokWebSearchTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<GrokWebSearchFilters>,
}

#[derive(Serialize)]
struct GrokWebSearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_domains: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct GrokSearchResponse {
    #[serde(default)]
    output: Vec<GrokSearchOutput>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum GrokSearchOutput {
    Message {
        #[serde(default)]
        content: Vec<GrokSearchContent>,
        #[serde(default)]
        search_sources: Vec<GrokSearchSource>,
    },
    Reasoning {},
    WebSearchCall {},
    #[serde(other)]
    Other,
}

/// Source URL listed in Grok's `search_sources` array.
///
/// Grok returns these alongside `content` to disclose every page the model
/// consulted, including ones that were not inlined as `url_citation`
/// annotations. We use them as a fallback to fill `max_results` so we do not
/// silently drop legitimate references (reddit/substack threads, etc.).
#[derive(Deserialize, Clone)]
struct GrokSearchSource {
    url: String,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum GrokSearchContent {
    OutputText {
        text: String,
        #[serde(default)]
        annotations: Vec<GrokSearchAnnotation>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum GrokSearchAnnotation {
    UrlCitation {
        url: String,
        title: Option<String>,
        start_index: Option<usize>,
        end_index: Option<usize>,
    },
    #[serde(other)]
    Other,
}
