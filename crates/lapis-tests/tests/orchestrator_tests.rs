use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use lapis_error::{Error, Result};
use lapis_model::ModelProvider;
use lapis_model::ModelService;
use lapis_model::{ModelInputItem, ModelRequest, ModelResponse, ModelToolCall};
use lapis_search::SearchProvider;
use lapis_search::SearchService;
use lapis_search::{SearchRequest, SearchResponse, SearchResult};
use lapis_workflow::AgentRuntime;
use lapis_workflow::Limit;
use lapis_workflow::aspect_research;
use lapis_workflow::{AgentBudget, BudgetConfig, ResearchBudget};
use lapis_workflow::{AgentBudgetGuard, ResearchBudgetGuard};
use lapis_workflow::{
    AspectReport, AspectResearchResult, Confidence, Evidence, Finding, FindingType, Importance,
    OpenQuestion, SourceType,
};
use lapis_workflow::{AspectResearchRequest, AspectResearchTask, AspectSpec, ResearchContext};
use lapis_workflow::{
    EvidencePolicy, ExecutionPolicy, ModelPolicy, OutputPolicy, SearchCategory, SearchContentLevel,
    SearchDepth, SearchPolicy, SearchRecency, ToolName,
};
use serde_json::json;

fn unlimited_budget_config() -> BudgetConfig {
    BudgetConfig {
        research: ResearchBudget::unlimited(),
        per_agent: AgentBudget::unlimited(),
    }
}

fn model_policy() -> ModelPolicy {
    ModelPolicy {
        allowed_providers: vec!["model".to_owned()],
        temperature: Some(0.2),
        max_tokens: None,
        require_tool_call_support: true,
    }
}

fn search_policy() -> SearchPolicy {
    SearchPolicy {
        allowed_providers: vec!["searcher".to_owned()],
        max_results_per_query: 2,
        freshness: None,
        depth: None,
        content_level: None,
        recency: None,
        category: None,
        language: None,
        region: None,
        include_domains: Vec::new(),
        exclude_domains: Vec::new(),
    }
}

fn evidence_policy() -> EvidencePolicy {
    EvidencePolicy {
        require_evidence_for_findings: true,
        min_evidence_per_finding: 1,
    }
}

fn output_policy() -> OutputPolicy {
    OutputPolicy {
        language: "zh-CN".to_owned(),
        max_findings_per_aspect: None,
    }
}

fn execution_policy(timeout_ms: Option<u64>) -> ExecutionPolicy {
    ExecutionPolicy {
        allow_partial_results: true,
        fail_fast: false,
        timeout_ms: timeout_ms.map_or(Limit::unlimited(), Limit::limited),
    }
}

const VALID_ASPECT_RESULT_SENTINEL: &str = "__VALID_ASPECT_RESULT_FROM_TOOL_OUTPUT__";
const UNKNOWN_EVIDENCE_SENTINEL: &str = "__UNKNOWN_EVIDENCE_FROM_TOOL_OUTPUT__";
const TAMPERED_EVIDENCE_SENTINEL: &str = "__TAMPERED_EVIDENCE_FROM_TOOL_OUTPUT__";
const INTERPRETIVE_EVIDENCE_SENTINEL: &str = "__INTERPRETIVE_EVIDENCE_FROM_TOOL_OUTPUT__";

struct SequenceModelProvider {
    calls: Arc<AtomicUsize>,
    responses: Mutex<VecDeque<ModelResponse>>,
    delay: Option<Duration>,
}

struct CapturingSequenceModelProvider {
    calls: Arc<AtomicUsize>,
    responses: Mutex<VecDeque<ModelResponse>>,
    requests: Arc<Mutex<Vec<ModelRequest>>>,
}

#[async_trait]
impl ModelProvider for SequenceModelProvider {
    fn name(&self) -> &'static str {
        "model"
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }
        let response = self
            .responses
            .lock()
            .expect("responses lock")
            .pop_front()
            .ok_or_else(|| Error::Internal {
                message: "missing fake model response".to_owned(),
            })?;
        Ok(resolve_final_response(response, &request.input))
    }
}

#[async_trait]
impl ModelProvider for CapturingSequenceModelProvider {
    fn name(&self) -> &'static str {
        "model"
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.requests
            .lock()
            .expect("requests lock")
            .push(request.clone());
        let response = self
            .responses
            .lock()
            .expect("responses lock")
            .pop_front()
            .ok_or_else(|| Error::Internal {
                message: "missing fake model response".to_owned(),
            })?;
        Ok(resolve_final_response(response, &request.input))
    }
}

struct CountingSearchProvider {
    calls: Arc<AtomicUsize>,
}

struct SequenceSearchProvider {
    calls: Arc<AtomicUsize>,
    responses: Mutex<VecDeque<Result<SearchResponse>>>,
}

struct CapturingSearchProvider {
    requests: Arc<Mutex<Vec<SearchRequest>>>,
}

#[async_trait]
impl SearchProvider for CountingSearchProvider {
    fn name(&self) -> &'static str {
        "searcher"
    }

    async fn search(&self, _request: SearchRequest) -> Result<SearchResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(search_response())
    }
}

#[async_trait]
impl SearchProvider for SequenceSearchProvider {
    fn name(&self) -> &'static str {
        "searcher"
    }

    async fn search(&self, _request: SearchRequest) -> Result<SearchResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.responses
            .lock()
            .expect("responses lock")
            .pop_front()
            .ok_or_else(|| Error::Internal {
                message: "missing fake search response".to_owned(),
            })?
    }
}

#[async_trait]
impl SearchProvider for CapturingSearchProvider {
    fn name(&self) -> &'static str {
        "searcher"
    }

    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        self.requests
            .lock()
            .expect("captured search requests")
            .push(request);
        Ok(search_response())
    }
}

