use std::collections::BTreeMap;
use std::time::Instant;

use futures::{StreamExt, stream};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::model::service::ModelService;
use crate::orchestrator::agent_loop::{AgentRuntime, AgentRuntimeFailure, AgentRuntimeOutput};
use crate::orchestrator::tool_policy::SEARCH_TOOL_NAME;
use crate::schema::budget::BudgetConfig;
use crate::schema::report::{
    AspectFailure, AspectReport, AspectResearchResult, Confidence, CoverageSummary,
    DeepResearchResult, Evidence, OpenQuestion, ResearchBudgetUsage, TokenUsage,
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
    request
        .validate_for_execution(&WorkflowValidationContext {
            budget_config,
            supported_schema_versions: SUPPORTED_SCHEMA_VERSIONS,
            supported_tool_name: SEARCH_TOOL_NAME,
        })
        .map_err(|error| AgentRuntimeFailure { error })?;
    AgentRuntime::new(model_service, search_service, &request)
        .run()
        .await
}

pub async fn deep_research(
    request: DeepResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    budget_config: &BudgetConfig,
) -> Result<DeepResearchResult> {
    request.validate_for_execution(&WorkflowValidationContext {
        budget_config,
        supported_schema_versions: SUPPORTED_SCHEMA_VERSIONS,
        supported_tool_name: SEARCH_TOOL_NAME,
    })?;

    let started = Instant::now();
    let run_id = Uuid::new_v4().to_string();
    let request_id = request.request_id.clone();
    let requested_aspects = request.aspect_tasks.len();
    tracing::info!(
        request_id = %request_id,
        run_id = %run_id,
        requested_aspects,
        "deep research started"
    );

    let mut run = execute_aspects(&request, model_service, search_service, budget_config).await;
    run.budget_usage.elapsed_ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
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
            error_code = ?error.code(),
            retryable = error.retryable(),
            status = "failed",
            "deep research budget check failed"
        );
        return Err(error);
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
        Err(error) => tracing::warn!(
            request_id = %request_id,
            run_id = %run_id,
            requested_aspects,
            error_code = ?error.code(),
            retryable = error.retryable(),
            status = "failed",
            "deep research failed"
        ),
    }
    result
}

struct DeepResearchRun {
    completed: Vec<String>,
    failures: Vec<AspectFailure>,
    aspect_reports: Vec<AspectReport>,
    evidence_by_id: BTreeMap<String, Evidence>,
    open_questions: Vec<OpenQuestion>,
    budget_usage: ResearchBudgetUsage,
    first_error: Option<Error>,
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
            first_error: None,
        }
    }
}

async fn execute_aspects(
    request: &DeepResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    budget_config: &BudgetConfig,
) -> DeepResearchRun {
    let mut run = DeepResearchRun::new();
    let mut results = stream::iter(aspect_requests(request).into_iter().map(
        |aspect_request| async move {
            let aspect_id = aspect_request.task.aspect.aspect_id.clone();
            let result =
                aspect_research(aspect_request, model_service, search_service, budget_config)
                    .await
                    .map_err(|failure| failure.error);
            (aspect_id, result)
        },
    ))
    .buffer_unordered(
        request
            .budget
            .max_concurrent_agents
            .as_concurrency(request.aspect_tasks.len()),
    );

    while let Some((aspect_id, result)) = results.next().await {
        run.budget_usage.agents_started += 1;
        record_aspect_result(&mut run, &aspect_id, result);
        if request.execution_policy.fail_fast && !run.failures.is_empty() {
            break;
        }
    }

    run
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
            if run.first_error.is_none() {
                run.first_error = Some(error);
            }
            run.failures.push(failure);
        }
    }
}

fn record_aspect_success(run: &mut DeepResearchRun, mut output: AgentRuntimeOutput) {
    namespace_aspect_evidence(&mut output.result);
    run.budget_usage.model_calls_used += output.budget_usage.turns_used;
    run.budget_usage.search_calls_used += output.budget_usage.search_calls_used;
    run.budget_usage.elapsed_ms = run
        .budget_usage
        .elapsed_ms
        .saturating_add(output.budget_usage.elapsed_ms);
    run.budget_usage.token_usage = merge_token_usage(
        run.budget_usage.token_usage.take(),
        output.token_usage.clone(),
    );
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
) -> Result<DeepResearchResult> {
    if run.completed.is_empty() {
        return Err(run.first_error.unwrap_or_else(|| Error::PartialResult {
            message: "all aspects failed".to_owned(),
        }));
    }

    if !run.failures.is_empty() && !request.execution_policy.allow_partial_results {
        return Err(run.first_error.unwrap_or_else(|| Error::PartialResult {
            message: "deep research produced partial results".to_owned(),
        }));
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
    let evidence_index = run.evidence_by_id.into_values().collect::<Vec<_>>();
    let coverage_summary = CoverageSummary {
        requested_aspects: request.aspect_tasks.len(),
        completed_aspects: run.completed.len(),
        failed_aspects: run.failures.len(),
        evidence_count: evidence_index.len(),
    };
    DeepResearchResult {
        run_id,
        completed_aspects: run.completed,
        failed_aspects: run.failures,
        confidence_summary: confidence_summary(&run.aspect_reports),
        aspect_reports: run.aspect_reports,
        evidence_index,
        open_questions: run.open_questions,
        coverage_summary,
        budget_usage: run.budget_usage,
    }
}

/// Builds the per-aspect failure record embedded inside a partial or failed
/// `DeepResearchResult`.
///
/// `error_code` is the `snake_case` `ToolErrorCode` identifier (matching the
/// MCP envelope's `error.code` field) rather than `Debug` output, so external
/// clients can dispatch on a single, stable string. `message` is the same
/// redacted summary used in the public envelope.
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

fn merge_token_usage(left: Option<TokenUsage>, right: Option<TokenUsage>) -> Option<TokenUsage> {
    match (left, right) {
        (None, None) => None,
        (Some(usage), None) | (None, Some(usage)) => Some(usage),
        (Some(left), Some(right)) => Some(TokenUsage {
            input_tokens: sum_options(left.input_tokens, right.input_tokens),
            output_tokens: sum_options(left.output_tokens, right.output_tokens),
            total_tokens: sum_options(left.total_tokens, right.total_tokens),
        }),
    }
}

fn sum_options(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (None, None) => None,
        (Some(value), None) | (None, Some(value)) => Some(value),
        (Some(left), Some(right)) => Some(left.saturating_add(right)),
    }
}
