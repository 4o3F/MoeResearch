use std::collections::BTreeMap;
use std::sync::Arc;

use moe_research_error::{Error, Result};

use crate::{SearchProvider, SearchRequest, SearchResponse};

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

    #[must_use]
    pub fn provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        request.validate()?;
        let provider_name = request.provider.clone();
        let configured_providers = self.provider_names();
        tracing::debug!(
            event = "search_provider_dispatching",
            status = "starting",
            provider_kind = "search",
            provider = %provider_name,
            configured_provider_count = configured_providers.len(),
            configured_providers = ?configured_providers,
            max_results = request.max_results,
            has_freshness = request.freshness.is_some(),
            depth = ?request.depth,
            content_level = ?request.content_level,
            recency = ?request.recency,
            category = ?request.category,
            include_domain_count = request.include_domains.len(),
            exclude_domain_count = request.exclude_domains.len(),
            "dispatching search request"
        );
        let provider =
            self.providers
                .get(&provider_name)
                .ok_or_else(|| Error::ProviderUnavailable {
                    provider: provider_name.clone(),
                    message: "search provider is not configured".to_owned(),
                    retryable: false,
                })?;

        let response = provider.search(request).await?;
        tracing::debug!(
            event = "search_provider_completed",
            status = "ok",
            provider_kind = "search",
            provider = %provider_name,
            result_count = response.results.len(),
            "search request completed"
        );
        Ok(response)
    }
}
