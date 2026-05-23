use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::error::{JsonSnafu, Result};
use crate::net::NetworkClient;
use crate::schema::network::{Header, NetworkRequest};
use crate::schema::search::{ProviderSearchRequest, SearchResponse, SearchResult};
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

    async fn search(&self, request: ProviderSearchRequest) -> Result<SearchResponse> {
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
