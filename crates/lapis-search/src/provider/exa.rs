use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use lapis_error::{JsonSnafu, Result};
use lapis_net::NetworkClient;
use lapis_net::provider_http::bearer_json_post;

use crate::{SearchProvider, SearchRequest, SearchResponse, SearchResult};

pub struct ExaSearchProvider {
    network: Arc<dyn NetworkClient>,
    base_url: String,
    api_key: String,
    timeout_ms: Option<u64>,
}

impl ExaSearchProvider {
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
impl SearchProvider for ExaSearchProvider {
    fn name(&self) -> &'static str {
        "exa"
    }

    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let (start_published_date, end_published_date) = match request.freshness.as_ref() {
            Some(freshness) => (freshness.since.clone(), freshness.until.clone()),
            None => (None, None),
        };
        let body = serde_json::to_value(ExaSearchRequest {
            query: request.query,
            num_results: request.max_results,
            include_domains: request.include_domains,
            exclude_domains: request.exclude_domains,
            start_published_date,
            end_published_date,
        })
        .context(JsonSnafu)?;

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

        let provider_response: ExaSearchResponse =
            serde_json::from_value(response.body).context(JsonSnafu)?;

        Ok(SearchResponse {
            provider: self.name().to_owned(),
            results: provider_response
                .results
                .into_iter()
                .map(|result| SearchResult {
                    title: result
                        .title
                        .unwrap_or_else(|| result.url.clone().unwrap_or_default()),
                    url: result.url,
                    snippet: result.text.unwrap_or_default(),
                    summary: result.summary,
                    published_at: result.published_date,
                })
                .collect(),
        })
    }
}

#[derive(Serialize)]
struct ExaSearchRequest {
    query: String,
    num_results: usize,
    include_domains: Vec<String>,
    exclude_domains: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_published_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_published_date: Option<String>,
}

#[derive(Deserialize)]
struct ExaSearchResponse {
    #[serde(default)]
    results: Vec<ExaResult>,
}

#[derive(Deserialize)]
struct ExaResult {
    title: Option<String>,
    url: Option<String>,
    text: Option<String>,
    summary: Option<String>,
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
}
