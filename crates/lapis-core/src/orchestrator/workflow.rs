use std::collections::BTreeMap;
use std::sync::Arc;

use futures::{StreamExt, stream};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::model::service::ModelService;
use crate::orchestrator::agent_loop::{AgentRuntime, AgentRuntimeFailure, AgentRuntimeOutput};
use crate::orchestrator::budget::ResearchBudgetGuard;
use crate::orchestrator::tool_policy::SEARCH_TOOL_NAME;
use crate::schema::budget::BudgetConfig;
use crate::schema::report::{
    AspectFailure, AspectReport, AspectResearchResult, Confidence, CoverageSummary,
    DeepResearchResult, Evidence, OpenQuestion, ResearchBudgetUsage,
};
use crate::schema::research::{
    AspectResearchRequest, DeepResearchRequest, WorkflowValidationContext,
};
use crate::search::service::SearchService;

const SUPPORTED_SCHEMA_VERSIONS: &[&str] = &["m4", "m5", "1", "1.0"];

pub async fn aspect_research(
    request: AspectResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    budget_config: &BudgetConfig,
) -> Result<AgentRuntimeOutput, AgentRuntimeFailure> {
    let research_budget = ResearchBudgetGuard::unlimited();
    research_budget.record_agent_started();
    run_aspect_runtime(
        request,
        model_service,
        search_service,
        budget_config,
        research_budget,
    )
    .await
}

