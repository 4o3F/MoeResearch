use std::path::{Component, Path};
use std::time::{Duration, Instant};

use serde_json::json;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::model::service::ModelService;
use crate::orchestrator::budget::AgentBudgetGuard;
use crate::orchestrator::tool_policy::{ToolPolicyGuard, search_model_tool};
use crate::orchestrator::validator::OutputValidator;
use crate::schema::limit::{DurationLimitMs, Limit};
use crate::schema::model::{
    ModelInputItem, ModelMessageRole, ModelRequest, ModelResponse, ModelToolCall, ModelToolOutput,
};
use crate::schema::policy::SearchPolicy;
use crate::schema::report::{
    AgentBudgetUsage, AspectResearchResult, Confidence, Evidence, PartialTrace,
    ProviderCallSummary, ProviderType, ProviderUsage, SearchQueryTrace, SearchSourceTrace,
    SearchToolCallTrace, SourceType, TerminationReason, TokenUsage, ToolCallTrace, TraceSummary,
};
use crate::schema::research::AspectResearchRequest;
use crate::schema::search::{SearchRequest, SearchResponse, SearchResult};
use crate::search::service::SearchService;

pub struct AgentRuntime<'a> {
    model_service: &'a ModelService,
    search_service: &'a SearchService,
    request: &'a AspectResearchRequest,
}

#[derive(Debug)]
pub struct AgentRuntimeOutput {
    pub result: AspectResearchResult,
    pub search_queries: Vec<SearchQueryTrace>,
    pub tool_calls: Vec<ToolCallTrace>,
    pub provider_usage: ProviderUsage,
    pub budget_usage: AgentBudgetUsage,
    pub trace_summary: TraceSummary,
}

#[derive(Debug)]
pub struct AgentRuntimeFailure {
    pub error: Error,
    pub partial_trace: Option<PartialTrace>,
}

impl AgentRuntimeOutput {
    #[must_use]
    pub fn into_result(self) -> AspectResearchResult {
        self.result
    }
}

struct RuntimeState {
    input: Vec<ModelInputItem>,
    replay_input: Vec<ModelInputItem>,
    previous_response_id: Option<String>,
    candidate_evidence: Vec<Evidence>,
    search_queries: Vec<SearchQueryTrace>,
    tool_calls: Vec<ToolCallTrace>,
    provider_usage: ProviderUsage,
    trace_summary: TraceSummary,
}

impl RuntimeState {
    fn new(input: Vec<ModelInputItem>, trace_summary: TraceSummary) -> Self {
        Self {
            replay_input: input.clone(),
            input,
            previous_response_id: None,
            candidate_evidence: Vec::new(),
            search_queries: Vec::new(),
            tool_calls: Vec::new(),
            provider_usage: ProviderUsage::default(),
            trace_summary,
        }
    }

    fn append_model_output_and_tool_outputs(
        &mut self,
        response: &ModelResponse,
        tool_outputs: Vec<ModelToolOutput>,
    ) {
        let output_items = Self::replayable_output_items(response);
        let tool_output_items = tool_outputs
            .into_iter()
            .map(ModelInputItem::ToolOutput)
            .collect::<Vec<_>>();
        self.replay_input.extend(output_items);
        self.replay_input.extend(tool_output_items.clone());

        if let Some(response_id) = &response.response_id {
            self.previous_response_id = Some(response_id.clone());
            self.input = tool_output_items;
        } else {
            self.previous_response_id = None;
            self.input.clone_from(&self.replay_input);
        }
    }

    fn replayable_output_items(response: &ModelResponse) -> Vec<ModelInputItem> {
        if response.output_items.is_empty() {
            response
                .tool_calls
                .iter()
                .cloned()
                .map(ModelInputItem::ToolCall)
                .collect()
        } else {
            response.output_items.clone()
        }
    }
}

impl<'a> AgentRuntime<'a> {
    #[must_use]
    pub fn new(
        model_service: &'a ModelService,
        search_service: &'a SearchService,
        request: &'a AspectResearchRequest,
    ) -> Self {
        Self {
            model_service,
            search_service,
            request,
        }
    }

