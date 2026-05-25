use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use lapis_core::error::{Error, Result};
use lapis_core::model::provider::ModelProvider;
use lapis_core::model::service::ModelService;
use lapis_core::orchestrator::workflow::deep_research;
use lapis_core::schema::budget::{AgentBudget, BudgetConfig, ResearchBudget};
use lapis_core::schema::limit::Limit;
use lapis_core::schema::model::{ModelInputItem, ModelRequest, ModelResponse, ModelToolCall};
use lapis_core::schema::policy::{
    EvidencePolicy, ExecutionPolicy, ModelPolicy, OutputPolicy, SearchPolicy, ToolName,
};
use lapis_core::schema::report::{
    AspectReport, AspectResearchResult, Confidence, Evidence, Finding, FindingType, Importance,
    OpenQuestion,
};
use lapis_core::schema::research::{
    AspectResearchTask, AspectSpec, DeepResearchRequest, ResearchContext,
};
use lapis_core::schema::search::{SearchRequest, SearchResponse, SearchResult};
use lapis_core::search::provider::SearchProvider;
use lapis_core::search::service::SearchService;
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
        timeout_ms,
    }
}

struct AdaptiveModelProvider {
    failing_aspects: BTreeSet<String>,
    calls: Arc<AtomicUsize>,
    in_flight: Arc<AtomicUsize>,
    max_in_flight: Arc<AtomicUsize>,
    delay: Duration,
}

#[async_trait]
impl ModelProvider for AdaptiveModelProvider {
    fn name(&self) -> &'static str {
        "model"
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let current = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_in_flight.fetch_max(current, Ordering::SeqCst);
        tokio::time::sleep(self.delay).await;
        self.in_flight.fetch_sub(1, Ordering::SeqCst);

        let aspect_id = aspect_field(&request.input, "Aspect ID");
        let aspect_name = aspect_field(&request.input, "Aspect name");

        if !has_tool_output(&request.input) {
            return Ok(tool_response());
        }

        if self.failing_aspects.contains(&aspect_id) {
            return Ok(final_response("{}".to_owned()));
        }

        let evidence = first_evidence_from_tool_output(&request.input);
        Ok(final_response(result_json(
            &aspect_id,
            &aspect_name,
            Confidence::Medium,
            evidence,
        )))
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

