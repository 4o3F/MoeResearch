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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::model::{ModelMessage, ModelMessageRole};

    struct StaticProvider(&'static str);

    struct CapturingProvider {
        seen: Arc<std::sync::Mutex<Option<ModelRequest>>>,
    }

    #[async_trait::async_trait]
    impl ModelProvider for StaticProvider {
        fn name(&self) -> &'static str {
            self.0
        }

        async fn complete(&self, _request: ModelRequest) -> Result<ModelResponse> {
            Ok(ModelResponse {
                provider: self.0.to_owned(),
                model: None,
                content: Some("content".to_owned()),
                tool_calls: vec![],
                usage: None,
            })
        }
    }

    #[async_trait::async_trait]
    impl ModelProvider for CapturingProvider {
        fn name(&self) -> &'static str {
            "alpha"
        }

        async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
            *self.seen.lock().expect("request lock") = Some(request.clone());
            Ok(ModelResponse {
                provider: request.provider,
                model: request.model,
                content: Some("content".to_owned()),
                tool_calls: vec![],
                usage: None,
            })
        }
    }

    fn request(provider: &str) -> ModelRequest {
        ModelRequest {
            provider: provider.to_owned(),
            model: None,
            messages: vec![ModelMessage {
                role: ModelMessageRole::User,
                content: "hello".to_owned(),
            }],
            tools: vec![],
            temperature: None,
            max_tokens: None,
        }
    }

    #[tokio::test]
    async fn routes_requested_allowed_provider() {
        let mut service = ModelService::new();
        service.register(StaticProvider("alpha"));
        service.register(StaticProvider("beta"));
        let policy = ModelPolicy {
            allowed_providers: vec!["beta".to_owned()],
            ..ModelPolicy::default()
        };

        let response = service
            .complete(request("beta"), &policy)
            .await
            .expect("model response");

        assert_eq!(response.provider, "beta");
    }

    #[tokio::test]
    async fn uses_default_provider_when_request_provider_is_empty() {
        let mut service = ModelService::new();
        service.register(StaticProvider("alpha"));
        let policy = ModelPolicy {
            default_provider: "alpha".to_owned(),
            allowed_providers: vec!["alpha".to_owned()],
            ..ModelPolicy::default()
        };

        let response = service
            .complete(request(""), &policy)
            .await
            .expect("model response");

        assert_eq!(response.provider, "alpha");
    }

    #[tokio::test]
    async fn rejects_disallowed_provider() {
        let mut service = ModelService::new();
        service.register(StaticProvider("beta"));
        let policy = ModelPolicy {
            allowed_providers: vec!["alpha".to_owned()],
            ..ModelPolicy::default()
        };

        let error = service
            .complete(request("beta"), &policy)
            .await
            .expect_err("disallowed provider error");

        assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "beta"));
    }

    #[tokio::test]
    async fn applies_policy_defaults_before_dispatch() {
        let seen = Arc::new(std::sync::Mutex::new(None));
        let mut service = ModelService::new();
        service.register(CapturingProvider { seen: seen.clone() });
        let policy = ModelPolicy {
            default_provider: "alpha".to_owned(),
            default_model: Some("model-a".to_owned()),
            allowed_providers: vec!["alpha".to_owned()],
            temperature: Some(0.7),
            max_tokens: Some(128),
            ..ModelPolicy::default()
        };

        service
            .complete(request(""), &policy)
            .await
            .expect("model response");
        let request = seen
            .lock()
            .expect("request lock")
            .clone()
            .expect("captured request");

        assert_eq!(request.provider, "alpha");
        assert_eq!(request.model.as_deref(), Some("model-a"));
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(128));
    }

    #[test]
    fn provider_names_returns_registered_names() {
        let mut service = ModelService::new();
        service.register(StaticProvider("beta"));
        service.register(StaticProvider("alpha"));

        assert_eq!(
            service.provider_names(),
            vec!["alpha".to_owned(), "beta".to_owned()]
        );
    }
}
