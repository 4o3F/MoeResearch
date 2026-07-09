//! Workflow orchestration for standalone aspect and multi-aspect deep research.
//!
//! This module owns the execution boundary: validate incoming requests, derive
//! the effective research limits from operator config and request limits, run
//! aspect agents, then aggregate successes and failures into the public result.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use futures::{StreamExt, stream};
use uuid::Uuid;

use crate::agent_loop::{AgentRuntime, AgentRuntimeFailure, AgentRuntimeOutput};
use crate::budget::BudgetConfig;
use crate::log_safe::error_message_for_log;
use crate::report::{
    AspectFailure, AspectReport, AspectResearchResult, Confidence, ConfidenceSummary,
    CoverageSummary, DeepResearchResult, Evidence, OpenQuestion, ResearchBudgetUsage,
};
use crate::research::{
    AspectResearchRequest, DeepResearchRequest, EffectiveAspectPlan, EffectiveResearchPlan,
    SUPPORTED_SCHEMA_VERSIONS, WorkflowValidationContext, effective_research_limits,
};
use crate::runtime_budget::ResearchBudgetGuard;
use crate::tool_policy::SEARCH_TOOL_NAME;
use moe_research_error::{Error, Result};
use moe_research_model::ModelService;
use moe_research_search::SearchService;

