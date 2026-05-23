use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::budget::{AgentBudget, ResearchBudget};
use super::config::BudgetConfig;
use super::limit::Limit;
use super::policy::{
    EvidencePolicy, EvidenceRequirement, ExecutionPolicy, ModelPolicy, ModelSelector, OutputPolicy,
    SearchPolicy, SearchSelector, ToolName,
};
use super::report::Evidence;

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ResearchContext {
    pub summary: String,
    pub known_facts: Vec<String>,
    pub excluded_assumptions: Vec<String>,
    pub prior_sources: Vec<Evidence>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct DeliverableSpec {
    pub kind: String,
    pub language: String,
    pub expected_sections: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ResearchConstraint {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct PromptAssets {
    pub aspect_agent_prompt_path: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AspectSpec {
    pub aspect_id: String,
    pub name: String,
    pub role: String,
    pub research_question: String,
    pub scope: Vec<String>,
    pub boundaries: Vec<String>,
    pub success_criteria: Vec<String>,
    pub prompt_assets: PromptAssets,
    pub required_evidence: EvidenceRequirement,
    pub allowed_tools: Vec<ToolName>,
    pub model_override: Option<ModelSelector>,
    pub search_override: Option<SearchSelector>,
    pub budget_override: Option<AgentBudget>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ResearchPlan {
    pub plan_id: String,
    pub user_question: String,
    pub deliverable: DeliverableSpec,
    pub constraints: Vec<ResearchConstraint>,
    pub aspects: Vec<AspectSpec>,
    pub budget: ResearchBudget,
    pub model_policy: ModelPolicy,
    pub search_policy: SearchPolicy,
    pub evidence_policy: EvidencePolicy,
    pub output_policy: OutputPolicy,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AspectResearchRequest {
    pub schema_version: String,
    pub request_id: String,
    pub aspect: AspectSpec,
    pub shared_context: ResearchContext,
    pub model_policy: ModelPolicy,
    pub search_policy: SearchPolicy,
    pub evidence_policy: EvidencePolicy,
    pub output_policy: OutputPolicy,
    pub budget: AgentBudget,
    pub execution_policy: ExecutionPolicy,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct DeepResearchRequest {
    pub schema_version: String,
    pub request_id: String,
    pub plan: ResearchPlan,
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
        ensure_non_empty("aspect.aspect_id", &self.aspect.aspect_id)?;
        ensure_non_empty("aspect.name", &self.aspect.name)?;
        ensure_non_empty("aspect.research_question", &self.aspect.research_question)?;

        if !ctx
            .supported_schema_versions
            .contains(&self.schema_version.as_str())
        {
            return Err(Error::SchemaValidationFailed {
                message: format!("unsupported schema version: {}", self.schema_version),
            });
        }

        ensure_non_empty(
            "aspect.prompt_assets.aspect_agent_prompt_path",
            &self.aspect.prompt_assets.aspect_agent_prompt_path,
        )?;
        self.search_policy.validate_for_search()?;
        self.budget.validate_against_config(ctx.budget_config)?;
        if let Some(timeout_ms) = self.execution_policy.timeout_ms
            && Limit::limited(timeout_ms).exceeds(self.budget.timeout_ms)
        {
            return Err(Error::BudgetExceeded {
                message: "execution timeout must not exceed agent budget timeout".to_owned(),
            });
        }
        ensure_runtime_tools_allowed(&self.aspect.allowed_tools, ctx.supported_tool_name)
    }
}

impl DeepResearchRequest {
    pub(crate) fn validate_for_execution(&self, ctx: &WorkflowValidationContext<'_>) -> Result<()> {
        ensure_non_empty("schema_version", &self.schema_version)?;
        ensure_non_empty("request_id", &self.request_id)?;
        ensure_non_empty("plan.plan_id", &self.plan.plan_id)?;

        if !ctx
            .supported_schema_versions
            .contains(&self.schema_version.as_str())
        {
            return Err(Error::SchemaValidationFailed {
                message: format!("unsupported schema version: {}", self.schema_version),
            });
        }

        if self.plan.aspects.is_empty() {
            return Err(Error::InvalidInput {
                message: "plan.aspects must not be empty".to_owned(),
            });
        }

        self.plan
            .budget
            .validate_against_config(ctx.budget_config)?;

        if Limit::limited(self.plan.aspects.len()).exceeds(self.plan.budget.max_agents) {
            return Err(Error::BudgetExceeded {
                message: "plan aspect count exceeds max_agents".to_owned(),
            });
        }

        if let Some(timeout_ms) = self.execution_policy.timeout_ms
            && Limit::limited(timeout_ms).exceeds(self.plan.budget.total_timeout_ms)
        {
            return Err(Error::BudgetExceeded {
                message: "execution timeout must not exceed research budget timeout".to_owned(),
            });
        }

        self.plan.search_policy.validate_for_search()?;

        for aspect in &self.plan.aspects {
            ensure_non_empty("aspect.aspect_id", &aspect.aspect_id)?;
            ensure_non_empty("aspect.name", &aspect.name)?;
            ensure_non_empty("aspect.research_question", &aspect.research_question)?;
            ensure_non_empty(
                "aspect.prompt_assets.aspect_agent_prompt_path",
                &aspect.prompt_assets.aspect_agent_prompt_path,
            )?;
            ensure_runtime_tools_allowed(&aspect.allowed_tools, ctx.supported_tool_name)?;

            if let Some(model_override) = &aspect.model_override
                && !self
                    .plan
                    .model_policy
                    .allowed_providers
                    .contains(&model_override.provider)
            {
                return Err(Error::ProviderUnavailable {
                    provider: model_override.provider.clone(),
                    message: "aspect model override is not allowed by policy".to_owned(),
                });
            }

            if let Some(search_override) = &aspect.search_override
                && let Some(provider) = search_override.providers.iter().find(|provider| {
                    !self
                        .plan
                        .search_policy
                        .allowed_providers
                        .contains(*provider)
                })
            {
                return Err(Error::ProviderUnavailable {
                    provider: provider.clone(),
                    message: "aspect search override is not allowed by policy".to_owned(),
                });
            }

            let aspect_budget = aspect
                .budget_override
                .as_ref()
                .map_or_else(AgentBudget::default, std::clone::Clone::clone);
            aspect_budget.validate_against_config(ctx.budget_config)?;
            let aspect_timeout = self
                .execution_policy
                .timeout_ms
                .map_or(aspect_budget.timeout_ms, Limit::limited);
            if aspect_timeout.exceeds(self.plan.budget.total_timeout_ms) {
                return Err(Error::BudgetExceeded {
                    message: "aspect budget timeout exceeds research budget timeout".to_owned(),
                });
            }
        }

        Ok(())
    }
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
