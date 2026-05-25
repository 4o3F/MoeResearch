use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::schema::limit::DurationLimitMs;

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

impl Freshness {
    /// Renders the freshness window as a single natural-language phrase that
    /// Grok-style search providers can embed in their prompt.
    ///
    /// Returns `None` when both `since` and `until` are absent; otherwise
    /// returns a description such as `"published between 2024-01-01 and
    /// 2024-12-31"`. Strings are echoed verbatim from the policy field, so
    /// callers should validate the format at policy ingest time.
    #[must_use]
    pub fn describe_for_prompt(&self) -> Option<String> {
        match (self.since.as_ref(), self.until.as_ref()) {
            (None, None) => None,
            (Some(since), None) => Some(format!("published on or after {since}")),
            (None, Some(until)) => Some(format!("published on or before {until}")),
            (Some(since), Some(until)) => {
                Some(format!("published between {since} and {until}"))
            }
        }
    }
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
    /// Per-call execution deadline. Promoted from `Option<u64>` to
    /// [`DurationLimitMs`] so callers can express "unlimited" with the
    /// same `-1` sentinel that [`AgentBudget`] and [`BudgetConfig`]
    /// accept, instead of mixing two encodings for the same concept.
    #[serde(default = "DurationLimitMs::unlimited")]
    pub timeout_ms: DurationLimitMs,
}
