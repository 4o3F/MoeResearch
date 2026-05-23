use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::policy::{Freshness, SearchPolicy};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchRequest {
    pub query: String,
    pub max_results: usize,
    pub freshness: Option<Freshness>,
    pub language: Option<String>,
    pub region: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSearchRequest {
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

impl ProviderSearchRequest {
    pub fn from_policy(request: SearchRequest, policy: &SearchPolicy) -> Self {
        Self {
            query: request.query,
            max_results: request.max_results,
            freshness: request.freshness.or_else(|| policy.freshness.clone()),
            language: request.language.or_else(|| policy.language.clone()),
            region: request.region.or_else(|| policy.region.clone()),
            include_domains: policy.include_domains.clone(),
            exclude_domains: policy.exclude_domains.clone(),
        }
    }
}

impl SearchRequest {
    pub fn validate(&self) -> Result<()> {
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

        Ok(())
    }

    pub(crate) fn validate_with_policy(&self, policy: &SearchPolicy) -> Result<()> {
        self.validate()?;

        policy.validate_for_search()?;

        if self.max_results > policy.max_results_per_query {
            return Err(Error::InvalidInput {
                message: "search request max_results exceeds policy max_results_per_query"
                    .to_owned(),
            });
        }

        Ok(())
    }

    pub fn from_query(query: impl Into<String>, max_results: usize) -> Self {
        Self {
            query: query.into(),
            max_results,
            freshness: None,
            language: None,
            region: None,
        }
    }
}
