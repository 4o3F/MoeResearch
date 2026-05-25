use std::collections::BTreeMap;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::net::NetworkClient;
use crate::schema::config::LapisConfig;
use crate::schema::policy::SearchPolicy;
use crate::schema::search::{SearchRequest, SearchResponse};
use crate::search::provider::{ExaSearchProvider, GrokSearchProvider, SearchProvider};

#[derive(Default)]
pub struct SearchService {
    providers: BTreeMap<String, Arc<dyn SearchProvider>>,
}

impl SearchService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P>(&mut self, provider: P)
    where
        P: SearchProvider + 'static,
    {
        self.providers
            .insert(provider.name().to_owned(), Arc::new(provider));
    }

    pub fn provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub async fn search(
        &self,
        request: SearchRequest,
        policy: &SearchPolicy,
    ) -> Result<SearchResponse> {
        request.validate_with_policy(policy)?;
        let provider_name = request.provider.clone();
        let provider =
            self.providers
                .get(&provider_name)
                .ok_or_else(|| Error::ProviderUnavailable {
                    provider: provider_name.clone(),
                    message: "search provider is not configured".to_owned(),
                })?;

        provider.search(request.with_policy(policy)).await
    }
}

pub fn build_search_service(
    config: &LapisConfig,
    network: &Arc<dyn NetworkClient>,
) -> Result<SearchService> {
    let mut service = SearchService::new();

    for (name, provider) in &config.search.providers {
        if !provider.enabled {
            continue;
        }

        let Some(api_key_env) = provider.api_key_env.as_ref() else {
            return Err(Error::ProviderUnavailable {
                provider: name.clone(),
                message: "enabled search provider must set api_key_env".to_owned(),
            });
        };
        let api_key = std::env::var(api_key_env).map_err(|_| Error::ProviderUnavailable {
            provider: name.clone(),
            message: format!("environment variable {api_key_env} is not set"),
        })?;

        match name.as_str() {
            "exa" => service.register(ExaSearchProvider::new(
                network.clone(),
                provider.base_url.clone(),
                api_key,
                provider.timeout_ms.or(Some(config.network.timeout_ms)),
            )),
            "grok" => {
                let Some(model) = provider
                    .model
                    .as_ref()
                    .map(|model| model.trim())
                    .filter(|model| !model.is_empty())
                else {
                    return Err(Error::ConfigInvalid {
                        message: format!("search.providers.{name}.model must be set"),
                    });
                };

                service.register(GrokSearchProvider::new(
                    network.clone(),
                    provider.base_url.clone(),
                    api_key,
                    provider.timeout_ms.or(Some(config.network.timeout_ms)),
                    model.to_owned(),
                ));
            }
            _ => {
                tracing::warn!(provider = name, "unknown search provider ignored");
            }
        }
    }

    Ok(service)
}
