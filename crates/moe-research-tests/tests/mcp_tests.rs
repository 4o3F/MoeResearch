mod support;

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use moe_research_error::{Error, Result};
use moe_research_mcp::MoeResearchMcpServer;
use moe_research_mcp::{ToolEnvelope, ToolError, ToolErrorCode, ToolStatus};
use moe_research_model::ModelProvider;
use moe_research_model::ModelService;
use moe_research_model::{ModelInputItem, ModelRequest, ModelResponse, ModelToolCall};
use moe_research_workflow::Limit;
use moe_research_workflow::{AgentLimits, BudgetConfig, ResearchLimits};
use moe_research_workflow::{AspectResearchResult, FailureStage, TokenUsage};
use rmcp::ServerHandler;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::schemars::schema_for;
use serde_json::json;
use support::research::{
    Services, aspect_field, aspect_request, deep_request, final_response,
    first_evidence_from_tool_output, medium_result_json, services, services_with_delay,
    services_with_token_usage, static_search_service, tool_response, unlimited_budget_config,
};

struct SequenceModelProvider {
    calls: Arc<AtomicUsize>,
    responses: Mutex<VecDeque<ModelResponse>>,
}

#[async_trait]
impl ModelProvider for SequenceModelProvider {
    fn name(&self) -> &'static str {
        "model"
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut response = self
            .responses
            .lock()
            .expect("responses lock")
            .pop_front()
            .ok_or_else(|| Error::Internal {
                message: "missing fake model response".to_owned(),
            })?;
        if response.content.as_deref() == Some("__RESULT_FROM_TOOL_OUTPUT__") {
            let aspect_id = aspect_field(&request.input, "Aspect ID");
            let aspect_name = aspect_field(&request.input, "Aspect name");
            response.content = Some(medium_result_json(
                &aspect_id,
                &aspect_name,
                first_evidence_from_tool_output(&request.input),
            ));
        } else if response.content.as_deref()
            == Some("__UNKNOWN_SELECTED_EVIDENCE_FROM_TOOL_OUTPUT__")
        {
            let aspect_id = aspect_field(&request.input, "Aspect ID");
            let aspect_name = aspect_field(&request.input, "Aspect name");
            let mut evidence = first_evidence_from_tool_output(&request.input);
            evidence.id = "ev-not-returned-by-search".to_owned();
            response.content = Some(medium_result_json(&aspect_id, &aspect_name, evidence));
        } else if response.content.as_deref() == Some("__TRANSIENT_PROVIDER_UNAVAILABLE__") {
            return Err(Error::ProviderUnavailable {
                provider: "openai".to_owned(),
                message: "SSE stream ended with terminal failure".to_owned(),
                retryable: true,
            });
        }
        Ok(response)
    }
}

fn sequence_services(responses: Vec<ModelResponse>) -> Services {
    let model_calls = Arc::new(AtomicUsize::new(0));
    let search_calls = Arc::new(AtomicUsize::new(0));
    let mut model = ModelService::new();
    model.register(SequenceModelProvider {
        calls: model_calls.clone(),
        responses: Mutex::new(responses.into()),
    });
    let search = static_search_service(search_calls.clone());

    Services {
        model,
        search,
        model_calls,
        search_calls,
        max_in_flight: Arc::new(AtomicUsize::new(0)),
    }
}

fn mcp_server(services: Services) -> MoeResearchMcpServer {
    MoeResearchMcpServer::new(services.model, services.search, unlimited_budget_config())
}

fn mcp_server_with_budget(services: Services, budget_config: BudgetConfig) -> MoeResearchMcpServer {
    MoeResearchMcpServer::new(services.model, services.search, budget_config)
}

#[test]
fn public_tool_lookup_exposes_m6_contract_tools() {
    let server = mcp_server(services(&[]));

    assert!(server.get_tool("aspect_research").is_some());
    assert!(server.get_tool("deep_research").is_some());
    assert!(server.get_tool("serve_stdio").is_none());
    assert!(server.get_tool("search").is_none());
}

#[test]
fn public_tool_descriptions_explain_direct_payload_shape() {
    let server = mcp_server(services(&[]));

    let aspect = server
        .get_tool("aspect_research")
        .expect("aspect research tool");
    let aspect_description = aspect.description.as_deref().unwrap_or("");
    assert!(aspect_description.contains("AspectResearchRequest"));
    assert!(aspect_description.contains("request object directly"));
    assert!(aspect_description.contains("Do not wrap"));
    assert!(aspect_description.contains("schema_version"));
    assert!(aspect_description.contains("instructions"));
    assert!(aspect_description.contains("model_provider"));
    assert!(aspect_description.contains("search_provider"));

    let deep = server
        .get_tool("deep_research")
        .expect("deep research tool");
    let deep_description = deep.description.as_deref().unwrap_or("");
    assert!(deep_description.contains("DeepResearchRequest"));
    assert!(deep_description.contains("request object directly"));
    assert!(deep_description.contains("Do not wrap"));
    assert!(deep_description.contains("schema_version"));
    assert!(deep_description.contains("instructions"));
    assert!(deep_description.contains("task.aspects"));
    assert!(deep_description.contains("limits"));
    assert!(deep_description.contains("policy"));
}

