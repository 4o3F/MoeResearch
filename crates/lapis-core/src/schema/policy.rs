use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ToolName(pub String);

impl ToolName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ToolName {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for ToolName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for ToolName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ModelPolicy {
    pub allowed_providers: Vec<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub require_tool_call_support: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct Freshness {
    pub since: Option<String>,
    pub until: Option<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchPolicy {
    pub allowed_providers: Vec<String>,
    pub max_results_per_query: usize,
    pub freshness: Option<Freshness>,
    pub language: Option<String>,
    pub region: Option<String>,
    pub include_domains: Vec<String>,
    pub exclude_domains: Vec<String>,
}

impl SearchPolicy {
    pub(crate) fn validate_for_search(&self) -> Result<()> {
        if self.max_results_per_query == 0 {
            return Err(Error::InvalidInput {
                message: "search policy max_results_per_query must be greater than zero".to_owned(),
            });
        }

        let include = self
            .include_domains
            .iter()
            .map(|domain| domain.to_ascii_lowercase())
            .collect::<BTreeSet<_>>();

        if let Some(domain) = self
            .exclude_domains
            .iter()
            .map(|domain| domain.to_ascii_lowercase())
            .find(|domain| include.contains(domain))
        {
            return Err(Error::InvalidInput {
                message: format!("domain appears in both include and exclude lists: {domain}"),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct EvidencePolicy {
    pub require_evidence_for_findings: bool,
    pub min_evidence_per_finding: usize,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct OutputPolicy {
    pub language: String,
    pub max_findings_per_aspect: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ExecutionPolicy {
    pub allow_partial_results: bool,
    pub fail_fast: bool,
    pub timeout_ms: Option<u64>,
}
