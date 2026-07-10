use async_trait::async_trait;
use moe_research_error::{Error, Result};
use moe_research_model::{ModelProvider, ModelRequest, ModelResponse, ModelService};
use moe_research_search::SearchService;
use moe_research_workflow::aspect_research;
use moe_research_workflow::{
    AgentLimits, AspectReport, AspectResearchRequest, AspectResearchResult, Confidence,
    EvidencePolicy, ExecutionPolicy, Finding, FindingType, Importance, Limit, ModelPolicy,
    OutputPolicy, ResearchContext, ResearchPolicy, SearchPolicy, SourceType,
};

struct StaticModelProvider {
    content: String,
}

#[async_trait]
impl ModelProvider for StaticModelProvider {
    fn name(&self) -> &'static str {
        "model"
    }

    async fn complete(&self, _request: ModelRequest) -> Result<ModelResponse> {
        Ok(ModelResponse {
            provider: "model".to_owned(),
            model: None,
            response_id: None,
            content: Some(self.content.clone()),
            tool_calls: Vec::new(),
            output_items: Vec::new(),
            usage: None,
        })
    }
}

fn aspect_request(output_policy: OutputPolicy) -> AspectResearchRequest {
    AspectResearchRequest {
        schema_version: "0.2".to_owned(),
        request_id: "request-1".to_owned(),
        task: moe_research_workflow::AspectRequest {
            id: "aspect-1".to_owned(),
            name: "Market".to_owned(),
            role: "researcher".to_owned(),
            question: "What matters?".to_owned(),
            scope: vec!["market".to_owned()],
            boundaries: Vec::new(),
            success_criteria: Vec::new(),
            instructions: "# Aspect Agent\n\nReturn AspectResearchResult JSON.".to_owned(),
            tools: Vec::new(),
            model_provider: "model".to_owned(),
            search_provider: None,
            limits: AgentLimits {
                max_turns: Limit::limited(1),
                max_tool_calls: Limit::limited(0),
                max_search_calls: Limit::limited(0),
                timeout_ms: Limit::limited(60_000),
            },
        },
        policy: ResearchPolicy {
            model: ModelPolicy {
                allowed_providers: vec!["model".to_owned()],
                temperature: Some(0.2),
                max_tokens: None,
                require_tool_call_support: false,
            },
            search: SearchPolicy {
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
            },
            evidence: EvidencePolicy {
                require_evidence_for_findings: false,
                min_evidence_per_finding: 1,
            },
            output: output_policy,
            execution: ExecutionPolicy {
                allow_partial_results: true,
                fail_fast: false,
            },
        },
        context: ResearchContext::empty(),
    }
}

fn output_policy() -> OutputPolicy {
    OutputPolicy {
        language: "zh-CN".to_owned(),
        max_findings_per_aspect: None,
    }
}

fn model_service(content: String) -> ModelService {
    let mut service = ModelService::new();
    service.register(StaticModelProvider { content });
    service
}

fn report() -> AspectReport {
    AspectReport {
        aspect_id: "aspect-1".to_owned(),
        aspect_name: "Market".to_owned(),
        question: "What matters?".to_owned(),
        scope: vec!["market".to_owned()],
        findings: Vec::new(),
        assumptions: Vec::new(),
        risks: Vec::new(),
        counterarguments: Vec::new(),
        open_questions: Vec::new(),
        confidence: Confidence::High,
        limitations: Vec::new(),
    }
}

fn finding(id: &str) -> Finding {
    Finding {
        id: id.to_owned(),
        claim: "A claim".to_owned(),
        finding_type: FindingType::Fact,
        importance: Importance::High,
        confidence: Confidence::High,
        evidence_refs: Vec::new(),
        contradicted_by: Vec::new(),
    }
}

fn result(report: AspectReport) -> AspectResearchResult {
    AspectResearchResult {
        aspect_report: report,
        evidence: Vec::new(),
    }
}