#[test]
fn aspect_research_tool_schema_uses_limit_wire_format() {
    let server = mcp_server(services(&[]));
    let tool = server
        .get_tool("aspect_research")
        .expect("aspect research tool");
    let schema = serde_json::Value::Object(tool.input_schema.as_ref().clone());
    let limit = schema.pointer("/$defs/Limit").expect("shared limit schema");

    assert_eq!(limit.get("type"), Some(&json!(["integer", "null"])));
    assert_eq!(limit.get("minimum"), Some(&json!(-1)));
    let schema_json = schema.to_string();
    assert!(!schema_json.contains("Limited"));
    assert!(!schema_json.contains("\"format\":\"uint"));
    assert!(
        schema_json.contains("\"minimum\":0"),
        "unsigned integer schema fields should advertise a non-negative minimum"
    );
}

#[test]
fn tool_envelope_schema_omits_trace_payloads() {
    let schema = schema_for!(ToolEnvelope<AspectResearchResult>);
    let schema = serde_json::to_value(&schema).expect("schema json");
    let properties = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .expect("schema properties");

    assert!(!properties.contains_key("partial_trace"));
    assert!(!properties.contains_key("trace_summary"));
    assert!(!properties.contains_key("warnings"));
    let schema_json = schema.to_string();
    assert!(!schema_json.contains("PartialTrace"));
    assert!(!schema_json.contains("TraceSummary"));
    assert!(!schema_json.contains("Sse"));
    assert!(!schema_json.contains("stream"));
    assert!(schema_json.contains("failed_aspects"));
}

#[test]
fn tool_error_schema_exposes_structured_failure_diagnostic() {
    let schema = serde_json::to_value(schema_for!(ToolError)).expect("schema json");
    let properties = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .expect("tool error properties");
    assert!(properties.contains_key("diagnostic"));

    let diagnostic = schema
        .pointer("/$defs/FailureDiagnostic/properties")
        .and_then(serde_json::Value::as_object)
        .expect("failure diagnostic schema");
    assert!(diagnostic.contains_key("stage"));
    assert!(diagnostic.contains_key("model_turn"));
    assert!(diagnostic.contains_key("search_turn"));
    assert!(
        !serde_json::to_string(&schema)
            .expect("schema text")
            .contains("tool_call_id")
    );
}

#[tokio::test]
async fn aspect_research_success_returns_ok_envelope() {
    let services = services(&[]);
    let model_calls = services.model_calls.clone();
    let search_calls = services.search_calls.clone();
    let envelope = mcp_server(services)
        .aspect_research(Parameters(aspect_request()))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Ok);
    assert_eq!(envelope.request_id, "request-1");
    assert!(envelope.data.is_some());
    assert!(envelope.error.is_none());
    assert!(envelope.run_id.is_none());
    assert_eq!(model_calls.load(Ordering::SeqCst), 2);
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn aspect_research_invalid_input_returns_failed_envelope() {
    let mut request = aspect_request();
    request.task.question.clear();
    let envelope = mcp_server(services(&[]))
        .aspect_research(Parameters(request))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    assert!(envelope.data.is_none());
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::InvalidInput);
    assert_eq!(error.diagnostic.stage, FailureStage::RequestValidation);
    assert_eq!(error.diagnostic.model_turn, None);
    assert_eq!(error.diagnostic.search_turn, None);
    assert!(!error.retryable);
    assert!(error.failed_aspects.is_empty());
}

#[tokio::test]
async fn aspect_research_budget_failure_envelope_returns_partial_with_tool_error() {
    let mut request = aspect_request();
    request.task.limits.max_search_calls = Limit::limited(2);
    let services = sequence_services(vec![tool_response(), tool_response(), tool_response()]);
    let search_calls = services.search_calls.clone();

    let envelope = mcp_server(services)
        .aspect_research(Parameters(request))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Partial);
    assert!(envelope.run_id.is_none());
    let data = envelope.data.expect("partial aspect data");
    assert_eq!(data.evidence.len(), 2);
    assert!(data.aspect_report.findings.is_empty());
    assert!(data.aspect_report.limitations[0].contains("budget_exceeded"));
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::BudgetExceeded);
    assert!(
        error.message.contains("budget exceeded")
            && error.message.contains("max_search_calls")
            && error.message.contains("effective cap 2"),
        "unexpected budget message: {}",
        error.message
    );
    assert_eq!(error.aspect_id.as_deref(), Some("aspect-1"));
    assert!(!error.retryable);
    assert!(error.failed_aspects.is_empty());
    assert_eq!(search_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn aspect_research_partial_failure_disabled_returns_failed_envelope() {
    let mut request = aspect_request();
    request.policy.execution.allow_partial_results = false;
    request.task.limits.max_search_calls = Limit::limited(1);
    let services = sequence_services(vec![tool_response(), tool_response()]);
    let search_calls = services.search_calls.clone();

    let envelope = mcp_server(services)
        .aspect_research(Parameters(request))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    assert!(envelope.data.is_none());
    assert!(envelope.run_id.is_none());
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::BudgetExceeded);
    assert_eq!(error.aspect_id.as_deref(), Some("aspect-1"));
    assert!(error.failed_aspects.is_empty());
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn aspect_research_unknown_selected_evidence_returns_partial_with_frozen_evidence() {
    let services = sequence_services(vec![
        tool_response(),
        final_response("__UNKNOWN_SELECTED_EVIDENCE_FROM_TOOL_OUTPUT__".to_owned()),
    ]);

    let envelope = mcp_server(services)
        .aspect_research(Parameters(aspect_request()))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Partial);
    assert!(envelope.run_id.is_none());

    let data = envelope.data.expect("partial aspect data");
    assert!(data.aspect_report.findings.is_empty());
    assert_eq!(data.evidence.len(), 1);
    assert_eq!(data.evidence[0].source_title, "Shared Source");
    assert_eq!(data.evidence[0].provider, "searcher");
    assert_eq!(data.evidence[0].snippet, "shared snippet");
    assert_eq!(data.evidence[0].summary, "shared summary");

    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::SchemaValidationFailed);
    assert_eq!(error.aspect_id.as_deref(), Some("aspect-1"));
    assert!(!error.retryable);
    assert!(error.message.contains("unknown_selected_evidence"));
    assert!(error.message.contains("selected_evidence[0]"));
    assert!(error.message.contains("not present in search tool output"));
}

