use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use lapis_core::error::{Error, Result};
use lapis_core::model::provider::ModelProvider;
use lapis_core::model::service::ModelService;
use lapis_core::orchestrator::workflow::deep_research;
use lapis_core::schema::common::{
    AgentBudget, AspectSpec, DeepResearchRequest, DeliverableSpec, EvidencePolicy,
    EvidenceRequirement, ExecutionPolicy, ModelPolicy, OutputPolicy, ResearchBudget,
    ResearchContext, ResearchPlan, SearchPolicy, ToolName,
};
use lapis_core::schema::model::{ModelMessage, ModelRequest, ModelResponse, ModelToolCall};
use lapis_core::schema::report::{
    AspectReport, Confidence, Finding, FindingType, Importance, OpenQuestion, TerminationReason,
};
use lapis_core::schema::search::{SearchRequest, SearchResponse, SearchResult};
use lapis_core::search::provider::SearchProvider;
use lapis_core::search::service::SearchService;
use serde_json::json;

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

        let aspect_id = aspect_field(&request.messages, "Aspect ID");
        let aspect_name = aspect_field(&request.messages, "Aspect name");

        if request.messages.len() <= 2 {
            return Ok(tool_response());
        }

        if self.failing_aspects.contains(&aspect_id) {
            return Ok(final_response("{}".to_owned()));
        }

        Ok(final_response(report_json(
            &aspect_id,
            &aspect_name,
            Confidence::Medium,
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
                max_agents: count,
                max_concurrent_agents: 2,
                max_total_model_calls: 20,
                max_total_search_calls: 20,
                total_timeout_ms: 180_000,
                max_tokens: None,
            },
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
        required_evidence: EvidenceRequirement::default(),
        allowed_tools: vec![ToolName("search".to_owned())],
        model_override: None,
        search_override: None,
        budget_override: Some(AgentBudget::default()),
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

fn report_json(aspect_id: &str, aspect_name: &str, confidence: Confidence) -> String {
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
            confidence,
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
        confidence,
        limitations: vec![],
    })
    .expect("report json")
}

fn aspect_field(messages: &[ModelMessage], label: &str) -> String {
    messages
        .iter()
        .find_map(|message| {
            message.content.lines().find_map(|line| {
                line.strip_prefix(label)
                    .and_then(|value| value.strip_prefix(": "))
                    .map(str::to_owned)
            })
        })
        .unwrap_or_default()
}

#[tokio::test]
async fn completes_three_aspects_with_bounded_concurrency() {
    let request = deep_request(3);
    let services = services(&[]);

    let result = deep_research(request, &services.model, &services.search)
        .await
        .expect("deep result");

    assert_eq!(result.completed_aspects.len(), 3);
    assert!(result.failed_aspects.is_empty());
    assert_eq!(result.aspect_reports.len(), 3);
    assert_eq!(result.evidence_index.len(), 1);
    assert_eq!(result.open_questions.len(), 3);
    assert_eq!(result.coverage_summary.requested_aspects, 3);
    assert_eq!(result.coverage_summary.completed_aspects, 3);
    assert_eq!(result.coverage_summary.failed_aspects, 0);
    assert_eq!(result.confidence_summary.medium, 3);
    assert_eq!(result.budget_usage.model_calls_used, 6);
    assert_eq!(result.budget_usage.search_calls_used, 3);
    assert_eq!(
        result.trace_summary.termination_reason,
        Some(TerminationReason::Completed)
    );
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 6);
    assert_eq!(services.search_calls.load(Ordering::SeqCst), 3);
    assert_eq!(services.max_in_flight.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn returns_partial_result_after_single_aspect_failure() {
    let request = deep_request(3);
    let services = services(&["aspect-2"]);

    let result = deep_research(request, &services.model, &services.search)
        .await
        .expect("partial result");

    assert_eq!(result.completed_aspects.len(), 2);
    assert_eq!(result.failed_aspects.len(), 1);
    assert_eq!(result.failed_aspects[0].aspect_id, "aspect-2");
    assert_eq!(
        result.trace_summary.termination_reason,
        Some(TerminationReason::PartialCompleted)
    );
}

#[tokio::test]
async fn all_aspects_failed_returns_error() {
    let request = deep_request(2);
    let services = services(&["aspect-1", "aspect-2"]);

    let error = deep_research(request, &services.model, &services.search)
        .await
        .expect_err("all failed");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
}

#[tokio::test]
async fn partial_results_disabled_returns_error() {
    let mut request = deep_request(3);
    request.execution_policy.allow_partial_results = false;
    let services = services(&["aspect-2"]);

    let error = deep_research(request, &services.model, &services.search)
        .await
        .expect_err("partial disabled");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
}

#[tokio::test]
async fn fail_fast_stops_before_scheduling_remaining_aspects() {
    let mut request = deep_request(2);
    request.plan.budget.max_concurrent_agents = 1;
    request.execution_policy.fail_fast = true;
    let services = services(&["aspect-1"]);

    let error = deep_research(request, &services.model, &services.search)
        .await
        .expect_err("fail fast error");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 2);
    assert_eq!(services.search_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn rejects_plan_exceeding_max_agents() {
    let mut request = deep_request(3);
    request.plan.budget.max_agents = 2;
    let services = services(&[]);

    let error = deep_research(request, &services.model, &services.search)
        .await
        .expect_err("too many aspects");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn global_search_budget_is_checked_after_aggregation() {
    let mut request = deep_request(2);
    request.plan.budget.max_total_search_calls = 1;
    let services = services(&[]);

    let error = deep_research(request, &services.model, &services.search)
        .await
        .expect_err("global search budget");

    assert!(matches!(error, Error::BudgetExceeded { .. }));
    assert_eq!(services.search_calls.load(Ordering::SeqCst), 2);
}
