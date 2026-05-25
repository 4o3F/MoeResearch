use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::policy::{Freshness, SearchPolicy};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchRequest {
    pub provider: String,
    pub query: String,
    pub max_results: usize,
    pub freshness: Option<Freshness>,
    pub language: Option<String>,
    pub region: Option<String>,
    pub include_domains: Vec<String>,
    pub exclude_domains: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchResponse {
    pub provider: String,
    pub results: Vec<SearchResult>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: Option<String>,
    pub snippet: String,
    pub summary: Option<String>,
    pub published_at: Option<String>,
}

impl SearchRequest {
    #[must_use]
    pub fn new(provider: impl Into<String>, query: impl Into<String>, max_results: usize) -> Self {
        Self {
            provider: provider.into(),
            query: query.into(),
            max_results,
            freshness: None,
            language: None,
            region: None,
            include_domains: Vec::new(),
            exclude_domains: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_policy(mut self, policy: &SearchPolicy) -> Self {
        self.freshness = self.freshness.or_else(|| policy.freshness.clone());
        self.language = self.language.or_else(|| policy.language.clone());
        self.region = self.region.or_else(|| policy.region.clone());
        self.include_domains.clone_from(&policy.include_domains);
        self.exclude_domains.clone_from(&policy.exclude_domains);
        self
    }

    pub(crate) fn validate_with_policy(&self, policy: &SearchPolicy) -> Result<()> {
        if self.provider.trim().is_empty() || self.provider.trim() != self.provider {
            return Err(Error::InvalidInput {
                message: "search provider must be explicitly selected".to_owned(),
            });
        }

        if self.query.trim().is_empty() {
            return Err(Error::InvalidInput {
                message: "search query must not be empty".to_owned(),
            });
        }

        if self.max_results == 0 {
            return Err(Error::InvalidInput {
                message: "search max_results must be greater than zero".to_owned(),
            });
        }

        policy.validate_for_search()?;

        if !policy.allowed_providers.contains(&self.provider) {
            return Err(Error::ProviderUnavailable {
                provider: self.provider.clone(),
                message: "search provider is not allowed by policy".to_owned(),
            });
        }

        if self.max_results > policy.max_results_per_query {
            return Err(Error::InvalidInput {
                message: "search request max_results exceeds policy max_results_per_query"
                    .to_owned(),
            });
        }

        Ok(())
    }
}