    async fn search(&self, _request: SearchRequest) -> Result<SearchResponse> {
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
    max_in_flight: Arc<AtomicUsize>,
}

fn services(failing_aspects: &[&str]) -> Services {
    let model_calls = Arc::new(AtomicUsize::new(0));
    let search_calls = Arc::new(AtomicUsize::new(0));
    let in_flight = Arc::new(AtomicUsize::new(0));
    let max_in_flight = Arc::new(AtomicUsize::new(0));
    let mut model = ModelService::new();
    model.register(AdaptiveModelProvider {
        failing_aspects: failing_aspects
            .iter()
            .map(|aspect| (*aspect).to_owned())
            .collect(),
        calls: model_calls.clone(),
        in_flight,
        max_in_flight: max_in_flight.clone(),
        delay: Duration::from_millis(10),
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
        max_in_flight,
    }
}

fn prompt_path() -> String {
    "prompts/layer2/aspect-agent.md".to_owned()
}

fn deep_request(count: usize) -> DeepResearchRequest {
    DeepResearchRequest {
        schema_version: "m5".to_owned(),
        request_id: "request-1".to_owned(),
        user_question: "What is true?".to_owned(),
        aspect_tasks: (1..=count).map(aspect_task).collect(),
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
        evidence_policy: evidence_policy(),
        output_policy: output_policy(),
        shared_context: ResearchContext::empty(),
        execution_policy: execution_policy(Some(180_000)),
    }
}

fn aspect_task(index: usize) -> AspectResearchTask {
    AspectResearchTask {
        aspect: AspectSpec {
            aspect_id: format!("aspect-{index}"),
            name: format!("Aspect {index}"),
            role: "researcher".to_owned(),
            research_question: format!("Question {index}?"),
            scope: vec!["scope".to_owned()],
            boundaries: vec![],
            success_criteria: vec!["answer".to_owned()],
            aspect_agent_prompt_path: prompt_path(),
            allowed_tools: vec![ToolName("search".to_owned())],
            model_provider: Some("model".to_owned()),
            search_provider: Some("searcher".to_owned()),
        },
        budget: AgentBudget::unlimited(),
    }
}

fn tool_response() -> ModelResponse {
    let tool_call = ModelToolCall {
        id: "call-1".to_owned(),
        name: "search".to_owned(),
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

fn report(
    aspect_id: &str,
    aspect_name: &str,
    confidence: Confidence,
    evidence_id: String,
) -> AspectReport {
    AspectReport {
        aspect_id: aspect_id.to_owned(),
        aspect_name: aspect_name.to_owned(),
        question: "What is true?".to_owned(),
        scope: vec!["scope".to_owned()],
        findings: vec![Finding {
            id: format!("finding-{aspect_id}"),
            claim: "A supported claim".to_owned(),
            finding_type: FindingType::Fact,
            importance: Importance::High,
            confidence,
            evidence_refs: vec![evidence_id],
            contradicted_by: vec![],
        }],
        assumptions: vec![],
        risks: vec![],
        counterarguments: vec![],
        open_questions: vec![OpenQuestion {
            id: format!("open-{aspect_id}"),
            question: "What remains uncertain?".to_owned(),
            reason: "Budget limited".to_owned(),
            suggested_follow_up: vec!["Search again".to_owned()],
        }],
        confidence,
        limitations: vec![],
    }
}

fn result_json(
    aspect_id: &str,
    aspect_name: &str,
    confidence: Confidence,
    mut evidence: Evidence,
) -> String {
    evidence.supports_findings = vec![format!("finding-{aspect_id}")];
    serde_json::to_string(&AspectResearchResult {
        aspect_report: report(aspect_id, aspect_name, confidence, evidence.id.clone()),
        evidence: vec![evidence],
    })
    .expect("result json")
}

fn first_evidence_from_tool_output(input: &[ModelInputItem]) -> Evidence {
    let output = input
        .iter()
        .rev()
        .find_map(|item| match item {
            ModelInputItem::ToolOutput(output) => Some(output.output.as_str()),
            _ => None,
        })
        .expect("tool output");
    let value = serde_json::from_str::<serde_json::Value>(output).expect("tool output json");
    serde_json::from_value(value["results"][0].clone()).expect("evidence result")
}

fn has_tool_output(input: &[ModelInputItem]) -> bool {
    input
        .iter()
        .any(|item| matches!(item, ModelInputItem::ToolOutput(_)))
}

fn aspect_field(input: &[ModelInputItem], label: &str) -> String {
    let pointer = match label {
        "Aspect ID" => "/task/aspect/aspect_id",
        "Aspect name" => "/task/aspect/name",
        _ => return String::new(),
    };

    input
        .iter()
        .find_map(|item| {
            let ModelInputItem::Message(message) = item else {
                return None;
            };

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

#[tokio::test]
async fn completes_three_aspects_with_bounded_concurrency() {
    let request = deep_request(3);
    let services = services(&[]);

    let result = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect("deep result");

    assert_eq!(result.completed_aspects.len(), 3);
    assert!(result.failed_aspects.is_empty());
    assert_eq!(result.aspect_reports.len(), 3);
    assert_eq!(result.evidence_index.len(), 3);
    let evidence_ids = result
        .evidence_index
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect::<BTreeSet<_>>();
    for report in &result.aspect_reports {
        for finding in &report.findings {
            assert!(
                finding
                    .evidence_refs
                    .iter()
                    .all(|id| evidence_ids.contains(id.as_str()))
            );
        }
    }
    assert_eq!(result.open_questions.len(), 3);
    assert_eq!(result.coverage_summary.requested_aspects, 3);
    assert_eq!(result.coverage_summary.completed_aspects, 3);
    assert_eq!(result.coverage_summary.failed_aspects, 0);
    assert_eq!(result.confidence_summary.medium, 3);
    assert_eq!(result.budget_usage.model_calls_used, 6);
    assert_eq!(result.budget_usage.search_calls_used, 3);
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 6);
    assert_eq!(services.search_calls.load(Ordering::SeqCst), 3);
    assert_eq!(services.max_in_flight.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn returns_partial_result_after_single_aspect_failure() {
    let request = deep_request(3);
    let services = services(&["aspect-2"]);

    let result = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect("partial result");

    assert_eq!(result.completed_aspects.len(), 2);
    assert_eq!(result.failed_aspects.len(), 1);
    assert_eq!(result.failed_aspects[0].aspect_id, "aspect-2");
}

#[tokio::test]
async fn all_aspects_failed_returns_error() {
    let request = deep_request(2);
    let services = services(&["aspect-1", "aspect-2"]);

    let error = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("all failed");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
}

#[tokio::test]
async fn partial_results_disabled_returns_error() {
    let mut request = deep_request(3);
    request.execution_policy.allow_partial_results = false;
    let services = services(&["aspect-2"]);

    let error = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("partial disabled");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
}

#[tokio::test]
async fn fail_fast_stops_before_scheduling_remaining_aspects() {
    let mut request = deep_request(2);
    request.budget.max_concurrent_agents = Limit::limited(1);
    request.execution_policy.fail_fast = true;
    let services = services(&["aspect-1"]);

    let error = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("fail fast error");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 2);
    assert_eq!(services.search_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn rejects_plan_exceeding_max_agents() {
    let mut request = deep_request(3);
    request.budget.max_agents = Limit::limited(2);
    let services = services(&[]);

    let error = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("too many aspects");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn rejects_research_budget_above_configured_limits() {
    let request = deep_request(3);
    let services = services(&[]);
    let limits = BudgetConfig {
        research: ResearchBudget {
            max_agents: Limit::limited(2),
            ..ResearchBudget::unlimited()
        },
        per_agent: AgentBudget::unlimited(),
    };

    let error = deep_research(request, &services.model, &services.search, &limits)
        .await
        .expect_err("budget exceeds configured limits");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn rejects_research_concurrency_above_configured_limits() {
    let request = deep_request(3);
    let services = services(&[]);
    let limits = BudgetConfig {
        research: ResearchBudget {
            max_concurrent_agents: Limit::limited(1),
            ..ResearchBudget::unlimited()
        },
        per_agent: AgentBudget::unlimited(),
    };

    let error = deep_research(request, &services.model, &services.search, &limits)
        .await
        .expect_err("concurrency exceeds configured limits");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn rejects_agent_budget_above_configured_limits() {
    let request = deep_request(2);
    let services = services(&[]);
    let limits = BudgetConfig {
        research: ResearchBudget::unlimited(),
        per_agent: AgentBudget {
            max_turns: Limit::limited(5),
            ..AgentBudget::unlimited()
        },
    };

    let error = deep_research(request, &services.model, &services.search, &limits)
        .await
        .expect_err("agent budget exceeds configured limits");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn global_search_budget_is_checked_after_aggregation() {
    let mut request = deep_request(2);
    request.budget.max_total_search_calls = Limit::limited(1);
    let services = services(&[]);

    let error = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("global search budget");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
    assert_eq!(services.search_calls.load(Ordering::SeqCst), 2);
}
