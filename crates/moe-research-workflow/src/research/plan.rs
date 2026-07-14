use std::collections::BTreeSet;

use moe_research_error::{Error, Result};

use super::super::budget::{AgentLimits, BudgetConfig, ResearchLimits};
use super::super::limit::Limit;
use super::super::policy::{ModelPolicy, SearchPolicy, ToolName};
use super::prompt::ASPECT_PROMPT_MAX_BYTES;
use super::request::{
    AspectRequest, AspectResearchRequest, DeepResearchRequest, ResearchContext, ResearchPolicy,
    ResearchTask, RuntimeCapabilitiesRequest, SUPPORTED_SCHEMA_VERSIONS,
};

/// Internal deep-research plan after request/config normalization.
///
/// This mirrors the public v0.2 request shape, but all research and per-aspect
/// limits have already been merged with operator config. Runtime orchestration
/// consumes this type instead of mutating or re-validating raw MCP payloads.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EffectiveResearchPlan {
    pub(crate) schema_version: String,
    pub(crate) request_id: String,
    pub(crate) task: ResearchTask,
    pub(crate) limits: ResearchLimits,
    pub(crate) policy: ResearchPolicy,
    pub(crate) context: ResearchContext,
}

/// Internal single-aspect plan after request/config normalization.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EffectiveAspectPlan {
    pub(crate) schema_version: String,
    pub(crate) request_id: String,
    pub(crate) task: AspectRequest,
    pub(crate) policy: ResearchPolicy,
    pub(crate) context: ResearchContext,
}

pub(crate) struct WorkflowValidationContext<'a> {
    pub budget_config: &'a BudgetConfig,
    pub supported_schema_versions: &'a [&'a str],
    pub supported_tool_names: &'a [&'a str],
}

impl RuntimeCapabilitiesRequest {
    /// Validates the capabilities request against the shared public schema version.
    ///
    /// # Errors
    /// Returns `InvalidInput` for empty identity fields and
    /// `UnsupportedSchemaVersion` for unsupported schema versions.
    pub fn validate(&self) -> Result<()> {
        ensure_non_empty("schema_version", &self.schema_version)?;
        ensure_non_empty("request_id", &self.request_id)?;
        ensure_schema_version_supported(&self.schema_version, SUPPORTED_SCHEMA_VERSIONS)
    }
}

impl AspectResearchRequest {
    pub(crate) fn normalize_for_execution(
        &self,
        ctx: &WorkflowValidationContext<'_>,
    ) -> Result<EffectiveAspectPlan> {
        ensure_non_empty("schema_version", &self.schema_version)?;
        ensure_non_empty("request_id", &self.request_id)?;
        ensure_schema_version_supported(&self.schema_version, ctx.supported_schema_versions)?;

        if self.task.tools.iter().any(|tool| tool.0 == "search") {
            self.policy.search.validate_for_search()?;
        }
        Ok(EffectiveAspectPlan {
            schema_version: self.schema_version.clone(),
            request_id: self.request_id.clone(),
            task: normalize_aspect_request(&self.task, &self.policy, ctx)?,
            policy: self.policy.clone(),
            context: self.context.clone(),
        })
    }
}

impl DeepResearchRequest {
    pub(crate) fn normalize_for_execution(
        &self,
        ctx: &WorkflowValidationContext<'_>,
    ) -> Result<EffectiveResearchPlan> {
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

        let limits = effective_research_limits(&ctx.budget_config.research, Some(&self.limits));
        limits.validate_against_config(ctx.budget_config)?;

        if Limit::limited(self.task.aspects.len()).exceeds(limits.max_agents) {
            return Err(Error::BudgetExceeded {
                message: "aspect task count exceeds max_agents".to_owned(),
            });
        }

        if self
            .task
            .aspects
            .iter()
            .any(|aspect| aspect.tools.iter().any(|tool| tool.0 == "search"))
        {
            self.policy.search.validate_for_search()?;
        }

        let mut aspects = Vec::with_capacity(self.task.aspects.len());
        for aspect in &self.task.aspects {
            let aspect = normalize_aspect_request(aspect, &self.policy, ctx)?;
            if aspect.limits.timeout_ms.exceeds(limits.total_timeout_ms) {
                return Err(Error::BudgetExceeded {
                    message: "aspect limits timeout exceeds research limits timeout".to_owned(),
                });
            }
            aspects.push(aspect);
        }

        Ok(EffectiveResearchPlan {
            schema_version: self.schema_version.clone(),
            request_id: self.request_id.clone(),
            task: ResearchTask {
                question: self.task.question.clone(),
                aspects,
            },
            limits,
            policy: self.policy.clone(),
            context: self.context.clone(),
        })
    }
}

