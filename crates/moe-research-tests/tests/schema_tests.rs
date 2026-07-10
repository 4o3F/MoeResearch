use moe_research_model::{ModelInputItem, ModelMessageRole, ModelRequest};
use moe_research_workflow::{AgentLimits, ResearchLimits};
use moe_research_workflow::{
    AspectReport, AspectResearchResult, Confidence, Evidence, Finding, FindingType, Importance,
    SourceType,
};
use moe_research_workflow::{
    AspectRequest, AspectResearchRequest, DeepResearchRequest, ResearchContext, ResearchPolicy,
    ResearchTask,
};
use moe_research_workflow::{CountLimit, DurationLimitMs, Limit};
use moe_research_workflow::{
    EvidencePolicy, ExecutionPolicy, ModelPolicy, OutputPolicy, SearchContentLevel, SearchDepth,
    SearchPolicy, SearchRecency, ToolName,
};
use schemars::schema_for;
use serde_json::{Value, json};

fn aspect() -> AspectRequest {
    AspectRequest {
        id: "market".to_owned(),
        name: "Market".to_owned(),
        role: "researcher".to_owned(),
        question: "What changed?".to_owned(),
        scope: vec!["market sizing".to_owned()],
        boundaries: vec!["no private data".to_owned()],
        success_criteria: vec!["evidence-backed findings".to_owned()],
        instructions: aspect_prompt(),
        tools: vec![ToolName("search".to_owned())],
        model_provider: "openai".to_owned(),
        search_provider: Some("exa".to_owned()),
        limits: AgentLimits::unlimited(),
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

fn research_policy(
    allowed_model_providers: &[&str],
    allowed_search_providers: &[&str],
) -> ResearchPolicy {
    ResearchPolicy {
        model: model_policy(allowed_model_providers),
        search: search_policy(allowed_search_providers),
        evidence: evidence_policy(),
        output: output_policy(),
        execution: execution_policy(),
    }
}

#[test]
fn deep_research_request_roundtrips_plan_fields_json() {
    let request = DeepResearchRequest {
        schema_version: "0.2".to_owned(),
        request_id: "request-1".to_owned(),
        task: ResearchTask {
            question: "What should MoeResearch build first?".to_owned(),
            aspects: vec![AspectRequest {
                id: "schema".to_owned(),
                name: "Schema".to_owned(),
                role: "contract reviewer".to_owned(),
                question: "Are contracts stable?".to_owned(),
                scope: vec!["schema".to_owned()],
                boundaries: vec![],
                success_criteria: vec!["roundtrip".to_owned()],
                instructions: aspect_prompt(),
                tools: vec![ToolName("search".to_owned())],
                model_provider: "openai".to_owned(),
                search_provider: Some("exa".to_owned()),
                limits: AgentLimits::unlimited(),
            }],
        },
        limits: ResearchLimits::unlimited(),
        policy: research_policy(&["openai"], &["exa"]),
        context: ResearchContext::empty(),
    };

    let value = serde_json::to_string(&request).expect("serialize request");
    let decoded: DeepResearchRequest = serde_json::from_str(&value).expect("deserialize request");

    assert_eq!(decoded.task.question, request.task.question);
    assert_eq!(decoded.task.aspects[0].role, "contract reviewer");
}

#[test]
fn aspect_research_request_roundtrips_json() {
    let request = AspectResearchRequest {
        schema_version: "0.2".to_owned(),
        request_id: "req-1".to_owned(),
        task: aspect(),
        context: ResearchContext {
            summary: "shared context".to_owned(),
            ..ResearchContext::empty()
        },
        policy: research_policy(&["openai"], &["exa"]),
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
    assert!(aspect_schema.pointer("/properties/limits").is_none());
    assert!(aspect_schema.pointer("/properties/user_question").is_none());
    assert!(aspect_schema.pointer("/properties/aspect_tasks").is_none());

    assert!(deep_schema.pointer("/properties/task").is_some());
    assert!(deep_schema.pointer("/properties/limits").is_some());
    assert!(deep_schema.pointer("/properties/user_question").is_none());
    assert!(deep_schema.pointer("/properties/aspect_tasks").is_none());
}

#[test]
fn deep_research_request_rejects_removed_execution_timeout_policy_field() {
    let mut value = serde_json::to_value(DeepResearchRequest {
        schema_version: "0.2".to_owned(),
        request_id: "request-1".to_owned(),
        task: ResearchTask {
            question: "What should MoeResearch build first?".to_owned(),
            aspects: vec![aspect()],
        },
        limits: ResearchLimits::unlimited(),
        policy: research_policy(&["openai"], &["exa"]),
        context: ResearchContext::empty(),
    })
    .expect("request json");

    value
        .pointer_mut("/policy/execution")
        .expect("execution policy")
        .as_object_mut()
        .expect("execution policy object")
        .insert("timeout_ms".to_owned(), json!(600_000));

    let error = serde_json::from_value::<DeepResearchRequest>(value)
        .expect_err("removed execution timeout must be rejected");

    assert!(error.to_string().contains("timeout_ms"));
}

#[test]
fn deep_research_request_rejects_unknown_nested_policy_fields() {
    let mut value = serde_json::to_value(DeepResearchRequest {
        schema_version: "0.2".to_owned(),
        request_id: "request-1".to_owned(),
        task: ResearchTask {
            question: "What should MoeResearch build first?".to_owned(),
            aspects: vec![aspect()],
        },
        limits: ResearchLimits::unlimited(),
        policy: research_policy(&["openai"], &["exa"]),
        context: ResearchContext::empty(),
    })
    .expect("request json");

    value
        .pointer_mut("/policy/search")
        .expect("search policy")
        .as_object_mut()
        .expect("search policy object")
        .insert("maxAgeHours".to_owned(), json!(24));

    let error = serde_json::from_value::<DeepResearchRequest>(value)
        .expect_err("provider-native search fields must be rejected");

    assert!(error.to_string().contains("maxAgeHours"));
}

#[test]
fn deep_research_request_rejects_old_flattened_policy_blocks() {
    let mut value = serde_json::to_value(DeepResearchRequest {
        schema_version: "0.2".to_owned(),
        request_id: "request-1".to_owned(),
        task: ResearchTask {
            question: "What should MoeResearch build first?".to_owned(),
            aspects: vec![aspect()],
        },
        limits: ResearchLimits::unlimited(),
        policy: research_policy(&["openai"], &["exa"]),
        context: ResearchContext::empty(),
    })
    .expect("request json");

    value
        .as_object_mut()
        .expect("request object")
        .insert("model_policy".to_owned(), json!({}));

    let error = serde_json::from_value::<DeepResearchRequest>(value)
        .expect_err("old flattened policy blocks must be rejected");

    assert!(error.to_string().contains("model_policy"));
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
fn research_limits_accepts_minus_one_as_unlimited() {
    let limits: ResearchLimits = serde_json::from_value(serde_json::json!({
        "max_agents": -1,
        "max_concurrent_agents": -1,
        "max_total_model_calls": -1,
        "max_total_search_calls": -1,
        "total_timeout_ms": -1,
        "max_tokens": -1
    }))
    .expect("unlimited research limits");

    assert!(limits.max_agents.is_unlimited());
    assert!(limits.max_concurrent_agents.is_unlimited());
    assert!(limits.max_total_model_calls.is_unlimited());
    assert!(limits.max_total_search_calls.is_unlimited());
    assert!(limits.total_timeout_ms.is_unlimited());
    assert!(limits.max_tokens.is_unlimited());
}

#[test]
fn limits_defaults_are_unlimited() {
    let research = ResearchLimits::unlimited();
    assert!(research.max_agents.is_unlimited());
    assert!(research.max_concurrent_agents.is_unlimited());
    assert!(research.max_total_model_calls.is_unlimited());
    assert!(research.max_total_search_calls.is_unlimited());
    assert!(research.total_timeout_ms.is_unlimited());
    assert!(research.max_tokens.is_unlimited());

    let agent = AgentLimits::unlimited();
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
    assert!(schema.get("format").is_none());
}

#[test]
fn duration_limit_schema_matches_wire_format() {
    let schema = schema_for!(DurationLimitMs);
    let schema = serde_json::to_value(&schema).expect("schema json");

    assert_eq!(schema.get("type"), Some(&json!(["integer", "null"])));
    assert_eq!(schema.get("minimum"), Some(&json!(-1)));
    assert!(schema.get("format").is_none());
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

#[test]
fn search_policy_schema_contains_provider_neutral_search_params_only() {
    let schema =
        serde_json::to_value(schema_for!(SearchPolicy)).expect("search policy schema json");
    let schema_text = schema.to_string();

    for expected in [
        "depth",
        "content_level",
        "recency",
        "category",
        "low_latency",
        "balanced",
        "high_recall",
        "compact",
        "standard",
        "detailed",
        "default",
        "live",
        "fresh",
        "recent",
        "cached",
        "organizations",
        "people",
        "academic",
        "news",
        "personal_sites",
        "financial_filings",
        "code",
    ] {
        assert!(schema_text.contains(expected), "missing {expected}");
    }

    for forbidden in ["maxAgeHours", "highlights", "deep-lite", "deep-reasoning"] {
        assert!(!schema_text.contains(forbidden), "leaked {forbidden}");
    }
}

/// The Layer 1 task-decomposition example MUST deserialize cleanly into a
/// `DeepResearchRequest`, including inline aspect instructions, snake_case
/// `tools`, structured per-aspect limits, and the `max_tokens` limit dimension.
#[test]
fn layer1_task_decomposition_fixture_deserializes_to_deep_research_request() {
    let fixture = include_str!("../fixtures/prompts/task_decomposition_valid.json");
    let request: DeepResearchRequest =
        serde_json::from_str(fixture).expect("task-decomposition fixture must deserialize");

    assert_eq!(request.policy.search.depth, Some(SearchDepth::Balanced));
    assert_eq!(
        request.policy.search.content_level,
        Some(SearchContentLevel::Standard)
    );
    assert_eq!(request.policy.search.recency, Some(SearchRecency::Default));

    let aspect = &request.task.aspects[0];
    assert_eq!(aspect.tools[0].as_str(), "search");
    assert!(!aspect.instructions.is_empty());
    assert_eq!(aspect.search_provider.as_deref(), Some("exa"));
    assert!(matches!(aspect.limits.max_turns, Limit::Limited(_)));
    assert!(matches!(request.limits.max_tokens, Limit::Unlimited));
    assert!(request.task.aspects.iter().all(|aspect| {
        !aspect
            .limits
            .timeout_ms
            .exceeds(request.limits.total_timeout_ms)
    }));
}

#[test]
fn direct_mcp_payload_fixtures_deserialize_without_wrappers() {
    let aspect_fixture = include_str!("../fixtures/mcp/aspect_research_direct_payload.json");
    let aspect_value: Value =
        serde_json::from_str(aspect_fixture).expect("aspect fixture is valid JSON");
    assert_direct_tool_payload(&aspect_value);
    let aspect: AspectResearchRequest =
        serde_json::from_str(aspect_fixture).expect("aspect direct payload must deserialize");

    assert_eq!(aspect.schema_version, "0.2");
    assert!(aspect.task.instructions.starts_with('#'));
    assert!(aspect.policy.execution.allow_partial_results);

    let deep_fixture = include_str!("../fixtures/mcp/deep_research_direct_payload.json");
    let deep_value: Value = serde_json::from_str(deep_fixture).expect("deep fixture is valid JSON");
    assert_direct_tool_payload(&deep_value);
    let deep: DeepResearchRequest =
        serde_json::from_str(deep_fixture).expect("deep direct payload must deserialize");

    assert_eq!(deep.schema_version, "0.2");
    assert!(!deep.task.aspects.is_empty());
    assert!(deep.task.aspects.iter().all(|aspect| {
        aspect.instructions.starts_with('#')
            && aspect.search_provider.is_some()
            && !aspect
                .limits
                .timeout_ms
                .exceeds(deep.limits.total_timeout_ms)
    }));
}

#[test]
fn layer1_prompt_search_policy_skeletons_include_complete_fields() {
    let prompts = [
        include_str!("../../../prompts/layer1/task-decomposition.md"),
        include_str!("../../../prompts/layer1/pm-deep-research/task-decomposition.md"),
        include_str!(
            "../../../prompts/layer1/pm-deep-research/task-decomposition-product-capability.md"
        ),
        include_str!(
            "../../../prompts/layer1/pm-deep-research/task-decomposition-innovation-direction.md"
        ),
        include_str!(
            "../../../prompts/layer1/pm-deep-research/task-decomposition-product-requirements.md"
        ),
    ];

    for prompt in prompts {
        for field in [
            "allowed_providers",
            "max_results_per_query",
            "freshness",
            "depth",
            "content_level",
            "recency",
            "category",
            "language",
            "region",
            "include_domains",
            "exclude_domains",
        ] {
            let marker = format!("\"{field}\"");
            assert!(prompt.contains(&marker), "prompt missing {marker}");
        }
        assert!(prompt.contains("AspectResearchRequest"));
        assert!(prompt.contains("top-level `task`"));
    }
}

#[test]
fn research_profile_task_decomposition_prompts_keep_schema_boundary() {
    let prompts = [
        (
            "academic",
            include_str!("../../../prompts/layer1/academic-deep-research/task-decomposition.md"),
        ),
        (
            "technical",
            include_str!("../../../prompts/layer1/technical-evaluation/task-decomposition.md"),
        ),
    ];

    for (name, prompt) in prompts {
        for marker in [
            "DeepResearchRequest",
            "instructions",
            "policy",
            "limits",
            "timeout_ms",
        ] {
            assert!(prompt.contains(marker), "{name} prompt missing {marker}");
        }

        for field in [
            "allowed_providers",
            "max_results_per_query",
            "include_domains",
            "exclude_domains",
        ] {
            let marker = format!("\"{field}\"");
            assert!(prompt.contains(&marker), "{name} prompt missing {marker}");
        }

        let lower = prompt.to_ascii_lowercase();
        assert!(
            lower.contains("rust core") && lower.contains("never reads prompt files"),
            "{name} prompt must keep prompt assets outside Rust runtime IO"
        );
        assert!(
            lower.contains("provider-native"),
            "{name} prompt must forbid provider-native request fields"
        );
    }
}

#[test]
fn layer1_task_decomposition_prompts_do_not_emit_removed_request_fields() {
    let prompts = [
        (
            "generic",
            include_str!("../../../prompts/layer1/task-decomposition.md"),
        ),
        (
            "pm-competitive",
            include_str!("../../../prompts/layer1/pm-deep-research/task-decomposition.md"),
        ),
        (
            "pm-product-capability",
            include_str!(
                "../../../prompts/layer1/pm-deep-research/task-decomposition-product-capability.md"
            ),
        ),
        (
            "pm-innovation-direction",
            include_str!(
                "../../../prompts/layer1/pm-deep-research/task-decomposition-innovation-direction.md"
            ),
        ),
        (
            "pm-product-requirements",
            include_str!(
                "../../../prompts/layer1/pm-deep-research/task-decomposition-product-requirements.md"
            ),
        ),
        (
            "academic",
            include_str!("../../../prompts/layer1/academic-deep-research/task-decomposition.md"),
        ),
        (
            "technical",
            include_str!("../../../prompts/layer1/technical-evaluation/task-decomposition.md"),
        ),
    ];

    for (name, prompt) in prompts {
        for marker in [
            "\"user_question\"",
            "\"aspect_tasks\"",
            "\"aspect_agent_prompt\"",
            "\"allowed_tools\"",
            "\"shared_context\"",
            "\"model_policy\"",
            "\"search_policy\"",
            "\"evidence_policy\"",
            "\"output_policy\"",
            "\"execution_policy\"",
            "\"budget\"",
            "budget_preset",
            "execution_policy.timeout_ms",
            "schema_version=\"0.1\"",
            "schema_version\": \"0.1\"",
        ] {
            assert!(!prompt.contains(marker), "{name} prompt leaked {marker}");
        }
    }
}

#[test]
fn layer1_agent_allocation_prompts_do_not_reference_removed_prompt_fields() {
    let prompts = [
        (
            "academic",
            include_str!("../../../prompts/layer1/academic-deep-research/agent-allocation.md"),
        ),
        (
            "technical",
            include_str!("../../../prompts/layer1/technical-evaluation/agent-allocation.md"),
        ),
        (
            "pm-competitive",
            include_str!("../../../prompts/layer1/pm-deep-research/agent-allocation.md"),
        ),
        (
            "pm-product-capability",
            include_str!(
                "../../../prompts/layer1/pm-deep-research/agent-allocation-product-capability.md"
            ),
        ),
        (
            "pm-innovation-direction",
            include_str!(
                "../../../prompts/layer1/pm-deep-research/agent-allocation-innovation-direction.md"
            ),
        ),
        (
            "pm-product-requirements",
            include_str!(
                "../../../prompts/layer1/pm-deep-research/agent-allocation-product-requirements.md"
            ),
        ),
    ];

    for (name, prompt) in prompts {
        for marker in [
            "aspect_agent_prompt",
            "AspectSpec",
            "AspectResearchTask",
            "shared_context",
            "budget {",
        ] {
            assert!(!prompt.contains(marker), "{name} prompt leaked {marker}");
        }
    }
}

fn assert_direct_tool_payload(value: &Value) {
    for wrapper_key in [
        "jsonrpc",
        "method",
        "params",
        "arguments",
        "request",
        "input",
        "tool_input",
    ] {
        assert!(value.get(wrapper_key).is_none(), "unexpected {wrapper_key}");
    }
}
