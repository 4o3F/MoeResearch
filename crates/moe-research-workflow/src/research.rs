use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use moe_research_error::{Error, Result};

use super::budget::{AgentLimits, BudgetConfig, ResearchLimits};
use super::limit::Limit;
use super::policy::{
    EvidencePolicy, ExecutionPolicy, ModelPolicy, OutputPolicy, SearchPolicy, ToolName,
};
use super::report::Evidence;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchContext {
    pub summary: String,
    pub known_facts: Vec<String>,
    pub excluded_assumptions: Vec<String>,
    pub prior_sources: Vec<Evidence>,
}

impl ResearchContext {
    /// Explicitly construct an empty research context that carries no prior
    /// knowledge.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            summary: String::new(),
            known_facts: Vec::new(),
            excluded_assumptions: Vec::new(),
            prior_sources: Vec::new(),
        }
    }
}

/// Policies that shape one MoeResearch request without carrying resource limits.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchPolicy {
    pub model: ModelPolicy,
    pub search: SearchPolicy,
    pub evidence: EvidencePolicy,
    pub output: OutputPolicy,
    pub execution: ExecutionPolicy,
}

/// One runnable research aspect.
///
/// Aspects are the unit of parallelism for deep research: each aspect runs
/// inside its own agent loop with its own limits, tool allowlist, and provider
/// selection. Layer 1 (the Claude Code skill) constructs one `AspectRequest`
/// per dimension of the user's question.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AspectRequest {
    /// Stable identifier used by the orchestrator to track per-aspect state
    /// (limits, evidence namespacing, failure records).
    pub id: String,
    /// Human-readable aspect name surfaced to the model and final report.
    pub name: String,
    /// Short role description (e.g. "competitive landscape analyst") used in
    /// the aspect agent's system prompt.
    pub role: String,
    /// Concrete research question this aspect must answer.
    pub question: String,
    /// Topical scope guides for the aspect agent (in-scope topics).
    pub scope: Vec<String>,
    /// Explicit out-of-scope boundaries.
    pub boundaries: Vec<String>,
    /// Acceptance criteria the aspect's findings must meet before completion.
    pub success_criteria: Vec<String>,
    /// Layer 2 aspect-agent system prompt content, provided inline by Layer 1.
    ///
    /// Rust core never performs prompt file IO; Layer 1 (the Claude Code skill)
    /// owns prompt selection, version pinning, and substitution, then passes
    /// the resolved Markdown verbatim as this field. Validation requires a
    /// non-empty string under `ASPECT_PROMPT_MAX_BYTES` (64 KiB) to guard
    /// against accidental payload bloat.
    pub instructions: String,
    /// Tools the aspect agent is allowed to call (currently only `search`).
    pub tools: Vec<ToolName>,
    /// Explicit model provider selection; must satisfy `ResearchPolicy.model`.
    pub model_provider: String,
    /// Explicit search provider selection (exactly one when `tools` includes
    /// `search`); must satisfy `ResearchPolicy.search`.
    pub search_provider: Option<String>,
    /// Per-aspect resource and timeout limits.
    pub limits: AgentLimits,
}

/// Upper bound on the inline aspect-agent prompt size, in bytes.
///
/// 64 KiB is comfortably above realistic prompt assets (current MoeResearch prompts
/// are 1-10 KiB) but bounds the per-request payload so a buggy Layer 1
/// implementation cannot accidentally explode token usage by inlining a huge
/// Markdown file.
pub(crate) const ASPECT_PROMPT_MAX_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchTask {
    pub question: String,
    pub aspects: Vec<AspectRequest>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AspectResearchRequest {
    pub schema_version: String,
    pub request_id: String,
    pub task: AspectRequest,
    pub policy: ResearchPolicy,
    pub context: ResearchContext,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeepResearchRequest {
    pub schema_version: String,
    pub request_id: String,
    pub task: ResearchTask,
    pub limits: ResearchLimits,
    pub policy: ResearchPolicy,
    pub context: ResearchContext,
}

pub(crate) struct WorkflowValidationContext<'a> {
    pub budget_config: &'a BudgetConfig,
    pub supported_schema_versions: &'a [&'a str],
    pub supported_tool_name: &'a str,
}

impl AspectResearchRequest {
    pub(crate) fn validate_for_execution(&self, ctx: &WorkflowValidationContext<'_>) -> Result<()> {
        ensure_non_empty("schema_version", &self.schema_version)?;
        ensure_non_empty("request_id", &self.request_id)?;
        ensure_schema_version_supported(&self.schema_version, ctx.supported_schema_versions)?;

        self.policy.search.validate_for_search()?;
        validate_aspect_request(&self.task, &self.policy, ctx)?;
        Ok(())
    }
}