#[tokio::test]
async fn aspect_research_unknown_selected_evidence_fails_when_partials_disabled() {
    let mut request = aspect_request();
    request.policy.execution.allow_partial_results = false;
    let services = sequence_services(vec![
        tool_response(),
        final_response("__UNKNOWN_SELECTED_EVIDENCE_FROM_TOOL_OUTPUT__".to_owned()),
    ]);

    let envelope = mcp_server(services)
        .aspect_research(Parameters(request))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    assert!(envelope.data.is_none());
    assert!(envelope.run_id.is_none());

    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::SchemaValidationFailed);
    assert_eq!(error.aspect_id.as_deref(), Some("aspect-1"));
    assert!(!error.retryable);
}

#[tokio::test]
async fn aspect_research_transient_provider_unavailable_partial_marks_tool_error_retryable() {
    let services = sequence_services(vec![
        tool_response(),
        final_response("__TRANSIENT_PROVIDER_UNAVAILABLE__".to_owned()),
    ]);

    let envelope = mcp_server(services)
        .aspect_research(Parameters(aspect_request()))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Partial);
    assert!(envelope.run_id.is_none());

    let data = envelope.data.expect("partial aspect data");
    assert!(data.aspect_report.findings.is_empty());
    assert_eq!(data.evidence.len(), 1);

    let error = envelope.error.expect("provider error");
    assert_eq!(error.code, ToolErrorCode::ProviderUnavailable);
    assert_eq!(error.message, "provider unavailable");
    assert_eq!(error.aspect_id.as_deref(), Some("aspect-1"));
    assert_eq!(error.diagnostic.stage, FailureStage::ModelTurn);
    assert_eq!(error.diagnostic.model_turn, Some(2));
    assert_eq!(error.diagnostic.search_turn, None);
    assert!(error.retryable);
    assert!(error.failed_aspects.is_empty());
}

#[tokio::test]
async fn aspect_research_policy_provider_unavailable_marks_tool_error_not_retryable() {
    let mut request = aspect_request();
    request.policy.model.allowed_providers = vec!["other".to_owned()];

    let envelope = mcp_server(services(&[]))
        .aspect_research(Parameters(request))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    assert!(envelope.data.is_none());

    let error = envelope.error.expect("provider error");
    assert_eq!(error.code, ToolErrorCode::ProviderUnavailable);
    assert_eq!(error.message, "provider unavailable");
    assert_eq!(error.aspect_id.as_deref(), Some("aspect-1"));
    assert!(!error.retryable);
    assert!(error.failed_aspects.is_empty());
}

