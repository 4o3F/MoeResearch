use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use futures::{StreamExt, stream};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::model::service::ModelService;
use crate::orchestrator::agent_loop::{AgentRuntime, AgentRuntimeOutput};
use crate::orchestrator::tool_policy::SEARCH_TOOL_NAME;
use crate::schema::common::{AspectResearchRequest, DeepResearchRequest, ResearchBudget};
use crate::schema::report::{
    AspectFailure, AspectReport, AspectResearchResult, Confidence, CoverageSummary,
    DeepResearchResult, Evidence, OpenQuestion, ResearchBudgetUsage, TerminationReason, TokenUsage,
    TraceSummary,
};
use crate::search::service::SearchService;

const SUPPORTED_SCHEMA_VERSIONS: &[&str] = &["m4", "m5", "1", "1.0"];

pub async fn aspect_research(
    request: AspectResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
) -> Result<AspectResearchResult> {
    validate_aspect_request(&request)?;
    AgentRuntime::new(model_service, search_service, &request)
        .run()
        .await
        .map(AgentRuntimeOutput::into_result)
}

pub async fn deep_research(
    request: DeepResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
) -> Result<DeepResearchResult> {
    validate_deep_request(&request)?;

    let started = Instant::now();
    let run_id = Uuid::new_v4().to_string();
    let mut run = execute_aspects(&request, model_service, search_service).await;
    run.budget_usage.elapsed_ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    validate_research_budget_usage(&request.plan.budget, &run.budget_usage)?;
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
) -> DeepResearchRun {
    let mut run = DeepResearchRun::default();
    let mut results = stream::iter(aspect_requests(request).into_iter().map(
        |aspect_request| async move {
            let aspect_id = aspect_request.aspect.aspect_id.clone();
            let result = aspect_research(aspect_request, model_service, search_service).await;
            (aspect_id, result)
        },
    ))
    .buffer_unordered(request.plan.budget.max_concurrent_agents);

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

fn validate_deep_request(request: &DeepResearchRequest) -> Result<()> {
    require_non_empty("schema_version", &request.schema_version)?;
    require_non_empty("request_id", &request.request_id)?;
    require_non_empty("plan.plan_id", &request.plan.plan_id)?;

    if !SUPPORTED_SCHEMA_VERSIONS.contains(&request.schema_version.as_str()) {
        return Err(Error::SchemaValidationFailed {
            message: format!("unsupported schema version: {}", request.schema_version),
        });
    }

    if request.plan.aspects.is_empty() {
        return Err(Error::InvalidInput {
            message: "plan.aspects must not be empty".to_owned(),
        });
    }

    validate_research_budget(&request.plan.budget)?;

    if request.plan.aspects.len() > request.plan.budget.max_agents {
        return Err(Error::BudgetExceeded {
            message: "plan aspect count exceeds max_agents".to_owned(),
        });
    }

    if let Some(timeout_ms) = request.execution_policy.timeout_ms
        && timeout_ms > request.plan.budget.total_timeout_ms
    {
        return Err(Error::BudgetExceeded {
            message: "execution timeout must not exceed research budget timeout".to_owned(),
        });
    }

    validate_domain_lists(
        &request.plan.search_policy.include_domains,
        &request.plan.search_policy.exclude_domains,
    )?;

    for aspect in &request.plan.aspects {
        require_non_empty("aspect.aspect_id", &aspect.aspect_id)?;
        require_non_empty("aspect.name", &aspect.name)?;
        require_non_empty("aspect.research_question", &aspect.research_question)?;
        validate_tools(&aspect.allowed_tools)?;

        if let Some(model_override) = &aspect.model_override
            && !request
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
                !request
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

        if let Some(budget) = &aspect.budget_override
            && budget.timeout_ms > request.plan.budget.total_timeout_ms
        {
            return Err(Error::BudgetExceeded {
                message: "aspect budget timeout exceeds research budget timeout".to_owned(),
            });
        }
    }

    Ok(())
}

fn validate_aspect_request(request: &AspectResearchRequest) -> Result<()> {
    require_non_empty("schema_version", &request.schema_version)?;
    require_non_empty("request_id", &request.request_id)?;
    require_non_empty("aspect.aspect_id", &request.aspect.aspect_id)?;
    require_non_empty("aspect.name", &request.aspect.name)?;
    require_non_empty(
        "aspect.research_question",
        &request.aspect.research_question,
    )?;

    if !SUPPORTED_SCHEMA_VERSIONS.contains(&request.schema_version.as_str()) {
        return Err(Error::SchemaValidationFailed {
            message: format!("unsupported schema version: {}", request.schema_version),
        });
    }

    if request.search_policy.max_results_per_query == 0 {
        return Err(Error::InvalidInput {
            message: "search_policy.max_results_per_query must be greater than 0".to_owned(),
        });
    }

    validate_domain_lists(
        &request.search_policy.include_domains,
        &request.search_policy.exclude_domains,
    )?;
    validate_aspect_timeout(request)?;
    validate_tools(&request.aspect.allowed_tools)
}

fn validate_research_budget(budget: &ResearchBudget) -> Result<()> {
    if budget.max_agents == 0 {
        return Err(Error::BudgetExceeded {
            message: "research budget requires at least one agent".to_owned(),
        });
    }

    if budget.max_concurrent_agents == 0 {
        return Err(Error::BudgetExceeded {
            message: "research budget requires non-zero concurrency".to_owned(),
        });
    }

    if budget.total_timeout_ms == 0 {
        return Err(Error::BudgetExceeded {
            message: "research budget requires a non-zero timeout".to_owned(),
        });
    }

    Ok(())
}

fn validate_research_budget_usage(
    budget: &ResearchBudget,
    usage: &ResearchBudgetUsage,
) -> Result<()> {
    if usage.model_calls_used > budget.max_total_model_calls {
        return Err(Error::BudgetExceeded {
            message: "research model call budget exhausted".to_owned(),
        });
    }

    if usage.search_calls_used > budget.max_total_search_calls {
        return Err(Error::BudgetExceeded {
            message: "research search call budget exhausted".to_owned(),
        });
    }

    if usage.elapsed_ms > budget.total_timeout_ms {
        return Err(Error::BudgetExceeded {
            message: "research timeout budget exhausted".to_owned(),
        });
    }

    Ok(())
}

fn require_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::InvalidInput {
            message: format!("{field} must not be empty"),
        });
    }
    Ok(())
}

fn validate_domain_lists(include_domains: &[String], exclude_domains: &[String]) -> Result<()> {
    let include = include_domains
        .iter()
        .map(|domain| domain.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();

    if let Some(domain) = exclude_domains
        .iter()
        .map(|domain| domain.to_ascii_lowercase())
        .find(|domain| include.contains(domain))
    {
        return Err(Error::InvalidInput {
            message: format!("domain appears in both include and exclude lists: {domain}"),
        });
    }

    Ok(())
}

fn validate_aspect_timeout(request: &AspectResearchRequest) -> Result<()> {
    if let Some(timeout_ms) = request.execution_policy.timeout_ms
        && timeout_ms > request.budget.timeout_ms
    {
        return Err(Error::BudgetExceeded {
            message: "execution timeout must not exceed agent budget timeout".to_owned(),
        });
    }
    Ok(())
}

fn validate_tools(tools: &[crate::schema::common::ToolName]) -> Result<()> {
    if let Some(tool) = tools.iter().find(|tool| tool.0 != SEARCH_TOOL_NAME) {
        return Err(Error::ToolPolicyDenied {
            message: format!("unsupported tool for aspect runtime: {}", tool.0),
        });
    }
    Ok(())
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
