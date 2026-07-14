use std::time::Instant;

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::format_description::well_known::Rfc3339;

use crate::report::{Confidence, Evidence, FailureStage, SourceType};
use crate::runtime::{AgentBudgetGuard, RuntimeDeadline};
use moe_research_error::{Error, Result};
use moe_research_model::{ModelTool, ModelToolCall, ModelToolOutput};
use moe_research_web_fetch::{WebFetchAnswerOutcome, WebFetchDocumentOutcome};

use super::policy::tool_args_error;
use super::{ToolExecutor, ToolRuntimeState};

pub const WEB_FETCH_TOOL_NAME: &str = "web_fetch";

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WebFetchToolArgs {
    pub url: String,
    pub prompt: String,
}

pub(crate) fn validate_web_fetch_call(call: &ModelToolCall) -> Result<WebFetchToolArgs> {
    let Some(arguments) = call.arguments.as_object() else {
        return Err(tool_args_error(WEB_FETCH_TOOL_NAME, "invalid_structure"));
    };
    if arguments.len() != 2 || !arguments.contains_key("url") || !arguments.contains_key("prompt") {
        return Err(tool_args_error(WEB_FETCH_TOOL_NAME, "required_fields"));
    }
    let args: WebFetchToolArgs = serde_json::from_value(call.arguments.clone())
        .map_err(|_| tool_args_error(WEB_FETCH_TOOL_NAME, "invalid_structure"))?;
    if args.url.trim().is_empty() || args.prompt.trim().is_empty() {
        return Err(tool_args_error(WEB_FETCH_TOOL_NAME, "empty_field"));
    }
    Ok(args)
}

pub fn web_fetch_model_tool() -> ModelTool {
    ModelTool {
        name: WEB_FETCH_TOOL_NAME.to_owned(),
        description: "Fetch one known public URL and answer a focused prompt from its content."
            .to_owned(),
        input_schema: serde_json::to_value(schema_for!(WebFetchToolArgs))
            .expect("web_fetch tool schema serializes to JSON"),
    }
}

impl ToolExecutor<'_> {
    pub(super) async fn execute_web_fetch(
        &self,
        tool_call: &ModelToolCall,
        args: WebFetchToolArgs,
        budget: &mut AgentBudgetGuard,
        state: &mut ToolRuntimeState,
        model_turn: usize,
        deadline: &RuntimeDeadline,
    ) -> Result<ModelToolOutput> {
        let service = self.web_fetch_service;
        if !service.is_enabled() {
            return Err(Error::ToolPolicyDenied {
                message: "web_fetch is not enabled on this runtime".to_owned(),
                public: false,
            });
        }
        state.set_diagnostic(FailureStage::WebFetchBudget, Some(model_turn), None);
        budget.consume_tool_call()?;
        let retrieval_turn = state.begin_retrieval_turn(model_turn, FailureStage::WebFetchDispatch);
        let operation_deadline = deadline
            .remaining()?
            .and_then(|remaining| Instant::now().checked_add(remaining));
        let document = match service
            .fetch_document(&args.url, operation_deadline)
            .await?
        {
            WebFetchDocumentOutcome::Redirect { redirect_url } => {
                return Ok(ModelToolOutput::new(
                    tool_call.id.clone(),
                    json!({
                        "tool": WEB_FETCH_TOOL_NAME,
                        "status": "redirect",
                        "code": "cross_origin_redirect",
                        "retryable": false,
                        "message": "the redirect was not followed; call web_fetch again only if the target origin is intentional",
                        "requested_url": args.url,
                        "redirect_url": redirect_url,
                        "results": []
                    })
                    .to_string(),
                ));
            }
            WebFetchDocumentOutcome::SoftError(error) => {
                return Ok(ModelToolOutput::new(
                    tool_call.id.clone(),
                    json!({
                        "tool": WEB_FETCH_TOOL_NAME,
                        "status": "error",
                        "code": error.code,
                        "retryable": error.retryable,
                        "message": error.message,
                        "results": []
                    })
                    .to_string(),
                ));
            }
            WebFetchDocumentOutcome::Document(document) => document,
        };

        state.set_diagnostic(
            FailureStage::ResearchBudget,
            Some(model_turn),
            Some(retrieval_turn),
        );
        self.research_budget.try_consume_model_call()?;
        state.set_diagnostic(
            FailureStage::WebFetchDispatch,
            Some(model_turn),
            Some(retrieval_turn),
        );
        let answered = service
            .answer_document(&document, &args.prompt, operation_deadline)
            .await?;
        let answer = match answered {
            WebFetchAnswerOutcome::SoftError(error) => {
                state.add_token_usage(error.token_usage.clone());
                self.research_budget
                    .record_token_usage(error.token_usage.clone())?;
                return Ok(ModelToolOutput::new(
                    tool_call.id.clone(),
                    json!({
                        "tool": WEB_FETCH_TOOL_NAME,
                        "status": "error",
                        "code": error.code,
                        "retryable": error.retryable,
                        "message": error.message,
                        "results": []
                    })
                    .to_string(),
                ));
            }
            WebFetchAnswerOutcome::Answer(answer) => answer,
        };
        state.add_token_usage(answer.token_usage.clone());
        self.research_budget
            .record_token_usage(answer.token_usage.clone())?;
        if !answer.found {
            return Ok(ModelToolOutput::new(
                tool_call.id.clone(),
                json!({
                    "tool": WEB_FETCH_TOOL_NAME,
                    "status": "ok",
                    "found": false,
                    "answer": answer.answer,
                    "results": []
                })
                .to_string(),
            ));
        }

        let evidence_index = state.candidate_evidence.len().saturating_add(1);
        let evidence = Evidence {
            id: format!("ev-{retrieval_turn}-{evidence_index}"),
            source_title: if document.title.trim().is_empty() {
                WEB_FETCH_TOOL_NAME.to_owned()
            } else {
                document.title.clone()
            },
            url: Some(document.final_url.clone()),
            provider: WEB_FETCH_TOOL_NAME.to_owned(),
            query: args.prompt,
            snippet: answer.supporting_excerpt.unwrap_or_default(),
            summary: answer.answer,
            published_at: None,
            retrieved_at: document
                .retrieved_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned()),
            supports_findings: Vec::new(),
            source_type: SourceType::Unknown,
            confidence: Confidence::Medium,
        };
        let output = json!({
            "tool": WEB_FETCH_TOOL_NAME,
            "status": "ok",
            "found": true,
            "answer": evidence.summary.clone(),
            "result_count": 1,
            "results": [&evidence]
        })
        .to_string();
        state.candidate_evidence.push(evidence);
        Ok(ModelToolOutput::new(tool_call.id.clone(), output))
    }
}