/// Runs one aspect agent.
///
/// `AspectResearchRequest` has no request-level [`ResearchLimits`], so the
/// standalone tool inherits the operator `limits.research` caps from config.
/// The request task still supplies the per-agent turn/tool/search limits.
pub async fn aspect_research(
    request: AspectResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    budget_config: &BudgetConfig,
) -> Result<AgentRuntimeOutput, AgentRuntimeFailure> {
    let plan = request
        .normalize_for_execution(&WorkflowValidationContext {
            budget_config,
            supported_schema_versions: SUPPORTED_SCHEMA_VERSIONS,
            supported_tool_name: SEARCH_TOOL_NAME,
        })
        .map_err(|error| AgentRuntimeFailure {
            error,
            partial_output: None,
        })?;
    let research_budget =
        ResearchBudgetGuard::new(effective_research_limits(&budget_config.research, None));
    research_budget.record_agent_started();
    run_aspect_runtime(plan, model_service, search_service, research_budget).await
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
    budget_config: &BudgetConfig,
) -> std::result::Result<DeepResearchResult, Box<DeepResearchFailure>> {
    let plan = request
        .normalize_for_execution(&WorkflowValidationContext {
            budget_config,
            supported_schema_versions: SUPPORTED_SCHEMA_VERSIONS,
            supported_tool_name: SEARCH_TOOL_NAME,
        })
        .map_err(DeepResearchFailure::top_level)?;
    if plan.limits != request.limits {
        tracing::debug!(
            request_id = %request.request_id,
            requested_limits = ?request.limits,
            effective_limits = ?plan.limits,
            "deep research limits constrained by effective limits"
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
        research_budget.clone(),
    )
    .await;
    run.budget_usage = match research_budget.snapshot() {
        Ok(usage) => usage,
        Err(error) => {
            return Err(DeepResearchFailure::with_aspects(
                error,
                order_failures_by_request(&plan, run.failures),
            ));
        }
    };
    if let Err(error) = plan.limits.ensure_usage_within(&run.budget_usage) {
        let failures_before = run.failures.len();
        let mut accounted = run.completed.iter().cloned().collect::<BTreeSet<_>>();
        accounted.extend(run.failures.iter().map(|failure| failure.aspect_id.clone()));
        for aspect in &plan.task.aspects {
            let aspect_id = &aspect.id;
            if accounted.insert(aspect_id.clone()) {
                run.failures.push(aspect_failure(aspect_id, &error));
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

#[derive(Debug)]
pub struct DeepResearchFailure {
    pub error: Error,
    pub failed_aspects: Vec<AspectFailure>,
}

impl DeepResearchFailure {
    fn top_level(error: Error) -> Box<Self> {
        Box::new(Self {
            error,
            failed_aspects: Vec::new(),
        })
    }

    fn with_aspects(error: Error, failed_aspects: Vec<AspectFailure>) -> Box<Self> {
        Box::new(Self {
            error,
            failed_aspects,
        })
    }
}

struct DeepResearchRun {
    completed: Vec<String>,
    failures: Vec<AspectFailure>,
    aspect_reports: Vec<AspectReport>,
    evidence_by_id: BTreeMap<String, Evidence>,
    open_questions: Vec<OpenQuestion>,
    budget_usage: ResearchBudgetUsage,
}

impl DeepResearchRun {
    fn new() -> Self {
        Self {
            completed: Vec::new(),
            failures: Vec::new(),
            aspect_reports: Vec::new(),
            evidence_by_id: BTreeMap::new(),
            open_questions: Vec::new(),
            budget_usage: ResearchBudgetUsage::zero(),
        }
    }
}

/// Executes every aspect with one shared research-level guard.
///
/// The request passed here already carries the effective merged limits. Its
/// concurrency cap controls scheduling, while the shared `ResearchBudgetGuard`
/// reserves global model/search/token capacity before provider dispatch.
async fn execute_aspects(
    request: &EffectiveResearchPlan,
    model_service: &ModelService,
    search_service: &SearchService,
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

async fn run_aspect_runtime(
    request: EffectiveAspectPlan,
    model_service: &ModelService,
    search_service: &SearchService,
    research_budget: Arc<ResearchBudgetGuard>,
) -> Result<AgentRuntimeOutput, AgentRuntimeFailure> {
    AgentRuntime::new(model_service, search_service, &request, research_budget)
        .run()
        .await
}

fn aspect_requests(request: &EffectiveResearchPlan) -> Vec<EffectiveAspectPlan> {
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

fn record_aspect_result(
    run: &mut DeepResearchRun,
    aspect_id: &str,
    result: std::result::Result<AgentRuntimeOutput, AgentRuntimeFailure>,
    allow_partial_results: bool,
) {
    match result {
        Ok(result) => record_aspect_success(run, result),
        Err(mut failure) => {
            let aspect_error = aspect_failure(aspect_id, &failure.error);
            let partial_evidence_count = failure
                .partial_output
                .as_ref()
                .map_or(0, |output| output.result.evidence.len());
            let preserve_partial_evidence = allow_partial_results && partial_evidence_count > 0;
            tracing::warn!(
                event = "aspect_failed",
                status = "failed",
                aspect_id,
                error_code = failure.error.code().as_str(),
                error_message = %error_message_for_log(&failure.error),
                retryable = failure.error.retryable(),
                partial_evidence_count,
                preserve_partial_evidence,
                "aspect failed"
            );
            if allow_partial_results && let Some(mut output) = failure.partial_output.take() {
                namespace_aspect_evidence(&mut output.result);
                for evidence in &output.result.evidence {
                    run.evidence_by_id
                        .entry(evidence.id.clone())
                        .or_insert_with(|| evidence.clone());
                }
            }
            run.failures.push(aspect_error);
        }
    }
}

fn record_aspect_success(run: &mut DeepResearchRun, mut output: AgentRuntimeOutput) {
    namespace_aspect_evidence(&mut output.result);
    run.completed
        .push(output.result.aspect_report.aspect_id.clone());
    run.open_questions
        .extend(output.result.aspect_report.open_questions.clone());
    for evidence in &output.result.evidence {
        run.evidence_by_id
            .entry(evidence.id.clone())
            .or_insert_with(|| evidence.clone());
    }
    run.aspect_reports.push(output.result.aspect_report);
}

fn namespace_aspect_evidence(result: &mut AspectResearchResult) {
    let aspect_id = result.aspect_report.aspect_id.clone();
    let mut remapped_ids = BTreeMap::new();

    for evidence in &mut result.evidence {
        let original_id = evidence.id.clone();
        let namespaced_id = format!("{aspect_id}:{original_id}");
        evidence.id.clone_from(&namespaced_id);
        remapped_ids.insert(original_id, namespaced_id);
    }

    for finding in &mut result.aspect_report.findings {
        for evidence_ref in &mut finding.evidence_refs {
            if let Some(namespaced_id) = remapped_ids.get(evidence_ref) {
                *evidence_ref = namespaced_id.clone();
            }
        }
    }
}

/// Finalizes a `DeepResearchRun` into either a `DeepResearchResult` or a
/// terminal error, honoring the `allow_partial_results` execution policy.
///
/// `request` is borrowed so the partial-result decision can read the policy
/// without taking ownership of the deep-research request.
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
            order_failures_by_request(request, run.failures),
        ));
    }

    if !run.failures.is_empty() && !request.policy.execution.allow_partial_results {
        return Err(DeepResearchFailure::with_aspects(
            Error::PartialResult {
                message: "deep research produced partial results".to_owned(),
            },
            order_failures_by_request(request, run.failures),
        ));
    }

    Ok(deep_result(request, run, run_id))
}

/// Builds the public `DeepResearchResult` from the request shape and the
/// accumulated `DeepResearchRun` state.
///
/// `request` is borrowed because we only need `task.aspects.len()` for the
/// coverage summary; `run` is consumed because the aggregated reports and
/// evidence are moved into the result.
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

fn order_failures_by_request(
    request: &EffectiveResearchPlan,
    failures: Vec<AspectFailure>,
) -> Vec<AspectFailure> {
    let mut by_aspect_id = failures
        .into_iter()
        .map(|failure| (failure.aspect_id.clone(), failure))
        .collect::<BTreeMap<_, _>>();

    request
        .task
        .aspects
        .iter()
        .filter_map(|aspect| by_aspect_id.remove(&aspect.id))
        .collect()
}

/// Builds the per-aspect failure record embedded inside a partial or failed
/// `DeepResearchResult`.
///
/// `error_code` is the `snake_case` transport-neutral `ErrorCode` identifier
/// rather than `Debug` output, so external clients can dispatch on a stable
/// string. `message` is the same redacted summary used in the public envelope.
fn aspect_failure(aspect_id: &str, error: &Error) -> AspectFailure {
    AspectFailure {
        aspect_id: aspect_id.to_owned(),
        error_code: error.code().as_str().to_owned(),
        message: error.public_message(),
        retryable: error.retryable(),
    }
}

fn confidence_summary(aspect_reports: &[AspectReport]) -> ConfidenceSummary {
    let mut summary = ConfidenceSummary::zero();
    for report in aspect_reports {
        match report.confidence {
            Confidence::High => summary.high += 1,
            Confidence::Medium => summary.medium += 1,
            Confidence::Low => summary.low += 1,
        }
    }
    summary
}