fn search_response() -> SearchResponse {
    SearchResponse {
        provider: "searcher".to_owned(),
        results: vec![SearchResult {
            title: "Source".to_owned(),
            url: Some("https://example.test/source".to_owned()),
            snippet: "snippet".to_owned(),
            summary: Some("summary".to_owned()),
            published_at: Some("2026-05-25".to_owned()),
        }],
    }
}

fn services(
    responses: Vec<ModelResponse>,
) -> (
    ModelService,
    SearchService,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
) {
    services_with_delay(responses, None)
}

fn services_with_delay(
    responses: Vec<ModelResponse>,
    delay: Option<Duration>,
) -> (
    ModelService,
    SearchService,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
) {
    let model_calls = Arc::new(AtomicUsize::new(0));
    let search_calls = Arc::new(AtomicUsize::new(0));
    let mut model_service = ModelService::new();
    model_service.register(SequenceModelProvider {
        calls: model_calls.clone(),
        responses: Mutex::new(responses.into()),
        delay,
    });
    let mut search_service = SearchService::new();
    search_service.register(CountingSearchProvider {
        calls: search_calls.clone(),
    });
    (model_service, search_service, model_calls, search_calls)
}

fn aspect_prompt() -> String {
    "# Aspect Agent\n\nDummy aspect agent prompt for tests.\n".to_owned()
}

fn aspect_request() -> AspectResearchRequest {
    AspectResearchRequest {
        schema_version: "0.1".to_owned(),
        request_id: "request-1".to_owned(),
        task: AspectResearchTask {
            aspect: AspectSpec {
                aspect_id: "aspect-1".to_owned(),
                name: "Aspect".to_owned(),
                role: "researcher".to_owned(),
                research_question: "What is true?".to_owned(),
                scope: vec!["scope".to_owned()],
                boundaries: vec![],
                success_criteria: vec!["answer".to_owned()],
                aspect_agent_prompt: aspect_prompt(),
                allowed_tools: vec![ToolName("search".to_owned())],
                model_provider: Some("model".to_owned()),
                search_provider: Some("searcher".to_owned()),
            },
            budget: AgentBudget::unlimited(),
        },
        shared_context: ResearchContext::empty(),
        model_policy: model_policy(),
        search_policy: search_policy(),
        evidence_policy: evidence_policy(),
        output_policy: output_policy(),
        execution_policy: execution_policy(Some(180_000)),
    }
}

fn tool_response(name: &str) -> ModelResponse {
    let tool_call = ModelToolCall {
        id: "call-1".to_owned(),
        name: name.to_owned(),
        arguments: json!({"query": "private query", "max_results": 1}),
    };
    ModelResponse {
        provider: "model".to_owned(),
        model: None,
        response_id: None,
        content: None,
        tool_calls: vec![tool_call.clone()],
        output_items: vec![ModelInputItem::tool_call(tool_call)],
        usage: None,
    }
}

fn tool_response_without_output_items(name: &str) -> ModelResponse {
    let mut response = tool_response(name);
    response.output_items.clear();
    response
}

fn tool_response_with_response_id(name: &str, response_id: &str) -> ModelResponse {
    let mut response = tool_response(name);
    response.response_id = Some(response_id.to_owned());
    response
}

fn final_response(content: String) -> ModelResponse {
    ModelResponse {
        provider: "model".to_owned(),
        model: None,
        response_id: None,
        content: Some(content),
        tool_calls: vec![],
        output_items: vec![],
        usage: None,
    }
}

fn valid_report_json() -> String {
    VALID_ASPECT_RESULT_SENTINEL.to_owned()
}

fn valid_aspect_report(evidence_refs: Vec<String>) -> AspectReport {
    AspectReport {
        aspect_id: "aspect-1".to_owned(),
        aspect_name: "Aspect".to_owned(),
        question: "What is true?".to_owned(),
        scope: vec!["scope".to_owned()],
        findings: vec![Finding {
            id: "finding-1".to_owned(),
            claim: "A supported claim".to_owned(),
            finding_type: FindingType::Fact,
            importance: Importance::High,
            confidence: Confidence::Medium,
            evidence_refs,
            contradicted_by: vec![],
        }],
        assumptions: vec![],
        risks: vec![],
        counterarguments: vec![],
        open_questions: Vec::<OpenQuestion>::new(),
        confidence: Confidence::Medium,
        limitations: vec![],
    }
}

fn aspect_result_json(mut evidence: Vec<Evidence>) -> String {
    let evidence_refs = evidence
        .iter()
        .map(|evidence| evidence.id.clone())
        .collect::<Vec<_>>();
    for item in &mut evidence {
        item.supports_findings = vec!["finding-1".to_owned()];
    }
    serde_json::to_string(&AspectResearchResult {
        aspect_report: valid_aspect_report(evidence_refs),
        evidence,
    })
    .expect("aspect result json")
}

fn resolve_final_response(mut response: ModelResponse, input: &[ModelInputItem]) -> ModelResponse {
    if response.content.as_deref() == Some(VALID_ASPECT_RESULT_SENTINEL) {
        response.content = Some(aspect_result_json(vec![first_evidence_from_tool_output(
            input,
        )]));
    } else if response.content.as_deref() == Some(UNKNOWN_EVIDENCE_SENTINEL) {
        let mut evidence = first_evidence_from_tool_output(input);
        evidence.id = "ev-missing".to_owned();
        response.content = Some(aspect_result_json(vec![evidence]));
    } else if response.content.as_deref() == Some(TAMPERED_EVIDENCE_SENTINEL) {
        let mut evidence = first_evidence_from_tool_output(input);
        evidence.summary = "tampered summary".to_owned();
        response.content = Some(aspect_result_json(vec![evidence]));
    } else if response.content.as_deref() == Some(INTERPRETIVE_EVIDENCE_SENTINEL) {
        let mut evidence = first_evidence_from_tool_output(input);
        evidence.source_type = SourceType::Official;
        evidence.confidence = Confidence::High;
        response.content = Some(aspect_result_json(vec![evidence]));
    }
    response
}