    pub async fn run(&self) -> Result<AgentRuntimeOutput, AgentRuntimeFailure> {
        let effective_budget = self.effective_budget();
        let deadline = RuntimeDeadline::new(effective_budget.timeout_ms);
        let mut budget = AgentBudgetGuard::new(effective_budget).map_err(Self::untraced_failure)?;
        let tool_policy = ToolPolicyGuard::new(&self.request.aspect);
        let validator = OutputValidator::new(
            &self.request.aspect,
            &self.request.evidence_policy,
            &self.request.output_policy,
        );
        let search_policy = self.effective_search_policy();
        let mut state = RuntimeState::new(
            self.initial_input().map_err(Self::untraced_failure)?,
            self.new_trace_summary(),
        );

        loop {
            let model_response = match self.complete_model_turn(&mut state, &mut budget).await {
                Ok(response) => response,
                Err(error) => return Err(Self::failure(error, state, &budget)),
            };
            if let Err(error) = deadline.ensure_not_elapsed() {
                return Err(Self::failure(error, state, &budget));
            }
            if model_response.tool_calls.is_empty() {
                let content = match model_response.content.as_deref().ok_or_else(|| {
                    Error::SchemaValidationFailed {
                        message: "model final response must include content".to_owned(),
                    }
                }) {
                    Ok(content) => content,
                    Err(error) => return Err(Self::failure(error, state, &budget)),
                };
                return self
                    .finish(content, state, &budget, &validator)
                    .map_err(|failure| *failure);
            }

            let mut tool_outputs = Vec::new();
            for tool_call in &model_response.tool_calls {
                let output = match self
                    .execute_tool_call(
                        tool_call,
                        &tool_policy,
                        &mut budget,
                        &search_policy,
                        &mut state,
                    )
                    .await
                {
                    Ok(output) => output,
                    Err(error) => return Err(Self::failure(error, state, &budget)),
                };
                if let Err(error) = deadline.ensure_not_elapsed() {
                    return Err(Self::failure(error, state, &budget));
                }
                tool_outputs.push(output);
            }
            state.append_model_output_and_tool_outputs(&model_response, tool_outputs);
        }
    }

    fn effective_budget(&self) -> crate::schema::budget::AgentBudget {
        let mut budget = self.request.budget.clone();
        if let Some(timeout_ms) = self.request.execution_policy.timeout_ms {
            budget.timeout_ms = Limit::limited(timeout_ms);
        }
        budget
    }

    async fn complete_model_turn(
        &self,
        state: &mut RuntimeState,
        budget: &mut AgentBudgetGuard,
    ) -> Result<ModelResponse> {
        budget.consume_model_turn()?;
        let model_started = Instant::now();
        let model_response = self
            .complete_model(state.previous_response_id.clone(), state.input.clone())
            .await?;
        let model_duration = elapsed_ms(model_started.elapsed());

        state.provider_usage.model_calls += 1;
        add_token_usage(
            &mut state.provider_usage.token_usage,
            model_response.usage.clone(),
        );
        state.trace_summary.model_calls.push(ProviderCallSummary {
            provider: model_response.provider.clone(),
            provider_type: ProviderType::Model,
            status: "ok".to_owned(),
            duration_ms: model_duration,
            retry_count: 0,
        });

        Ok(model_response)
    }

    async fn execute_tool_call(
        &self,
        tool_call: &ModelToolCall,
        tool_policy: &ToolPolicyGuard,
        budget: &mut AgentBudgetGuard,
        search_policy: &SearchPolicy,
        state: &mut RuntimeState,
    ) -> Result<ModelToolOutput> {
        let args = tool_policy.validate_search_call(tool_call)?;
        budget.consume_search_tool_call()?;

        let max_results = args
            .max_results
            .unwrap_or(self.request.search_policy.max_results_per_query);
        let search_started_at = now_rfc3339();
        let search_started = Instant::now();
        let search_request = self.search_request(&args.query, max_results);
        let response = self
            .search_service
            .search(search_request, search_policy)
            .await?;
        let search_duration = elapsed_ms(search_started.elapsed());

        state.provider_usage.search_calls += 1;
        state.trace_summary.search_calls.push(ProviderCallSummary {
            provider: response.provider.clone(),
            provider_type: ProviderType::Search,
            status: "ok".to_owned(),
            duration_ms: search_duration,
            retry_count: 0,
        });

        let new_evidence = self.evidence_from_search(
            &args.query,
            &response,
            state.candidate_evidence.len(),
            state.search_queries.len() + 1,
        );
        let result_count = response.results.len();
        let sources = self.source_traces_from_response(&response);
        state.search_queries.push(SearchQueryTrace {
            provider: response.provider.clone(),
            query: args.query.clone(),
            result_count,
            sources: sources.clone(),
            started_at: search_started_at.clone(),
            duration_ms: search_duration,
        });
        state.tool_calls.push(ToolCallTrace {
            tool_call_id: Some(tool_call.id.clone()),
            tool_name: tool_call.name.clone(),
            input_summary: Self::tool_input_summary(max_results),
            output_summary: format!("{result_count} result(s) from {}", response.provider),
            search: Some(SearchToolCallTrace {
                provider: response.provider.clone(),
                query: args.query.clone(),
                result_count,
                sources,
            }),
            started_at: search_started_at,
            duration_ms: search_duration,
        });

        let tool_output = self.search_result_message(&args.query, &response, &new_evidence);
        state.candidate_evidence.extend(new_evidence);

        Ok(ModelToolOutput::new(tool_call.id.clone(), tool_output))
    }

