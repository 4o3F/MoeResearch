use std::collections::BTreeMap;
use std::sync::Arc;

use lapis_error::{Error, Result};

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

    pub async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        request.validate()?;
        let provider_name = request.provider.clone();
        let provider =
            self.providers
                .get(&provider_name)
                .ok_or_else(|| Error::ProviderUnavailable {
                    provider: provider_name.clone(),
                    message: "search provider is not configured".to_owned(),
                })?;

        provider.search(request).await
    }
}
