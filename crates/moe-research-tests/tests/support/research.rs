#![allow(dead_code)]

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use moe_research_error::Result;
use moe_research_model::ModelProvider;
use moe_research_model::ModelService;
use moe_research_model::{ModelInputItem, ModelRequest, ModelResponse, ModelToolCall};
use moe_research_search::SearchProvider;
use moe_research_search::SearchService;
use moe_research_search::{SearchRequest, SearchResponse, SearchResult};
use moe_research_workflow::Limit;
use moe_research_workflow::{AgentLimits, BudgetConfig, ResearchLimits};
use moe_research_workflow::{
    AspectReport, AspectResearchResult, Confidence, Evidence, Finding, FindingType, Importance,
    OpenQuestion, TokenUsage,
};
use moe_research_workflow::{
    AspectRequest, AspectResearchRequest, DeepResearchRequest, ResearchContext, ResearchPolicy,
    ResearchTask,
};
use moe_research_workflow::{
    EvidencePolicy, ExecutionPolicy, ModelPolicy, OutputPolicy, SearchPolicy, ToolName,
};
use serde_json::json;

pub struct Services {
    pub model: ModelService,
    pub search: SearchService,
    pub model_calls: Arc<AtomicUsize>,
    pub search_calls: Arc<AtomicUsize>,
    pub max_in_flight: Arc<AtomicUsize>,
}

struct AdaptiveModelProvider {
    failing_aspects: BTreeSet<String>,
    calls: Arc<AtomicUsize>,
    in_flight: Arc<AtomicUsize>,
    max_in_flight: Arc<AtomicUsize>,
    delay: Duration,
    usage: Option<TokenUsage>,
}

struct StaticSearchProvider {
    calls: Arc<AtomicUsize>,
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

        let mut response = if !has_tool_output(&request.input) {
            tool_response()
        } else if self.failing_aspects.contains(&aspect_id) {
            final_response("{}".to_owned())
        } else {
            final_response(result_json(
                &aspect_id,
                &aspect_name,
                Confidence::Medium,
                first_evidence_from_tool_output(&request.input),
            ))
        };
        response.usage = self.usage.clone();
        Ok(response)
    }
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

pub fn unlimited_budget_config() -> BudgetConfig {
    BudgetConfig {
        research: ResearchLimits::unlimited(),
        per_agent: AgentLimits::unlimited(),
    }
}

pub fn services(failing_aspects: &[&str]) -> Services {
    services_with_options(failing_aspects, None, Duration::from_millis(10))
}

pub fn services_with_token_usage(failing_aspects: &[&str], usage: Option<TokenUsage>) -> Services {
    services_with_options(failing_aspects, usage, Duration::from_millis(10))
}

pub fn services_with_delay(failing_aspects: &[&str], delay: Duration) -> Services {
    services_with_options(failing_aspects, None, delay)
}

fn services_with_options(
    failing_aspects: &[&str],
    usage: Option<TokenUsage>,
    delay: Duration,
) -> Services {
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
        delay,
        usage,
    });
    let search = static_search_service(search_calls.clone());

    Services {
        model,
        search,
        model_calls,
        search_calls,
        max_in_flight,
    }
}

pub fn static_search_service(search_calls: Arc<AtomicUsize>) -> SearchService {
    let mut search = SearchService::new();
    search.register(StaticSearchProvider {
        calls: search_calls,
    });
    search
}

pub fn aspect_request() -> AspectResearchRequest {
    AspectResearchRequest {
        schema_version: "0.2".to_owned(),
        request_id: "request-1".to_owned(),
        task: aspect(1),
        policy: research_policy(),
        context: ResearchContext::empty(),
    }
}

pub fn deep_request(count: usize) -> DeepResearchRequest {
    DeepResearchRequest {
        schema_version: "0.2".to_owned(),
        request_id: "request-1".to_owned(),
        task: ResearchTask {
            question: "What is true?".to_owned(),
            aspects: (1..=count).map(aspect).collect(),
        },
        limits: ResearchLimits {
            max_agents: Limit::limited(count),
            max_concurrent_agents: Limit::limited(2),
            max_total_model_calls: Limit::limited(20),
            max_total_search_calls: Limit::limited(20),
            total_timeout_ms: Limit::limited(180_000),
            max_tokens: Limit::unlimited(),
        },
        policy: research_policy(),
        context: ResearchContext::empty(),
    }
}

pub fn tool_response() -> ModelResponse {
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

pub fn final_response(content: String) -> ModelResponse {
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

pub fn medium_result_json(aspect_id: &str, aspect_name: &str, evidence: Evidence) -> String {
    result_json(aspect_id, aspect_name, Confidence::Medium, evidence)
}

pub fn first_evidence_from_tool_output(input: &[ModelInputItem]) -> Evidence {
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

pub fn aspect_field(input: &[ModelInputItem], label: &str) -> String {
    let pointer = match label {
        "Aspect ID" => "/task/id",
        "Aspect name" => "/task/name",
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

fn research_policy() -> ResearchPolicy {
    ResearchPolicy {
        model: model_policy(),
        search: search_policy(),
        evidence: evidence_policy(),
        output: output_policy(),
        execution: execution_policy(),
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

fn execution_policy() -> ExecutionPolicy {
    ExecutionPolicy {
        allow_partial_results: true,
        fail_fast: false,
    }
}

fn aspect(index: usize) -> AspectRequest {
    AspectRequest {
        id: format!("aspect-{index}"),
        name: format!("Aspect {index}"),
        role: "researcher".to_owned(),
        question: format!("Question {index}?"),
        scope: vec!["scope".to_owned()],
        boundaries: vec![],
        success_criteria: vec!["answer".to_owned()],
        instructions: "# Aspect Agent\n\nDummy aspect agent prompt for tests.\n".to_owned(),
        tools: vec![ToolName("search".to_owned())],
        model_provider: "model".to_owned(),
        search_provider: Some("searcher".to_owned()),
        limits: AgentLimits {
            max_turns: Limit::limited(8),
            max_tool_calls: Limit::limited(12),
            max_search_calls: Limit::limited(6),
            timeout_ms: Limit::limited(180_000),
        },
    }
}

fn has_tool_output(input: &[ModelInputItem]) -> bool {
    input
        .iter()
        .any(|item| matches!(item, ModelInputItem::ToolOutput(_)))
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
