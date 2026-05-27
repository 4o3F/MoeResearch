use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use lapis_error::{Error, Result};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct Freshness {
    pub since: Option<String>,
    pub until: Option<String>,
}

impl Freshness {
    #[must_use]
    pub fn describe_for_prompt(&self) -> Option<String> {
        match (self.since.as_ref(), self.until.as_ref()) {
            (None, None) => None,
            (Some(since), None) => Some(format!("published on or after {since}")),
            (None, Some(until)) => Some(format!("published on or before {until}")),
            (Some(since), Some(until)) => Some(format!("published between {since} and {until}")),
        }
    }
}

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

    pub fn validate(&self) -> Result<()> {
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

        Ok(())
    }
}
