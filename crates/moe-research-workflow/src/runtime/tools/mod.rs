mod policy;
mod search;
mod web_fetch;

use std::sync::Arc;

use crate::error_log_safe::{error_message_for_log, safe_model_identifier_for_log};
use crate::report::{Evidence, FailureDiagnostic, FailureStage, TokenUsage};
use crate::research::EffectiveAspectPlan;
use crate::runtime::{AgentBudgetGuard, ResearchBudgetGuard, RuntimeDeadline};
use moe_research_error::Result;
use moe_research_model::{ModelToolCall, ModelToolOutput};
use moe_research_search::SearchService;
use moe_research_web_fetch::WebFetchService;

use policy::ValidatedToolCall;
pub(crate) use policy::{SUPPORTED_ASPECT_TOOLS, ToolPolicyGuard};

pub(crate) struct ToolRuntimeState {
    candidate_evidence: Vec<Evidence>,
    token_usage: Option<TokenUsage>,
    diagnostic: FailureDiagnostic,
    retrieval_turns_started: usize,
}

impl ToolRuntimeState {
    pub(crate) fn new() -> Self {
        Self {
            candidate_evidence: Vec::new(),
            token_usage: None,
            diagnostic: FailureDiagnostic::new(FailureStage::RequestValidation, None, None),
            retrieval_turns_started: 0,
        }
    }

    pub(crate) fn begin_retrieval_turn(&mut self, model_turn: usize, stage: FailureStage) -> usize {
        self.retrieval_turns_started += 1;
        let retrieval_turn = self.retrieval_turns_started;
        self.set_diagnostic(stage, Some(model_turn), Some(retrieval_turn));
        retrieval_turn
    }

    pub(crate) fn set_diagnostic(
        &mut self,
        stage: FailureStage,
        model_turn: Option<usize>,
        search_turn: Option<usize>,
    ) {
        self.diagnostic = FailureDiagnostic::new(stage, model_turn, search_turn);
    }

    pub(crate) fn candidate_evidence(&self) -> &[Evidence] {
        &self.candidate_evidence
    }

    pub(crate) fn candidate_evidence_count(&self) -> usize {
        self.candidate_evidence.len()
    }

    pub(crate) fn token_usage(&self) -> Option<TokenUsage> {
        self.token_usage.clone()
    }

    pub(crate) fn add_token_usage(&mut self, usage: Option<TokenUsage>) {
        crate::runtime::add_token_usage(&mut self.token_usage, usage);
    }

    pub(crate) fn diagnostic(&self) -> FailureDiagnostic {
        self.diagnostic.clone()
    }

    pub(crate) fn into_token_usage(self) -> Option<TokenUsage> {
        self.token_usage
    }
}

pub(crate) struct ToolExecutor<'a> {
    search_service: &'a SearchService,
    web_fetch_service: &'a WebFetchService,
    request: &'a EffectiveAspectPlan,
    research_budget: Arc<ResearchBudgetGuard>,
}

impl<'a> ToolExecutor<'a> {
    pub(crate) fn new(
        search_service: &'a SearchService,
        web_fetch_service: &'a WebFetchService,
        request: &'a EffectiveAspectPlan,
        research_budget: Arc<ResearchBudgetGuard>,
    ) -> Self {
        Self {
            search_service,
            web_fetch_service,
            request,
            research_budget,
        }
    }

    pub(crate) async fn dispatch(
        &self,
        tool_call: &ModelToolCall,
        tool_policy: &ToolPolicyGuard,
        budget: &mut AgentBudgetGuard,
        state: &mut ToolRuntimeState,
        deadline: &RuntimeDeadline,
    ) -> Result<ModelToolOutput> {
        let model_turn = budget.usage().turns_used;
        state.set_diagnostic(FailureStage::ToolValidation, Some(model_turn), None);
        let validated = match tool_policy.validate_call(tool_call) {
            Ok(validated) => validated,
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

        match validated {
            ValidatedToolCall::Search(args) => {
                self.execute_search(tool_call, args, budget, state, model_turn)
                    .await
            }
            ValidatedToolCall::WebFetch(args) => {
                self.execute_web_fetch(tool_call, args, budget, state, model_turn, deadline)
                    .await
            }
        }
    }
}