fn first_evidence_from_tool_output(input: &[ModelInputItem]) -> Evidence {
    evidence_from_tool_output(input, 0)
}

fn evidence_from_tool_output(input: &[ModelInputItem], index: usize) -> Evidence {
    let output = input
        .iter()
        .rev()
        .find_map(|item| match item {
            ModelInputItem::ToolOutput(output) => Some(output.output.as_str()),
            _ => None,
        })
        .expect("tool output");
    let value = serde_json::from_str::<serde_json::Value>(output).expect("tool output json");
    serde_json::from_value(value["results"][index].clone()).expect("evidence result")
}

fn budget(max_turns: usize, max_tool_calls: usize, max_search_calls: usize) -> AgentBudget {
    AgentBudget {
        max_turns: Limit::limited(max_turns),
        max_tool_calls: Limit::limited(max_tool_calls),
        max_search_calls: Limit::limited(max_search_calls),
        timeout_ms: Limit::limited(60_000),
    }
}

#[test]
fn accepts_minus_one_as_unlimited_agent_budget() {
    let budget: AgentBudget = serde_json::from_value(json!({
        "max_turns": -1,
        "max_tool_calls": -1,
        "max_search_calls": -1,
        "timeout_ms": -1
    }))
    .expect("unlimited budget");
    assert!(budget.max_turns.is_unlimited());
    let mut guard = AgentBudgetGuard::new(budget).expect("valid unlimited budget");
    for _ in 0..3 {
        guard.consume_model_turn().expect("unlimited model turn");
        guard.consume_tool_call().expect("unlimited tool call");
        guard.consume_search_call().expect("unlimited search call");
    }

    assert_eq!(guard.usage().turns_used, 3);
    assert_eq!(guard.usage().tool_calls_used, 3);
    assert_eq!(guard.usage().search_calls_used, 3);
}

#[test]
fn allows_boundary_usage_and_tracks_counters() {
    let mut guard = AgentBudgetGuard::new(budget(2, 1, 1)).expect("valid budget");

    guard.consume_model_turn().expect("first model turn");
    guard.consume_model_turn().expect("second model turn");
    guard.consume_tool_call().expect("tool call");
    guard.consume_search_call().expect("search call");

    let usage = guard.usage();
    assert_eq!(usage.turns_used, 2);
    assert_eq!(usage.tool_calls_used, 1);
    assert_eq!(usage.search_calls_used, 1);
}

#[test]
fn rejects_exhausted_model_tool_and_search_budgets() {
    let mut turn_guard = AgentBudgetGuard::new(budget(1, 1, 1)).expect("valid budget");
    turn_guard.consume_model_turn().expect("within turn budget");
    assert!(matches!(
        turn_guard.consume_model_turn(),
        Err(Error::BudgetExceeded { .. })
    ));

    let mut tool_guard = AgentBudgetGuard::new(budget(1, 0, 1)).expect("valid budget");
    assert!(matches!(
        tool_guard.consume_tool_call(),
        Err(Error::BudgetExceeded { .. })
    ));

    let mut search_guard = AgentBudgetGuard::new(budget(1, 1, 0)).expect("valid budget");
    assert!(matches!(
        search_guard.consume_search_call(),
        Err(Error::BudgetExceeded { .. })
    ));

    let mut search_tool_guard = AgentBudgetGuard::new(budget(1, 1, 0)).expect("valid budget");
    assert!(matches!(
        search_tool_guard.consume_search_tool_call(),
        Err(Error::BudgetExceeded { .. })
    ));
    assert_eq!(search_tool_guard.usage().tool_calls_used, 0);
    assert_eq!(search_tool_guard.usage().search_calls_used, 0);
}

#[test]
fn rejects_zero_turns_zero_timeout_and_elapsed_timeout() {
    assert!(matches!(
        AgentBudgetGuard::new(budget(0, 1, 1)),
        Err(Error::BudgetExceeded { .. })
    ));

    let mut zero_timeout = budget(1, 1, 1);
    zero_timeout.timeout_ms = Limit::limited(0);
    assert!(matches!(
        AgentBudgetGuard::new(zero_timeout),
        Err(Error::BudgetExceeded { .. })
    ));

    let mut guard = AgentBudgetGuard::new(AgentBudget {
        timeout_ms: Limit::limited(1),
        ..budget(1, 1, 1)
    })
    .expect("valid budget");
    std::thread::sleep(Duration::from_millis(5));

    assert!(matches!(
        guard.consume_model_turn(),
        Err(Error::BudgetExceeded { .. })
    ));
    assert_eq!(guard.usage().turns_used, 0);
}

