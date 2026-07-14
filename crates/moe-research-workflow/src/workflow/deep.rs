use std::collections::BTreeSet;
use std::sync::Arc;

use futures::{StreamExt, stream};
use uuid::Uuid;

use crate::budget::BudgetConfig;
use crate::error_log_safe::error_message_for_log;
use crate::report::{
    AspectFailure, CoverageSummary, DeepResearchResult, FailureDiagnostic, FailureStage,
};
use crate::research::{
    DeepResearchRequest, EffectiveAspectPlan, EffectiveResearchPlan, SUPPORTED_SCHEMA_VERSIONS,
    WorkflowValidationContext,
};
use crate::runtime::{
    AgentRuntime, AgentRuntimeFailure, AgentRuntimeOutput, ResearchBudgetGuard,
    SUPPORTED_ASPECT_TOOLS,
};
use moe_research_error::{Error, Result};
use moe_research_model::ModelService;
use moe_research_search::SearchService;
use moe_research_web_fetch::WebFetchService;

use super::aggregation::{
    DeepResearchRun, aspect_failure, confidence_summary, order_failures_by_request,
    record_aspect_result,
};

#[derive(Debug)]
pub struct DeepResearchFailure {
    pub error: Error,
    pub diagnostic: FailureDiagnostic,
    pub failed_aspects: Vec<AspectFailure>,
}

impl DeepResearchFailure {
    pub(super) fn top_level(error: Error) -> Box<Self> {
        Box::new(Self {
            error,
            diagnostic: FailureDiagnostic::new(FailureStage::RequestValidation, None, None),
            failed_aspects: Vec::new(),
        })
    }

    pub(super) fn with_aspects(
        error: Error,
        diagnostic: FailureDiagnostic,
        failed_aspects: Vec<AspectFailure>,
    ) -> Box<Self> {
        Box::new(Self {
            error,
            diagnostic,
            failed_aspects,
        })
    }
}

/// Runs a Layer 1 deep-research plan.
///
/// The runtime limits are the stricter value for each research-limit dimension:
/// operator config is the hard ceiling, and the Layer 1 request can only narrow
/// a single run. `Limit::Unlimited` means "this layer adds no cap", not
/// "ignore the other layer's finite cap". Finalization still honors
/// `policy.execution.fail_fast` during execution and `allow_partial_results`
/// when shaping the final result.
pub async fn deep_research(
    request: DeepResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    web_fetch_service: &WebFetchService,
    budget_config: &BudgetConfig,
) -> std::result::Result<DeepResearchResult, Box<DeepResearchFailure>> {
    let plan = request
        .normalize_for_execution(&WorkflowValidationContext {
            budget_config,
            supported_schema_versions: SUPPORTED_SCHEMA_VERSIONS,
            supported_tool_names: SUPPORTED_ASPECT_TOOLS,
        })
        .map_err(DeepResearchFailure::top_level)?;
    if plan.limits != request.limits {
        tracing::info!(
            event = "effective_limits_applied",
            request_id = %request.request_id,
            requested_limits = ?request.limits,
            effective_limits = ?plan.limits,
            "request limits tightened by operator config ceilings"
        );
    }

    let run_id = Uuid::new_v4().to_string();
    let request_id = plan.request_id.clone();
    let requested_aspects = plan.task.aspects.len();
    tracing::info!(
        request_id = %request_id,
        run_id = %run_id,
        requested_aspects,
        "deep research started"
    );

    let research_budget = ResearchBudgetGuard::new(plan.limits.clone());
    let mut run = execute_aspects(
        &plan,
        model_service,
        search_service,
        web_fetch_service,
        research_budget.clone(),
    )
    .await;
    run.budget_usage = match research_budget.snapshot() {
        Ok(usage) => usage,
        Err(error) => {
            return Err(DeepResearchFailure::with_aspects(
                error,
                FailureDiagnostic::new(FailureStage::ResearchBudget, None, None),
                order_failures_by_request(&plan, run.failures),
            ));
        }
    };
    if let Err(error) = plan.limits.ensure_usage_within(&run.budget_usage) {
        let diagnostic = FailureDiagnostic::new(FailureStage::ResearchBudget, None, None);
        let failures_before = run.failures.len();
        let mut accounted = run.completed.iter().cloned().collect::<BTreeSet<_>>();
        accounted.extend(run.failures.iter().map(|failure| failure.aspect_id.clone()));
        for aspect in &plan.task.aspects {
            let aspect_id = &aspect.id;
            if accounted.insert(aspect_id.clone()) {
                run.failures
                    .push(aspect_failure(aspect_id, &error, diagnostic.clone()));
            }
        }
        let terminal_failures_added = run.failures.len() - failures_before;
        let has_partial_payload = !run.completed.is_empty() || !run.evidence_by_id.is_empty();
        let return_partial = plan.policy.execution.allow_partial_results
            && has_partial_payload
            && (!run.failures.is_empty() || terminal_failures_added > 0);
        tracing::warn!(
            request_id = %request_id,
            run_id = %run_id,
            requested_aspects,
            agents_started = run.budget_usage.agents_started,
            completed_aspects = run.completed.len(),
            failed_aspects = run.failures.len(),
            terminal_failures_added,
            model_calls_used = run.budget_usage.model_calls_used,
            search_calls_used = run.budget_usage.search_calls_used,
            elapsed_ms = run.budget_usage.elapsed_ms,
            error_code = error.code().as_str(),
            error_message = %error_message_for_log(&error),
            retryable = error.retryable(),
            status = if return_partial { "partial" } else { "failed" },
            "deep research budget check failed"
        );
        if !return_partial {
            return Err(DeepResearchFailure::with_aspects(
                error,
                diagnostic,
                order_failures_by_request(&plan, run.failures),
            ));
        }
    }

    let result = finalize_deep_result(&plan, run, run_id.clone());
    match &result {
        Ok(result) => tracing::info!(
            request_id = %request_id,
            run_id = %run_id,
            requested_aspects,
            completed_aspects = result.completed_aspects.len(),
            failed_aspects = result.failed_aspects.len(),
            evidence_count = result.coverage_summary.evidence_count,
            elapsed_ms = result.budget_usage.elapsed_ms,
            status = if result.failed_aspects.is_empty() { "ok" } else { "partial" },
            "deep research completed"
        ),
        Err(failure) => tracing::warn!(
            request_id = %request_id,
            run_id = %run_id,
            requested_aspects,
            error_code = failure.error.code().as_str(),
            error_message = %error_message_for_log(&failure.error),
            retryable = failure.error.retryable(),
            failed_aspects = failure.failed_aspects.len(),
            status = "failed",
            "deep research failed"
        ),
    }
    result
}