impl DeepResearchRequest {
    pub(crate) fn validate_for_execution(&self, ctx: &WorkflowValidationContext<'_>) -> Result<()> {
        ensure_non_empty("schema_version", &self.schema_version)?;
        ensure_non_empty("request_id", &self.request_id)?;
        ensure_non_empty("task.question", &self.task.question)?;
        ensure_schema_version_supported(&self.schema_version, ctx.supported_schema_versions)?;

        if self.task.aspects.is_empty() {
            return Err(Error::InvalidInput {
                message: "task.aspects must not be empty".to_owned(),
            });
        }

        let mut aspect_ids = BTreeSet::new();
        for aspect in &self.task.aspects {
            let aspect_id = aspect.id.as_str();
            if !aspect_id.is_empty() && !aspect_ids.insert(aspect_id) {
                return Err(Error::InvalidInput {
                    message: format!("task.aspects[].id must be unique: {aspect_id}"),
                });
            }
        }

        self.limits.validate_against_config(ctx.budget_config)?;

        if Limit::limited(self.task.aspects.len()).exceeds(self.limits.max_agents) {
            return Err(Error::BudgetExceeded {
                message: "aspect task count exceeds max_agents".to_owned(),
            });
        }

        self.policy.search.validate_for_search()?;

        for aspect in &self.task.aspects {
            validate_aspect_request(aspect, &self.policy, ctx)?;
            if aspect
                .limits
                .timeout_ms
                .exceeds(self.limits.total_timeout_ms)
            {
                return Err(Error::BudgetExceeded {
                    message: "aspect limits timeout exceeds research limits timeout".to_owned(),
                });
            }
        }

        Ok(())
    }
}

fn validate_aspect_request(
    aspect: &AspectRequest,
    policy: &ResearchPolicy,
    ctx: &WorkflowValidationContext<'_>,
) -> Result<()> {
    ensure_non_empty("task.id", &aspect.id)?;
    ensure_non_empty("task.name", &aspect.name)?;
    ensure_non_empty("task.question", &aspect.question)?;
    ensure_non_empty("task.instructions", &aspect.instructions)?;
    if aspect.instructions.len() > ASPECT_PROMPT_MAX_BYTES {
        return Err(Error::SchemaValidationFailed {
            message: format!("task.instructions exceeds {ASPECT_PROMPT_MAX_BYTES} bytes"),
        });
    }
    ensure_runtime_tools_allowed(&aspect.tools, ctx.supported_tool_name)?;
    validate_explicit_model_provider(aspect, &policy.model)?;
    validate_explicit_search_provider(aspect, &policy.search, ctx.supported_tool_name)?;
    aspect.limits.validate_against_config(ctx.budget_config)?;
    Ok(())
}

/// Validates that the request's `schema_version` is in the supported list.
///
/// # Errors
/// Returns `Error::UnsupportedSchemaVersion { version }` when the supplied
/// value is not in `supported`, so the public error code is the dedicated
/// `ToolErrorCode::UnsupportedSchemaVersion` instead of the generic
/// `SchemaValidationFailed`.
fn ensure_schema_version_supported(version: &str, supported: &[&str]) -> Result<()> {
    if supported.contains(&version) {
        return Ok(());
    }
    Err(Error::UnsupportedSchemaVersion {
        version: version.to_owned(),
    })
}

fn validate_explicit_model_provider(aspect: &AspectRequest, policy: &ModelPolicy) -> Result<()> {
    let provider = validate_explicit_provider_name(
        Some(aspect.model_provider.as_str()),
        "aspect must specify model_provider",
        "model_provider must be a non-empty provider name",
    )?;

    if !policy.allowed_providers.iter().any(|p| p == provider) {
        return Err(Error::ProviderUnavailable {
            provider: provider.to_owned(),
            message: "aspect model provider is not allowed by policy".to_owned(),
            retryable: false,
        });
    }

    Ok(())
}

fn validate_explicit_search_provider(
    aspect: &AspectRequest,
    policy: &SearchPolicy,
    supported_tool_name: &str,
) -> Result<()> {
    if !aspect
        .tools
        .iter()
        .any(|tool| tool.0 == supported_tool_name)
    {
        return Ok(());
    }

    let provider = validate_explicit_provider_name(
        aspect.search_provider.as_deref(),
        "search-enabled aspect must specify search_provider",
        "search_provider must be a non-empty provider name",
    )?;

    if !policy.allowed_providers.iter().any(|p| p == provider) {
        return Err(Error::ProviderUnavailable {
            provider: provider.to_owned(),
            message: "aspect search provider is not allowed by policy".to_owned(),
            retryable: false,
        });
    }

    Ok(())
}

fn validate_explicit_provider_name<'a>(
    provider: Option<&'a str>,
    missing_message: &str,
    invalid_message: &str,
) -> Result<&'a str> {
    let provider = provider.ok_or_else(|| Error::InvalidInput {
        message: missing_message.to_owned(),
    })?;

    let trimmed = provider.trim();
    if trimmed.is_empty() || trimmed != provider {
        return Err(Error::InvalidInput {
            message: invalid_message.to_owned(),
        });
    }

    Ok(provider)
}

fn ensure_runtime_tools_allowed(tools: &[ToolName], supported_tool_name: &str) -> Result<()> {
    if let Some(tool) = tools.iter().find(|tool| tool.0 != supported_tool_name) {
        return Err(Error::ToolPolicyDenied {
            message: format!("unsupported tool for aspect runtime: {}", tool.0),
        });
    }
    Ok(())
}

fn ensure_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::InvalidInput {
            message: format!("{field} must not be empty"),
        });
    }
    Ok(())
}