    fn finish(
        &self,
        content: &str,
        mut state: RuntimeState,
        budget: &AgentBudgetGuard,
        validator: &OutputValidator<'_>,
    ) -> std::result::Result<AgentRuntimeOutput, Box<AgentRuntimeFailure>> {
        let (result, _) = match validator.validate_content(content, &state.candidate_evidence) {
            Ok(result) => result,
            Err(error) => return Err(Box::new(Self::failure(error, state, budget))),
        };
        state.trace_summary.finished_at = Some(now_rfc3339());
        state.trace_summary.termination_reason = Some(TerminationReason::Completed);

        Ok(AgentRuntimeOutput {
            result,
            search_queries: state.search_queries,
            tool_calls: state.tool_calls,
            provider_usage: state.provider_usage,
            budget_usage: budget.usage(),
            trace_summary: state.trace_summary,
        })
    }

    fn initial_input(&self) -> Result<Vec<ModelInputItem>> {
        Ok(vec![
            ModelInputItem::message(ModelMessageRole::System, self.system_prompt()?),
            ModelInputItem::message(ModelMessageRole::User, self.user_prompt()),
        ])
    }

    fn system_prompt(&self) -> Result<String> {
        read_prompt_asset(&self.request.aspect.prompt_assets.aspect_agent_prompt_path)
    }

    fn user_prompt(&self) -> String {
        match serde_json::to_string_pretty(self.request) {
            Ok(request) => request,
            Err(error) => json!({ "serialization_error": error.to_string() }).to_string(),
        }
    }

    async fn complete_model(
        &self,
        previous_response_id: Option<String>,
        input: Vec<ModelInputItem>,
    ) -> Result<ModelResponse> {
        let model_override = self.request.aspect.model_override.as_ref();
        let request = ModelRequest {
            provider: model_override.map_or_else(String::new, |selector| selector.provider.clone()),
            model: model_override.and_then(|selector| selector.model.clone()),
            previous_response_id,
            input,
            tools: vec![search_model_tool()],
            temperature: self.request.model_policy.temperature,
            max_tokens: self.request.model_policy.max_tokens,
        };
        self.model_service
            .complete(request, &self.request.model_policy)
            .await
    }

    fn effective_search_policy(&self) -> SearchPolicy {
        let mut policy = self.request.search_policy.clone();
        if let Some(selector) = &self.request.aspect.search_override
            && !selector.providers.is_empty()
        {
            policy.allowed_providers.clone_from(&selector.providers);
            policy.preferred_providers.clone_from(&selector.providers);
        }
        policy
    }

    fn search_request(&self, query: &str, max_results: usize) -> SearchRequest {
        let policy = &self.request.search_policy;
        SearchRequest {
            query: query.to_owned(),
            max_results: max_results.min(policy.max_results_per_query),
            freshness: policy.freshness.clone(),
            language: policy.language.clone(),
            region: policy.region.clone(),
        }
    }

