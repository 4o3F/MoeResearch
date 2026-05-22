use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::error::{JsonSnafu, Result};
use crate::net::NetworkClient;
use crate::schema::common::{Header, NetworkRequest};
use crate::schema::search::{SearchRequest, SearchResponse, SearchResult};
use crate::search::provider::SearchProvider;

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
        let body = serde_json::to_value(ExaSearchRequest {
            query: request.query,
            num_results: request.max_results,
            include_domains: request.include_domains,
            exclude_domains: request.exclude_domains,
        })
        .context(JsonSnafu)?;

        let response = self
            .network
            .send(NetworkRequest {
                method: "POST".to_owned(),
                url: format!("{}/search", self.base_url.trim_end_matches('/')),
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use super::*;
    use crate::net::client::MockNetworkClient;
    use crate::schema::common::NetworkResponse;

    #[tokio::test]
    async fn maps_exa_response_to_standard_search_response() {
        let network = Arc::new(MockNetworkClient::new([NetworkResponse {
            status: 200,
            headers: vec![],
            body: json!({
                "results": [{
                    "title": "Lapis",
                    "url": "https://example.com/lapis",
                    "text": "snippet",
                    "summary": "summary",
                    "publishedDate": "2026-01-01"
                }]
            }),
        }]));
        let provider = ExaSearchProvider::new(
            network,
            "https://api.exa.ai".to_owned(),
            "key".to_owned(),
            None,
        );

        let response = provider
            .search(SearchRequest::from_query("lapis", 1))
            .await
            .expect("exa response");

        assert_eq!(response.provider, "exa");
        assert_eq!(response.results[0].title, "Lapis");
    }
}
