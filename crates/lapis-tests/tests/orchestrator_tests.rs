use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use lapis_core::error::{Error, Result};
use lapis_core::model::provider::ModelProvider;
use lapis_core::model::service::ModelService;
use lapis_core::orchestrator::agent_loop::AgentRuntime;
use lapis_core::orchestrator::budget::AgentBudgetGuard;
use lapis_core::orchestrator::workflow::aspect_research;
use lapis_core::schema::budget::AgentBudget;
use lapis_core::schema::config::BudgetConfig;
use lapis_core::schema::limit::Limit;
use lapis_core::schema::model::{ModelRequest, ModelResponse, ModelToolCall};
use lapis_core::schema::policy::{
    EvidencePolicy, EvidenceRequirement, ExecutionPolicy, ModelPolicy, OutputPolicy, SearchPolicy,
    ToolName,
};
use lapis_core::schema::report::{
    AspectReport, Confidence, Finding, FindingType, Importance, OpenQuestion,
};
use lapis_core::schema::research::{
    AspectResearchRequest, AspectSpec, PromptAssets, ResearchContext,
};
use lapis_core::schema::search::{ProviderSearchRequest, SearchResponse, SearchResult};
use lapis_core::search::provider::SearchProvider;
use lapis_core::search::service::SearchService;
use serde_json::json;

struct SequenceModelProvider {
    calls: Arc<AtomicUsize>,
    responses: Mutex<VecDeque<ModelResponse>>,
    delay: Option<Duration>,
}

#[async_trait]
impl ModelProvider for SequenceModelProvider {
    fn name(&self) -> &'static str {
        "model"
    }

    async fn complete(&self, _request: ModelRequest) -> Result<ModelResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }
        self.responses
            .lock()
            .expect("responses lock")
            .pop_front()
            .ok_or_else(|| Error::Internal {
                message: "missing fake model response".to_owned(),
            })
    }
}

struct CountingSearchProvider {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl SearchProvider for CountingSearchProvider {
    fn name(&self) -> &'static str {
        "searcher"
    }

