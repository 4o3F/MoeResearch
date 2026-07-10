use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use schemars::schema_for;
use serde_json::json;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::error_log_safe::{error_message_for_log, safe_model_identifier_for_log};
use crate::limit::{DurationLimitMs, Limit};
use crate::policy::SearchPolicy;
use crate::report::{
    AgentBudgetUsage, AspectReport, AspectResearchResult, Confidence, Evidence, SourceType,
    TokenUsage,
};
use crate::research::{ASPECT_PROMPT_MAX_BYTES, AspectPromptInput, EffectiveAspectPlan};
use crate::runtime_budget::{AgentBudgetGuard, ResearchBudgetGuard};
use crate::tool_policy::{SearchToolArgs, ToolPolicyGuard};
use crate::validator::OutputValidator;
use moe_research_error::{Error, Result};
use moe_research_model::ModelService;
use moe_research_model::{
    JsonSchemaFormat, ModelInputItem, ModelMessageRole, ModelRequest, ModelResponse,
    ModelResponseFormat, ModelToolCall, ModelToolOutput,
};
use moe_research_search::SearchService;
use moe_research_search::{SearchRequest, SearchResponse, SearchResult};

pub(crate) struct AgentRuntime<'a> {
    model_service: &'a ModelService,
    search_service: &'a SearchService,
    request: &'a EffectiveAspectPlan,
    research_budget: Arc<ResearchBudgetGuard>,
}

#[derive(Debug)]
pub(crate) struct AgentRuntimeOutput {
    pub(crate) result: AspectResearchResult,
    pub(crate) budget_usage: AgentBudgetUsage,
    pub(crate) token_usage: Option<TokenUsage>,
}

#[derive(Debug)]
pub(crate) struct AgentRuntimeFailure {
    pub(crate) error: Error,
    pub(crate) partial_output: Option<AgentRuntimeOutput>,
}

struct RuntimeState {
    input: Vec<ModelInputItem>,
    replay_input: Vec<ModelInputItem>,
    previous_response_id: Option<String>,
    candidate_evidence: Vec<Evidence>,
    token_usage: Option<TokenUsage>,
}