/// Executes every aspect with one shared research-level guard.
///
/// The request passed here already carries the effective merged limits. Its
/// concurrency cap controls scheduling, while the shared `ResearchBudgetGuard`
/// reserves global model/search/token capacity before provider dispatch.
pub(super) async fn execute_aspects(
    request: &EffectiveResearchPlan,
    model_service: &ModelService,
    search_service: &SearchService,
    web_fetch_service: &WebFetchService,
    research_budget: Arc<ResearchBudgetGuard>,
) -> DeepResearchRun {
    let mut run = DeepResearchRun::new();
    let mut results = stream::iter(aspect_requests(request).into_iter().map(|aspect_request| {
        let research_budget = research_budget.clone();
        async move {
            research_budget.record_agent_started();
            let aspect_id = aspect_request.task.id.clone();
            let result = run_aspect_runtime(
                aspect_request,
                model_service,
                search_service,
                web_fetch_service,
                research_budget,
            )
            .await;
            (aspect_id, result)
        }
    }))
    .buffer_unordered(
        request
            .limits
            .max_concurrent_agents
            .as_concurrency(request.task.aspects.len()),
    );

    while let Some((aspect_id, result)) = results.next().await {
        record_aspect_result(
            &mut run,
            &aspect_id,
            result,
            request.policy.execution.allow_partial_results,
        );
        if request.policy.execution.fail_fast && !run.failures.is_empty() {
            break;
        }
    }

    run
}

pub(super) async fn run_aspect_runtime(
    request: EffectiveAspectPlan,
    model_service: &ModelService,
    search_service: &SearchService,
    web_fetch_service: &WebFetchService,
    research_budget: Arc<ResearchBudgetGuard>,
) -> Result<AgentRuntimeOutput, AgentRuntimeFailure> {
    AgentRuntime::new(
        model_service,
        search_service,
        web_fetch_service,
        &request,
        research_budget,
    )
    .run()
    .await
}

pub(super) fn aspect_requests(request: &EffectiveResearchPlan) -> Vec<EffectiveAspectPlan> {
    request
        .task
        .aspects
        .iter()
        .cloned()
        .map(|task| EffectiveAspectPlan {
            schema_version: request.schema_version.clone(),
            request_id: request.request_id.clone(),
            task,
            policy: request.policy.clone(),
            context: request.context.clone(),
        })
        .collect()
}

/// Finalizes a `DeepResearchRun` into either a `DeepResearchResult` or a
/// terminal error, honoring the `allow_partial_results` execution policy.
fn finalize_deep_result(
    request: &EffectiveResearchPlan,
    run: DeepResearchRun,
    run_id: String,
) -> std::result::Result<DeepResearchResult, Box<DeepResearchFailure>> {
    if run.completed.is_empty()
        && (!request.policy.execution.allow_partial_results || run.evidence_by_id.is_empty())
    {
        return Err(DeepResearchFailure::with_aspects(
            Error::PartialResult {
                message: "all aspects failed".to_owned(),
            },
            FailureDiagnostic::new(FailureStage::ResultAggregation, None, None),
            order_failures_by_request(request, run.failures),
        ));
    }

    if !run.failures.is_empty() && !request.policy.execution.allow_partial_results {
        return Err(DeepResearchFailure::with_aspects(
            Error::PartialResult {
                message: "deep research produced partial results".to_owned(),
            },
            FailureDiagnostic::new(FailureStage::ResultAggregation, None, None),
            order_failures_by_request(request, run.failures),
        ));
    }

    Ok(deep_result(request, run, run_id))
}

/// Builds the public `DeepResearchResult` from the request shape and the
/// accumulated `DeepResearchRun` state.
fn deep_result(
    request: &EffectiveResearchPlan,
    run: DeepResearchRun,
    run_id: String,
) -> DeepResearchResult {
    let failed_aspects = order_failures_by_request(request, run.failures);
    let evidence_index = run.evidence_by_id.into_values().collect::<Vec<_>>();
    let coverage_summary = CoverageSummary {
        requested_aspects: request.task.aspects.len(),
        completed_aspects: run.completed.len(),
        failed_aspects: failed_aspects.len(),
        evidence_count: evidence_index.len(),
    };
    DeepResearchResult {
        run_id,
        completed_aspects: run.completed,
        failed_aspects,
        confidence_summary: confidence_summary(&run.aspect_reports),
        aspect_reports: run.aspect_reports,
        evidence_index,
        open_questions: run.open_questions,
        coverage_summary,
        budget_usage: run.budget_usage,
    }
}
