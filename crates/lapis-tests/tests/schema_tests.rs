use lapis_core::schema::budget::{AgentBudget, ResearchBudget};
use lapis_core::schema::limit::{CountLimit, DurationLimitMs, Limit};
use lapis_core::schema::model::{ModelInputItem, ModelMessageRole, ModelRequest};
use lapis_core::schema::policy::{
    EvidencePolicy, ExecutionPolicy, ModelPolicy, OutputPolicy, SearchPolicy, ToolName,
};
use lapis_core::schema::report::{
    AspectReport, AspectResearchResult, Confidence, Evidence, Finding, FindingType, Importance,
    SourceType,
};
use lapis_core::schema::research::{
    AspectResearchRequest, AspectResearchTask, AspectSpec, DeepResearchRequest, ResearchContext,
};
use schemars::schema_for;
use serde_json::json;

fn aspect() -> AspectSpec {
    AspectSpec {
        aspect_id: "market".to_owned(),
        name: "Market".to_owned(),
        role: "researcher".to_owned(),
        research_question: "What changed?".to_owned(),
        scope: vec!["market sizing".to_owned()],
        boundaries: vec!["no private data".to_owned()],
        success_criteria: vec!["evidence-backed findings".to_owned()],
        aspect_agent_prompt: aspect_prompt(),
        allowed_tools: vec![ToolName("search".to_owned())],
        model_provider: Some("openai".to_owned()),
        search_provider: Some("exa".to_owned()),
    }
}

fn minimal_request() -> ModelRequest {
    ModelRequest {
        provider: String::new(),
        model: None,
        previous_response_id: None,
        input: vec![ModelInputItem::message(ModelMessageRole::User, "hello")],
        tools: Vec::new(),
        response_format: None,
        temperature: None,
        max_tokens: None,
    }
}

fn aspect_prompt() -> String {
    "# Aspect Agent\n\nDummy aspect agent prompt for tests.\n".to_owned()
}

fn model_policy(allowed_providers: &[&str]) -> ModelPolicy {
    ModelPolicy {
        allowed_providers: allowed_providers
            .iter()
            .map(|provider| (*provider).to_owned())
            .collect(),
        temperature: Some(0.2),
        max_tokens: None,
        require_tool_call_support: true,
    }
}

fn search_policy(allowed_providers: &[&str]) -> SearchPolicy {
    SearchPolicy {
        allowed_providers: allowed_providers
            .iter()
            .map(|provider| (*provider).to_owned())
            .collect(),
        max_results_per_query: 5,
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
        timeout_ms: timeout_ms.map_or(Limit::unlimited(), Limit::limited),
    }
}

#[test]
fn deep_research_request_roundtrips_plan_fields_json() {
    let request = DeepResearchRequest {
        schema_version: "m5".to_owned(),
        request_id: "request-1".to_owned(),
        user_question: "What should Lapis build first?".to_owned(),
        aspect_tasks: vec![AspectResearchTask {
            aspect: AspectSpec {
                aspect_id: "schema".to_owned(),
                name: "Schema".to_owned(),
                role: "contract reviewer".to_owned(),
                research_question: "Are contracts stable?".to_owned(),
                scope: vec!["schema".to_owned()],
                boundaries: vec![],
                success_criteria: vec!["roundtrip".to_owned()],
                aspect_agent_prompt: aspect_prompt(),
                allowed_tools: vec![ToolName("search".to_owned())],
                model_provider: Some("openai".to_owned()),
                search_provider: Some("exa".to_owned()),
            },
            budget: AgentBudget::unlimited(),
        }],
        budget: ResearchBudget::unlimited(),
        model_policy: model_policy(&["openai"]),
        search_policy: search_policy(&["exa"]),
        evidence_policy: evidence_policy(),
        output_policy: output_policy(),
        shared_context: ResearchContext::empty(),
        execution_policy: execution_policy(Some(300_000)),
    };

    let value = serde_json::to_string(&request).expect("serialize request");
    let decoded: DeepResearchRequest = serde_json::from_str(&value).expect("deserialize request");

    assert_eq!(decoded.user_question, request.user_question);
    assert_eq!(decoded.aspect_tasks[0].aspect.role, "contract reviewer");
}

