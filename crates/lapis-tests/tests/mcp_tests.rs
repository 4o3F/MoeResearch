use std::collections::{BTreeSet, VecDeque};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use async_trait::async_trait;
use lapis_core::error::{Error, Result};
use lapis_core::mcp::LapisMcpServer;
use lapis_core::model::provider::ModelProvider;
use lapis_core::model::service::ModelService;
use lapis_core::orchestrator::workflow::deep_research;
use lapis_core::schema::budget::{AgentBudget, ResearchBudget};
use lapis_core::schema::config::BudgetConfig;
use lapis_core::schema::limit::Limit;
use lapis_core::schema::mcp::{ToolEnvelope, ToolErrorCode, ToolStatus};
use lapis_core::schema::model::{ModelMessage, ModelRequest, ModelResponse, ModelToolCall};
use lapis_core::schema::policy::{
    EvidencePolicy, EvidenceRequirement, ExecutionPolicy, ModelPolicy, OutputPolicy, SearchPolicy,
    ToolName,
};
use lapis_core::schema::report::{
    AspectReport, AspectResearchResult, Confidence, Finding, FindingType, Importance, OpenQuestion,
    TerminationReason,
};
use lapis_core::schema::research::{
    AspectResearchRequest, AspectSpec, DeepResearchRequest, DeliverableSpec, PromptAssets,
    ResearchContext, ResearchPlan,
};
use lapis_core::schema::search::{ProviderSearchRequest, SearchResponse, SearchResult};
use lapis_core::search::provider::SearchProvider;
use lapis_core::search::service::SearchService;
use rmcp::ServerHandler;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::schemars::schema_for;
use serde_json::json;

struct AdaptiveModelProvider {
    failing_aspects: BTreeSet<String>,
    calls: Arc<AtomicUsize>,
}

struct SequenceModelProvider {
    calls: Arc<AtomicUsize>,
    responses: Mutex<VecDeque<ModelResponse>>,
}

#[async_trait]
impl ModelProvider for AdaptiveModelProvider {
    fn name(&self) -> &'static str {
        "model"
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let aspect_id = aspect_field(&request.messages, "Aspect ID");
        let aspect_name = aspect_field(&request.messages, "Aspect name");

        if request.messages.len() <= 2 {
            return Ok(tool_response());
        }

        if self.failing_aspects.contains(&aspect_id) {
            return Ok(final_response("{}".to_owned()));
        }

        Ok(final_response(report_json(&aspect_id, &aspect_name)))
    }
}

#[async_trait]
impl ModelProvider for SequenceModelProvider {
    fn name(&self) -> &'static str {
        "model"
    }

    async fn complete(&self, _request: ModelRequest) -> Result<ModelResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.responses
            .lock()
            .expect("responses lock")
            .pop_front()
            .ok_or_else(|| Error::Internal {
                message: "missing fake model response".to_owned(),
            })
    }
}

struct StaticSearchProvider {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl SearchProvider for StaticSearchProvider {
    fn name(&self) -> &'static str {
        "searcher"
    }

    async fn search(&self, _request: ProviderSearchRequest) -> Result<SearchResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(SearchResponse {
            provider: "searcher".to_owned(),
            results: vec![SearchResult {
                title: "Shared Source".to_owned(),
                url: Some("https://example.test/shared".to_owned()),
                snippet: "shared snippet".to_owned(),
                summary: Some("shared summary".to_owned()),
                published_at: None,
            }],
        })
    }
}

struct Services {
    model: ModelService,
    search: SearchService,
    model_calls: Arc<AtomicUsize>,
    search_calls: Arc<AtomicUsize>,
}