#[tokio::test]
async fn aspect_research_config_research_budget_failure_returns_tool_error() {
    let services = sequence_services(vec![tool_response()]);
    let search_calls = services.search_calls.clone();
    let budget_config = BudgetConfig {
        research: ResearchLimits {
            max_total_search_calls: Limit::limited(0),
            ..ResearchLimits::unlimited()
        },
        per_agent: AgentLimits::unlimited(),
    };

    let envelope = mcp_server_with_budget(services, budget_config)
        .aspect_research(Parameters(aspect_request()))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    assert!(envelope.data.is_none());
    assert!(envelope.run_id.is_none());
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::BudgetExceeded);
    assert_eq!(error.aspect_id.as_deref(), Some("aspect-1"));
    assert_eq!(error.diagnostic.stage, FailureStage::ResearchBudget);
    assert_eq!(error.diagnostic.model_turn, Some(1));
    assert_eq!(error.diagnostic.search_turn, Some(1));
    assert!(error.failed_aspects.is_empty());
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn aspect_research_token_budget_failure_returns_research_budget_diagnostic() {
    let services = services_with_token_usage(
        &[],
        Some(TokenUsage {
            input_tokens: None,
            output_tokens: None,
            total_tokens: Some(200),
        }),
    );
    let model_calls = services.model_calls.clone();
    let search_calls = services.search_calls.clone();
    let budget_config = BudgetConfig {
        research: ResearchLimits {
            max_tokens: Limit::limited(100),
            ..ResearchLimits::unlimited()
        },
        per_agent: AgentLimits::unlimited(),
    };

    let envelope = mcp_server_with_budget(services, budget_config)
        .aspect_research(Parameters(aspect_request()))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    assert!(envelope.data.is_none());
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::BudgetExceeded);
    assert_eq!(error.diagnostic.stage, FailureStage::ResearchBudget);
    assert_eq!(error.diagnostic.model_turn, Some(1));
    assert_eq!(error.diagnostic.search_turn, None);
    assert_eq!(model_calls.load(Ordering::SeqCst), 1);
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn deep_research_all_success_returns_ok_envelope() {
    let envelope = mcp_server(services(&[]))
        .deep_research(Parameters(deep_request(2)))
        .await
        .0;
    let data = envelope.data.expect("deep data");

    assert_eq!(envelope.status, ToolStatus::Ok);
    assert_eq!(envelope.request_id, "request-1");
    assert!(envelope.error.is_none());
    assert_eq!(envelope.run_id.as_deref(), Some(data.run_id.as_str()));
    assert_eq!(data.completed_aspects.len(), 2);
    assert!(data.failed_aspects.is_empty());
}

/// Partial deep-research envelopes MUST report failed aspects with stable
/// snake_case error codes matching the public `ToolErrorCode` contract.
#[tokio::test]
async fn deep_research_partial_success_returns_partial_envelope() {
    let envelope = mcp_server(services(&["aspect-2"]))
        .deep_research(Parameters(deep_request(3)))
        .await
        .0;
    let data = envelope.data.expect("partial deep data");

    assert_eq!(envelope.status, ToolStatus::Partial);
    assert!(envelope.error.is_none());
    assert_eq!(data.completed_aspects.len(), 2);
    assert_eq!(data.failed_aspects.len(), 1);
    assert_eq!(data.failed_aspects[0].aspect_id, "aspect-2");
    assert_eq!(
        data.failed_aspects[0].error_code,
        "schema_validation_failed"
    );
}

#[tokio::test]
async fn deep_research_post_run_budget_partial_returns_partial_envelope_with_data() {
    let mut request = deep_request(3);
    request.limits.max_concurrent_agents = Limit::limited(1);
    request.limits.total_timeout_ms = Limit::limited(110);
    for aspect in &mut request.task.aspects {
        aspect.limits.timeout_ms = Limit::limited(110);
    }
    request.policy.execution.fail_fast = true;

    let envelope = mcp_server(services_with_delay(
        &["aspect-2"],
        Duration::from_millis(30),
    ))
    .deep_research(Parameters(request))
    .await
    .0;

    assert_eq!(envelope.status, ToolStatus::Partial);
    assert!(envelope.error.is_none());

    let data = envelope.data.expect("partial deep data");
    assert_eq!(envelope.run_id.as_deref(), Some(data.run_id.as_str()));
    assert_eq!(data.completed_aspects, vec!["aspect-1".to_owned()]);
    assert_eq!(data.failed_aspects.len(), 2);
    assert_eq!(data.failed_aspects[0].aspect_id, "aspect-2");
    assert_eq!(data.failed_aspects[1].aspect_id, "aspect-3");
    assert_eq!(data.failed_aspects[1].error_code, "budget_exceeded");
    assert!(!data.failed_aspects[1].aspect_id.starts_with("__"));
    assert_eq!(data.coverage_summary.requested_aspects, 3);
    assert_eq!(data.coverage_summary.completed_aspects, 1);
    assert_eq!(data.coverage_summary.failed_aspects, 2);
}

#[tokio::test]
async fn deep_research_all_failed_with_partial_evidence_returns_partial_envelope() {
    let envelope = mcp_server(services(&["aspect-1", "aspect-2"]))
        .deep_research(Parameters(deep_request(2)))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Partial);
    assert!(envelope.error.is_none());
    let data = envelope.data.expect("partial deep data");
    assert!(data.completed_aspects.is_empty());
    assert!(data.aspect_reports.is_empty());
    assert_eq!(data.failed_aspects.len(), 2);
    assert_eq!(data.failed_aspects[0].aspect_id, "aspect-1");
    assert_eq!(data.failed_aspects[1].aspect_id, "aspect-2");
    assert_eq!(
        data.failed_aspects[0].error_code,
        "schema_validation_failed"
    );
    assert_eq!(
        data.failed_aspects[1].error_code,
        "schema_validation_failed"
    );
    assert_eq!(data.evidence_index.len(), 2);
    assert_eq!(data.coverage_summary.completed_aspects, 0);
}

#[tokio::test]
async fn deep_research_duplicate_aspect_ids_is_top_level_invalid_input() {
    let mut request = deep_request(2);
    request.task.aspects[1].id = request.task.aspects[0].id.clone();

    let envelope = mcp_server(services(&[]))
        .deep_research(Parameters(request))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::InvalidInput);
    assert!(error.failed_aspects.is_empty());
}