impl RuntimeState {
    fn new(input: Vec<ModelInputItem>) -> Self {
        Self {
            replay_input: input.clone(),
            input,
            previous_response_id: None,
            candidate_evidence: Vec::new(),
            token_usage: None,
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
    pub(crate) fn new(
        model_service: &'a ModelService,
        search_service: &'a SearchService,
        request: &'a EffectiveAspectPlan,
        research_budget: Arc<ResearchBudgetGuard>,
    ) -> Self {
        Self {
            model_service,
            search_service,
            request,
            research_budget,
        }
    }

    pub(crate) async fn run(&self) -> Result<AgentRuntimeOutput, AgentRuntimeFailure> {
        self.validate_inline_prompt()
            .map_err(Self::untraced_failure)?;
        let effective_budget = self.effective_budget();
        let deadline = RuntimeDeadline::new(effective_budget.timeout_ms);
        let mut budget = AgentBudgetGuard::new(effective_budget).map_err(Self::untraced_failure)?;
        let tool_policy = ToolPolicyGuard::new(&self.request.task);
        let validator = OutputValidator::new(
            &self.request.task,
            &self.request.policy.evidence,
            &self.request.policy.output,
        );
        let search_policy = self.request.policy.search.clone();
        let search_provider = self.selected_search_provider();
        let mut state = RuntimeState::new(self.initial_input());

        loop {
            let model_response = match deadline
                .run(self.complete_model_turn(&mut state, &mut budget, &tool_policy))
                .await
            {
                Ok(response) => response,
                Err(error) => return Err(self.failure(error, &state, &budget)),
            };
            if model_response.tool_calls.is_empty() {
                let content = match model_response.content.as_deref().ok_or_else(|| {
                    Error::SchemaValidationFailed {
                        message: "model final response must include content".to_owned(),
                    }
                }) {
                    Ok(content) => content,
                    Err(error) => return Err(self.failure(error, &state, &budget)),
                };
                return self
                    .finish(content, state, &budget, &validator)
                    .map_err(|failure| *failure);
            }

            if let Err(error) = Self::ensure_unique_tool_call_ids(&model_response.tool_calls) {
                return Err(self.failure(error, &state, &budget));
            }

            let mut tool_outputs = Vec::new();
            for tool_call in &model_response.tool_calls {
                let output = match deadline
                    .run(self.execute_tool_call(
                        tool_call,
                        &tool_policy,
                        &mut budget,
                        &search_policy,
                        search_provider.as_deref(),
                        &mut state,
                    ))
                    .await
                {
                    Ok(output) => output,
                    Err(error) => return Err(self.failure(error, &state, &budget)),
                };
                tool_outputs.push(output);
            }
            state.append_model_output_and_tool_outputs(&model_response, tool_outputs);
        }
    }

    /// Re-checks the inline prompt invariants before the agent loop starts.
    ///
    /// The request normalizer enforces the same invariants at the workflow
    /// boundary. This method keeps the runtime entrypoint defensive because
    /// crate-internal callers can still construct effective plans. The check is
    /// O(1) for the empty case and O(n) only on the length comparison.
    ///
    /// # Errors
    /// Returns `Error::InvalidInput` when the prompt is empty or whitespace.
    /// Returns `Error::SchemaValidationFailed` when the prompt exceeds
    /// `ASPECT_PROMPT_MAX_BYTES`.
    fn validate_inline_prompt(&self) -> Result<()> {
        let prompt = &self.request.task.instructions;
        if prompt.trim().is_empty() {
            return Err(Error::InvalidInput {
                message: "task.instructions must not be empty".to_owned(),
            });
        }
        if prompt.len() > ASPECT_PROMPT_MAX_BYTES {
            return Err(Error::SchemaValidationFailed {
                message: format!("task.instructions exceeds {ASPECT_PROMPT_MAX_BYTES} bytes"),
            });
        }
        Ok(())
    }

    /// Rejects a model response whose tool-call list contains duplicate
    /// identifiers before any tool is dispatched.
    ///
    /// Duplicate `tool_call.id` values would let a misbehaving model issue
    /// the same call twice with different arguments and observe both budget
    /// consumption and output ordering, so we treat the situation as a
    /// policy violation and stop the agent loop. Validation is whole-batch:
    /// no tool is dispatched if any id repeats, so the orchestrator never
    /// observes partial side effects.
    ///
    /// # Errors
    /// Returns `Error::ToolPolicyDenied` with a generic message; the offending
    /// id is logged through `tracing` rather than echoed into the envelope.
    fn ensure_unique_tool_call_ids(tool_calls: &[ModelToolCall]) -> Result<()> {
        let mut seen = std::collections::HashSet::with_capacity(tool_calls.len());
        for tool_call in tool_calls {
            if !seen.insert(tool_call.id.as_str()) {
                tracing::warn!(
                    event = "tool_call_duplicate_rejected",
                    status = "rejected",
                    tool_call_id = %safe_model_identifier_for_log(&tool_call.id),
                    error_code = "tool_policy_denied",
                    error_message = "model returned duplicate tool call id",
                    retryable = false,
                    "duplicate tool call id rejected before dispatch"
                );
                return Err(Error::ToolPolicyDenied {
                    message: "model returned duplicate tool call id".to_owned(),
                });
            }
        }
        Ok(())
    }

    fn effective_budget(&self) -> crate::budget::AgentLimits {
        self.request.task.limits.clone()
    }

    async fn complete_model_turn(
        &self,
        state: &mut RuntimeState,
        budget: &mut AgentBudgetGuard,
        tool_policy: &ToolPolicyGuard,
    ) -> Result<ModelResponse> {
        budget.consume_model_turn()?;
        if let Err(error) = self.research_budget.try_consume_model_call() {
            tracing::warn!(
                request_id = %self.request.request_id,
                aspect_id = %self.request.task.id,
                error_code = error.code().as_str(),
                error_message = %error_message_for_log(&error),
                retryable = error.retryable(),
                status = "rejected",
                "research model budget rejected before model dispatch"
            );
            return Err(error);
        }
        let model_started = Instant::now();
        let model_response = match self
            .complete_model(
                state.previous_response_id.clone(),
                state.input.clone(),
                tool_policy.allowed_model_tools(),
            )
            .await
        {
            Ok(response) => response,
            Err(error) => {
                tracing::warn!(
                    request_id = %self.request.request_id,
                    aspect_id = %self.request.task.id,
                    duration_ms = elapsed_ms(model_started.elapsed()),
                    error_code = error.code().as_str(),
                    error_message = %error_message_for_log(&error),
                    retryable = error.retryable(),
                    status = "failed",
                    "model turn failed"
                );
                return Err(error);
            }
        };
        let model_duration = elapsed_ms(model_started.elapsed());
        let usage = model_response.usage.clone();
        add_token_usage(&mut state.token_usage, usage.clone());
        if let Err(error) = self.research_budget.record_token_usage(usage.clone()) {
            tracing::warn!(
                request_id = %self.request.request_id,
                aspect_id = %self.request.task.id,
                error_code = error.code().as_str(),
                error_message = %error_message_for_log(&error),
                retryable = error.retryable(),
                status = "rejected",
                "research token budget exhausted after model dispatch"
            );
            return Err(error);
        }

        tracing::info!(
            request_id = %self.request.request_id,
            aspect_id = %self.request.task.id,
            provider = %model_response.provider,
            duration_ms = model_duration,
            input_tokens = ?usage.as_ref().and_then(|usage| usage.input_tokens),
            output_tokens = ?usage.as_ref().and_then(|usage| usage.output_tokens),
            total_tokens = ?usage.as_ref().and_then(|usage| usage.total_tokens),
            status = "ok",
            "model turn completed"
        );

        Ok(model_response)
    }

    /// Dispatches a single model-requested tool call through the policy guard,
    /// the budget guard, and the relevant provider.
    ///
    /// This method is intentionally long-lived: it owns the search-tool fast
    /// path, evidence collection, and the policy-rejection branch. A finer
    /// split is planned alongside the Commit 3 tool-boundary rework; until
    /// then `clippy::too_many_lines` is suppressed locally.
    #[allow(clippy::too_many_lines)]
    async fn execute_tool_call(
        &self,
        tool_call: &ModelToolCall,
        tool_policy: &ToolPolicyGuard,
        budget: &mut AgentBudgetGuard,
        search_policy: &SearchPolicy,
        search_provider: Option<&str>,
        state: &mut RuntimeState,
    ) -> Result<ModelToolOutput> {
        let args = match tool_policy.validate_search_call(tool_call) {
            Ok(args) => args,
            Err(error) => {
                tracing::warn!(
                    request_id = %self.request.request_id,
                    aspect_id = %self.request.task.id,
                    tool_call_id = %safe_model_identifier_for_log(&tool_call.id),
                    tool_name = %safe_model_identifier_for_log(&tool_call.name),
                    error_code = error.code().as_str(),
                    error_message = %error_message_for_log(&error),
                    retryable = error.retryable(),
                    status = "denied",
                    "tool call denied"
                );
                return Err(error);
            }
        };
        let search_provider = search_provider.ok_or_else(|| Error::InvalidInput {
            message: "search provider must be explicitly selected".to_owned(),
        })?;
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

        let max_results = args
            .max_results
            .unwrap_or(self.request.policy.search.max_results_per_query);
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
        let search_started = Instant::now();
        let search_request = self.search_request(search_provider, &args, max_results);
        let response = match self
            .search_service
            .search(search_policy.apply_to(search_request)?)
            .await
        {
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

        let search_index = budget.usage().search_calls_used;
        let new_evidence = Self::evidence_from_search(
            &args.query,
            &response,
            state.candidate_evidence.len(),
            search_index,
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

        let tool_output = Self::search_result_message(&args.query, &response, &new_evidence);
        state.candidate_evidence.extend(new_evidence);

        Ok(ModelToolOutput::new(tool_call.id.clone(), tool_output))
    }

    fn finish(
        &self,
        content: &str,
        state: RuntimeState,
        budget: &AgentBudgetGuard,
        validator: &OutputValidator<'_>,
    ) -> std::result::Result<AgentRuntimeOutput, Box<AgentRuntimeFailure>> {
        let (result, _) = match validator.validate_content(content, &state.candidate_evidence) {
            Ok(result) => result,
            Err(error) => {
                let budget_usage = budget.usage();
                tracing::warn!(
                    event = "agent_finish_failed",
                    status = "failed",
                    request_id = %self.request.request_id,
                    aspect_id = %self.request.task.id,
                    turns_used = budget_usage.turns_used,
                    tool_calls_used = budget_usage.tool_calls_used,
                    search_calls_used = budget_usage.search_calls_used,
                    elapsed_ms = budget_usage.elapsed_ms,
                    candidate_evidence_count = state.candidate_evidence.len(),
                    error_code = error.code().as_str(),
                    error_message = %error_message_for_log(&error),
                    retryable = error.retryable(),
                    "agent finish failed"
                );
                return Err(Box::new(self.failure(error, &state, budget)));
            }
        };
        let budget_usage = budget.usage();
        tracing::info!(
            event = "agent_runtime_completed",
            status = "ok",
            request_id = %self.request.request_id,
            aspect_id = %self.request.task.id,
            turns_used = budget_usage.turns_used,
            tool_calls_used = budget_usage.tool_calls_used,
            search_calls_used = budget_usage.search_calls_used,
            elapsed_ms = budget_usage.elapsed_ms,
            candidate_evidence_count = state.candidate_evidence.len(),
            "agent runtime completed"
        );

        Ok(AgentRuntimeOutput {
            result,
            budget_usage,
            token_usage: state.token_usage,
        })
    }

    /// Builds the initial agent input: the inline aspect-agent system prompt
    /// followed by the narrow user-prompt projection.
    fn initial_input(&self) -> Vec<ModelInputItem> {
        vec![
            ModelInputItem::message(ModelMessageRole::System, self.system_prompt().to_owned()),
            ModelInputItem::message(ModelMessageRole::User, self.user_prompt()),
        ]
    }

    /// Returns the Layer 2 aspect-agent system prompt supplied inline by Layer 1.
    ///
    /// No filesystem IO is performed here; the string is taken verbatim from
    /// the MCP request after normalization has already enforced non-empty and
    /// size-bound invariants. Eliminating runtime prompt file IO closes
    /// the arbitrary-file-read attack surface that earlier path-based variants
    /// of this code carried.
    fn system_prompt(&self) -> &str {
        &self.request.task.instructions
    }

    fn user_prompt(&self) -> String {
        let prompt_input = AspectPromptInput::from(self.request);
        match serde_json::to_string_pretty(&prompt_input) {
            Ok(request) => request,
            Err(error) => json!({ "serialization_error": error.to_string() }).to_string(),
        }
    }

    async fn complete_model(
        &self,
        previous_response_id: Option<String>,
        input: Vec<ModelInputItem>,
        tools: Vec<moe_research_model::ModelTool>,
    ) -> Result<ModelResponse> {
        let request = ModelRequest {
            provider: self.request.task.model_provider.clone(),
            model: None,
            previous_response_id,
            input,
            tools,
            response_format: Some(aspect_response_format()),
            temperature: self.request.policy.model.temperature,
            max_tokens: self.request.policy.model.max_tokens,
        };
        self.model_service
            .complete(self.request.policy.model.apply_to(request)?)
            .await
    }

    fn selected_search_provider(&self) -> Option<String> {
        self.request
            .task
            .search_provider
            .clone()
            .filter(|provider| !provider.trim().is_empty())
    }

    fn search_request(
        &self,
        provider: &str,
        args: &SearchToolArgs,
        max_results: usize,
    ) -> SearchRequest {
        let policy = &self.request.policy.search;
        let mut request = SearchRequest::new(
            provider,
            &args.query,
            max_results.min(policy.max_results_per_query),
        );
        request.depth = args.depth;
        request.content_level = args.content_level;
        request.recency = args.recency;
        request.category = args.category;
        request
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
                Self::evidence_from_result(
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

    fn partial_output(
        &self,
        error: &Error,
        state: &RuntimeState,
        budget: &AgentBudgetGuard,
    ) -> Option<AgentRuntimeOutput> {
        if state.candidate_evidence.is_empty() {
            return None;
        }

        Some(AgentRuntimeOutput {
            result: AspectResearchResult {
                aspect_report: AspectReport {
                    aspect_id: self.request.task.id.clone(),
                    aspect_name: self.request.task.name.clone(),
                    question: self.request.task.question.clone(),
                    scope: self.request.task.scope.clone(),
                    findings: Vec::new(),
                    assumptions: Vec::new(),
                    risks: Vec::new(),
                    counterarguments: Vec::new(),
                    open_questions: Vec::new(),
                    confidence: Confidence::Low,
                    limitations: vec![format!(
                        "terminal failure [{}]: {}",
                        error.code().as_str(),
                        error.public_message()
                    )],
                },
                evidence: state.candidate_evidence.clone(),
            },
            budget_usage: budget.usage(),
            token_usage: state.token_usage.clone(),
        })
    }

    fn untraced_failure(error: Error) -> AgentRuntimeFailure {
        AgentRuntimeFailure {
            error,
            partial_output: None,
        }
    }

    /// Records a terminal agent failure with full diagnostic context and
    /// wraps the error in `AgentRuntimeFailure` so the caller can surface a
    /// per-aspect failure to the orchestrator.
    ///
    /// `state` is borrowed because only the candidate evidence count is read
    /// here; the runtime state is otherwise owned by the caller.
    fn failure(
        &self,
        error: Error,
        state: &RuntimeState,
        budget: &AgentBudgetGuard,
    ) -> AgentRuntimeFailure {
        let budget_usage = budget.usage();
        let partial_output = self.partial_output(&error, state, budget);
        tracing::warn!(
            event = "agent_runtime_failed",
            status = "failed",
            request_id = %self.request.request_id,
            aspect_id = %self.request.task.id,
            turns_used = budget_usage.turns_used,
            tool_calls_used = budget_usage.tool_calls_used,
            search_calls_used = budget_usage.search_calls_used,
            elapsed_ms = budget_usage.elapsed_ms,
            candidate_evidence_count = state.candidate_evidence.len(),
            error_code = error.code().as_str(),
            error_message = %error_message_for_log(&error),
            retryable = error.retryable(),
            "agent runtime failed"
        );
        AgentRuntimeFailure {
            error,
            partial_output,
        }
    }
}

fn elapsed_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
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

    fn remaining(&self) -> Result<Option<Duration>> {
        match self.timeout_ms {
            Limit::Unlimited => Ok(None),
            Limit::Limited(limit_ms) => {
                let elapsed = elapsed_ms(self.started.elapsed());
                if elapsed >= limit_ms {
                    return Err(Error::BudgetExceeded {
                        message: "agent runtime budget timeout exhausted".to_owned(),
                    });
                }
                Ok(Some(Duration::from_millis(limit_ms - elapsed)))
            }
        }
    }

    async fn run<F, T>(&self, future: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match self.remaining()? {
            None => future.await,
            Some(remaining) => tokio::time::timeout(remaining, future).await.map_err(|_| {
                Error::BudgetExceeded {
                    message: "agent runtime budget timeout exhausted".to_owned(),
                }
            })?,
        }
    }
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}

fn aspect_response_format() -> ModelResponseFormat {
    ModelResponseFormat::JsonSchema(JsonSchemaFormat {
        name: "aspect_research_result_v1".to_owned(),
        strict: true,
        schema: serde_json::to_value(schema_for!(AspectResearchResult))
            .expect("AspectResearchResult schema serializes"),
    })
}

fn add_token_usage(total: &mut Option<TokenUsage>, delta: Option<TokenUsage>) {
    let Some(delta) = delta else {
        return;
    };
    let usage = total.get_or_insert_with(TokenUsage::zero);
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
