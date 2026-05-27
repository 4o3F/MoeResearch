use std::sync::Arc;

use crate::error::{Error, Result};
use crate::model::provider::OpenAiProvider;
use crate::net::NetworkClient;
use crate::schema::config::LapisConfig;
use crate::schema::model::{ModelRequest, ModelResponse};
use crate::schema::policy::ModelPolicy;

#[derive(Default)]
pub struct ModelService {
    inner: lapis_model::ModelService,
}

impl ModelService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P>(&mut self, provider: P)
    where
        P: lapis_model::ModelProvider + 'static,
    {
        self.inner.register(provider);
    }

    pub fn provider_names(&self) -> Vec<String> {
        self.inner.provider_names()
    }

    pub fn inner(&self) -> &lapis_model::ModelService {
        &self.inner
    }

    pub async fn complete(
        &self,
        mut request: ModelRequest,
        policy: &ModelPolicy,
    ) -> Result<ModelResponse> {
        let provider_name = selected_provider(&request, policy)?;
        request.provider = provider_name;
        if request.temperature.is_none() {
            request.temperature = policy.temperature;
        }
        if request.max_tokens.is_none() {
            request.max_tokens = policy.max_tokens;
        }
        self.inner.complete(request).await
    }
}

impl std::ops::Deref for ModelService {
    type Target = lapis_model::ModelService;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn selected_provider(request: &ModelRequest, policy: &ModelPolicy) -> Result<String> {
    if request.provider.trim().is_empty() || request.provider.trim() != request.provider {
        return Err(Error::InvalidInput {
            message: "model provider must be explicitly selected".to_owned(),
        });
    }

    let provider = request.provider.clone();

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
            "openai" => {
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

                let Some(model) = provider
                    .model
                    .as_ref()
                    .map(|model| model.trim())
                    .filter(|model| !model.is_empty())
                else {
                    return Err(Error::ConfigInvalid {
                        message: format!("model.providers.{name}.model must be set"),
                    });
                };

                service.register(OpenAiProvider::new(
                    network.clone(),
                    provider.base_url.clone(),
                    api_key,
                    provider.timeout_ms.or(Some(config.network.timeout_ms)),
                    model.to_owned(),
                ));
            }
            other => {
                return Err(Error::ConfigInvalid {
                    message: format!("unknown model provider `{other}`"),
                });
            }
        }
    }

    Ok(service)
}
