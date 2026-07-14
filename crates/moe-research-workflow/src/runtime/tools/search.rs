use std::time::Instant;

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::error_log_safe::{error_message_for_log, safe_model_identifier_for_log};
use crate::report::{Confidence, Evidence, FailureStage, SourceType};
use crate::runtime::{AgentBudgetGuard, elapsed_ms};
use moe_research_error::{Error, Result};
use moe_research_model::{ModelTool, ModelToolCall, ModelToolOutput};
use moe_research_search::{
    IntentDimensionResolution, SearchIntent, SearchRequest, SearchResponse, SearchResult,
};

use super::policy::tool_args_error;
use super::{ToolExecutor, ToolRuntimeState};

pub const SEARCH_TOOL_NAME: &str = "search";

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SearchToolArgs {
    pub query: String,
    #[schemars(schema_with = "crate::limit::optional_positive_integer_schema")]
    pub max_results: Option<usize>,
    pub intent: SearchIntent,
}

pub(crate) fn validate_search_call(call: &ModelToolCall) -> Result<SearchToolArgs> {
    let Some(arguments) = call.arguments.as_object() else {
        return Err(tool_args_error(SEARCH_TOOL_NAME, "invalid_structure"));
    };
    if !arguments.contains_key("intent") {
        return Err(tool_args_error(SEARCH_TOOL_NAME, "missing_intent"));
    }
    if arguments
        .keys()
        .any(|key| !matches!(key.as_str(), "query" | "max_results" | "intent"))
    {
        return Err(tool_args_error(SEARCH_TOOL_NAME, "unknown_field"));
    }
    let args: SearchToolArgs = serde_json::from_value(call.arguments.clone())
        .map_err(|_| tool_args_error(SEARCH_TOOL_NAME, "invalid_structure"))?;
    if args.query.trim().is_empty() {
        return Err(tool_args_error(SEARCH_TOOL_NAME, "empty_query"));
    }
    if args.max_results == Some(0) {
        return Err(tool_args_error(SEARCH_TOOL_NAME, "zero_max_results"));
    }
    Ok(args)
}

pub fn search_model_tool() -> ModelTool {
    ModelTool {
        name: SEARCH_TOOL_NAME.to_owned(),
        description: "Search trusted external sources for evidence relevant to the aspect."
            .to_owned(),
        input_schema: serde_json::to_value(schema_for!(SearchToolArgs))
            .expect("search tool schema serializes to JSON"),
    }
}

