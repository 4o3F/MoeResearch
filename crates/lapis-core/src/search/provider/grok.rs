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
    search_context_size: String,
    max_output_tokens: Option<u32>,
}

impl GrokSearchProvider {
    /// Default search-context-size used when the operator does not override
    /// it via TOML configuration.
    pub const DEFAULT_SEARCH_CONTEXT_SIZE: &'static str = "low";

    pub fn new(
        network: Arc<dyn NetworkClient>,
        base_url: String,
        api_key: String,
        timeout_ms: Option<u64>,
        model: String,
    ) -> Self {
        Self::with_search_knobs(
            network,
            base_url,
            api_key,
            timeout_ms,
            model,
            None,
            None,
        )
    }

    /// Constructs the provider with the search-tuning knobs explicitly
    /// supplied from configuration.
    ///
    /// `search_context_size` falls back to [`Self::DEFAULT_SEARCH_CONTEXT_SIZE`]
    /// when `None`. `max_output_tokens` of `None` leaves the cap to the
    /// upstream provider default. Validation of these inputs is owned by
    /// `ProviderEndpoint::validate`; this constructor trusts its caller.
    #[must_use]
    pub fn with_search_knobs(
        network: Arc<dyn NetworkClient>,
        base_url: String,
        api_key: String,
        timeout_ms: Option<u64>,
        model: String,
        search_context_size: Option<String>,
        max_output_tokens: Option<u32>,
    ) -> Self {
        Self {
            network,
            base_url,
            api_key,
            timeout_ms,
            model,
            search_context_size: search_context_size
                .unwrap_or_else(|| Self::DEFAULT_SEARCH_CONTEXT_SIZE.to_owned()),
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
                search_context_size: Some(self.search_context_size.as_str()),
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

        results.push(SearchResult {
            title: citation.title.unwrap_or_else(|| citation.url.clone()),
            url: Some(citation.url),
            snippet: citation_snippet(&citation.text, citation.start_index, citation.end_index),
            summary: Some(full_text.clone()),
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
    for source in fallback_sources {
        if results.len() >= max_results {
            break;
        }
        if !seen_urls.insert(source.url.clone()) {
            continue;
        }

        let title = source
            .title
            .clone()
            .unwrap_or_else(|| source.url.clone());
        let snippet = source
            .title
            .clone()
            .unwrap_or_else(|| source.url.clone());
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

fn citation_snippet(text: &str, start_index: Option<usize>, end_index: Option<usize>) -> String {
    let (Some(start_index), Some(end_index)) = (start_index, end_index) else {
        return text.to_owned();
    };

    if start_index < end_index
        && end_index <= text.len()
        && text.is_char_boundary(start_index)
        && text.is_char_boundary(end_index)
    {
        text[start_index..end_index].to_owned()
    } else {
        text.to_owned()
    }
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
struct GrokSearchRequest<'a> {
    model: String,
    input: Vec<GrokSearchInputMessage>,
    tools: Vec<GrokSearchTool<'a>>,
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
enum GrokSearchTool<'a> {
    #[serde(rename = "web_search")]
    WebSearch(GrokWebSearchTool<'a>),
}

#[derive(Serialize)]
struct GrokWebSearchTool<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<GrokWebSearchFilters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_context_size: Option<&'a str>,
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