#[tokio::test]
async fn deep_research_all_agent_budget_failures_include_failed_aspects() {
    let mut request = deep_request(2);
    request.limits.max_concurrent_agents = Limit::limited(1);
    for task in &mut request.task.aspects {
        task.limits.max_search_calls = Limit::limited(0);
    }
    let services = services(&[]);
    let model_calls = services.model_calls.clone();
    let search_calls = services.search_calls.clone();

    let envelope = mcp_server(services)
        .deep_research(Parameters(request))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    assert!(envelope.data.is_none());
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::PartialResult);
    assert_eq!(error.diagnostic.stage, FailureStage::ResultAggregation);
    assert_eq!(error.failed_aspects.len(), 2);
    assert!(
        error
            .failed_aspects
            .iter()
            .all(|failure| failure.error_code == "budget_exceeded")
    );
    assert!(error.failed_aspects.iter().all(|failure| {
        failure.message.contains("budget exceeded") && failure.message.contains("max_search_calls")
    }));
    assert!(error.failed_aspects.iter().all(|failure| {
        failure.diagnostic.stage == FailureStage::SearchBudget
            && failure.diagnostic.model_turn == Some(1)
            && failure.diagnostic.search_turn == Some(1)
    }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 2);
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn error_retryability_mapping_is_stable() {
    assert!(
        Error::NetworkFailed {
            message: "temporary network failure".to_owned(),
        }
        .retryable()
    );
    assert!(
        Error::Timeout {
            message: "deadline exceeded".to_owned(),
        }
        .retryable()
    );
    assert!(
        Error::HttpStatus {
            status: 503,
            message: "service unavailable".to_owned(),
            retryable: true,
        }
        .retryable()
    );
    assert!(
        Error::ProviderUnavailable {
            provider: "openai".to_owned(),
            message: "runtime transient".to_owned(),
            retryable: true,
        }
        .retryable()
    );
    assert!(
        !Error::ProviderUnavailable {
            provider: "openai".to_owned(),
            message: "not configured".to_owned(),
            retryable: false,
        }
        .retryable()
    );
    assert!(
        !Error::InvalidInput {
            message: "missing question".to_owned(),
        }
        .retryable()
    );
}

/// `SchemaValidationFailed.message` is public-safe validator output and MUST
/// survive into the MCP `ToolError.message`, while raw JSON conversion errors
/// remain generic because they may include parser/provider details.
#[test]
fn schema_validation_failed_preserves_message_but_json_stays_generic() {
    let validation_message = concat!(
        "final output failed validation: unknown_selected_evidence ",
        "at selected_evidence[0] (selected evidence was not present in search tool output)"
    );
    let validation_error = Error::SchemaValidationFailed {
        message: validation_message.to_owned(),
    };

    assert_eq!(validation_error.code().as_str(), "schema_validation_failed");
    assert_eq!(validation_error.public_message(), validation_message);

    let json_source = serde_json::from_str::<serde_json::Value>("{not json")
        .expect_err("malformed JSON must fail");
    let json_error = Error::Json {
        source: json_source,
    };

    assert_eq!(json_error.code().as_str(), "schema_validation_failed");
    assert_eq!(json_error.public_message(), "schema validation failed");
}

#[tokio::test]
async fn aspect_research_schema_failure_envelope_preserves_validator_message() {
    let invalid_result = json!({
        "aspect_report": {
            "aspect_id": "wrong-aspect",
            "aspect_name": "Aspect 1",
            "question": "Question 1?",
            "scope": ["scope"],
            "findings": [],
            "assumptions": [],
            "risks": [],
            "counterarguments": [],
            "open_questions": [],
            "confidence": "medium",
            "limitations": []
        },
        "selected_evidence": []
    })
    .to_string();

    let envelope = mcp_server(sequence_services(vec![final_response(invalid_result)]))
        .aspect_research(Parameters(aspect_request()))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::SchemaValidationFailed);
    assert!(error.message.contains("aspect_id_mismatch"));
    assert!(error.message.contains("aspect_report.aspect_id"));
    assert!(
        error
            .message
            .contains("report aspect_id does not match requested aspect")
    );
}

#[tokio::test]
async fn aspect_research_envelope_surfaces_safe_intent_policy_diagnostic() {
    let mut request = aspect_request();
    request.policy.search.depth = Some(moe_research_workflow::SearchDepth::Balanced);
    let tool_call = ModelToolCall {
        id: "call-1".to_owned(),
        name: "search".to_owned(),
        arguments: json!({
            "query": "must-not-appear-in-the-envelope",
            "intent": {
                "source_focus": "general",
                "timeliness": "any",
                "coverage": "broad",
                "detail": "standard"
            }
        }),
    };
    let response = ModelResponse {
        provider: "model".to_owned(),
        model: None,
        response_id: None,
        content: None,
        tool_calls: vec![tool_call.clone()],
        output_items: vec![ModelInputItem::tool_call(tool_call)],
        usage: None,
    };

    let envelope = mcp_server(sequence_services(vec![response]))
        .aspect_research(Parameters(request))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::ToolPolicyDenied);
    assert_eq!(
        error.message,
        "search intent conflicts with policy [branch=intent_policy key=coverage]"
    );
    assert!(!error.message.contains("must-not-appear-in-the-envelope"));
}

