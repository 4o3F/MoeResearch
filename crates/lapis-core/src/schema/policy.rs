use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ToolName(pub String);

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ModelSelector {
    pub provider: String,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchSelector {
    pub providers: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct Freshness {
    pub since: Option<String>,
    pub until: Option<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct EvidenceRequirement {
    pub min_sources: usize,
    pub min_independent_sources: usize,
    pub allow_low_confidence_findings: bool,
}

impl Default for EvidenceRequirement {
    fn default() -> Self {
        Self {
            min_sources: 2,
            min_independent_sources: 2,
            allow_low_confidence_findings: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ModelPolicy {
    pub default_provider: String,
    pub default_model: Option<String>,
    pub allowed_providers: Vec<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub require_tool_call_support: bool,
}

impl Default for ModelPolicy {
    fn default() -> Self {
        Self {
            default_provider: "openai-compatible".to_owned(),
            default_model: None,
            allowed_providers: vec!["openai-compatible".to_owned()],
            temperature: Some(0.2),
            max_tokens: None,
            require_tool_call_support: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchPolicy {
    pub allowed_providers: Vec<String>,
    pub preferred_providers: Vec<String>,
    pub max_results_per_query: usize,
    pub freshness: Option<Freshness>,
    pub language: Option<String>,
    pub region: Option<String>,
    pub include_domains: Vec<String>,
    pub exclude_domains: Vec<String>,
}

impl Default for SearchPolicy {
    fn default() -> Self {
        Self {
            allowed_providers: vec!["exa".to_owned(), "grok".to_owned()],
            preferred_providers: vec!["exa".to_owned(), "grok".to_owned()],
            max_results_per_query: 5,
            freshness: None,
            language: None,
            region: None,
            include_domains: vec![],
            exclude_domains: vec![],
        }
    }
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
    pub include_query_trace: bool,
    pub include_source_urls: bool,
}

impl Default for EvidencePolicy {
    fn default() -> Self {
        Self {
            require_evidence_for_findings: true,
            min_evidence_per_finding: 1,
            include_query_trace: true,
            include_source_urls: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct OutputPolicy {
    pub language: String,
    pub include_trace_summary: bool,
    pub include_raw_search_snippets: bool,
    pub max_findings_per_aspect: Option<usize>,
}

impl Default for OutputPolicy {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_owned(),
            include_trace_summary: true,
            include_raw_search_snippets: false,
            max_findings_per_aspect: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ExecutionPolicy {
    pub allow_partial_results: bool,
    pub fail_fast: bool,
    pub timeout_ms: Option<u64>,
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            allow_partial_results: true,
            fail_fast: false,
            timeout_ms: Some(300_000),
        }
    }
}
