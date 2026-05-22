use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::Freshness;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchRequest {
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
    pub fn from_query(query: impl Into<String>, max_results: usize) -> Self {
        Self {
            query: query.into(),
            max_results,
            freshness: None,
            language: None,
            region: None,
            include_domains: vec![],
            exclude_domains: vec![],
        }
    }
}