#[test]
fn non_public_tool_policy_message_remains_redacted() {
    let error = Error::ToolPolicyDenied {
        message: "untrusted-input".to_owned(),
        public: false,
    };

    assert_eq!(error.public_message(), "tool policy denied request");
}

/// `ToolErrorCode::as_str` MUST emit the same snake_case string that serde
/// produces under `#[serde(rename_all = "snake_case")]`, so external clients
/// can rely on either path to dispatch on the same identifier.
#[test]
fn tool_error_code_as_str_matches_serde() {
    let codes = [
        ToolErrorCode::InvalidInput,
        ToolErrorCode::UnsupportedSchemaVersion,
        ToolErrorCode::ConfigInvalid,
        ToolErrorCode::ProviderUnavailable,
        ToolErrorCode::NetworkFailed,
        ToolErrorCode::BudgetExceeded,
        ToolErrorCode::ToolPolicyDenied,
        ToolErrorCode::SchemaValidationFailed,
        ToolErrorCode::Timeout,
        ToolErrorCode::PartialResult,
        ToolErrorCode::Internal,
    ];
    for code in codes {
        let serde_value = serde_json::to_value(code).expect("serialize");
        let serde_str = serde_value.as_str().expect("string");
        assert_eq!(serde_str, code.as_str(), "mismatch for {code:?}");
    }
}

/// Every transport-neutral `ErrorCode` must map 1:1 onto `ToolErrorCode` with
/// identical `as_str()` values. Adding a new `ErrorCode` without `ToolErrorCode`
/// (or vice versa) must fail this test.
#[test]
fn tool_error_code_mirrors_error_code_1to1() {
    use moe_research_error::ErrorCode;

    let pairs = [
        (ErrorCode::InvalidInput, ToolErrorCode::InvalidInput),
        (
            ErrorCode::UnsupportedSchemaVersion,
            ToolErrorCode::UnsupportedSchemaVersion,
        ),
        (ErrorCode::ConfigInvalid, ToolErrorCode::ConfigInvalid),
        (
            ErrorCode::ProviderUnavailable,
            ToolErrorCode::ProviderUnavailable,
        ),
        (ErrorCode::NetworkFailed, ToolErrorCode::NetworkFailed),
        (ErrorCode::BudgetExceeded, ToolErrorCode::BudgetExceeded),
        (ErrorCode::ToolPolicyDenied, ToolErrorCode::ToolPolicyDenied),
        (
            ErrorCode::SchemaValidationFailed,
            ToolErrorCode::SchemaValidationFailed,
        ),
        (ErrorCode::Timeout, ToolErrorCode::Timeout),
        (ErrorCode::PartialResult, ToolErrorCode::PartialResult),
        (ErrorCode::Internal, ToolErrorCode::Internal),
    ];

    assert_eq!(
        pairs.len(),
        11,
        "update this table when ErrorCode variants change"
    );

    for (domain, tool) in pairs {
        let mapped: ToolErrorCode = domain.into();
        assert_eq!(mapped, tool, "From mapping mismatch for {domain:?}");
        assert_eq!(
            domain.as_str(),
            tool.as_str(),
            "as_str mismatch for {domain:?}"
        );
        assert_eq!(
            domain.as_str(),
            serde_json::to_value(tool)
                .expect("serialize")
                .as_str()
                .expect("string"),
            "serde rename drift for {tool:?}"
        );
    }
}

/// The MCP envelope MUST serialize `run_id: null` and `error: null` (not
/// omitted) on `status = "ok"` so external clients can rely on a fixed key
/// set per the public contract in `docs/research-agent-product.md` §10.1.
#[tokio::test]
async fn tool_envelope_ok_serializes_null_run_id_and_null_error() {
    let envelope = mcp_server(services(&[]))
        .aspect_research(Parameters(aspect_request()))
        .await
        .0;
    assert_eq!(envelope.status, ToolStatus::Ok);
    let value = serde_json::to_value(&envelope).expect("envelope json");
    let object = value.as_object().expect("object envelope");
    assert!(object.contains_key("run_id"), "run_id key must be present");
    assert!(object.contains_key("error"), "error key must be present");
    assert!(object["run_id"].is_null(), "run_id must serialize as null");
    assert!(object["error"].is_null(), "error must serialize as null");
}

/// Deep-research partial envelopes MUST surface `run_id`, populated `data`, and
/// an explicit `error: null` so clients can distinguish the aggregated partial
/// path from a failed envelope without inspecting `status`.
#[tokio::test]
async fn tool_envelope_partial_includes_data_and_null_error_with_run_id() {
    let envelope = mcp_server(services(&["aspect-2"]))
        .deep_research(Parameters(deep_request(3)))
        .await
        .0;
    assert_eq!(envelope.status, ToolStatus::Partial);
    let value = serde_json::to_value(&envelope).expect("envelope json");
    let object = value.as_object().expect("object envelope");
    assert!(object["run_id"].is_string(), "run_id must be populated");
    assert!(object["data"].is_object(), "data must be populated");
    assert!(object.contains_key("error"));
    assert!(object["error"].is_null(), "error must serialize as null");
}