fn services(failing_aspects: &[&str]) -> Services {
    let model_calls = Arc::new(AtomicUsize::new(0));
    let search_calls = Arc::new(AtomicUsize::new(0));
    let mut model = ModelService::new();
    model.register(AdaptiveModelProvider {
        failing_aspects: failing_aspects
            .iter()
            .map(|aspect| (*aspect).to_owned())
            .collect(),
        calls: model_calls.clone(),
    });
    let mut search = SearchService::new();
    search.register(StaticSearchProvider {
        calls: search_calls.clone(),
    });

    Services {
        model,
        search,
        model_calls,
        search_calls,
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
    let mut search = SearchService::new();
    search.register(StaticSearchProvider {
        calls: search_calls.clone(),
    });

    Services {
        model,
        search,
        model_calls,
        search_calls,
    }
}

fn prompt_assets() -> PromptAssets {
    PromptAssets {
        aspect_agent_prompt_path: "prompts/layer2/aspect-agent.md".to_owned(),
    }
}

fn aspect_request() -> AspectResearchRequest {
    AspectResearchRequest {
        schema_version: "m4".to_owned(),
        request_id: "request-1".to_owned(),
        aspect: aspect(1),
        shared_context: ResearchContext::default(),
        model_policy: model_policy(),
        search_policy: search_policy(),
        evidence_policy: EvidencePolicy {
            include_query_trace: false,
            include_source_urls: true,
            ..EvidencePolicy::default()
        },
        output_policy: OutputPolicy::default(),
        budget: AgentBudget::default(),
        execution_policy: ExecutionPolicy {
            timeout_ms: Some(180_000),
            ..ExecutionPolicy::default()
        },
    }
}

fn deep_request(count: usize) -> DeepResearchRequest {
    DeepResearchRequest {
        schema_version: "m5".to_owned(),
        request_id: "request-1".to_owned(),
        plan: ResearchPlan {
            plan_id: "plan-1".to_owned(),
            user_question: "What is true?".to_owned(),
            deliverable: DeliverableSpec {
                kind: "brief".to_owned(),
                language: "en".to_owned(),
                expected_sections: vec!["summary".to_owned()],
                notes: vec![],
            },
            constraints: vec![],
            aspects: (1..=count).map(aspect).collect(),
            budget: ResearchBudget {
                max_agents: Limit::limited(count),
                max_concurrent_agents: Limit::limited(2),
                max_total_model_calls: Limit::limited(20),
                max_total_search_calls: Limit::limited(20),
                total_timeout_ms: Limit::limited(180_000),
                max_tokens: Limit::unlimited(),
            },
            model_policy: model_policy(),
            search_policy: search_policy(),
            evidence_policy: EvidencePolicy {
                include_query_trace: false,
                include_source_urls: true,
                ..EvidencePolicy::default()
            },
            output_policy: OutputPolicy::default(),
        },
        shared_context: ResearchContext::default(),
        execution_policy: ExecutionPolicy {
            timeout_ms: Some(180_000),
            ..ExecutionPolicy::default()
        },
    }
}

fn aspect(index: usize) -> AspectSpec {
    AspectSpec {
        aspect_id: format!("aspect-{index}"),
        name: format!("Aspect {index}"),
        role: "researcher".to_owned(),
        research_question: format!("Question {index}?"),
        scope: vec!["scope".to_owned()],
        boundaries: vec![],
        success_criteria: vec!["answer".to_owned()],
        prompt_assets: prompt_assets(),
        required_evidence: EvidenceRequirement::default(),
        allowed_tools: vec![ToolName("search".to_owned())],
        model_override: None,
        search_override: None,
        budget_override: Some(AgentBudget::default()),
    }
}

fn model_policy() -> ModelPolicy {
    ModelPolicy {
        default_provider: "model".to_owned(),
        allowed_providers: vec!["model".to_owned()],
        ..ModelPolicy::default()
    }
}

fn search_policy() -> SearchPolicy {
    SearchPolicy {
        allowed_providers: vec!["searcher".to_owned()],
        preferred_providers: vec!["searcher".to_owned()],
        max_results_per_query: 2,
        ..SearchPolicy::default()
    }
}

fn tool_response() -> ModelResponse {
    ModelResponse {
        provider: "model".to_owned(),
        model: None,
        content: None,
        tool_calls: vec![ModelToolCall {
            id: "call-1".to_owned(),
            name: "search".to_owned(),
            arguments: json!({"query": "private query", "max_results": 1}),
        }],
        usage: None,
    }
}

fn final_response(content: String) -> ModelResponse {
    ModelResponse {
        provider: "model".to_owned(),
        model: None,
        content: Some(content),
        tool_calls: vec![],
        usage: None,
    }
}

fn report_json(aspect_id: &str, aspect_name: &str) -> String {
    serde_json::to_string(&AspectReport {
        aspect_id: aspect_id.to_owned(),
        aspect_name: aspect_name.to_owned(),
        question: "What is true?".to_owned(),
        scope: vec!["scope".to_owned()],
        findings: vec![Finding {
            id: format!("finding-{aspect_id}"),
            claim: "A supported claim".to_owned(),
            finding_type: FindingType::Fact,
            importance: Importance::High,
            confidence: Confidence::Medium,
            evidence_refs: vec!["ev-1-1".to_owned()],
            contradicted_by: vec![],
        }],
        evidence: vec![],
        assumptions: vec![],
        risks: vec![],
        counterarguments: vec![],
        open_questions: vec![OpenQuestion {
            id: format!("open-{aspect_id}"),
            question: "What remains uncertain?".to_owned(),
            reason: "Budget limited".to_owned(),
            suggested_follow_up: vec!["Search again".to_owned()],
        }],
        confidence: Confidence::Medium,
        limitations: vec![],
    })
    .expect("report json")
}

fn aspect_field(messages: &[ModelMessage], label: &str) -> String {
    let pointer = match label {
        "Aspect ID" => "/aspect/aspect_id",
        "Aspect name" => "/aspect/name",
        _ => return String::new(),
    };

    messages
        .iter()
        .find_map(|message| {
            serde_json::from_str::<serde_json::Value>(&message.content)
                .ok()
                .and_then(|value| {
                    value
                        .pointer(pointer)
                        .and_then(|field| field.as_str())
                        .map(str::to_owned)
                })
                .or_else(|| {
                    message.content.lines().find_map(|line| {
                        line.strip_prefix(label)
                            .and_then(|value| value.strip_prefix(": "))
                            .map(str::to_owned)
                    })
                })
        })
        .unwrap_or_default()
}

fn mcp_server(services: Services) -> LapisMcpServer {
    LapisMcpServer::new(services.model, services.search, BudgetConfig::default())
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
fn aspect_research_tool_schema_uses_limit_wire_format() {
    let server = mcp_server(services(&[]));
    let tool = server
        .get_tool("aspect_research")
        .expect("aspect research tool");
    let schema = serde_json::Value::Object(tool.input_schema.as_ref().clone());
    let limit = schema
        .pointer("/$defs/Limit_uint")
        .expect("count limit schema");
    let duration_limit = schema
        .pointer("/$defs/Limit_uint64")
        .expect("duration limit schema");

    assert_eq!(limit.get("type"), Some(&json!(["integer", "null"])));
    assert_eq!(limit.get("minimum"), Some(&json!(-1)));
    assert_eq!(
        duration_limit.get("type"),
        Some(&json!(["integer", "null"]))
    );
    assert_eq!(duration_limit.get("minimum"), Some(&json!(-1)));
    assert!(!schema.to_string().contains("Limited"));
}

#[test]
fn tool_envelope_schema_exposes_partial_trace() {
    let schema = schema_for!(ToolEnvelope<AspectResearchResult>);
    let schema = serde_json::to_value(&schema).expect("schema json");
    let properties = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .expect("schema properties");

    assert!(properties.contains_key("trace_summary"));
    assert!(properties.contains_key("partial_trace"));
    assert!(!schema.to_string().contains("partialTrace"));
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
    assert!(envelope.run_id.is_some());
    assert!(envelope.partial_trace.is_none());
    assert_eq!(model_calls.load(Ordering::SeqCst), 2);
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn aspect_research_invalid_input_returns_failed_envelope() {
    let mut request = aspect_request();
    request.aspect.research_question.clear();
    let envelope = mcp_server(services(&[]))
        .aspect_research(Parameters(request))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    assert!(envelope.data.is_none());
    assert!(envelope.trace_summary.is_none());
    assert!(envelope.partial_trace.is_none());
    let error = envelope.error.expect("tool error");
    assert_eq!(error.code, ToolErrorCode::InvalidInput);
    assert!(!error.retryable);
}

#[tokio::test]
async fn aspect_research_budget_failure_envelope_includes_partial_trace() {
    let mut request = aspect_request();
    request.evidence_policy.include_query_trace = true;
    request.evidence_policy.include_source_urls = true;
    request.budget.max_search_calls = Limit::limited(2);
    let services = sequence_services(vec![tool_response(), tool_response(), tool_response()]);
    let search_calls = services.search_calls.clone();

    let envelope = mcp_server(services)
        .aspect_research(Parameters(request))
        .await
        .0;

    assert_eq!(envelope.status, ToolStatus::Failed);
    assert!(envelope.data.is_none());
    assert_eq!(
        envelope.error.expect("tool error").code,
        ToolErrorCode::BudgetExceeded
    );
    assert!(envelope.trace_summary.is_some());
    assert!(envelope.run_id.is_some());
    assert_eq!(search_calls.load(Ordering::SeqCst), 2);
    let partial = envelope.partial_trace.expect("partial trace");
    assert_eq!(partial.search_queries.len(), 2);
    assert_eq!(partial.tool_calls.len(), 2);
    assert_eq!(partial.provider_usage.search_calls, 2);
    assert_eq!(partial.budget_usage.tool_calls_used, 2);
    assert_eq!(partial.budget_usage.search_calls_used, 2);
    assert_eq!(
        partial.trace_summary.termination_reason,
        Some(TerminationReason::BudgetExceeded)
    );
    assert_eq!(partial.search_queries[0].query, "private query");
    assert_eq!(partial.search_queries[0].sources[0].title, "Shared Source");
    assert_eq!(
        partial.search_queries[0].sources[0].url.as_deref(),
        Some("https://example.test/shared")
    );
    assert_eq!(
        partial.tool_calls[0]
            .search
            .as_ref()
            .expect("search trace")
            .provider,
        "searcher"
    );
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
    assert_eq!(data.failed_aspects[0].error_code, "SchemaValidationFailed");
}

#[tokio::test]
async fn deep_research_all_failed_returns_failed_envelope_with_tool_error() {
    let request = deep_request(2);
    let envelope = mcp_server(services(&["aspect-1", "aspect-2"]))
        .deep_research(Parameters(request.clone()))
        .await
        .0;
    let expected_services = services(&["aspect-1", "aspect-2"]);
    let expected_error = deep_research(
        request,
        &expected_services.model,
        &expected_services.search,
        &BudgetConfig::default(),
    )
    .await
    .expect_err("deep error")
    .to_tool_error();

    assert_eq!(envelope.status, ToolStatus::Failed);
    assert!(envelope.data.is_none());
    assert_eq!(envelope.error.expect("tool error"), expected_error);
}

#[test]
fn error_retryability_mapping_is_stable() {
    assert!(
        Error::NetworkFailed {
            message: "temporary network failure".to_owned(),
        }
        .to_tool_error()
        .retryable
    );
    assert!(
        Error::Timeout {
            message: "deadline exceeded".to_owned(),
        }
        .to_tool_error()
        .retryable
    );
    assert!(
        Error::HttpStatus {
            status: 503,
            message: "service unavailable".to_owned(),
            retryable: true,
        }
        .to_tool_error()
        .retryable
    );
    assert!(
        !Error::InvalidInput {
            message: "missing question".to_owned(),
        }
        .to_tool_error()
        .retryable
    );
}

#[test]
fn cli_serve_writes_startup_logs_to_stderr_not_stdout() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root");
    let mut child = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .args([
            "run",
            "--quiet",
            "--locked",
            "-p",
            "lapis-cli",
            "--",
            "serve",
            "--config",
            "missing-mcp-stdio-test.toml",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn lapis serve");

    thread::sleep(Duration::from_secs(5));
    if child.try_wait().expect("poll lapis serve").is_none() {
        child.kill().expect("stop lapis serve");
    }
    let output = child
        .wait_with_output()
        .expect("collect lapis serve output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!stdout.contains("lapis core initialized"));
    assert!(!stdout.contains("user config not found"));
    assert!(
        stderr.contains("lapis core initialized") || stderr.contains("user config not found"),
        "expected startup logs on stderr, got stderr: {stderr}"
    );
}