#[tokio::test]
async fn rejects_invalid_request_fields() {
    let mut request = aspect_request();
    request.task.aspect.research_question.clear();
    let model_service = ModelService::new();
    let search_service = SearchService::new();

    let error = aspect_research(
        request,
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("invalid request");

    assert!(matches!(error.error, Error::InvalidInput { .. }));
}

/// Rust core never performs filesystem IO for prompts; Layer 1 supplies the
/// system-prompt content inline as `AspectSpec.aspect_agent_prompt`. The
/// first input the model sees MUST be a System message whose body equals
/// the request field byte-for-byte, with no further transformation.
#[tokio::test]
async fn aspect_agent_prompt_content_is_passed_as_system_message() {
    let mut request = aspect_request();
    request.task.aspect.aspect_agent_prompt = "# Custom Agent\n\nFollow these rules.\n".to_owned();

    let model_calls = Arc::new(AtomicUsize::new(0));
    let captured_requests = Arc::new(Mutex::new(Vec::new()));
    let mut model_service = ModelService::new();
    model_service.register(CapturingSequenceModelProvider {
        calls: model_calls,
        responses: Mutex::new(
            vec![tool_response("search"), final_response(valid_report_json())].into(),
        ),
        requests: captured_requests.clone(),
    });
    let search_calls = Arc::new(AtomicUsize::new(0));
    let mut search_service = SearchService::new();
    search_service.register(CountingSearchProvider {
        calls: search_calls,
    });

    aspect_research(
        request.clone(),
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect("valid inline prompt");

    let requests = captured_requests.lock().expect("requests lock").clone();
    assert!(
        matches!(
            &requests[0].input[0],
            ModelInputItem::Message(message)
                if message.role == lapis_model::ModelMessageRole::System
                    && message.content == request.task.aspect.aspect_agent_prompt
        ),
        "first input must be System message equal to inline prompt content"
    );
}

/// `AgentRuntime::run()` is a public entry that callers can hit without
/// going through workflow validation; it must therefore reject empty inline
/// prompts before any model call dispatches.
#[tokio::test]
async fn agent_runtime_rejects_empty_inline_prompt_before_dispatch() {
    let mut request = aspect_request();
    request.task.aspect.aspect_agent_prompt = String::new();

    let (model_service, search_service, model_calls, _search_calls) = services(vec![]);
    let failure = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("empty prompt must be rejected at runtime entry");

    assert!(matches!(failure.error, Error::InvalidInput { .. }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 0);
}

/// Direct `AgentRuntime::run()` must also reject prompts larger than the
/// 64 KiB cap before dispatch.
#[tokio::test]
async fn agent_runtime_rejects_oversized_inline_prompt_before_dispatch() {
    let mut request = aspect_request();
    request.task.aspect.aspect_agent_prompt = "x".repeat(64 * 1024 + 1);

    let (model_service, search_service, model_calls, _search_calls) = services(vec![]);
    let failure = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("oversized prompt must be rejected at runtime entry");

    assert!(matches!(
        failure.error,
        Error::SchemaValidationFailed { .. }
    ));
    assert_eq!(model_calls.load(Ordering::SeqCst), 0);
}

/// Schema validation MUST reject an empty inline prompt before any agent loop
/// runs.
#[tokio::test]
async fn aspect_agent_prompt_empty_string_is_rejected_at_schema_validation() {
    let mut request = aspect_request();
    request.task.aspect.aspect_agent_prompt = String::new();
    let model_service = ModelService::new();
    let search_service = SearchService::new();

    let error = aspect_research(
        request,
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("empty prompt must be rejected");

    assert!(matches!(error.error, Error::InvalidInput { .. }));
}

/// Schema validation MUST reject an inline prompt larger than 64 KiB to keep
/// a single MCP payload bounded.
#[tokio::test]
async fn aspect_agent_prompt_exceeding_max_bytes_is_rejected() {
    let mut request = aspect_request();
    request.task.aspect.aspect_agent_prompt = "x".repeat(64 * 1024 + 1);
    let model_service = ModelService::new();
    let search_service = SearchService::new();

    let error = aspect_research(
        request,
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("oversized prompt must be rejected");

    assert!(matches!(error.error, Error::SchemaValidationFailed { .. }));
}

#[tokio::test]
async fn rejects_conflicting_domains() {
    let mut request = aspect_request();
    request.search_policy.include_domains = vec!["Example.com".to_owned()];
    request.search_policy.exclude_domains = vec!["example.com".to_owned()];
    let model_service = ModelService::new();
    let search_service = SearchService::new();

    let error = aspect_research(
        request,
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("domain conflict");

    assert!(matches!(error.error, Error::InvalidInput { .. }));
}

#[tokio::test]
async fn rejects_search_enabled_aspect_without_provider() {
    let mut request = aspect_request();
    request.task.aspect.search_provider = None;
    let model_service = ModelService::new();
    let search_service = SearchService::new();

    let error = aspect_research(
        request,
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("missing search provider");

    assert!(matches!(error.error, Error::InvalidInput { .. }));
}

#[tokio::test]
async fn rejects_search_provider_not_allowed_by_policy() {
    let mut request = aspect_request();
    request.task.aspect.search_provider = Some("blocked".to_owned());
    let model_service = ModelService::new();
    let search_service = SearchService::new();

    let error = aspect_research(
        request,
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("disallowed search provider");

    assert!(
        matches!(error.error, Error::ProviderUnavailable { provider, .. } if provider == "blocked")
    );
}

#[tokio::test]
async fn rejects_empty_search_provider_allowlist() {
    let mut request = aspect_request();
    request.search_policy.allowed_providers.clear();
    let model_service = ModelService::new();
    let search_service = SearchService::new();

    let error = aspect_research(
        request,
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("empty search provider allowlist");

    assert!(
        matches!(error.error, Error::ProviderUnavailable { provider, .. } if provider == "searcher")
    );
}

#[tokio::test]
async fn non_search_aspect_runs_without_search_provider() {
    let mut request = aspect_request();
    request.task.aspect.allowed_tools.clear();
    request.task.aspect.search_provider = None;
    request.evidence_policy.require_evidence_for_findings = false;
    let (model_service, search_service, _model_calls, search_calls) =
        services(vec![final_response(
            serde_json::to_string(&AspectResearchResult {
                aspect_report: AspectReport {
                    aspect_id: "aspect-1".to_owned(),
                    aspect_name: "Aspect".to_owned(),
                    question: "What is true?".to_owned(),
                    scope: vec!["scope".to_owned()],
                    findings: Vec::new(),
                    assumptions: Vec::new(),
                    risks: Vec::new(),
                    counterarguments: Vec::new(),
                    open_questions: Vec::new(),
                    confidence: Confidence::Medium,
                    limitations: Vec::new(),
                },
                evidence: Vec::new(),
            })
            .expect("report json"),
        )]);

    let output = aspect_research(
        request,
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect("non-search aspect result");

    assert_eq!(output.result.aspect_report.aspect_id, "aspect-1");
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn rejects_execution_timeout_above_budget() {
    let mut request = aspect_request();
    request.task.budget.timeout_ms = Limit::limited(100);
    request.execution_policy.timeout_ms = Limit::limited(101);
    let model_service = ModelService::new();
    let search_service = SearchService::new();

    let error = aspect_research(
        request,
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("timeout conflict");

    assert!(matches!(error.error, Error::BudgetExceeded { .. }));
}

#[tokio::test]
async fn delegates_valid_request_to_runtime() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, search_calls) = services(vec![
        tool_response("search"),
        final_response(valid_report_json()),
    ]);

    let result = aspect_research(
        request,
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect("aspect result");

    assert_eq!(result.result.aspect_report.aspect_id, "aspect-1");
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn aspect_research_uses_configured_research_model_call_budget() {
    let request = aspect_request();
    let (model_service, search_service, model_calls, search_calls) = services(vec![
        tool_response("search"),
        final_response(valid_report_json()),
    ]);
    let limits = BudgetConfig {
        research: ResearchBudget {
            max_total_model_calls: Limit::limited(1),
            ..ResearchBudget::unlimited()
        },
        per_agent: AgentBudget::unlimited(),
    };

    let failure = aspect_research(request, &model_service, &search_service, &limits)
        .await
        .expect_err("configured research model budget should cap aspect_research");

    assert!(matches!(failure.error, Error::BudgetExceeded { .. }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 1);
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn aspect_research_uses_configured_research_search_budget() {
    let request = aspect_request();
    let (model_service, search_service, model_calls, search_calls) =
        services(vec![tool_response("search")]);
    let limits = BudgetConfig {
        research: ResearchBudget {
            max_total_search_calls: Limit::limited(0),
            ..ResearchBudget::unlimited()
        },
        per_agent: AgentBudget::unlimited(),
    };

    let failure = aspect_research(request, &model_service, &search_service, &limits)
        .await
        .expect_err("configured research search budget should cap aspect_research");

    assert!(matches!(failure.error, Error::BudgetExceeded { .. }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 1);
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn fake_model_and_search_complete_successfully() {
    let request = aspect_request();
    let (model_service, search_service, model_calls, search_calls) = services(vec![
        tool_response("search"),
        final_response(valid_report_json()),
    ]);

    let output = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("runtime output");

    assert_eq!(model_calls.load(Ordering::SeqCst), 2);
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
    assert_eq!(output.result.evidence[0].query, "private query");
    assert_eq!(output.result.evidence[0].snippet, "snippet");
    assert_eq!(
        output.result.evidence[0].url.as_deref(),
        Some("https://example.test/source")
    );
    assert_eq!(output.budget_usage.turns_used, 2);
    assert_eq!(output.budget_usage.tool_calls_used, 1);
    assert_eq!(output.budget_usage.search_calls_used, 1);
}

#[tokio::test]
async fn success_output_includes_resource_accounting() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, _search_calls) = services(vec![
        tool_response("search"),
        final_response(valid_report_json()),
    ]);

    let output = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("runtime output");

    assert_eq!(output.budget_usage.turns_used, 2);
    assert_eq!(output.budget_usage.tool_calls_used, 1);
    assert_eq!(output.budget_usage.search_calls_used, 1);
    assert_eq!(output.result.evidence.len(), 1);
}

#[tokio::test]
async fn search_tool_keeps_query_in_business_evidence() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, _search_calls) = services(vec![
        tool_response("search"),
        final_response(valid_report_json()),
    ]);

    let output = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("runtime output");

    assert_eq!(output.result.evidence[0].query, "private query");
    assert_eq!(output.budget_usage.tool_calls_used, 1);
}

#[tokio::test]
async fn model_tool_outputs_use_ordered_responses_items() {
    let request = aspect_request();
    let model_calls = Arc::new(AtomicUsize::new(0));
    let search_calls = Arc::new(AtomicUsize::new(0));
    let captured_requests = Arc::new(Mutex::new(Vec::new()));
    let mut model_service = ModelService::new();
    model_service.register(CapturingSequenceModelProvider {
        calls: model_calls.clone(),
        responses: Mutex::new(
            vec![tool_response("search"), final_response(valid_report_json())].into(),
        ),
        requests: captured_requests.clone(),
    });
    let mut search_service = SearchService::new();
    search_service.register(CountingSearchProvider {
        calls: search_calls.clone(),
    });

    AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("agent output");

    assert_eq!(model_calls.load(Ordering::SeqCst), 2);
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
    let requests = captured_requests.lock().expect("requests lock").clone();
    assert_eq!(requests.len(), 2);
    let second_input = &requests[1].input;
    assert!(second_input.iter().any(|item| {
        matches!(
            item,
            ModelInputItem::ToolCall(call)
                if call.id == "call-1" && call.name == "search"
        )
    }));
    assert!(second_input.iter().any(|item| {
        matches!(
            item,
            ModelInputItem::ToolOutput(output)
                if output.call_id == "call-1" && output.output.contains("\"tool\":\"search\"")
        )
    }));
    assert!(!second_input.iter().any(|item| {
        matches!(
            item,
            ModelInputItem::Message(message)
                if message.content == "Tool calls accepted and executed."
        )
    }));
}

#[tokio::test]
async fn search_tool_output_includes_full_results_for_layer2() {
    let request = aspect_request();
    let model_calls = Arc::new(AtomicUsize::new(0));
    let search_calls = Arc::new(AtomicUsize::new(0));
    let captured_requests = Arc::new(Mutex::new(Vec::new()));
    let mut model_service = ModelService::new();
    model_service.register(CapturingSequenceModelProvider {
        calls: model_calls.clone(),
        responses: Mutex::new(
            vec![tool_response("search"), final_response(valid_report_json())].into(),
        ),
        requests: captured_requests.clone(),
    });
    let mut search_service = SearchService::new();
    search_service.register(CountingSearchProvider {
        calls: search_calls.clone(),
    });

    AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("agent output");

    let requests = captured_requests.lock().expect("requests lock").clone();
    let tool_output = requests[1]
        .input
        .iter()
        .find_map(|item| match item {
            ModelInputItem::ToolOutput(output) => Some(output.output.as_str()),
            _ => None,
        })
        .expect("tool output");
    let value = serde_json::from_str::<serde_json::Value>(tool_output).expect("tool output json");

    assert!(value.get("evidence_ids").is_none());
    assert_eq!(value["tool"], "search");
    assert_eq!(value["query"], "private query");
    assert_eq!(value["provider"], "searcher");
    assert_eq!(value["result_count"], 1);
    assert_eq!(value["results"][0]["id"], "ev-1-1");
    assert_eq!(value["results"][0]["source_title"], "Source");
    assert_eq!(value["results"][0]["url"], "https://example.test/source");
    assert_eq!(value["results"][0]["query"], "private query");
    assert_eq!(value["results"][0]["snippet"], "snippet");
    assert_eq!(value["results"][0]["summary"], "summary");
    assert_eq!(value["results"][0]["published_at"], "2026-05-25");
    assert!(value["results"][0]["retrieved_at"].as_str().is_some());
}

#[tokio::test]
async fn rejects_selected_evidence_not_seen_in_tool_output() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, _search_calls) = services(vec![
        tool_response("search"),
        final_response(UNKNOWN_EVIDENCE_SENTINEL.to_owned()),
    ]);

    let error = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("unknown evidence rejected");

    assert!(matches!(error.error, Error::SchemaValidationFailed { .. }));
}

#[tokio::test]
async fn rejects_tampered_evidence_provenance() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, _search_calls) = services(vec![
        tool_response("search"),
        final_response(TAMPERED_EVIDENCE_SENTINEL.to_owned()),
    ]);

    let error = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("tampered evidence rejected");

    assert!(matches!(error.error, Error::SchemaValidationFailed { .. }));
}

#[tokio::test]
async fn allows_layer2_to_set_interpretive_evidence_fields() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, _search_calls) = services(vec![
        tool_response("search"),
        final_response(INTERPRETIVE_EVIDENCE_SENTINEL.to_owned()),
    ]);

    let output = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("interpretive fields accepted");

    assert_eq!(output.result.evidence[0].source_type, SourceType::Official);
    assert_eq!(output.result.evidence[0].confidence, Confidence::High);
}

#[tokio::test]
async fn model_tool_outputs_fallback_replays_tool_calls() {
    let request = aspect_request();
    let model_calls = Arc::new(AtomicUsize::new(0));
    let search_calls = Arc::new(AtomicUsize::new(0));
    let captured_requests = Arc::new(Mutex::new(Vec::new()));
    let mut model_service = ModelService::new();
    model_service.register(CapturingSequenceModelProvider {
        calls: model_calls.clone(),
        responses: Mutex::new(
            vec![
                tool_response_without_output_items("search"),
                final_response(valid_report_json()),
            ]
            .into(),
        ),
        requests: captured_requests.clone(),
    });
    let mut search_service = SearchService::new();
    search_service.register(CountingSearchProvider {
        calls: search_calls.clone(),
    });

    AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("agent output");

    let requests = captured_requests.lock().expect("requests lock").clone();
    let second_input = &requests[1].input;
    assert!(second_input.iter().any(|item| {
        matches!(
            item,
            ModelInputItem::ToolCall(call)
                if call.id == "call-1" && call.name == "search"
        )
    }));
    assert!(second_input.iter().any(|item| {
        matches!(
            item,
            ModelInputItem::ToolOutput(output) if output.call_id == "call-1"
        )
    }));
}

#[tokio::test]
async fn model_tool_outputs_use_previous_response_id_when_available() {
    let request = aspect_request();
    let model_calls = Arc::new(AtomicUsize::new(0));
    let search_calls = Arc::new(AtomicUsize::new(0));
    let captured_requests = Arc::new(Mutex::new(Vec::new()));
    let mut model_service = ModelService::new();
    model_service.register(CapturingSequenceModelProvider {
        calls: model_calls.clone(),
        responses: Mutex::new(
            vec![
                tool_response_with_response_id("search", "resp_1"),
                final_response(valid_report_json()),
            ]
            .into(),
        ),
        requests: captured_requests.clone(),
    });
    let mut search_service = SearchService::new();
    search_service.register(CountingSearchProvider {
        calls: search_calls.clone(),
    });

    AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("agent output");

    assert_eq!(model_calls.load(Ordering::SeqCst), 2);
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
    let requests = captured_requests.lock().expect("requests lock").clone();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[1].previous_response_id.as_deref(), Some("resp_1"));
    assert_eq!(requests[1].input.len(), 1);
    assert!(matches!(
        &requests[1].input[0],
        ModelInputItem::ToolOutput(output)
            if output.call_id == "call-1" && output.output.contains("\"tool\":\"search\"")
    ));
}

#[tokio::test]
async fn model_tool_outputs_can_fall_back_after_previous_response_id() {
    let request = aspect_request();
    let model_calls = Arc::new(AtomicUsize::new(0));
    let search_calls = Arc::new(AtomicUsize::new(0));
    let captured_requests = Arc::new(Mutex::new(Vec::new()));
    let mut model_service = ModelService::new();
    model_service.register(CapturingSequenceModelProvider {
        calls: model_calls.clone(),
        responses: Mutex::new(
            vec![
                tool_response_with_response_id("search", "resp_1"),
                tool_response_without_output_items("search"),
                final_response(valid_report_json()),
            ]
            .into(),
        ),
        requests: captured_requests.clone(),
    });
    let mut search_service = SearchService::new();
    search_service.register(CountingSearchProvider {
        calls: search_calls.clone(),
    });

    AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("agent output");

    assert_eq!(model_calls.load(Ordering::SeqCst), 3);
    assert_eq!(search_calls.load(Ordering::SeqCst), 2);
    let requests = captured_requests.lock().expect("requests lock").clone();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[1].previous_response_id.as_deref(), Some("resp_1"));
    assert_eq!(requests[1].input.len(), 1);
    assert_eq!(requests[2].previous_response_id, None);
    assert!(requests[2]
        .input
        .iter()
        .any(|item| matches!(item, ModelInputItem::Message(message) if message.content.contains("aspect-1"))));
    assert_eq!(
        requests[2]
            .input
            .iter()
            .filter(|item| matches!(item, ModelInputItem::ToolCall(_)))
            .count(),
        2
    );
    assert_eq!(
        requests[2]
            .input
            .iter()
            .filter(|item| matches!(item, ModelInputItem::ToolOutput(_)))
            .count(),
        2
    );
}

#[tokio::test]
async fn evidence_includes_structured_sources() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, _search_calls) = services(vec![
        tool_response("search"),
        final_response(valid_report_json()),
    ]);

    let output = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("runtime output");

    assert_eq!(output.result.evidence[0].query, "private query");
    assert_eq!(output.result.evidence[0].provider, "searcher");
    assert_eq!(output.result.evidence[0].source_title, "Source");
    assert_eq!(
        output.result.evidence[0].url.as_deref(),
        Some("https://example.test/source")
    );
}

#[tokio::test]
async fn success_output_retains_budget_usage_and_token_usage() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, _search_calls) = services(vec![
        tool_response("search"),
        final_response(valid_report_json()),
    ]);

    let output = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("runtime output");

    assert_eq!(output.budget_usage.turns_used, 2);
    assert_eq!(output.budget_usage.search_calls_used, 1);
    assert!(output.token_usage.is_none());
    assert_eq!(output.result.evidence[0].query, "private query");
    assert_eq!(
        output.result.evidence[0].url.as_deref(),
        Some("https://example.test/source")
    );
}