impl ToolExecutor<'_> {
    pub(super) async fn execute_search(
        &self,
        tool_call: &ModelToolCall,
        args: SearchToolArgs,
        budget: &mut AgentBudgetGuard,
        state: &mut ToolRuntimeState,
        model_turn: usize,
    ) -> Result<ModelToolOutput> {
        let search_policy = &self.request.policy.search;
        let search_turn =
            state.begin_retrieval_turn(model_turn, FailureStage::SearchIntentResolution);
        let search_provider = self
            .request
            .task
            .search_provider
            .as_deref()
            .filter(|provider| !provider.trim().is_empty())
            .ok_or_else(|| Error::InvalidInput {
                message: "search provider must be explicitly selected".to_owned(),
            })?;
        let max_results = args
            .max_results
            .unwrap_or(self.request.policy.search.max_results_per_query);
        let resolved = match self.search_service.resolve_intent(
            search_provider,
            self.base_search_request(search_provider, &args, max_results),
            &args.intent,
            &search_policy.intent_constraints(),
        ) {
            Ok(resolved) => resolved,
            Err(error) => {
                tracing::warn!(
                    request_id = %self.request.request_id,
                    aspect_id = %self.request.task.id,
                    tool_call_id = %safe_model_identifier_for_log(&tool_call.id),
                    provider = %search_provider,
                    diagnostic_branch = "search_intent_resolution",
                    diagnostic_key = search_resolution_diagnostic_key(&error),
                    error_code = error.code().as_str(),
                    error_message = %error_message_for_log(&error),
                    retryable = error.retryable(),
                    status = "denied",
                    "search intent resolution rejected"
                );
                return Err(error);
            }
        };
        let intent_resolution = resolved.resolution;
        state.set_diagnostic(
            FailureStage::SearchPolicy,
            Some(model_turn),
            Some(search_turn),
        );
        let search_request = match search_policy.apply_to(resolved.request) {
            Ok(request) => request,
            Err(error) => {
                tracing::warn!(
                    request_id = %self.request.request_id,
                    aspect_id = %self.request.task.id,
                    tool_call_id = %safe_model_identifier_for_log(&tool_call.id),
                    provider = %search_provider,
                    diagnostic_branch = "search_policy_guard",
                    diagnostic_key = "final_guard_rejected",
                    error_code = error.code().as_str(),
                    error_message = %error_message_for_log(&error),
                    retryable = error.retryable(),
                    status = "denied",
                    "resolved search request rejected by policy"
                );
                return Err(error);
            }
        };
        state.set_diagnostic(
            FailureStage::SearchBudget,
            Some(model_turn),
            Some(search_turn),
        );
        if let Err(error) = budget.consume_search_tool_call() {
            let budget_usage = budget.usage();
            tracing::warn!(
                request_id = %self.request.request_id,
                aspect_id = %self.request.task.id,
                tool_call_id = %safe_model_identifier_for_log(&tool_call.id),
                tool_name = %safe_model_identifier_for_log(&tool_call.name),
                provider = %search_provider,
                turns_used = budget_usage.turns_used,
                tool_calls_used = budget_usage.tool_calls_used,
                search_calls_used = budget_usage.search_calls_used,
                elapsed_ms = budget_usage.elapsed_ms,
                error_code = error.code().as_str(),
                error_message = %error_message_for_log(&error),
                retryable = error.retryable(),
                status = "rejected",
                "search tool call budget rejected"
            );
            return Err(error);
        }
        if let Err(error) = self.research_budget.try_consume_search_call() {
            state.set_diagnostic(
                FailureStage::ResearchBudget,
                Some(model_turn),
                Some(search_turn),
            );
            tracing::warn!(
                request_id = %self.request.request_id,
                aspect_id = %self.request.task.id,
                tool_call_id = %safe_model_identifier_for_log(&tool_call.id),
                tool_name = %safe_model_identifier_for_log(&tool_call.name),
                provider = %search_provider,
                error_code = error.code().as_str(),
                error_message = %error_message_for_log(&error),
                retryable = error.retryable(),
                status = "rejected",
                "research search budget rejected before search dispatch"
            );
            return Err(error);
        }

        tracing::debug!(
            request_id = %self.request.request_id,
            aspect_id = %self.request.task.id,
            tool_call_id = %safe_model_identifier_for_log(&tool_call.id),
            tool_name = %safe_model_identifier_for_log(&tool_call.name),
            provider = %search_provider,
            max_results,
            status = "accepted",
            "search tool call accepted"
        );
        state.set_diagnostic(
            FailureStage::SearchDispatch,
            Some(model_turn),
            Some(search_turn),
        );
        let search_started = Instant::now();
        let response = match self.search_service.search(search_request).await {
            Ok(response) => response,
            Err(error) => {
                tracing::warn!(
                    request_id = %self.request.request_id,
                    aspect_id = %self.request.task.id,
                    provider = %search_provider,
                    duration_ms = elapsed_ms(search_started.elapsed()),
                    error_code = error.code().as_str(),
                    error_message = %error_message_for_log(&error),
                    retryable = error.retryable(),
                    status = "failed",
                    "search call failed"
                );
                return Err(error);
            }
        };
        let search_duration = elapsed_ms(search_started.elapsed());
        let result_count = response.results.len();

        tracing::info!(
            request_id = %self.request.request_id,
            aspect_id = %self.request.task.id,
            provider = %response.provider,
            result_count,
            duration_ms = search_duration,
            status = "ok",
            "search call completed"
        );

        let new_evidence = evidence_from_search(
            &args.query,
            &response,
            state.candidate_evidence.len(),
            search_turn,
        );
        tracing::debug!(
            request_id = %self.request.request_id,
            aspect_id = %self.request.task.id,
            tool_call_id = %safe_model_identifier_for_log(&tool_call.id),
            tool_name = %safe_model_identifier_for_log(&tool_call.name),
            provider = %response.provider,
            result_count,
            duration_ms = search_duration,
            status = "completed",
            "tool call completed"
        );

        let tool_output =
            search_result_message(&args.query, &response, &new_evidence, &intent_resolution);
        state.candidate_evidence.extend(new_evidence);

        Ok(ModelToolOutput::new(tool_call.id.clone(), tool_output))
    }

    fn base_search_request(
        &self,
        provider: &str,
        args: &SearchToolArgs,
        max_results: usize,
    ) -> SearchRequest {
        SearchRequest::new(
            provider,
            &args.query,
            max_results.min(self.request.policy.search.max_results_per_query),
        )
    }
}

fn evidence_from_search(
    query: &str,
    response: &SearchResponse,
    existing_count: usize,
    search_index: usize,
) -> Vec<Evidence> {
    response
        .results
        .iter()
        .enumerate()
        .map(|(index, result)| {
            evidence_from_result(
                query,
                &response.provider,
                result,
                existing_count + index + 1,
                search_index,
            )
        })
        .collect()
}

fn evidence_from_result(
    query: &str,
    provider: &str,
    result: &SearchResult,
    evidence_index: usize,
    search_index: usize,
) -> Evidence {
    let snippet = result.snippet.clone();
    let summary = result
        .summary
        .clone()
        .unwrap_or_else(|| result.snippet.clone());
    Evidence {
        id: format!("ev-{search_index}-{evidence_index}"),
        source_title: result.title.clone(),
        url: result.url.clone(),
        provider: provider.to_owned(),
        query: query.to_owned(),
        snippet,
        summary,
        published_at: result.published_at.clone(),
        retrieved_at: now_rfc3339(),
        supports_findings: Vec::new(),
        source_type: SourceType::Unknown,
        confidence: Confidence::Medium,
    }
}

fn search_result_message(
    query: &str,
    response: &SearchResponse,
    evidence: &[Evidence],
    intent_resolution: &[IntentDimensionResolution],
) -> String {
    json!({
        "tool": SEARCH_TOOL_NAME,
        "query": query,
        "provider": response.provider,
        "result_count": response.results.len(),
        "intent_resolution": { "dimensions": intent_resolution },
        "results": evidence,
    })
    .to_string()
}

fn search_resolution_diagnostic_key(error: &Error) -> &'static str {
    match error {
        Error::ToolPolicyDenied { .. } => "intent_policy_conflict",
        Error::ProviderUnavailable { .. } => "provider_unavailable",
        Error::InvalidInput { .. } => "provider_intent_incompatible",
        _ => "resolution_failed",
    }
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}