#[test]
fn aspect_research_request_roundtrips_json() {
    let request = AspectResearchRequest {
        schema_version: "m4".to_owned(),
        request_id: "req-1".to_owned(),
        task: AspectResearchTask {
            aspect: aspect(),
            budget: AgentBudget::unlimited(),
        },
        shared_context: ResearchContext {
            summary: "shared context".to_owned(),
            ..ResearchContext::empty()
        },
        model_policy: model_policy(&["openai"]),
        search_policy: search_policy(&["exa"]),
        evidence_policy: evidence_policy(),
        output_policy: output_policy(),
        execution_policy: execution_policy(Some(300_000)),
    };

    let value = serde_json::to_string(&request).expect("serialize request");
    let decoded: AspectResearchRequest = serde_json::from_str(&value).expect("deserialize request");

    assert_eq!(decoded, request);
}

#[test]
fn aspect_and_deep_research_request_schemas_remain_distinct() {
    let aspect_schema =
        serde_json::to_value(schema_for!(AspectResearchRequest)).expect("aspect request schema");
    let deep_schema =
        serde_json::to_value(schema_for!(DeepResearchRequest)).expect("deep request schema");

    assert!(aspect_schema.pointer("/properties/task").is_some());
    assert!(aspect_schema.pointer("/properties/aspect_tasks").is_none());
    assert!(aspect_schema.pointer("/properties/user_question").is_none());

    assert!(deep_schema.pointer("/properties/aspect_tasks").is_some());
    assert!(deep_schema.pointer("/properties/user_question").is_some());
    assert!(deep_schema.pointer("/properties/task").is_none());
}

#[test]
fn aspect_report_schema_omits_embedded_evidence() {
    let report = AspectReport {
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
        assumptions: vec![],
        risks: vec![],
        counterarguments: vec![],
        open_questions: vec![],
        confidence: Confidence::Medium,
        limitations: vec![],
    };

    let value = serde_json::to_value(&report).expect("serialize report");
    assert!(value.get("evidence").is_none());
}

#[test]
fn model_message_role_uses_snake_case() {
    assert_eq!(
        serde_json::to_string(&ModelMessageRole::System).unwrap(),
        "\"system\""
    );
    assert_eq!(
        serde_json::to_string(&ModelMessageRole::User).unwrap(),
        "\"user\""
    );
    assert_eq!(
        serde_json::from_str::<ModelMessageRole>("\"assistant\"").unwrap(),
        ModelMessageRole::Assistant
    );
}

#[test]
fn research_budget_accepts_minus_one_as_unlimited() {
    let budget: ResearchBudget = serde_json::from_value(serde_json::json!({
        "max_agents": -1,
        "max_concurrent_agents": -1,
        "max_total_model_calls": -1,
        "max_total_search_calls": -1,
        "total_timeout_ms": -1,
        "max_tokens": -1
    }))
    .expect("unlimited research budget");

    assert!(budget.max_agents.is_unlimited());
    assert!(budget.max_concurrent_agents.is_unlimited());
    assert!(budget.max_total_model_calls.is_unlimited());
    assert!(budget.max_total_search_calls.is_unlimited());
    assert!(budget.total_timeout_ms.is_unlimited());
    assert!(budget.max_tokens.is_unlimited());
}

#[test]
fn budget_defaults_are_unlimited() {
    let research = ResearchBudget::unlimited();
    assert!(research.max_agents.is_unlimited());
    assert!(research.max_concurrent_agents.is_unlimited());
    assert!(research.max_total_model_calls.is_unlimited());
    assert!(research.max_total_search_calls.is_unlimited());
    assert!(research.total_timeout_ms.is_unlimited());
    assert!(research.max_tokens.is_unlimited());

    let agent = AgentBudget::unlimited();
    assert!(agent.max_turns.is_unlimited());
    assert!(agent.max_tool_calls.is_unlimited());
    assert!(agent.max_search_calls.is_unlimited());
    assert!(agent.timeout_ms.is_unlimited());
}

#[test]
fn validate_accepts_valid_minimal_request() {
    assert!(minimal_request().validate().is_ok());
}

#[test]
fn validate_rejects_invalid_temperature_and_zero_max_tokens() {
    for temperature in [f32::NAN, -0.1, 2.1] {
        let mut request = minimal_request();
        request.temperature = Some(temperature);

        assert!(request.validate().is_err());
    }

    let mut request = minimal_request();
    request.max_tokens = Some(0);

    assert!(request.validate().is_err());
}

