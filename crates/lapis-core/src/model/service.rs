use std::collections::BTreeMap;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::model::provider::ModelProvider;
use crate::model::providers::OpenAiCompatibleProvider;
use crate::net::NetworkClient;
use crate::schema::common::ModelPolicy;
use crate::schema::config::LapisConfig;
use crate::schema::model::{ModelRequest, ModelResponse};

#[derive(Default)]
pub struct ModelService {
    providers: BTreeMap<String, Arc<dyn ModelProvider>>,
}

impl ModelService {
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

    pub fn register_arc(&mut self, provider: Arc<dyn ModelProvider>) {
        self.providers.insert(provider.name().to_owned(), provider);
    }

    pub fn provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub async fn complete(
        &self,
        mut request: ModelRequest,
        policy: &ModelPolicy,
    ) -> Result<ModelResponse> {
        let provider_name = selected_provider(&request, policy)?;
        let provider =
            self.providers
                .get(&provider_name)
                .ok_or_else(|| Error::ProviderUnavailable {
                    provider: provider_name.clone(),
                    message: "model provider is not configured".to_owned(),
                })?;

        request.provider = provider_name;
        if request.model.is_none() {
            request.model = policy.default_model.clone();
        }
        if request.temperature.is_none() {
            request.temperature = policy.temperature;
        }
        if request.max_tokens.is_none() {
            request.max_tokens = policy.max_tokens;
        }
        provider.complete(request).await
    }
}

fn selected_provider(request: &ModelRequest, policy: &ModelPolicy) -> Result<String> {
    let provider = if request.provider.is_empty() {
        policy.default_provider.clone()
    } else {
        request.provider.clone()
    };

    if !policy.allowed_providers.is_empty() && !policy.allowed_providers.contains(&provider) {
        return Err(Error::ProviderUnavailable {
            provider,
            message: "model provider is not allowed by policy".to_owned(),
        });
    }

    Ok(provider)
}

pub fn build_model_service(
    config: &LapisConfig,
    network: &Arc<dyn NetworkClient>,
) -> Result<ModelService> {
    let mut service = ModelService::new();

    for (name, provider) in &config.model.providers {
        if !provider.enabled {
            continue;
        }

        match name.as_str() {
            "openai-compatible" => {
                let Some(api_key_env) = provider.api_key_env.as_ref() else {
                    return Err(Error::ProviderUnavailable {
                        provider: name.clone(),
                        message: "enabled model provider must set api_key_env".to_owned(),
                    });
                };
                let api_key =
                    std::env::var(api_key_env).map_err(|_| Error::ProviderUnavailable {
                        provider: name.clone(),
                        message: format!("environment variable {api_key_env} is not set"),
                    })?;

                service.register(OpenAiCompatibleProvider::new(
                    network.clone(),
                    provider.base_url.clone(),
                    api_key,
                    provider.timeout_ms.or(Some(config.network.timeout_ms)),
                ));
            }
            _ => {
                tracing::error!(provider = name, "unknown model provider ignored");
            }
        }
    }

    Ok(service)
}
