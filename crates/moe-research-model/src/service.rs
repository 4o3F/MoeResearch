use std::collections::BTreeMap;
use std::sync::Arc;

use moe_research_error::{Error, Result};

use crate::{ModelProvider, ModelRequest, ModelResponse};

#[derive(Default)]
pub struct ModelService {
    providers: BTreeMap<String, Arc<dyn ModelProvider>>,
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

        let provider =
            self.providers
                .get(&request.provider)
                .ok_or_else(|| Error::ProviderUnavailable {
                    provider: request.provider.clone(),
                    message: "model provider is not configured".to_owned(),
                    retryable: false,
                })?;

        request.validate()?;
        provider.complete(request).await
    }
}
