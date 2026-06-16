use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use lapis_error::{JsonSnafu, Result};
use lapis_net::NetworkClient;
use lapis_net::provider_http::bearer_json_post;

use crate::{
    Freshness, SearchCategory, SearchContentLevel, SearchDepth, SearchProvider, SearchRecency,
    SearchRequest, SearchResponse, SearchResult,
};

pub struct TavilySearchProvider {
    network: Arc<dyn NetworkClient>,
    base_url: String,
    api_key: String,
    timeout_ms: Option<u64>,
}

impl TavilySearchProvider {
    pub fn new(
        network: Arc<dyn NetworkClient>,
        base_url: String,
        api_key: String,
        timeout_ms: Option<u64>,
    ) -> Self {
        Self {
            network,
            base_url,
            api_key,
            timeout_ms,
        }
    }
}

#[async_trait]
impl SearchProvider for TavilySearchProvider {
    fn name(&self) -> &'static str {
        "tavily"
    }

    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let max_results = request.max_results;
        let body = serde_json::to_value(TavilySearchRequest::from(request)).context(JsonSnafu)?;

        let response = self
            .network
            .send_json(bearer_json_post(
                &self.base_url,
                "search",
                &self.api_key,
                body,
                self.timeout_ms,
            ))
            .await?;

        let provider_response: TavilySearchResponse =
            serde_json::from_value(response.body).context(JsonSnafu)?;

        Ok(SearchResponse {
            provider: self.name().to_owned(),
            results: provider_response
                .results
                .into_iter()
                .take(max_results)
                .map(map_tavily_result)
                .collect(),
        })
    }
}

#[derive(Serialize)]
struct TavilySearchRequest {
    query: String,
    max_results: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_depth: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_range: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_date: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    include_domains: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    exclude_domains: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_raw_content: Option<bool>,
}

impl From<SearchRequest> for TavilySearchRequest {
    fn from(request: SearchRequest) -> Self {
        let (start_date, end_date) = match request.freshness {
            Some(Freshness { since, until }) => (since, until),
            None => (None, None),
        };
        let has_explicit_dates = start_date.is_some() || end_date.is_some();

        Self {
            query: request.query,
            max_results: request.max_results,
            search_depth: tavily_search_depth(request.depth),
            topic: tavily_topic(request.category),
            time_range: (!has_explicit_dates)
                .then(|| tavily_time_range(request.recency))
                .flatten(),
            start_date,
            end_date,
            include_domains: request.include_domains,
            exclude_domains: request.exclude_domains,
            include_raw_content: tavily_include_raw_content(request.content_level),
        }
    }
}

fn tavily_search_depth(depth: Option<SearchDepth>) -> Option<&'static str> {
    match depth {
        None => None,
        Some(SearchDepth::LowLatency) => Some("fast"),
        Some(SearchDepth::Balanced) => Some("basic"),
        Some(SearchDepth::HighRecall) => Some("advanced"),
    }
}

fn tavily_topic(category: Option<SearchCategory>) -> Option<&'static str> {
    match category {
        Some(SearchCategory::News) => Some("news"),
        Some(SearchCategory::FinancialFilings) => Some("finance"),
        _ => None,
    }
}

fn tavily_time_range(recency: Option<SearchRecency>) -> Option<&'static str> {
    match recency {
        Some(SearchRecency::Live | SearchRecency::Fresh) => Some("day"),
        Some(SearchRecency::Recent) => Some("week"),
        None | Some(SearchRecency::Default | SearchRecency::Cached) => None,
    }
}

fn tavily_include_raw_content(level: Option<SearchContentLevel>) -> Option<bool> {
    match level {
        Some(SearchContentLevel::Detailed) => Some(true),
        None | Some(SearchContentLevel::Compact | SearchContentLevel::Standard) => None,
    }
}

#[derive(Deserialize)]
struct TavilySearchResponse {
    #[serde(default)]
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: Option<String>,
    url: Option<String>,
    link: Option<String>,
    content: Option<String>,
    snippet: Option<String>,
    raw_content: Option<String>,
    published_date: Option<String>,
}

const TAVILY_SUMMARY_MAX_BYTES: usize = 4_000;

fn map_tavily_result(result: TavilyResult) -> SearchResult {
    let url = result.url.or(result.link);
    let snippet = result.content.or(result.snippet).unwrap_or_default();
    SearchResult {
        title: result
            .title
            .unwrap_or_else(|| url.clone().unwrap_or_default()),
        url,
        snippet,
        summary: bounded_summary(result.raw_content),
        published_at: result.published_date,
    }
}

fn bounded_summary(raw_content: Option<String>) -> Option<String> {
    let raw_content = raw_content?;
    let trimmed = raw_content.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(clamp_to_max(trimmed, TAVILY_SUMMARY_MAX_BYTES))
}

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
