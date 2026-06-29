use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::limit::DurationLimitMs;
use moe_research_error::{Error, Result};

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
    #[schemars(schema_with = "crate::limit::optional_non_negative_integer_schema")]
    pub max_tokens: Option<u32>,
    pub require_tool_call_support: bool,
}

pub use moe_research_search::{
    Freshness, SearchCategory, SearchContentLevel, SearchDepth, SearchRecency,
};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchPolicy {
    pub allowed_providers: Vec<String>,
    #[schemars(schema_with = "crate::limit::non_negative_integer_schema")]
    pub max_results_per_query: usize,
    pub freshness: Option<Freshness>,
    pub depth: Option<SearchDepth>,
    pub content_level: Option<SearchContentLevel>,
    pub recency: Option<SearchRecency>,
    pub category: Option<SearchCategory>,
    pub language: Option<String>,
    pub region: Option<String>,
    pub include_domains: Vec<String>,
    pub exclude_domains: Vec<String>,
}

impl ModelPolicy {
    pub fn apply_to(
        &self,
        mut request: moe_research_model::ModelRequest,
    ) -> Result<moe_research_model::ModelRequest> {
        if request.provider.trim().is_empty() || request.provider.trim() != request.provider {
            return Err(Error::InvalidInput {
                message: "model provider must be explicitly selected".to_owned(),
            });
        }

        if !self.allowed_providers.is_empty() && !self.allowed_providers.contains(&request.provider)
        {
            return Err(Error::ProviderUnavailable {
                provider: request.provider.clone(),
                message: "model provider is not allowed by policy".to_owned(),
            });
        }

        if request.temperature.is_none() {
            request.temperature = self.temperature;
        }
        if request.max_tokens.is_none() {
            request.max_tokens = self.max_tokens;
        }

        Ok(request)
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

    pub fn apply_to(
        &self,
        mut request: moe_research_search::SearchRequest,
    ) -> Result<moe_research_search::SearchRequest> {
        self.validate_for_search()?;
        request.validate()?;

        if !self.allowed_providers.contains(&request.provider) {
            return Err(Error::ProviderUnavailable {
                provider: request.provider.clone(),
                message: "search provider is not allowed by policy".to_owned(),
            });
        }

        if request.max_results > self.max_results_per_query {
            return Err(Error::InvalidInput {
                message: "search request max_results exceeds policy max_results_per_query"
                    .to_owned(),
            });
        }

        if let (Some(request_depth), Some(policy_depth)) = (request.depth, self.depth)
            && request_depth.rank() > policy_depth.rank()
        {
            return Err(Error::InvalidInput {
                message: "search request depth exceeds policy depth".to_owned(),
            });
        }

        if let (Some(request_level), Some(policy_level)) =
            (request.content_level, self.content_level)
            && request_level.rank() > policy_level.rank()
        {
            return Err(Error::InvalidInput {
                message: "search request content_level exceeds policy content_level".to_owned(),
            });
        }

        if let (Some(request_recency), Some(policy_recency)) = (request.recency, self.recency)
            && request_recency.rank() > policy_recency.rank()
        {
            return Err(Error::InvalidInput {
                message: "search request recency exceeds policy recency".to_owned(),
            });
        }

        if let (Some(request_category), Some(policy_category)) = (request.category, self.category)
            && request_category != policy_category
        {
            return Err(Error::InvalidInput {
                message: "search request category conflicts with policy category".to_owned(),
            });
        }

        request.freshness = request.freshness.or_else(|| self.freshness.clone());
        request.depth = request.depth.or(self.depth);
        request.content_level = request.content_level.or(self.content_level);
        request.recency = request.recency.or(self.recency);
        request.category = request.category.or(self.category);
        request.language = request.language.or_else(|| self.language.clone());
        request.region = request.region.or_else(|| self.region.clone());
        request.include_domains.clone_from(&self.include_domains);
        request.exclude_domains.clone_from(&self.exclude_domains);
        Ok(request)
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct EvidencePolicy {
    pub require_evidence_for_findings: bool,
    #[schemars(schema_with = "crate::limit::non_negative_integer_schema")]
    pub min_evidence_per_finding: usize,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct OutputPolicy {
    pub language: String,
    #[schemars(schema_with = "crate::limit::optional_non_negative_integer_schema")]
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
