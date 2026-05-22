use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::net::NetworkClient;
use crate::schema::common::SearchPolicy;
use crate::schema::config::LapisConfig;
use crate::schema::search::{SearchRequest, SearchResponse};
use crate::search::provider::SearchProvider;
use crate::search::providers::{ExaSearchProvider, GrokSearchProvider};

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

    pub fn register_arc(&mut self, provider: Arc<dyn SearchProvider>) {
        self.providers.insert(provider.name().to_owned(), provider);
    }

    pub fn provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub async fn search(
        &self,
        request: SearchRequest,
        policy: &SearchPolicy,
    ) -> Result<SearchResponse> {
        let provider_names = self.candidate_providers(policy);
        let mut last_error = None;

        for name in provider_names {
            let Some(provider) = self.providers.get(&name) else {
                continue;
            };

            match provider.search(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    tracing::warn!(provider = name, error = %error, "search provider failed");
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::ProviderUnavailable {
            provider: "search".to_owned(),
            message: "no configured search provider matched the request policy".to_owned(),
        }))
    }

    fn candidate_providers(&self, policy: &SearchPolicy) -> Vec<String> {
        let allowed = if policy.allowed_providers.is_empty() {
            self.providers.keys().cloned().collect::<BTreeSet<_>>()
        } else {
            policy
                .allowed_providers
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
        };

        let mut names = Vec::new();
        for provider in &policy.preferred_providers {
            if allowed.contains(provider) && !names.contains(provider) {
                names.push(provider.clone());
            }
        }
        for provider in allowed {
            if !names.contains(&provider) {
                names.push(provider);
            }
        }
        names
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
            "grok" => service.register(GrokSearchProvider::new(
                network.clone(),
                provider.base_url.clone(),
                api_key,
                provider.timeout_ms.or(Some(config.network.timeout_ms)),
            )),
            _ => {
                tracing::warn!(provider = name, "unknown search provider ignored");
            }
        }
    }

    Ok(service)
}