async fn run_result(
    result: AspectResearchResult,
    output_policy: OutputPolicy,
) -> std::result::Result<
    moe_research_workflow::AspectResearchOutput,
    Box<moe_research_workflow::AspectResearchFailure>,
> {
    let content = serde_json::to_string(&result).expect("serialize result");
    let model_service = model_service(content);
    let search_service = SearchService::new();

    aspect_research(
        aspect_request(output_policy),
        &model_service,
        &search_service,
        &moe_research_workflow::BudgetConfig {
            research: moe_research_workflow::ResearchLimits::unlimited(),
            per_agent: AgentLimits::unlimited(),
        },
    )
    .await
}

#[tokio::test]
async fn public_workflow_accepts_valid_report() {
    let output = run_result(result(report()), output_policy())
        .await
        .expect("valid report");

    assert_eq!(output.result.aspect_report.aspect_id, "aspect-1");
    assert!(output.result.evidence.is_empty());
}

#[tokio::test]
async fn public_workflow_rejects_malformed_json() {
    let model_service = model_service("{not json".to_owned());
    let search_service = SearchService::new();

    let err = aspect_research(
        aspect_request(output_policy()),
        &model_service,
        &search_service,
        &moe_research_workflow::BudgetConfig {
            research: moe_research_workflow::ResearchLimits::unlimited(),
            per_agent: AgentLimits::unlimited(),
        },
    )
    .await
    .expect_err("malformed JSON must fail");

    assert!(matches!(err.error, Error::SchemaValidationFailed { .. }));
    assert!(err.partial_output.is_none());
}

#[tokio::test]
async fn public_workflow_rejects_wrong_aspect_id() {
    let mut report = report();
    report.aspect_id = "other".to_owned();

    let err = run_result(result(report), output_policy())
        .await
        .expect_err("wrong aspect id must fail");

    assert!(matches!(err.error, Error::SchemaValidationFailed { .. }));
}

#[tokio::test]
async fn public_workflow_rejects_too_many_findings() {
    let mut report = report();
    report.findings = vec![finding("finding-1"), finding("finding-2")];
    let mut output_policy = output_policy();
    output_policy.max_findings_per_aspect = Some(1);

    let err = run_result(result(report), output_policy)
        .await
        .expect_err("too many findings must fail");

    assert!(matches!(err.error, Error::SchemaValidationFailed { .. }));
}

#[tokio::test]
async fn public_workflow_rejects_selected_evidence_not_seen_in_search_output() {
    let mut report = report();
    report.findings = vec![Finding {
        id: "finding-1".to_owned(),
        claim: "A supported claim".to_owned(),
        finding_type: FindingType::Fact,
        importance: Importance::High,
        confidence: Confidence::High,
        evidence_refs: vec!["evidence-1".to_owned()],
        contradicted_by: Vec::new(),
    }];
    let mut request = aspect_request(output_policy());
    request.policy.evidence.require_evidence_for_findings = true;
    let result = AspectResearchResult {
        aspect_report: report,
        evidence: vec![moe_research_workflow::Evidence {
            id: "evidence-1".to_owned(),
            source_title: "Source".to_owned(),
            url: None,
            provider: "manual".to_owned(),
            query: "query".to_owned(),
            snippet: "snippet".to_owned(),
            summary: String::new(),
            published_at: None,
            retrieved_at: "2026-05-22T00:00:00Z".to_owned(),
            supports_findings: vec!["finding-1".to_owned()],
            source_type: SourceType::Official,
            confidence: Confidence::High,
        }],
    };
    let content = serde_json::to_string(&result).expect("serialize result");
    let model_service = model_service(content);
    let search_service = SearchService::new();

    let err = aspect_research(
        request,
        &model_service,
        &search_service,
        &moe_research_workflow::BudgetConfig {
            research: moe_research_workflow::ResearchLimits::unlimited(),
            per_agent: AgentLimits::unlimited(),
        },
    )
    .await
    .expect_err("manual evidence not returned by search must fail");

    assert!(matches!(err.error, Error::SchemaValidationFailed { .. }));
}