    fn evidence_from_search(
        &self,
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
                self.evidence_from_result(
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
        &self,
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
        &self,
        query: &str,
        response: &SearchResponse,
        evidence: &[Evidence],
    ) -> String {
        json!({
            "tool": "search",
            "query": query,
            "provider": response.provider,
            "result_count": response.results.len(),
            "results": evidence,
        })
        .to_string()
    }

    fn source_traces_from_response(&self, response: &SearchResponse) -> Vec<SearchSourceTrace> {
        response
            .results
            .iter()
            .map(|result| SearchSourceTrace {
                title: result.title.clone(),
                url: result.url.clone(),
            })
            .collect()
    }

    fn untraced_failure(error: Error) -> AgentRuntimeFailure {
        AgentRuntimeFailure {
            error,
            partial_trace: None,
        }
    }

    fn failure(
        error: Error,
        mut state: RuntimeState,
        budget: &AgentBudgetGuard,
    ) -> AgentRuntimeFailure {
        state.trace_summary.finished_at = Some(now_rfc3339());
        state.trace_summary.termination_reason = Some(Self::termination_reason_for_error(&error));
        let partial_trace = PartialTrace {
            trace_summary: state.trace_summary,
            search_queries: state.search_queries,
            tool_calls: state.tool_calls,
            provider_usage: state.provider_usage,
            budget_usage: budget.usage(),
            evidence_count: state.candidate_evidence.len(),
        };
        AgentRuntimeFailure {
            error,
            partial_trace: Some(partial_trace),
        }
    }

    fn termination_reason_for_error(error: &Error) -> TerminationReason {
        match error {
            Error::BudgetExceeded { message } if message.contains("timeout") => {
                TerminationReason::Timeout
            }
            Error::BudgetExceeded { .. } => TerminationReason::BudgetExceeded,
            Error::Timeout { .. } => TerminationReason::Timeout,
            Error::ToolPolicyDenied { .. } => TerminationReason::ToolPolicyDenied,
            Error::SchemaValidationFailed { .. } | Error::Json { .. } => {
                TerminationReason::SchemaValidationFailed
            }
            _ => TerminationReason::ProviderError,
        }
    }

    fn tool_input_summary(max_results: usize) -> String {
        format!("search query accepted, max_results={max_results}")
    }

    fn new_trace_summary(&self) -> TraceSummary {
        TraceSummary {
            trace_id: Uuid::new_v4().to_string(),
            root_span: format!("aspect_research:{}", self.request.aspect.aspect_id),
            started_at: now_rfc3339(),
            finished_at: None,
            model_calls: Vec::new(),
            search_calls: Vec::new(),
            termination_reason: None,
        }
    }
}

fn elapsed_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn read_prompt_asset(path: &str) -> Result<String> {
    let path = path.trim();
    if path.is_empty() {
        return Err(Error::InvalidInput {
            message: "prompt asset path must not be empty".to_owned(),
        });
    }

    let path_ref = Path::new(path);
    if !path_ref.is_absolute()
        && path_ref
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(Error::InvalidInput {
            message: "relative prompt asset path must not contain parent traversal".to_owned(),
        });
    }

    if path_ref
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("md")
    {
        return Err(Error::InvalidInput {
            message: "prompt asset path must point to a markdown file".to_owned(),
        });
    }

    let resolved_path = if path_ref.is_absolute() || path_ref.exists() {
        path_ref.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path_ref)
    };

    if !resolved_path.is_file() {
        return Err(Error::InvalidInput {
            message: format!(
                "prompt asset path must point to a file: {}",
                resolved_path.display()
            ),
        });
    }

    std::fs::read_to_string(&resolved_path).map_err(|source| Error::InvalidInput {
        message: format!(
            "unable to read prompt asset {}: {source}",
            resolved_path.display()
        ),
    })
}

struct RuntimeDeadline {
    started: Instant,
    timeout_ms: DurationLimitMs,
}

impl RuntimeDeadline {
    fn new(timeout_ms: DurationLimitMs) -> Self {
        Self {
            started: Instant::now(),
            timeout_ms,
        }
    }

    fn ensure_not_elapsed(&self) -> Result<()> {
        if self
            .timeout_ms
            .is_elapsed(elapsed_ms(self.started.elapsed()))
        {
            return Err(Error::BudgetExceeded {
                message: "agent runtime budget timeout exhausted".to_owned(),
            });
        }
        Ok(())
    }
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}

fn add_token_usage(total: &mut Option<TokenUsage>, delta: Option<TokenUsage>) {
    let Some(delta) = delta else {
        return;
    };
    let usage = total.get_or_insert_with(TokenUsage::default);
    usage.input_tokens = sum_optional(usage.input_tokens, delta.input_tokens);
    usage.output_tokens = sum_optional(usage.output_tokens, delta.output_tokens);
    usage.total_tokens = sum_optional(usage.total_tokens, delta.total_tokens);
}

fn sum_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.saturating_add(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}
