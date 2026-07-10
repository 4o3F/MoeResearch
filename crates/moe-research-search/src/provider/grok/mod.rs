use std::sync::Arc;

use async_trait::async_trait;
use snafu::ResultExt;

use moe_research_error::{Error, JsonSnafu, Result};
use moe_research_net::NetworkClient;
use moe_research_net::provider_http::{bearer_sse_post, provider_status_retryable};

use crate::{SearchProvider, SearchRequest, SearchResponse};

mod excerpt;
mod map;
mod sse;

use map::{
    GrokReasoning, GrokSearchRequest, GrokSearchRequestParts, GrokSearchResponse, map_grok_response,
};
use sse::assemble_grok_sse;

pub struct GrokSearchProvider {
    network: Arc<dyn NetworkClient>,
    base_url: String,
    api_key: String,
    inactivity_timeout_ms: Option<u64>,
    model: String,
    max_output_tokens: Option<u32>,
    reasoning_effort: Option<GrokReasoningEffort>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrokReasoningEffort {
    None,
    Low,
    Medium,
    High,
}

impl GrokReasoningEffort {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

impl GrokSearchProvider {
    pub fn new(
        network: Arc<dyn NetworkClient>,
        base_url: String,
        api_key: String,
        inactivity_timeout_ms: Option<u64>,
        model: String,
    ) -> Self {
        Self::with_request_options(
            network,
            base_url,
            api_key,
            inactivity_timeout_ms,
            model,
            None,
            None,
        )
    }

    /// Constructs the provider with the response-size cap supplied from
    /// configuration. `None` leaves the cap to the upstream provider default.
    /// Validation of this input is owned by `SearchProviderEndpoint::validate`;
    /// this constructor trusts its caller.
    #[must_use]
    pub fn with_max_output_tokens(
        network: Arc<dyn NetworkClient>,
        base_url: String,
        api_key: String,
        inactivity_timeout_ms: Option<u64>,
        model: String,
        max_output_tokens: Option<u32>,
    ) -> Self {
        Self::with_request_options(
            network,
            base_url,
            api_key,
            inactivity_timeout_ms,
            model,
            max_output_tokens,
            None,
        )
    }

    #[must_use]
    pub fn with_request_options(
        network: Arc<dyn NetworkClient>,
        base_url: String,
        api_key: String,
        inactivity_timeout_ms: Option<u64>,
        model: String,
        max_output_tokens: Option<u32>,
        reasoning_effort: Option<GrokReasoningEffort>,
    ) -> Self {
        Self {
            network,
            base_url,
            api_key,
            inactivity_timeout_ms,
            model,
            max_output_tokens,
            reasoning_effort,
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
        let parts = GrokSearchRequestParts::from(request);
        let body = serde_json::to_value(GrokSearchRequest {
            model: self.model.clone(),
            input: parts.input,
            tools: parts.tools,
            reasoning: self.reasoning_effort.map(|effort| GrokReasoning {
                effort: effort.as_str().to_owned(),
            }),
            max_output_tokens: self.max_output_tokens,
            stream: true,
        })
        .context(JsonSnafu)?;

        let mut response = self
            .network
            .send_sse(bearer_sse_post(
                &self.base_url,
                "responses",
                &self.api_key,
                body,
                self.inactivity_timeout_ms,
            ))
            .await?;

        if !(200..300).contains(&response.status) {
            return Err(Error::HttpStatus {
                status: response.status,
                message: "grok search provider returned non-success status".to_owned(),
                retryable: provider_status_retryable(response.status),
            });
        }

        let provider_response: GrokSearchResponse =
            serde_json::from_value(assemble_grok_sse(&mut response).await?).context(JsonSnafu)?;

        Ok(SearchResponse {
            provider: self.name().to_owned(),
            results: map_grok_response(provider_response, max_results),
        })
    }
}
