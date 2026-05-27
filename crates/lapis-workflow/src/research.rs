use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use lapis_error::{Error, Result};

use super::budget::{AgentBudget, BudgetConfig, ResearchBudget};
use super::limit::Limit;
use super::policy::{
    EvidencePolicy, ExecutionPolicy, ModelPolicy, OutputPolicy, SearchPolicy, ToolName,
};
use super::report::Evidence;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
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

/// Specification of a single research aspect.
///
/// Aspects are the unit of parallelism for deep research: each aspect runs
/// inside its own agent loop with its own budget, tool allowlist, and
/// provider selection. Layer 1 (the Claude Code skill) constructs one
/// `AspectSpec` per dimension of the user's question.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AspectSpec {
    /// Stable identifier used by the orchestrator to track per-aspect state
    /// (budgets, evidence namespacing, failure records).
    pub aspect_id: String,
    /// Human-readable aspect name surfaced to the model and final report.
    pub name: String,
    /// Short role description (e.g. "competitive landscape analyst") used in
    /// the aspect agent's system prompt.
    pub role: String,
    /// Concrete research question this aspect must answer.
    pub research_question: String,
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
    pub aspect_agent_prompt: String,
    /// Tools the aspect agent is allowed to call (currently only `search`).
    pub allowed_tools: Vec<ToolName>,
    /// Explicit model provider selection; must satisfy `ModelPolicy`.
    pub model_provider: Option<String>,
    /// Explicit search provider selection (exactly one); must satisfy
    /// `SearchPolicy`.
    pub search_provider: Option<String>,
}

/// Upper bound on the inline aspect-agent prompt size, in bytes.
///
/// 64 KiB is comfortably above realistic prompt assets (current Lapis prompts
/// are 1-10 KiB) but bounds the per-request payload so a buggy Layer 1
/// implementation cannot accidentally explode token usage by inlining a huge
/// Markdown file.
pub(crate) const ASPECT_PROMPT_MAX_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AspectResearchTask {
    pub aspect: AspectSpec,
    pub budget: AgentBudget,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AspectResearchRequest {
    pub schema_version: String,
    pub request_id: String,
    pub task: AspectResearchTask,
    pub shared_context: ResearchContext,
    pub model_policy: ModelPolicy,
    pub search_policy: SearchPolicy,
    pub evidence_policy: EvidencePolicy,
    pub output_policy: OutputPolicy,
    pub execution_policy: ExecutionPolicy,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct DeepResearchRequest {
    pub schema_version: String,
    pub request_id: String,
    pub user_question: String,
    pub aspect_tasks: Vec<AspectResearchTask>,
    pub budget: ResearchBudget,
    pub model_policy: ModelPolicy,
    pub search_policy: SearchPolicy,
    pub evidence_policy: EvidencePolicy,
    pub output_policy: OutputPolicy,
    pub shared_context: ResearchContext,
    pub execution_policy: ExecutionPolicy,
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

        self.search_policy.validate_for_search()?;

        let parent_timeout = self.task.budget.timeout_ms;
        validate_aspect_task(
            &self.task,
            &self.model_policy,
            &self.search_policy,
            &self.execution_policy,
            parent_timeout,
            ctx,
            "execution timeout must not exceed agent budget timeout",
        )
    }
}

impl DeepResearchRequest {
    pub(crate) fn validate_for_execution(&self, ctx: &WorkflowValidationContext<'_>) -> Result<()> {
        ensure_non_empty("schema_version", &self.schema_version)?;
        ensure_non_empty("request_id", &self.request_id)?;
        ensure_schema_version_supported(&self.schema_version, ctx.supported_schema_versions)?;

        if self.aspect_tasks.is_empty() {
            return Err(Error::InvalidInput {
                message: "aspect_tasks must not be empty".to_owned(),
            });
        }

        let mut aspect_ids = BTreeSet::new();
        for task in &self.aspect_tasks {
            let aspect_id = task.aspect.aspect_id.as_str();
            if !aspect_id.is_empty() && !aspect_ids.insert(aspect_id) {
                return Err(Error::InvalidInput {
                    message: format!("aspect.aspect_id must be unique: {aspect_id}"),
                });
            }
        }

        self.budget.validate_against_config(ctx.budget_config)?;

        if Limit::limited(self.aspect_tasks.len()).exceeds(self.budget.max_agents) {
            return Err(Error::BudgetExceeded {
                message: "aspect task count exceeds max_agents".to_owned(),
            });
        }

        if self
            .execution_policy
            .timeout_ms
            .exceeds(self.budget.total_timeout_ms)
        {
            return Err(Error::BudgetExceeded {
                message: "execution timeout must not exceed research budget timeout".to_owned(),
            });
        }

        self.search_policy.validate_for_search()?;

        for task in &self.aspect_tasks {
            validate_aspect_task(
                task,
                &self.model_policy,
                &self.search_policy,
                &self.execution_policy,
                self.budget.total_timeout_ms,
                ctx,
                "aspect budget timeout exceeds research budget timeout",
            )?;
        }

        Ok(())
    }
}

fn validate_aspect_task(
    task: &AspectResearchTask,
    model_policy: &ModelPolicy,
    search_policy: &SearchPolicy,
    execution_policy: &ExecutionPolicy,
    parent_timeout: Limit<u64>,
    ctx: &WorkflowValidationContext<'_>,
    timeout_violation_message: &str,
) -> Result<()> {
    let aspect = &task.aspect;
    ensure_non_empty("aspect.aspect_id", &aspect.aspect_id)?;
    ensure_non_empty("aspect.name", &aspect.name)?;
    ensure_non_empty("aspect.research_question", &aspect.research_question)?;
    ensure_non_empty("aspect.aspect_agent_prompt", &aspect.aspect_agent_prompt)?;
    if aspect.aspect_agent_prompt.len() > ASPECT_PROMPT_MAX_BYTES {
        return Err(Error::SchemaValidationFailed {
            message: format!("aspect.aspect_agent_prompt exceeds {ASPECT_PROMPT_MAX_BYTES} bytes"),
        });
    }
    ensure_runtime_tools_allowed(&aspect.allowed_tools, ctx.supported_tool_name)?;
    validate_explicit_model_provider(aspect, model_policy)?;
    validate_explicit_search_provider(aspect, search_policy, ctx.supported_tool_name)?;
    task.budget.validate_against_config(ctx.budget_config)?;

    // Treat an unset `execution_policy.timeout_ms` as "inherit the
    // parent budget", so an `Unlimited` policy does not silently widen
    // a finite parent budget.
    let effective_timeout = match execution_policy.timeout_ms {
        Limit::Unlimited => parent_timeout,
        Limit::Limited(_) => execution_policy.timeout_ms,
    };
    if effective_timeout.exceeds(parent_timeout) {
        return Err(Error::BudgetExceeded {
            message: timeout_violation_message.to_owned(),
        });
    }
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

fn validate_explicit_model_provider(aspect: &AspectSpec, policy: &ModelPolicy) -> Result<()> {
    let provider = validate_explicit_provider_name(
        aspect.model_provider.as_deref(),
        "aspect must specify model_provider",
        "model_provider must be a non-empty provider name",
    )?;

    if !policy.allowed_providers.iter().any(|p| p == provider) {
        return Err(Error::ProviderUnavailable {
            provider: provider.to_owned(),
            message: "aspect model provider is not allowed by policy".to_owned(),
        });
    }

    Ok(())
}

fn validate_explicit_search_provider(
    aspect: &AspectSpec,
    policy: &SearchPolicy,
    supported_tool_name: &str,
) -> Result<()> {
    if !aspect
        .allowed_tools
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