#[tokio::test]
async fn budget_failure_stops_after_completed_searches() {
    let mut request = aspect_request();
    request.task.budget.max_search_calls = Limit::limited(2);
    let (model_service, search_service, _model_calls, search_calls) = services(vec![
        tool_response("search"),
        tool_response("search"),
        tool_response("search"),
    ]);

    let failure = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("budget failure");

    assert!(matches!(failure.error, Error::BudgetExceeded { .. }));
    assert_eq!(search_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn provider_failure_returns_error_after_prior_successful_search() {
    let request = aspect_request();

    let model_calls = Arc::new(AtomicUsize::new(0));
    let search_calls = Arc::new(AtomicUsize::new(0));
    let mut model_service = ModelService::new();
    model_service.register(SequenceModelProvider {
        calls: model_calls,
        responses: Mutex::new(vec![tool_response("search"), tool_response("search")].into()),
        delay: None,
    });
    let mut search_service = SearchService::new();
    search_service.register(SequenceSearchProvider {
        calls: search_calls.clone(),
        responses: Mutex::new(
            vec![
                Ok(search_response()),
                Err(Error::NetworkFailed {
                    message: "provider unavailable".to_owned(),
                }),
            ]
            .into(),
        ),
    });

    let failure = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("provider failure");

    assert!(matches!(failure.error, Error::NetworkFailed { .. }));
    assert_eq!(search_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn budget_exhaustion_stops_before_actions() {
    let mut zero_turn_request = aspect_request();
    zero_turn_request.task.budget.max_turns = Limit::limited(0);
    let (model_service, search_service, model_calls, search_calls) = services(vec![]);

    let error = AgentRuntime::new(
        &model_service,
        &search_service,
        &zero_turn_request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("budget error");

    assert!(matches!(error.error, Error::BudgetExceeded { .. }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 0);
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);

    let mut request = aspect_request();
    request.task.budget.max_search_calls = Limit::limited(0);
    let (model_service, search_service, model_calls, search_calls) =
        services(vec![tool_response("search")]);

    let error = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("search budget error");

    assert!(matches!(error.error, Error::BudgetExceeded { .. }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 1);
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn slow_final_model_call_exhausts_effective_timeout() {
    let mut request = aspect_request();
    request.task.budget.timeout_ms = Limit::limited(60_000);
    request.execution_policy.timeout_ms = Limit::limited(1);
    let (model_service, search_service, model_calls, search_calls) = services_with_delay(
        vec![final_response("{}".to_owned())],
        Some(Duration::from_millis(5)),
    );

    let error = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("execution timeout error");

    assert!(matches!(error.error, Error::BudgetExceeded { .. }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 1);
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn lower_execution_timeout_is_enforced_before_search() {
    let mut request = aspect_request();
    request.task.budget.timeout_ms = Limit::limited(60_000);
    request.execution_policy.timeout_ms = Limit::limited(1);
    let (model_service, search_service, model_calls, search_calls) = services_with_delay(
        vec![tool_response("search")],
        Some(Duration::from_millis(5)),
    );

    let error = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("execution timeout error");

    assert!(matches!(error.error, Error::BudgetExceeded { .. }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 1);
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn invalid_tool_stops_without_search() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, search_calls) =
        services(vec![tool_response("filesystem")]);

    let error = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("tool policy error");

    assert!(matches!(error.error, Error::ToolPolicyDenied { .. }));
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn invalid_final_output_returns_schema_failure() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, _search_calls) =
        services(vec![final_response("{}".to_owned())]);

    let error = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("schema error");

    assert!(matches!(error.error, Error::SchemaValidationFailed { .. }));
}

/// Returns a model response with two tool calls sharing the same `id`, so the
/// orchestrator's pre-dispatch dedup guard can reject the batch.
fn duplicate_id_tool_response() -> ModelResponse {
    let tool_call_a = ModelToolCall {
        id: "dup-1".to_owned(),
        name: "search".to_owned(),
        arguments: json!({"query": "first", "max_results": 1}),
    };
    let tool_call_b = ModelToolCall {
        id: "dup-1".to_owned(),
        name: "search".to_owned(),
        arguments: json!({"query": "second", "max_results": 1}),
    };
    ModelResponse {
        provider: "model".to_owned(),
        model: None,
        response_id: None,
        content: None,
        tool_calls: vec![tool_call_a.clone(), tool_call_b.clone()],
        output_items: vec![
            ModelInputItem::tool_call(tool_call_a),
            ModelInputItem::tool_call(tool_call_b),
        ],
        usage: None,
    }
}

/// Whole-batch tool-call validation MUST run before any tool dispatches, so a
/// duplicate `tool_call.id` is rejected with `ToolPolicyDenied` and the
/// search service sees zero calls. Regression guard for M8.
#[tokio::test]
async fn duplicate_tool_call_ids_are_rejected_before_dispatch() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, search_calls) =
        services(vec![duplicate_id_tool_response()]);

    let error = AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect_err("duplicate tool call must fail");

    assert!(matches!(error.error, Error::ToolPolicyDenied { .. }));
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn agent_loop_forwards_search_tool_call_time_params() {
    let request = aspect_request();
    let tool_call = ModelToolCall {
        id: "call-1".to_owned(),
        name: "search".to_owned(),
        arguments: json!({
            "query": "private query",
            "max_results": 1,
            "depth": "balanced",
            "content_level": "standard",
            "recency": "fresh",
            "category": "news"
        }),
    };
    let model_response = ModelResponse {
        provider: "model".to_owned(),
        model: None,
        response_id: None,
        content: None,
        tool_calls: vec![tool_call.clone()],
        output_items: vec![ModelInputItem::tool_call(tool_call)],
        usage: None,
    };
    let mut model_service = ModelService::new();
    model_service.register(SequenceModelProvider {
        calls: Arc::new(AtomicUsize::new(0)),
        responses: Mutex::new(vec![model_response, final_response(valid_report_json())].into()),
        delay: None,
    });
    let captured_requests = Arc::new(Mutex::new(Vec::new()));
    let mut search_service = SearchService::new();
    search_service.register(CapturingSearchProvider {
        requests: captured_requests.clone(),
    });

    AgentRuntime::new(
        &model_service,
        &search_service,
        &request,
        ResearchBudgetGuard::unlimited(),
    )
    .run()
    .await
    .expect("agent output");

    let requests = captured_requests.lock().expect("requests lock");
    assert_eq!(requests[0].depth, Some(SearchDepth::Balanced));
    assert_eq!(
        requests[0].content_level,
        Some(SearchContentLevel::Standard)
    );
    assert_eq!(requests[0].recency, Some(SearchRecency::Fresh));
    assert_eq!(requests[0].category, Some(SearchCategory::News));
}

/// When an aspect's `allowed_tools` is empty the tool guard MUST advertise no
/// tools, closing the gap where a model could still see denied tools.
/// Regression guard for M13 (dynamic tool advertisement by policy).
#[test]
fn aspect_with_empty_allowed_tools_advertises_no_tools() {
    let mut request = aspect_request();
    request.task.aspect.allowed_tools.clear();
    let guard = lapis_workflow::ToolPolicyGuard::new(&request.task.aspect);
    assert!(
        guard.allowed_model_tools().is_empty(),
        "tools list must be empty when allowed_tools is empty"
    );
}

/// Integration guard for M13: an aspect with empty `allowed_tools` MUST
/// reach the model provider with an actually empty `ModelRequest.tools`
/// list, not merely a guard-level abstraction. This pins the wire shape.
#[tokio::test]
async fn aspect_with_empty_allowed_tools_sends_empty_model_tools() {
    let mut request = aspect_request();
    request.task.aspect.allowed_tools.clear();
    request.task.aspect.search_provider = None;
    request.evidence_policy.require_evidence_for_findings = false;

    let final_payload = serde_json::to_string(&AspectResearchResult {
        aspect_report: AspectReport {
            aspect_id: "aspect-1".to_owned(),
            aspect_name: "Aspect".to_owned(),
            question: "What is true?".to_owned(),
            scope: vec!["scope".to_owned()],
            findings: Vec::new(),
            assumptions: Vec::new(),
            risks: Vec::new(),
            counterarguments: Vec::new(),
            open_questions: Vec::new(),
            confidence: Confidence::Medium,
            limitations: Vec::new(),
        },
        evidence: Vec::new(),
    })
    .expect("report json");

    let model_calls = Arc::new(AtomicUsize::new(0));
    let captured_requests = Arc::new(Mutex::new(Vec::new()));
    let mut model_service = ModelService::new();
    model_service.register(CapturingSequenceModelProvider {
        calls: model_calls,
        responses: Mutex::new(vec![final_response(final_payload)].into()),
        requests: captured_requests.clone(),
    });
    let search_service = SearchService::new();

    aspect_research(
        request,
        &model_service,
        &search_service,
        &unlimited_budget_config(),
    )
    .await
    .expect("non-search aspect result");

    let requests = captured_requests.lock().expect("requests lock").clone();
    assert!(
        requests[0].tools.is_empty(),
        "ModelRequest.tools must be empty when allowed_tools is empty"
    );
}

/// When an aspect permits the search tool the guard MUST advertise exactly
/// the search tool (no extras). Regression guard for M13.
#[test]
fn aspect_with_search_tool_advertises_only_search_tool() {
    let request = aspect_request();
    let guard = lapis_workflow::ToolPolicyGuard::new(&request.task.aspect);
    let tools = guard.allowed_model_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "search");
}