    async fn search(&self, _request: ProviderSearchRequest) -> Result<SearchResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(SearchResponse {
            provider: "searcher".to_owned(),
            results: vec![SearchResult {
                title: "Source".to_owned(),
                url: Some("https://example.test/source".to_owned()),
                snippet: "snippet".to_owned(),
                summary: Some("summary".to_owned()),
                published_at: None,
            }],
        })
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

fn prompt_assets() -> PromptAssets {
    PromptAssets {
        aspect_agent_prompt_path: "prompts/layer2/aspect-agent.md".to_owned(),
    }
}

fn aspect_request() -> AspectResearchRequest {
    AspectResearchRequest {
        schema_version: "m4".to_owned(),
        request_id: "request-1".to_owned(),
        aspect: AspectSpec {
            aspect_id: "aspect-1".to_owned(),
            name: "Aspect".to_owned(),
            role: "researcher".to_owned(),
            research_question: "What is true?".to_owned(),
            scope: vec!["scope".to_owned()],
            boundaries: vec![],
            success_criteria: vec!["answer".to_owned()],
            prompt_assets: prompt_assets(),
            required_evidence: EvidenceRequirement::default(),
            allowed_tools: vec![ToolName("search".to_owned())],
            model_override: None,
            search_override: None,
            budget_override: None,
        },
        shared_context: ResearchContext::default(),
        model_policy: ModelPolicy {
            default_provider: "model".to_owned(),
            allowed_providers: vec!["model".to_owned()],
            ..ModelPolicy::default()
        },
        search_policy: SearchPolicy {
            allowed_providers: vec!["searcher".to_owned()],
            preferred_providers: vec!["searcher".to_owned()],
            max_results_per_query: 2,
            ..SearchPolicy::default()
        },
        evidence_policy: EvidencePolicy {
            include_query_trace: false,
            include_source_urls: false,
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

fn tool_response(name: &str) -> ModelResponse {
    ModelResponse {
        provider: "model".to_owned(),
        model: None,
        content: None,
        tool_calls: vec![ModelToolCall {
            id: "call-1".to_owned(),
            name: name.to_owned(),
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

fn valid_report_json() -> String {
    serde_json::to_string(&AspectReport {
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
            evidence_refs: vec!["ev-1-1".to_owned()],
            contradicted_by: vec![],
        }],
        evidence: vec![],
        assumptions: vec![],
        risks: vec![],
        counterarguments: vec![],
        open_questions: Vec::<OpenQuestion>::new(),
        confidence: Confidence::Medium,
        limitations: vec![],
    })
    .expect("report json")
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
    request.aspect.research_question.clear();
    let model_service = ModelService::new();
    let search_service = SearchService::new();

    let error = aspect_research(
        request,
        &model_service,
        &search_service,
        &BudgetConfig::default(),
    )
    .await
    .expect_err("invalid request");

    assert!(matches!(error, Error::InvalidInput { .. }));
}

#[tokio::test]
async fn rejects_unsafe_prompt_asset_path() {
    let mut request = aspect_request();
    request.aspect.prompt_assets.aspect_agent_prompt_path = "../secret.md".to_owned();
    let model_service = ModelService::new();
    let search_service = SearchService::new();

    let error = aspect_research(
        request,
        &model_service,
        &search_service,
        &BudgetConfig::default(),
    )
    .await
    .expect_err("unsafe prompt path");

    assert!(matches!(error, Error::InvalidInput { .. }));
}

#[tokio::test]
async fn accepts_absolute_prompt_asset_path() {
    let mut request = aspect_request();
    request.aspect.prompt_assets.aspect_agent_prompt_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("prompts/layer2/aspect-agent.md")
            .canonicalize()
            .expect("absolute prompt path")
            .display()
            .to_string();
    let (model_service, search_service, _model_calls, search_calls) = services(vec![
        tool_response("search"),
        final_response(valid_report_json()),
    ]);

    let result = aspect_research(
        request,
        &model_service,
        &search_service,
        &BudgetConfig::default(),
    )
    .await
    .expect("valid absolute prompt path");

    assert_eq!(result.aspect_report.aspect_id, "aspect-1");
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
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
        &BudgetConfig::default(),
    )
    .await
    .expect_err("domain conflict");

    assert!(matches!(error, Error::InvalidInput { .. }));
}

#[tokio::test]
async fn rejects_execution_timeout_above_budget() {
    let mut request = aspect_request();
    request.budget.timeout_ms = Limit::limited(100);
    request.execution_policy.timeout_ms = Some(101);
    let model_service = ModelService::new();
    let search_service = SearchService::new();

    let error = aspect_research(
        request,
        &model_service,
        &search_service,
        &BudgetConfig::default(),
    )
    .await
    .expect_err("timeout conflict");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
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
        &BudgetConfig::default(),
    )
    .await
    .expect("aspect result");

    assert_eq!(result.aspect_report.aspect_id, "aspect-1");
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn fake_model_and_search_complete_successfully() {
    let request = aspect_request();
    let (model_service, search_service, model_calls, search_calls) = services(vec![
        tool_response("search"),
        final_response(valid_report_json()),
    ]);

    let output = AgentRuntime::new(&model_service, &search_service, &request)
        .run()
        .await
        .expect("runtime output");

    assert_eq!(model_calls.load(Ordering::SeqCst), 2);
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
    assert_ne!(output.evidence[0].query, "private query");
    assert_eq!(
        output.evidence[0].snippet,
        "raw search snippet omitted by output policy"
    );
    assert!(output.evidence[0].url.is_none());
    assert_ne!(output.search_queries[0].query, "private query");
    assert!(!output.tool_calls[0].input_summary.contains('{'));
}

#[tokio::test]
async fn tool_trace_summary_redacts_query_when_query_trace_is_enabled() {
    let mut request = aspect_request();
    request.evidence_policy.include_query_trace = true;
    let (model_service, search_service, _model_calls, _search_calls) = services(vec![
        tool_response("search"),
        final_response(valid_report_json()),
    ]);

    let output = AgentRuntime::new(&model_service, &search_service, &request)
        .run()
        .await
        .expect("runtime output");

    assert_eq!(output.evidence[0].query, "private query");
    assert_eq!(output.search_queries[0].query, "private query");
    assert_eq!(
        output.tool_calls[0].input_summary,
        "search query accepted, max_results=1"
    );
    assert!(!output.tool_calls[0].input_summary.contains("private query"));
}

#[tokio::test]
async fn budget_exhaustion_stops_before_actions() {
    let mut zero_turn_request = aspect_request();
    zero_turn_request.budget.max_turns = Limit::limited(0);
    let (model_service, search_service, model_calls, search_calls) = services(vec![]);

    let error = AgentRuntime::new(&model_service, &search_service, &zero_turn_request)
        .run()
        .await
        .expect_err("budget error");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 0);
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);

    let mut request = aspect_request();
    request.budget.max_search_calls = Limit::limited(0);
    let (model_service, search_service, model_calls, search_calls) =
        services(vec![tool_response("search")]);

    let error = AgentRuntime::new(&model_service, &search_service, &request)
        .run()
        .await
        .expect_err("search budget error");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 1);
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn slow_final_model_call_exhausts_effective_timeout() {
    let mut request = aspect_request();
    request.budget.timeout_ms = Limit::limited(60_000);
    request.execution_policy.timeout_ms = Some(1);
    let (model_service, search_service, model_calls, search_calls) = services_with_delay(
        vec![final_response(valid_report_json())],
        Some(Duration::from_millis(5)),
    );

    let error = AgentRuntime::new(&model_service, &search_service, &request)
        .run()
        .await
        .expect_err("execution timeout error");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 1);
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn lower_execution_timeout_is_enforced_before_search() {
    let mut request = aspect_request();
    request.budget.timeout_ms = Limit::limited(60_000);
    request.execution_policy.timeout_ms = Some(1);
    let (model_service, search_service, model_calls, search_calls) = services_with_delay(
        vec![tool_response("search")],
        Some(Duration::from_millis(5)),
    );

    let error = AgentRuntime::new(&model_service, &search_service, &request)
        .run()
        .await
        .expect_err("execution timeout error");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
    assert_eq!(model_calls.load(Ordering::SeqCst), 1);
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn invalid_tool_stops_without_search() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, search_calls) =
        services(vec![tool_response("filesystem")]);

    let error = AgentRuntime::new(&model_service, &search_service, &request)
        .run()
        .await
        .expect_err("tool policy error");

    assert!(matches!(error, Error::ToolPolicyDenied { .. }));
    assert_eq!(search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn invalid_final_output_returns_schema_failure() {
    let request = aspect_request();
    let (model_service, search_service, _model_calls, _search_calls) =
        services(vec![final_response("{}".to_owned())]);

    let error = AgentRuntime::new(&model_service, &search_service, &request)
        .run()
        .await
        .expect_err("schema error");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
}
