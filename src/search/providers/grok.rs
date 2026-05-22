use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::error::{JsonSnafu, Result};
use crate::net::NetworkClient;
use crate::schema::common::{Header, NetworkRequest};
use crate::schema::search::{SearchRequest, SearchResponse, SearchResult};
use crate::search::provider::SearchProvider;

pub struct GrokSearchProvider {
    network: Arc<dyn NetworkClient>,
    base_url: String,
    api_key: String,
    timeout_ms: Option<u64>,
}

impl GrokSearchProvider {
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
impl SearchProvider for GrokSearchProvider {
    fn name(&self) -> &'static str {
        "grok"
    }

    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let body = serde_json::to_value(GrokSearchRequest {
            query: request.query,
            max_results: request.max_results,
            language: request.language,
            region: request.region,
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

        let provider_response: GrokSearchResponse =
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
                    snippet: result.snippet.unwrap_or_default(),
                    summary: result.summary,
                    published_at: result.published_at,
                })
                .collect(),
        })
    }
}

#[derive(Serialize)]
struct GrokSearchRequest {
    query: String,
    max_results: usize,
    language: Option<String>,
    region: Option<String>,
}

#[derive(Deserialize)]
struct GrokSearchResponse {
    #[serde(default)]
    results: Vec<GrokResult>,
}

#[derive(Deserialize)]
struct GrokResult {
    title: Option<String>,
    url: Option<String>,
    snippet: Option<String>,
    summary: Option<String>,
    published_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use super::*;
    use crate::net::client::MockNetworkClient;
    use crate::schema::common::NetworkResponse;

    #[tokio::test]
    async fn maps_grok_response_to_standard_search_response() {
        let network = Arc::new(MockNetworkClient::new([NetworkResponse {
            status: 200,
            headers: vec![],
            body: json!({
                "results": [{
                    "title": "Grok result",
                    "url": "https://example.com/grok",
                    "snippet": "snippet",
                    "summary": "summary",
                    "published_at": "2026-01-01"
                }]
            }),
        }]));
        let provider = GrokSearchProvider::new(
            network,
            "https://api.x.ai".to_owned(),
            "key".to_owned(),
            None,
        );

        let response = provider
            .search(SearchRequest::from_query("lapis", 1))
            .await
            .expect("grok response");

        assert_eq!(response.provider, "grok");
        assert_eq!(response.results[0].title, "Grok result");
    }
}
