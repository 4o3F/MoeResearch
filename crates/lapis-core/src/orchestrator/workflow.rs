use std::collections::BTreeMap;
use std::time::Instant;

use futures::{StreamExt, stream};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::model::service::ModelService;
use crate::orchestrator::agent_loop::{AgentRuntime, AgentRuntimeFailure, AgentRuntimeOutput};
use crate::orchestrator::tool_policy::SEARCH_TOOL_NAME;
use crate::schema::config::BudgetConfig;
use crate::schema::report::{
    AspectFailure, AspectReport, AspectResearchResult, Confidence, CoverageSummary,
    DeepResearchResult, Evidence, OpenQuestion, ResearchBudgetUsage, TerminationReason, TokenUsage,
    TraceSummary,
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
) -> Result<AspectResearchResult, AgentRuntimeFailure> {
    request
        .validate_for_execution(&WorkflowValidationContext {
            budget_config,
            supported_schema_versions: SUPPORTED_SCHEMA_VERSIONS,
            supported_tool_name: SEARCH_TOOL_NAME,
        })
        .map_err(|error| AgentRuntimeFailure {
            error,
            partial_trace: None,
        })?;
    AgentRuntime::new(model_service, search_service, &request)
        .run()
        .await
        .map(AgentRuntimeOutput::into_result)
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
    let mut run = execute_aspects(&request, model_service, search_service, budget_config).await;
    run.budget_usage.elapsed_ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    request.plan.budget.ensure_usage_within(&run.budget_usage)?;
    finalize_deep_result(request, run, run_id)
}

#[derive(Default)]
struct DeepResearchRun {
    completed: Vec<String>,
    failures: Vec<AspectFailure>,
    aspect_reports: Vec<AspectReport>,
    evidence_by_id: BTreeMap<String, Evidence>,
    open_questions: Vec<OpenQuestion>,
    budget_usage: ResearchBudgetUsage,
    model_calls: Vec<crate::schema::report::ProviderCallSummary>,
    search_calls: Vec<crate::schema::report::ProviderCallSummary>,
    first_error: Option<Error>,
}

async fn execute_aspects(
    request: &DeepResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    budget_config: &BudgetConfig,
) -> DeepResearchRun {
    let mut run = DeepResearchRun::default();
    let mut results = stream::iter(aspect_requests(request).into_iter().map(
        |aspect_request| async move {
            let aspect_id = aspect_request.aspect.aspect_id.clone();
            let result =
                aspect_research(aspect_request, model_service, search_service, budget_config)
                    .await
                    .map_err(|failure| failure.error);
            (aspect_id, result)
        },
    ))
    .buffer_unordered(
        request
            .plan
            .budget
            .max_concurrent_agents
            .as_concurrency(request.plan.aspects.len()),
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
        .plan
        .aspects
        .iter()
        .cloned()
        .map(|aspect| {
            let budget = aspect.budget_override.clone().unwrap_or_default();
            AspectResearchRequest {
                schema_version: request.schema_version.clone(),
                request_id: request.request_id.clone(),
                aspect,
                shared_context: request.shared_context.clone(),
                model_policy: request.plan.model_policy.clone(),
                search_policy: request.plan.search_policy.clone(),
                evidence_policy: request.plan.evidence_policy.clone(),
                output_policy: request.plan.output_policy.clone(),
                budget,
                execution_policy: request.execution_policy.clone(),
            }
        })
        .collect()
}

fn record_aspect_result(
    run: &mut DeepResearchRun,
    aspect_id: &str,
    result: Result<AspectResearchResult>,
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

fn record_aspect_success(run: &mut DeepResearchRun, result: AspectResearchResult) {
    run.budget_usage.model_calls_used += result.provider_usage.model_calls;
    run.budget_usage.search_calls_used += result.budget_usage.search_calls_used;
    run.budget_usage.elapsed_ms = run
        .budget_usage
        .elapsed_ms
        .saturating_add(result.budget_usage.elapsed_ms);
    run.budget_usage.token_usage = merge_token_usage(
        run.budget_usage.token_usage.take(),
        result.provider_usage.token_usage.clone(),
    );
    run.model_calls
        .extend(result.trace_summary.model_calls.clone());
    run.search_calls
        .extend(result.trace_summary.search_calls.clone());
    run.completed.push(result.aspect_report.aspect_id.clone());
    run.open_questions
        .extend(result.aspect_report.open_questions.clone());
    for evidence in &result.evidence {
        run.evidence_by_id
            .entry(evidence.id.clone())
            .or_insert_with(|| evidence.clone());
    }
    run.aspect_reports.push(result.aspect_report);
}

fn finalize_deep_result(
    request: DeepResearchRequest,
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

fn deep_result(
    request: DeepResearchRequest,
    run: DeepResearchRun,
    run_id: String,
) -> DeepResearchResult {
    let evidence_index = run.evidence_by_id.into_values().collect::<Vec<_>>();
    let coverage_summary = CoverageSummary {
        requested_aspects: request.plan.aspects.len(),
        completed_aspects: run.completed.len(),
        failed_aspects: run.failures.len(),
        evidence_count: evidence_index.len(),
    };
    let termination_reason = if run.failures.is_empty() {
        TerminationReason::Completed
    } else {
        TerminationReason::PartialCompleted
    };

    DeepResearchResult {
        run_id: run_id.clone(),
        plan_id: request.plan.plan_id,
        completed_aspects: run.completed,
        failed_aspects: run.failures,
        confidence_summary: confidence_summary(&run.aspect_reports),
        aspect_reports: run.aspect_reports,
        evidence_index,
        open_questions: run.open_questions,
        coverage_summary,
        budget_usage: run.budget_usage,
        trace_summary: TraceSummary {
            trace_id: run_id,
            root_span: "deep_research".to_owned(),
            started_at: now_rfc3339(),
            finished_at: Some(now_rfc3339()),
            model_calls: run.model_calls,
            search_calls: run.search_calls,
            termination_reason: Some(termination_reason),
        },
    }
}

fn aspect_failure(aspect_id: &str, error: &Error) -> AspectFailure {
    AspectFailure {
        aspect_id: aspect_id.to_owned(),
        error_code: format!("{:?}", error.code()),
        message: error.to_string(),
        retryable: error.to_tool_error().retryable,
    }
}

fn confidence_summary(
    aspect_reports: &[crate::schema::report::AspectReport],
) -> crate::schema::report::ConfidenceSummary {
    let mut summary = crate::schema::report::ConfidenceSummary::default();
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

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}
