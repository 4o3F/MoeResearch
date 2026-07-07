use std::collections::BTreeMap;
use std::sync::Arc;

use moe_research_error::{Error, Result};

use crate::{ModelProvider, ModelRequest, ModelResponse, ModelResponseFormat};

#[derive(Default)]
pub struct ModelService {
    providers: BTreeMap<String, Arc<dyn ModelProvider>>,
}

fn response_format_name(format: &ModelResponseFormat) -> &'static str {
    match format {
        ModelResponseFormat::Text => "text",
        ModelResponseFormat::JsonSchema(_) => "json_schema",
    }
}

impl ModelService {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P>(&mut self, provider: P)
    where
        P: ModelProvider + 'static,
    {
        self.providers
            .insert(provider.name().to_owned(), Arc::new(provider));
    }

    #[must_use]
    pub fn provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        if request.provider.trim().is_empty() || request.provider.trim() != request.provider {
            return Err(Error::InvalidInput {
                message: "model provider must be explicitly selected".to_owned(),
            });
        }

        let provider_name = request.provider.clone();
        let configured_providers = self.provider_names();
        tracing::debug!(
            event = "model_provider_dispatching",
            status = "starting",
            provider_kind = "model",
            provider = %provider_name,
            configured_provider_count = configured_providers.len(),
            configured_providers = ?configured_providers,
            input_item_count = request.input.len(),
            tool_count = request.tools.len(),
            has_previous_response_id = request.previous_response_id.is_some(),
            response_format = request.response_format.as_ref().map(response_format_name),
            "dispatching model request"
        );

        let provider =
            self.providers
                .get(&provider_name)
                .ok_or_else(|| Error::ProviderUnavailable {
                    provider: provider_name.clone(),
                    message: "model provider is not configured".to_owned(),
                    retryable: false,
                })?;

        request.validate()?;
        let response = provider.complete(request).await?;
        tracing::debug!(
            event = "model_provider_completed",
            status = "ok",
            provider_kind = "model",
            provider = %provider_name,
            "model request completed"
        );
        Ok(response)
    }
}
