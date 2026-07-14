use std::sync::Arc;
use std::time::Instant;

use serde_json::json;

use crate::error_log_safe::{error_message_for_log, safe_model_identifier_for_log};
use crate::report::OutputValidator;
use crate::report::{
    AgentBudgetUsage, AspectReport, AspectResearchResult, Confidence, FailureDiagnostic,
    FailureStage, TokenUsage,
};
use crate::research::{ASPECT_PROMPT_MAX_BYTES, AspectPromptInput, EffectiveAspectPlan};
use crate::runtime::tools::{ToolExecutor, ToolPolicyGuard, ToolRuntimeState};
use crate::runtime::{
    AgentBudgetGuard, ResearchBudgetGuard, RuntimeDeadline, aspect_response_format, elapsed_ms,
};
use moe_research_error::{Error, Result};
use moe_research_model::ModelService;
use moe_research_model::{
    ModelInputItem, ModelMessageRole, ModelRequest, ModelResponse, ModelToolCall, ModelToolOutput,
};
use moe_research_search::SearchService;
use moe_research_web_fetch::WebFetchService;

pub(crate) struct AgentRuntime<'a> {
    model_service: &'a ModelService,
    search_service: &'a SearchService,
    web_fetch_service: &'a WebFetchService,
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
    pub(crate) diagnostic: FailureDiagnostic,
    pub(crate) partial_output: Option<AgentRuntimeOutput>,
}

struct RuntimeState {
    input: Vec<ModelInputItem>,
    replay_input: Vec<ModelInputItem>,
    previous_response_id: Option<String>,
    tools: ToolRuntimeState,
}

impl RuntimeState {
    fn new(input: Vec<ModelInputItem>) -> Self {
        Self {
            replay_input: input.clone(),
            input,
            previous_response_id: None,
            tools: ToolRuntimeState::new(),
        }
    }

    fn set_diagnostic(
        &mut self,
        stage: FailureStage,
        model_turn: Option<usize>,
        search_turn: Option<usize>,
    ) {
        self.tools.set_diagnostic(stage, model_turn, search_turn);
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
        web_fetch_service: &'a WebFetchService,
        request: &'a EffectiveAspectPlan,
        research_budget: Arc<ResearchBudgetGuard>,
    ) -> Self {
        Self {
            model_service,
            search_service,
            web_fetch_service,
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
        let tool_executor = ToolExecutor::new(
            self.search_service,
            self.web_fetch_service,
            self.request,
            self.research_budget.clone(),
        );
        let mut state = RuntimeState::new(self.initial_input());

        loop {
            let next_model_turn = budget.usage().turns_used.saturating_add(1);
            state.set_diagnostic(FailureStage::ModelTurn, Some(next_model_turn), None);
            let model_response = match deadline
                .run(self.complete_model_turn(&mut state, &mut budget, &tool_policy))
                .await
            {
                Ok(response) => response,
                Err(error) => return Err(self.failure(error, &state, &budget)),
            };
            if model_response.tool_calls.is_empty() {
                state.set_diagnostic(
                    FailureStage::OutputValidation,
                    Some(budget.usage().turns_used),
                    None,
                );
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

            state.set_diagnostic(
                FailureStage::ToolValidation,
                Some(budget.usage().turns_used),
                None,
            );
            if let Err(error) = Self::ensure_unique_tool_call_ids(&model_response.tool_calls) {
                return Err(self.failure(error, &state, &budget));
            }

            let mut tool_outputs = Vec::new();
            for tool_call in &model_response.tool_calls {
                let output = match deadline
                    .run(tool_executor.dispatch(
                        tool_call,
                        &tool_policy,
                        &mut budget,
                        &mut state.tools,
                        &deadline,
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
                    public: false,
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
            state.set_diagnostic(
                FailureStage::ResearchBudget,
                Some(budget.usage().turns_used),
                None,
            );
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
        state.tools.add_token_usage(usage.clone());
        if let Err(error) = self.research_budget.record_token_usage(usage.clone()) {
            state.set_diagnostic(
                FailureStage::ResearchBudget,
                Some(budget.usage().turns_used),
                None,
            );
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

    fn finish(
        &self,
        content: &str,
        mut state: RuntimeState,
        budget: &AgentBudgetGuard,
        validator: &OutputValidator<'_>,
    ) -> std::result::Result<AgentRuntimeOutput, Box<AgentRuntimeFailure>> {
        state.set_diagnostic(
            FailureStage::OutputValidation,
            Some(budget.usage().turns_used),
            None,
        );
        let (result, _) =
            match validator.validate_content(content, state.tools.candidate_evidence()) {
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
                        candidate_evidence_count = state.tools.candidate_evidence_count(),
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
            candidate_evidence_count = state.tools.candidate_evidence_count(),
            "agent runtime completed"
        );

        Ok(AgentRuntimeOutput {
            result,
            budget_usage,
            token_usage: state.tools.into_token_usage(),
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

    fn partial_output(
        &self,
        error: &Error,
        state: &RuntimeState,
        budget: &AgentBudgetGuard,
    ) -> Option<AgentRuntimeOutput> {
        if state.tools.candidate_evidence().is_empty() {
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
                selected_evidence: state
                    .tools
                    .candidate_evidence()
                    .iter()
                    .map(|evidence| evidence.id.clone())
                    .collect(),
                evidence: state.tools.candidate_evidence().to_vec(),
            },
            budget_usage: budget.usage(),
            token_usage: state.tools.token_usage(),
        })
    }

    fn untraced_failure(error: Error) -> AgentRuntimeFailure {
        AgentRuntimeFailure {
            error,
            diagnostic: FailureDiagnostic::new(FailureStage::RequestValidation, None, None),
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
            candidate_evidence_count = state.tools.candidate_evidence_count(),
            error_code = error.code().as_str(),
            error_message = %error_message_for_log(&error),
            retryable = error.retryable(),
            "agent runtime failed"
        );
        AgentRuntimeFailure {
            error,
            diagnostic: state.tools.diagnostic(),
            partial_output,
        }
    }
}