pub async fn deep_research(
    request: DeepResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    budget_config: &BudgetConfig,
) -> std::result::Result<DeepResearchResult, Box<DeepResearchFailure>> {
    request
        .validate_for_execution(&WorkflowValidationContext {
            budget_config,
            supported_schema_versions: SUPPORTED_SCHEMA_VERSIONS,
            supported_tool_name: SEARCH_TOOL_NAME,
        })
        .map_err(DeepResearchFailure::top_level)?;

    let run_id = Uuid::new_v4().to_string();
    let request_id = request.request_id.clone();
    let requested_aspects = request.aspect_tasks.len();
    tracing::info!(
        request_id = %request_id,
        run_id = %run_id,
        requested_aspects,
        "deep research started"
    );

    let research_budget = ResearchBudgetGuard::new(request.budget.clone());
    let mut run = execute_aspects(
        &request,
        model_service,
        search_service,
        budget_config,
        research_budget.clone(),
    )
    .await;
    run.budget_usage = research_budget.snapshot();
    if let Err(error) = request.budget.ensure_usage_within(&run.budget_usage) {
        tracing::warn!(
            request_id = %request_id,
            run_id = %run_id,
            requested_aspects,
            agents_started = run.budget_usage.agents_started,
            completed_aspects = run.completed.len(),
            failed_aspects = run.failures.len(),
            model_calls_used = run.budget_usage.model_calls_used,
            search_calls_used = run.budget_usage.search_calls_used,
            elapsed_ms = run.budget_usage.elapsed_ms,
            error_code = error.code().as_str(),
            retryable = error.retryable(),
            status = "failed",
            "deep research budget check failed"
        );
        return Err(DeepResearchFailure::with_aspects(
            error,
            order_failures_by_request(&request, run.failures),
        ));
    }

    let result = finalize_deep_result(&request, run, run_id.clone());
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

async fn execute_aspects(
    request: &DeepResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    budget_config: &BudgetConfig,
    research_budget: Arc<ResearchBudgetGuard>,
) -> DeepResearchRun {
    let mut run = DeepResearchRun::new();
    let mut results = stream::iter(aspect_requests(request).into_iter().map(|aspect_request| {
        let research_budget = research_budget.clone();
        async move {
            research_budget.record_agent_started();
            let aspect_id = aspect_request.task.aspect.aspect_id.clone();
            let result = run_aspect_runtime(
                aspect_request,
                model_service,
                search_service,
                budget_config,
                research_budget,
            )
            .await
            .map_err(|failure| failure.error);
            (aspect_id, result)
        }
    }))
    .buffer_unordered(
        request
            .budget
            .max_concurrent_agents
            .as_concurrency(request.aspect_tasks.len()),
    );

    while let Some((aspect_id, result)) = results.next().await {
        record_aspect_result(&mut run, &aspect_id, result);
        if request.execution_policy.fail_fast && !run.failures.is_empty() {
            break;
        }
    }

    run
}

async fn run_aspect_runtime(
    request: AspectResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    budget_config: &BudgetConfig,
    research_budget: Arc<ResearchBudgetGuard>,
) -> Result<AgentRuntimeOutput, AgentRuntimeFailure> {
    request
        .validate_for_execution(&WorkflowValidationContext {
            budget_config,
            supported_schema_versions: SUPPORTED_SCHEMA_VERSIONS,
            supported_tool_name: SEARCH_TOOL_NAME,
        })
        .map_err(|error| AgentRuntimeFailure { error })?;
    AgentRuntime::new(model_service, search_service, &request, research_budget)
        .run()
        .await
}

fn aspect_requests(request: &DeepResearchRequest) -> Vec<AspectResearchRequest> {
    request
        .aspect_tasks
        .iter()
        .cloned()
        .map(|task| AspectResearchRequest {
            schema_version: request.schema_version.clone(),
            request_id: request.request_id.clone(),
            task,
            shared_context: request.shared_context.clone(),
            model_policy: request.model_policy.clone(),
            search_policy: request.search_policy.clone(),
            evidence_policy: request.evidence_policy.clone(),
            output_policy: request.output_policy.clone(),
            execution_policy: request.execution_policy.clone(),
        })
        .collect()
}

fn record_aspect_result(
    run: &mut DeepResearchRun,
    aspect_id: &str,
    result: Result<AgentRuntimeOutput>,
) {
    match result {
        Ok(result) => record_aspect_success(run, result),
        Err(error) => {
            let failure = aspect_failure(aspect_id, &error);
            run.failures.push(failure);
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
    request: &DeepResearchRequest,
    run: DeepResearchRun,
    run_id: String,
) -> std::result::Result<DeepResearchResult, Box<DeepResearchFailure>> {
    if run.completed.is_empty() {
        return Err(DeepResearchFailure::with_aspects(
            Error::PartialResult {
                message: "all aspects failed".to_owned(),
            },
            order_failures_by_request(request, run.failures),
        ));
    }

    if !run.failures.is_empty() && !request.execution_policy.allow_partial_results {
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
/// `request` is borrowed because we only need `aspect_tasks.len()` for the
/// coverage summary; `run` is consumed because the aggregated reports and
/// evidence are moved into the result.
fn deep_result(
    request: &DeepResearchRequest,
    run: DeepResearchRun,
    run_id: String,
) -> DeepResearchResult {
    let failed_aspects = order_failures_by_request(request, run.failures);
    let evidence_index = run.evidence_by_id.into_values().collect::<Vec<_>>();
    let coverage_summary = CoverageSummary {
        requested_aspects: request.aspect_tasks.len(),
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
    request: &DeepResearchRequest,
    failures: Vec<AspectFailure>,
) -> Vec<AspectFailure> {
    let mut by_aspect_id = failures
        .into_iter()
        .map(|failure| (failure.aspect_id.clone(), failure))
        .collect::<BTreeMap<_, _>>();

    request
        .aspect_tasks
        .iter()
        .filter_map(|task| by_aspect_id.remove(&task.aspect.aspect_id))
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

fn confidence_summary(
    aspect_reports: &[crate::schema::report::AspectReport],
) -> crate::schema::report::ConfidenceSummary {
    let mut summary = crate::schema::report::ConfidenceSummary::zero();
    for report in aspect_reports {
        match report.confidence {
            Confidence::High => summary.high += 1,
            Confidence::Medium => summary.medium += 1,
            Confidence::Low => summary.low += 1,
        }
    }
    summary
}