#[test]
fn aspect_research_result_schema_excludes_runtime_metadata() {
    let schema = serde_json::to_value(rmcp::schemars::schema_for!(AspectResearchResult))
        .expect("schema json");
    let properties = schema["properties"].as_object().expect("properties");

    assert!(properties.contains_key("aspect_report"));
    assert!(properties.contains_key("evidence"));
    assert!(!properties.contains_key("provider_usage"));
    assert!(!properties.contains_key("budget_usage"));
    assert!(!properties.contains_key("trace_summary"));
    assert!(!properties.contains_key("search_queries"));
    assert!(!properties.contains_key("tool_calls"));
}

#[test]
fn output_policy_schema_omits_trace_controls() {
    let schema =
        serde_json::to_value(rmcp::schemars::schema_for!(OutputPolicy)).expect("schema json");
    let properties = schema["properties"].as_object().expect("properties");

    assert!(properties.contains_key("language"));
    assert!(properties.contains_key("max_findings_per_aspect"));
    assert!(!properties.contains_key("include_trace_summary"));
}

#[test]
fn aspect_research_result_roundtrips_json() {
    let result = AspectResearchResult {
        aspect_report: AspectReport {
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
            assumptions: vec![],
            risks: vec![],
            counterarguments: vec![],
            open_questions: vec![],
            confidence: Confidence::Medium,
            limitations: vec![],
        },
        evidence: vec![Evidence {
            id: "ev-1-1".to_owned(),
            source_title: "Source".to_owned(),
            url: Some("https://example.test/source".to_owned()),
            provider: "searcher".to_owned(),
            query: "query".to_owned(),
            snippet: "snippet".to_owned(),
            summary: "summary".to_owned(),
            published_at: None,
            retrieved_at: "2026-05-25T00:00:00Z".to_owned(),
            supports_findings: vec!["finding-1".to_owned()],
            source_type: SourceType::Official,
            confidence: Confidence::High,
        }],
    };

    let json = serde_json::to_string(&result).expect("serialize result");
    let decoded = serde_json::from_str::<AspectResearchResult>(&json).expect("decode result");

    assert_eq!(decoded, result);
}

#[test]
fn count_limit_schema_matches_wire_format() {
    let schema = schema_for!(CountLimit);
    let schema = serde_json::to_value(&schema).expect("schema json");

    assert_eq!(schema.get("type"), Some(&json!(["integer", "null"])));
    assert_eq!(schema.get("minimum"), Some(&json!(-1)));
}

#[test]
fn duration_limit_schema_matches_wire_format() {
    let schema = schema_for!(DurationLimitMs);
    let schema = serde_json::to_value(&schema).expect("schema json");

    assert_eq!(schema.get("type"), Some(&json!(["integer", "null"])));
    assert_eq!(schema.get("minimum"), Some(&json!(-1)));
}

#[test]
fn limit_deserializes_schema_advertised_values() {
    assert_eq!(
        serde_json::from_value::<CountLimit>(json!(null)).expect("null limit"),
        Limit::Unlimited
    );
    assert_eq!(
        serde_json::from_value::<CountLimit>(json!(-1)).expect("unlimited limit"),
        Limit::Unlimited
    );
    assert_eq!(
        serde_json::from_value::<CountLimit>(json!(3)).expect("finite limit"),
        Limit::Limited(3)
    );
    assert!(serde_json::from_value::<CountLimit>(json!(-2)).is_err());
}

/// The Layer 1 task-decomposition example MUST deserialize cleanly into a
/// `DeepResearchRequest`, including the inline aspect prompt, snake_case
/// `allowed_tools`, structured per-aspect budget, and the `max_tokens`
/// budget dimension.
#[test]
fn layer1_task_decomposition_fixture_deserializes_to_deep_research_request() {
    let fixture = include_str!("../fixtures/prompts/task_decomposition_valid.json");
    let request: DeepResearchRequest =
        serde_json::from_str(fixture).expect("task-decomposition fixture must deserialize");

    let aspect = &request.aspect_tasks[0].aspect;
    assert_eq!(aspect.allowed_tools[0].as_str(), "search");
    assert!(!aspect.aspect_agent_prompt.is_empty());
    assert_eq!(aspect.search_provider.as_deref(), Some("exa"));
    assert!(matches!(
        request.aspect_tasks[0].budget.max_turns,
        Limit::Limited(_)
    ));
    assert!(matches!(request.budget.max_tokens, Limit::Unlimited));
}