fn normalize_aspect_request(
    aspect: &AspectRequest,
    policy: &ResearchPolicy,
    ctx: &WorkflowValidationContext<'_>,
) -> Result<AspectRequest> {
    ensure_non_empty("task.id", &aspect.id)?;
    ensure_non_empty("task.name", &aspect.name)?;
    ensure_non_empty("task.question", &aspect.question)?;
    ensure_non_empty("task.instructions", &aspect.instructions)?;
    if aspect.instructions.len() > ASPECT_PROMPT_MAX_BYTES {
        return Err(Error::SchemaValidationFailed {
            message: format!("task.instructions exceeds {ASPECT_PROMPT_MAX_BYTES} bytes"),
        });
    }
    ensure_runtime_tools_allowed(&aspect.tools, ctx.supported_tool_names)?;
    validate_explicit_model_provider(aspect, &policy.model)?;
    validate_explicit_search_provider(aspect, &policy.search, "search")?;

    let mut normalized = aspect.clone();
    normalized.limits = effective_agent_limits(&ctx.budget_config.per_agent, &aspect.limits);
    normalized
        .limits
        .validate_against_config(ctx.budget_config)?;
    Ok(normalized)
}

/// Merges operator config and optional Layer 1 request limits (stricter-wins).
///
/// Each field chooses the stricter limit. `Unlimited` means the corresponding
/// layer does not constrain that dimension, so a finite limit from the other
/// layer wins. If both layers are unlimited, the effective field remains
/// unlimited; no hidden hard cap is introduced here.
#[must_use]
pub fn effective_research_limits(
    configured: &ResearchLimits,
    requested: Option<&ResearchLimits>,
) -> ResearchLimits {
    let Some(requested) = requested else {
        return configured.clone();
    };

    ResearchLimits {
        max_agents: stricter_limit(configured.max_agents, requested.max_agents),
        max_concurrent_agents: stricter_limit(
            configured.max_concurrent_agents,
            requested.max_concurrent_agents,
        ),
        max_total_model_calls: stricter_limit(
            configured.max_total_model_calls,
            requested.max_total_model_calls,
        ),
        max_total_search_calls: stricter_limit(
            configured.max_total_search_calls,
            requested.max_total_search_calls,
        ),
        total_timeout_ms: stricter_limit(configured.total_timeout_ms, requested.total_timeout_ms),
        max_tokens: stricter_limit(configured.max_tokens, requested.max_tokens),
    }
}

fn effective_agent_limits(configured: &AgentLimits, requested: &AgentLimits) -> AgentLimits {
    AgentLimits {
        max_turns: stricter_limit(configured.max_turns, requested.max_turns),
        max_tool_calls: stricter_limit(configured.max_tool_calls, requested.max_tool_calls),
        max_search_calls: stricter_limit(configured.max_search_calls, requested.max_search_calls),
        timeout_ms: stricter_limit(configured.timeout_ms, requested.timeout_ms),
    }
}

fn stricter_limit<T>(configured: Limit<T>, requested: Limit<T>) -> Limit<T>
where
    T: Copy + Ord,
{
    match (configured, requested) {
        (Limit::Unlimited, Limit::Unlimited) => Limit::Unlimited,
        (Limit::Unlimited, Limit::Limited(value)) | (Limit::Limited(value), Limit::Unlimited) => {
            Limit::Limited(value)
        }
        (Limit::Limited(configured), Limit::Limited(requested)) => {
            Limit::Limited(configured.min(requested))
        }
    }
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

fn ensure_runtime_tools_allowed(tools: &[ToolName], supported_tool_names: &[&str]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for tool in tools {
        if !supported_tool_names.contains(&tool.0.as_str()) {
            return Err(Error::ToolPolicyDenied {
                message: format!("unsupported tool for aspect runtime: {}", tool.0),
                public: false,
            });
        }
        if !seen.insert(tool.0.as_str()) {
            return Err(Error::ToolPolicyDenied {
                message: format!("duplicate tool for aspect runtime: {}", tool.0),
                public: false,
            });
        }
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
