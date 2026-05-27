use std::sync::Arc;

use crate::error::{Error, Result};
use crate::net::NetworkClient;
use crate::schema::config::LapisConfig;
use crate::schema::policy::SearchPolicy;
use crate::schema::search::{SearchRequest, SearchResponse};
use crate::search::provider::{ExaSearchProvider, GrokSearchProvider, SearchProvider};

#[derive(Default)]
pub struct SearchService {
    inner: lapis_search::SearchService,
}

impl SearchService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P>(&mut self, provider: P)
    where
        P: SearchProvider + 'static,
    {
        self.inner.register(provider);
    }

    pub fn inner(&self) -> &lapis_search::SearchService {
        &self.inner
    }

    pub async fn search(
        &self,
        request: SearchRequest,
        policy: &SearchPolicy,
    ) -> Result<SearchResponse> {
        self.inner.search(policy.apply_to(request)?).await
    }
}

impl std::ops::Deref for SearchService {
    type Target = lapis_search::SearchService;

    fn deref(&self) -> &Self::Target {
        &self.inner
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

                service.register(GrokSearchProvider::with_max_output_tokens(
                    network.clone(),
                    provider.base_url.clone(),
                    api_key,
                    provider.timeout_ms.or(Some(config.network.timeout_ms)),
                    model.to_owned(),
                    provider.max_output_tokens,
                ));
            }
            other => {
                return Err(Error::ConfigInvalid {
                    message: format!("unknown search provider `{other}`"),
                });
            }
        }
    }

    Ok(service)
}