/// Single-aspect partial envelopes preserve the original failure metadata in
/// `error` while returning the collected evidence in `data`.
#[tokio::test]
async fn tool_envelope_aspect_partial_serializes_data_and_error() {
    let mut request = aspect_request();
    request.task.limits.max_search_calls = Limit::limited(1);
    let envelope = mcp_server(sequence_services(vec![tool_response(), tool_response()]))
        .aspect_research(Parameters(request))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Partial);
    let value = serde_json::to_value(&envelope).expect("envelope json");
    let object = value.as_object().expect("object envelope");
    assert!(object["run_id"].is_null(), "run_id must serialize as null");
    assert!(object["data"].is_object(), "data must be populated");
    assert!(object["error"].is_object(), "error must be populated");
    assert_eq!(object["data"]["aspect_report"]["findings"], json!([]));
    assert_eq!(object["error"]["code"], json!("budget_exceeded"));
    assert_eq!(object["error"]["aspect_id"], json!("aspect-1"));
    assert_eq!(
        object["error"]["diagnostic"]["stage"],
        json!("search_budget")
    );
    assert_eq!(object["error"]["diagnostic"]["model_turn"], json!(2));
    assert_eq!(object["error"]["diagnostic"]["search_turn"], json!(2));
}

/// Failed envelopes MUST serialize `run_id: null` and `data: null` so clients
/// see the same key set across `ok` / `partial` / `failed` responses.
#[tokio::test]
async fn tool_envelope_failed_serializes_null_run_id_and_null_data() {
    let mut request = aspect_request();
    request.task.question.clear();
    let envelope = mcp_server(services(&[]))
        .aspect_research(Parameters(request))
        .await
        .0;
    assert_eq!(envelope.status, ToolStatus::Failed);
    let value = serde_json::to_value(&envelope).expect("envelope json");
    let object = value.as_object().expect("object envelope");
    assert!(object.contains_key("run_id"));
    assert!(object["run_id"].is_null(), "run_id must serialize as null");
    assert!(object["data"].is_null(), "data must serialize as null");
    assert!(object["error"].is_object(), "error must be populated");
    assert_eq!(
        object["error"]["diagnostic"]["stage"],
        json!("request_validation")
    );
    assert!(object["error"]["diagnostic"].get("model_turn").is_none());
    assert!(object["error"]["diagnostic"].get("search_turn").is_none());
    assert_eq!(object["error"]["failed_aspects"], json!([]));
}

/// Aspect-research failures MUST carry the failing aspect id in `error.aspect_id`
/// so external clients can pinpoint the failure without parsing the message.
#[tokio::test]
async fn tool_envelope_failed_aspect_research_carries_aspect_id() {
    let mut request = aspect_request();
    request.task.question.clear();
    let expected_aspect_id = request.task.id.clone();
    let envelope = mcp_server(services(&[]))
        .aspect_research(Parameters(request))
        .await
        .0;
    let error = envelope.error.expect("tool error");
    assert_eq!(
        error.aspect_id.as_deref(),
        Some(expected_aspect_id.as_str())
    );
    assert!(error.failed_aspects.is_empty());
}

/// Top-level deep-research failures cannot be tied to a single aspect, so
/// the envelope MUST set `error.aspect_id` to `None`.
#[tokio::test]
async fn tool_envelope_failed_deep_research_aspect_id_is_none() {
    let mut request = deep_request(1);
    request.schema_version = "not-a-supported-version".to_owned();
    let envelope = mcp_server(services(&[]))
        .deep_research(Parameters(request))
        .await
        .0;
    assert_eq!(envelope.status, ToolStatus::Failed);
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::UnsupportedSchemaVersion);
    assert!(error.aspect_id.is_none());
    assert!(error.failed_aspects.is_empty());
}

/// Public messages MUST not leak provider names, request bodies, header values,
/// caller-supplied schema versions, or host file paths. Curated schema
/// validation diagnostics are tested separately.
#[test]
fn public_message_redacts_provider_path_and_api_key() {
    let cases = vec![
        (
            Error::HttpTransport {
                message: concat!(
                    "POST https://api.openai.com/v1/responses?api_key=sk-query-secret ",
                    "Authorization=sk-abcdef response={\"api_key\":\"raw-provider-secret\"}"
                )
                .to_owned(),
                retryable: true,
            },
            vec![
                "Authorization",
                "sk-abcdef",
                "api.openai.com",
                "api_key",
                "sk-query-secret",
                "raw-provider-secret",
            ],
        ),
        (
            Error::ProviderUnavailable {
                provider: "openai".to_owned(),
                message: "missing OPENAI_API_KEY in /home/user/moeresearch.toml".to_owned(),
                retryable: false,
            },
            vec!["openai", "OPENAI_API_KEY", "/home/user/moeresearch.toml"],
        ),
        (
            Error::UnsupportedSchemaVersion {
                version: "../../Authorization=sk-abcdef".to_owned(),
            },
            vec!["Authorization", "sk-abcdef", "../"],
        ),
    ];

    for (error, forbidden_fragments) in cases {
        let message = error.public_message();
        for forbidden in forbidden_fragments {
            assert!(
                !message.contains(forbidden),
                "public message leaked forbidden fragment `{forbidden}`: {message}"
            );
        }
    }
}

