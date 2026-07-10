use crate::agent_loop::{AgentRuntimeFailure, AgentRuntimeOutput};
use crate::budget::BudgetConfig;
use crate::report::{AgentBudgetUsage, AspectResearchResult, TokenUsage};
use crate::research::{
    AspectResearchRequest, SUPPORTED_SCHEMA_VERSIONS, WorkflowValidationContext,
    effective_research_limits,
};
use crate::runtime::ResearchBudgetGuard;
use crate::runtime::SEARCH_TOOL_NAME;
use moe_research_error::Error;
use moe_research_model::ModelService;
use moe_research_search::SearchService;

use super::deep::run_aspect_runtime;

/// Public output from a standalone `aspect_research` run.
#[derive(Debug)]
pub struct AspectResearchOutput {
    pub result: AspectResearchResult,
    pub budget_usage: AgentBudgetUsage,
    pub token_usage: Option<TokenUsage>,
}

impl AspectResearchOutput {
    pub(super) fn from_runtime(output: AgentRuntimeOutput) -> Self {
        Self {
            result: output.result,
            budget_usage: output.budget_usage,
            token_usage: output.token_usage,
        }
    }
}

/// Public failure from a standalone `aspect_research` run.
#[derive(Debug)]
pub struct AspectResearchFailure {
    pub error: Error,
    pub partial_output: Option<AspectResearchOutput>,
}

impl AspectResearchFailure {
    fn top_level(error: Error) -> Box<Self> {
        Box::new(Self {
            error,
            partial_output: None,
        })
    }

    fn from_runtime(failure: AgentRuntimeFailure) -> Box<Self> {
        Box::new(Self {
            error: failure.error,
            partial_output: failure
                .partial_output
                .map(AspectResearchOutput::from_runtime),
        })
    }
}

/// Runs one aspect agent.
///
/// `AspectResearchRequest` has no request-level [`crate::budget::ResearchLimits`], so the
/// standalone tool inherits the operator `limits.research` caps from config.
/// The request task still supplies the per-agent turn/tool/search limits.
pub async fn aspect_research(
    request: AspectResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    budget_config: &BudgetConfig,
) -> std::result::Result<AspectResearchOutput, Box<AspectResearchFailure>> {
    let plan = request
        .normalize_for_execution(&WorkflowValidationContext {
            budget_config,
            supported_schema_versions: SUPPORTED_SCHEMA_VERSIONS,
            supported_tool_name: SEARCH_TOOL_NAME,
        })
        .map_err(AspectResearchFailure::top_level)?;
    let allow_partial_results = plan.policy.execution.allow_partial_results;
    let research_budget =
        ResearchBudgetGuard::new(effective_research_limits(&budget_config.research, None));
    research_budget.record_agent_started();
    run_aspect_runtime(plan, model_service, search_service, research_budget)
        .await
        .map(AspectResearchOutput::from_runtime)
        .map_err(|failure| {
            let mut failure = AspectResearchFailure::from_runtime(failure);
            if !allow_partial_results {
                failure.partial_output = None;
            }
            failure
        })
}
