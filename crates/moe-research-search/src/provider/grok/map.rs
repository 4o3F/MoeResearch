use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{Freshness, SearchRequest, SearchResult};

use super::excerpt::{citation_local_summary, citation_snippet};

fn grok_filters(include_domains: Vec<String>) -> Option<GrokWebSearchFilters> {
    if include_domains.is_empty() {
        None
    } else {
        Some(GrokWebSearchFilters {
            allowed_domains: Some(include_domains),
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

    if let Some(depth) = request.depth {
        prompt.push_str("\nSearch depth preference: ");
        prompt.push_str(depth.prompt_hint());
    }

    if let Some(content_level) = request.content_level {
        prompt.push_str("\nContent detail preference: ");
        prompt.push_str(content_level.prompt_hint());
    }

    if let Some(recency) = request.recency {
        prompt.push_str("\nSource recency preference: ");
        prompt.push_str(recency.prompt_hint());
    }

    if let Some(category) = request.category {
        prompt.push_str("\nCategory focus: ");
        prompt.push_str(category.prompt_hint());
    }

    prompt
}

pub(super) fn map_grok_response(
    response: GrokSearchResponse,
    max_results: usize,
) -> Vec<SearchResult> {
    let mut full_text = String::new();
    let mut citations = Vec::new();
    let mut fallback_sources: Vec<GrokSearchSource> = Vec::new();
    let mut output_message_count = 0usize;
    let mut citation_annotation_count = 0usize;
    let mut fallback_source_count = 0usize;
    let mut duplicate_source_count = 0usize;

    for output in response.output {
        match output {
            GrokSearchOutput::Message {
                content,
                search_sources,
            } => {
                output_message_count += 1;
                fallback_source_count = fallback_source_count.saturating_add(search_sources.len());
                for item in content {
                    match item {
                        GrokSearchContent::OutputText { text, annotations } => {
                            citation_annotation_count =
                                citation_annotation_count.saturating_add(annotations.len());
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
            duplicate_source_count = duplicate_source_count.saturating_add(1);
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
            duplicate_source_count = duplicate_source_count.saturating_add(1);
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

    tracing::debug!(
        event = "search_response_mapped",
        status = "ok",
        provider_kind = "search",
        provider = "grok",
        max_results,
        output_message_count,
        citation_annotation_count,
        fallback_source_count,
        duplicate_source_count,
        emitted_result_count = results.len(),
        capped_by_max_results = results.len() == max_results,
        "search response mapped"
    );

    results
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
pub(super) struct GrokSearchRequest {
    pub(super) model: String,
    pub(super) input: Vec<GrokSearchInputMessage>,
    pub(super) tools: Vec<GrokSearchTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) reasoning: Option<GrokReasoning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) max_output_tokens: Option<u32>,
    pub(super) stream: bool,
}

pub(super) struct GrokSearchRequestParts {
    pub(super) input: Vec<GrokSearchInputMessage>,
    pub(super) tools: Vec<GrokSearchTool>,
}

impl From<SearchRequest> for GrokSearchRequestParts {
    fn from(request: SearchRequest) -> Self {
        let content = search_prompt(&request);
        let filters = grok_filters(request.include_domains);

        Self {
            input: vec![GrokSearchInputMessage {
                role: "user",
                content,
            }],
            tools: vec![GrokSearchTool::WebSearch(GrokWebSearchTool { filters })],
        }
    }
}

#[derive(Serialize)]
pub(super) struct GrokReasoning {
    pub(super) effort: String,
}

#[derive(Serialize)]
pub(super) struct GrokSearchInputMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum GrokSearchTool {
    #[serde(rename = "web_search")]
    WebSearch(GrokWebSearchTool),
}

#[derive(Serialize)]
pub(super) struct GrokWebSearchTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<GrokWebSearchFilters>,
}

#[derive(Serialize)]
struct GrokWebSearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_domains: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub(super) struct GrokSearchResponse {
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