/// An unsupported `schema_version` MUST produce the dedicated
/// `ToolErrorCode::UnsupportedSchemaVersion` rather than the generic
/// `SchemaValidationFailed`, so clients can differentiate the two.
#[tokio::test]
async fn unsupported_schema_version_returns_dedicated_code_aspect_research() {
    let mut request = aspect_request();
    request.schema_version = "../../Authorization=sk-abcdef".to_owned();
    let envelope = mcp_server(services(&[]))
        .aspect_research(Parameters(request))
        .await
        .0;
    assert_eq!(envelope.status, ToolStatus::Failed);
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::UnsupportedSchemaVersion);
    assert_eq!(error.message, "unsupported schema version");
    assert!(!error.message.contains("Authorization"));
    assert!(!error.message.contains("sk-abcdef"));
    assert!(error.failed_aspects.is_empty());
}

/// Same dedicated-code guarantee for the deep_research entry point.
#[tokio::test]
async fn unsupported_schema_version_returns_dedicated_code_deep_research() {
    let mut request = deep_request(1);
    request.schema_version = "v999".to_owned();
    let envelope = mcp_server(services(&[]))
        .deep_research(Parameters(request))
        .await
        .0;
    assert_eq!(envelope.status, ToolStatus::Failed);
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::UnsupportedSchemaVersion);
    assert!(error.failed_aspects.is_empty());
}

#[test]
fn cli_serve_writes_startup_logs_to_stderr_not_stdout() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root");
    let config_path = std::env::temp_dir().join(format!(
        "moe-research-mcp-stdio-test-{}.toml",
        std::process::id()
    ));
    std::fs::write(
        &config_path,
        r#"
[logging]
format = "json"

[network]
inactivity_timeout_ms = 120000
max_retries = 2
retry_backoff_ms = 200
user_agent = "moeresearch/0.1.0"

[search.providers.exa]
enabled = false
base_url = "https://api.exa.ai"
api_key_env = "EXA_API_KEY"
inactivity_timeout_ms = 120000

[search.providers.grok]
enabled = false
base_url = "https://api.x.ai/v1"
api_key_env = "XAI_API_KEY"
inactivity_timeout_ms = 120000
model = "grok-4.3"

[model.providers.openai]
enabled = false
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY"
inactivity_timeout_ms = 120000
model = "gpt-5.5"

[limits.research]
max_agents = -1
max_concurrent_agents = -1
max_total_model_calls = -1
max_total_search_calls = -1
total_timeout_ms = -1
max_tokens = -1

[limits.per_agent]
max_turns = -1
max_tool_calls = -1
max_search_calls = -1
timeout_ms = -1
"#,
    )
    .expect("write test config");

    let mut child = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .args([
            "run",
            "--quiet",
            "--locked",
            "-p",
            "moe-research-cli",
            "--",
            "serve",
            "--config",
        ])
        .arg(&config_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn moeresearch serve");

    let mut stdout_pipe = child.stdout.take().expect("stdout pipe");
    let stderr_pipe = child.stderr.take().expect("stderr pipe");
    let stdout_reader = thread::spawn(move || {
        let mut output = String::new();
        stdout_pipe
            .read_to_string(&mut output)
            .expect("read stdout");
        output
    });
    let (startup_tx, startup_rx) = mpsc::channel();
    let stderr_reader = thread::spawn(move || {
        let mut output = String::new();
        let reader = BufReader::new(stderr_pipe);
        for line in reader.lines() {
            let line = line.expect("read stderr line");
            if line.contains("moeresearch initialized") {
                let _ = startup_tx.send(());
            }
            output.push_str(&line);
            output.push('\n');
        }
        output
    });

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if startup_rx.try_recv().is_ok() {
            break;
        }
        if child.try_wait().expect("poll moeresearch serve").is_some() {
            break;
        }
        if Instant::now() >= deadline {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    if child.try_wait().expect("poll moeresearch serve").is_none() {
        child.kill().expect("stop moeresearch serve");
    }
    child.wait().expect("collect moeresearch serve status");
    let _ = std::fs::remove_file(&config_path);
    let stdout = stdout_reader.join().expect("join stdout reader");
    let stderr = stderr_reader.join().expect("join stderr reader");

    assert!(!stdout.contains("moeresearch initialized"));
    assert!(
        stderr.contains("moeresearch initialized"),
        "expected startup logs on stderr, got stderr: {stderr}"
    );
    assert!(
        stderr.contains("operator_limits_research"),
        "expected startup logs to include operator_limits_research, got stderr: {stderr}"
    );
    assert!(
        stderr.contains("operator_limits_per_agent"),
        "expected startup logs to include operator_limits_per_agent, got stderr: {stderr}"
    );
    assert!(
        !stderr.contains("budget_research") && !stderr.contains("budget_per_agent"),
        "startup logs should not expose legacy budget_* config field names, got stderr: {stderr}"
    );
}
