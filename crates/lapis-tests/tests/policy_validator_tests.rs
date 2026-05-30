use lapis_error::Error;
use lapis_model::ModelToolCall;
use lapis_net::policy::RedactionPolicy;
use lapis_workflow::AspectSpec;
use lapis_workflow::validate_output;
use lapis_workflow::{
    AspectReport, AspectResearchResult, Confidence, Evidence, Finding, FindingType, Importance,
    SourceType,
};
use lapis_workflow::{
    EvidencePolicy, OutputPolicy, SearchCategory, SearchContentLevel, SearchDepth, SearchRecency,
    ToolName,
};
use lapis_workflow::{SEARCH_TOOL_NAME, ToolPolicyGuard, search_model_tool};
use serde_json::json;

fn aspect_prompt() -> String {
    "# Aspect Agent\n\nDummy aspect agent prompt for tests.\n".to_owned()
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

fn aspect_with_tools(allowed_tools: Vec<ToolName>) -> AspectSpec {
    AspectSpec {
        aspect_id: "market".to_owned(),
        name: "Market".to_owned(),
        role: "researcher".to_owned(),
        research_question: "What changed?".to_owned(),
        scope: vec![],
        boundaries: vec![],
        success_criteria: vec![],
        aspect_agent_prompt: aspect_prompt(),
        allowed_tools,
        model_provider: None,
        search_provider: Some("exa".to_owned()),
    }
}

fn call(name: &str, arguments: serde_json::Value) -> ModelToolCall {
    ModelToolCall {
        id: "call-1".to_owned(),
        name: name.to_owned(),
        arguments,
    }
}

fn validator_aspect() -> AspectSpec {
    AspectSpec {
        aspect_id: "aspect-1".to_owned(),
        name: "Market".to_owned(),
        role: "researcher".to_owned(),
        research_question: "What matters?".to_owned(),
        scope: vec!["market".to_owned()],
        boundaries: Vec::new(),
        success_criteria: Vec::new(),
        aspect_agent_prompt: aspect_prompt(),
        allowed_tools: vec![ToolName("search".to_owned())],
        model_provider: None::<String>,
        search_provider: Some("exa".to_owned()),
    }
}

fn report() -> AspectReport {
    AspectReport {
        aspect_id: "aspect-1".to_owned(),
        aspect_name: "Market".to_owned(),
        question: "What matters?".to_owned(),
        scope: vec!["market".to_owned()],
        findings: vec![Finding {
            id: "finding-1".to_owned(),
            claim: "A supported claim".to_owned(),
            finding_type: FindingType::Fact,
            importance: Importance::High,
            confidence: Confidence::High,
            evidence_refs: vec!["evidence-1".to_owned()],
            contradicted_by: Vec::new(),
        }],
        assumptions: Vec::new(),
        risks: Vec::new(),
        counterarguments: Vec::new(),
        open_questions: Vec::new(),
        confidence: Confidence::High,
        limitations: Vec::new(),
    }
}

fn evidence() -> Vec<Evidence> {
    vec![Evidence {
        id: "evidence-1".to_owned(),
        source_title: "Source".to_owned(),
        url: None,
        provider: "fake".to_owned(),
        query: "query".to_owned(),
        snippet: "snippet".to_owned(),
        summary: String::new(),
        published_at: None,
        retrieved_at: "2026-05-22T00:00:00Z".to_owned(),
        supports_findings: vec!["finding-1".to_owned()],
        source_type: SourceType::Documentation,
        confidence: Confidence::High,
    }]
}

fn result(report: AspectReport, evidence: Vec<Evidence>) -> AspectResearchResult {
    AspectResearchResult {
        aspect_report: report,
        evidence,
    }
}

fn validate(
    report: &AspectReport,
) -> lapis_error::Result<(AspectResearchResult, lapis_workflow::ValidationStatus)> {
    validate_result(&result(report.clone(), evidence()))
}

fn validate_result(
    result: &AspectResearchResult,
) -> lapis_error::Result<(AspectResearchResult, lapis_workflow::ValidationStatus)> {
    validate_output(
        &serde_json::to_string(result).expect("serialize result"),
        &validator_aspect(),
        &evidence(),
        &evidence_policy(),
        &output_policy(),
    )
}

#[test]
fn accepts_valid_search_call() {
    let guard = ToolPolicyGuard::new(&aspect_with_tools(vec![ToolName(
        SEARCH_TOOL_NAME.to_owned(),
    )]));

    let args = guard
        .validate_search_call(&call(
            SEARCH_TOOL_NAME,
            json!({ "query": "rust async runtime", "max_results": 3 }),
        ))
        .expect("valid search call");

    assert_eq!(args.query, "rust async runtime");
    assert_eq!(args.max_results, Some(3));
}

#[test]
fn accepts_valid_search_call_with_provider_neutral_params() {
    let guard = ToolPolicyGuard::new(&aspect_with_tools(vec![ToolName(
        SEARCH_TOOL_NAME.to_owned(),
    )]));

    let args = guard
        .validate_search_call(&call(
            SEARCH_TOOL_NAME,
            json!({
                "query": "rust async runtime",
                "max_results": 3,
                "depth": "balanced",
                "content_level": "standard",
                "recency": "fresh",
                "category": "news"
            }),
        ))
        .expect("valid search call");

    assert_eq!(args.depth, Some(SearchDepth::Balanced));
    assert_eq!(args.content_level, Some(SearchContentLevel::Standard));
    assert_eq!(args.recency, Some(SearchRecency::Fresh));
    assert_eq!(args.category, Some(SearchCategory::News));
}

#[test]
fn rejects_unknown_tool_and_disallowed_search() {
    let guard = ToolPolicyGuard::new(&aspect_with_tools(vec![ToolName(
        SEARCH_TOOL_NAME.to_owned(),
    )]));
    assert!(matches!(
        guard.validate_search_call(&call("exa_search", json!({ "query": "test" }))),
        Err(Error::ToolPolicyDenied { .. })
    ));

    let disallowed_guard = ToolPolicyGuard::new(&aspect_with_tools(vec![]));
    assert!(matches!(
        disallowed_guard.validate_search_call(&call(SEARCH_TOOL_NAME, json!({ "query": "test" }))),
        Err(Error::ToolPolicyDenied { .. })
    ));
}

#[test]
fn rejects_empty_query_and_malformed_arguments() {
    let guard = ToolPolicyGuard::new(&aspect_with_tools(vec![ToolName(
        SEARCH_TOOL_NAME.to_owned(),
    )]));

    assert!(matches!(
        guard.validate_search_call(&call(SEARCH_TOOL_NAME, json!({ "query": "   " }))),
        Err(Error::ToolPolicyDenied { .. })
    ));

    assert!(matches!(
        guard.validate_search_call(&call(SEARCH_TOOL_NAME, json!({ "max_results": 3 }))),
        Err(Error::ToolPolicyDenied { .. })
    ));

    assert!(matches!(
        guard.validate_search_call(&call(
            SEARCH_TOOL_NAME,
            json!({ "query": "test", "max_results": 0 })
        )),
        Err(Error::ToolPolicyDenied { .. })
    ));
}

#[test]
fn rejects_provider_field_in_search_tool_args() {
    let guard = ToolPolicyGuard::new(&aspect_with_tools(vec![ToolName(
        SEARCH_TOOL_NAME.to_owned(),
    )]));

    assert!(matches!(
        guard.validate_search_call(&call(
            SEARCH_TOOL_NAME,
            json!({ "query": "rust async runtime", "max_results": 3, "provider": "exa" }),
        )),
        Err(Error::ToolPolicyDenied { .. })
    ));
}

#[test]
fn rejects_provider_specific_fields_in_search_tool_args() {
    let guard = ToolPolicyGuard::new(&aspect_with_tools(vec![ToolName(
        SEARCH_TOOL_NAME.to_owned(),
    )]));

    for arguments in [
        json!({ "query": "test", "type": "auto" }),
        json!({ "query": "test", "contents": { "highlights": true } }),
        json!({ "query": "test", "maxAgeHours": 24 }),
        json!({ "query": "test", "provider_overrides": {} }),
    ] {
        assert!(matches!(
            guard.validate_search_call(&call(SEARCH_TOOL_NAME, arguments)),
            Err(Error::ToolPolicyDenied { .. })
        ));
    }
}

#[test]
fn search_model_tool_uses_provider_neutral_schema() {
    let tool = search_model_tool();

    assert_eq!(tool.name, SEARCH_TOOL_NAME);
    assert!(tool.input_schema.get("title").is_some());
    let schema = tool.input_schema.to_string();
    assert!(schema.contains("query"));
    assert!(schema.contains("depth"));
    assert!(schema.contains("content_level"));
    assert!(schema.contains("recency"));
    assert!(schema.contains("category"));
    assert!(schema.contains("low_latency"));
    assert!(schema.contains("high_recall"));
    assert!(!schema.contains("maxAgeHours"));
    assert!(!schema.contains("deep-lite"));
    assert!(!schema.contains("deep\""));
    assert!(!schema.contains("deep-reasoning"));
}

#[test]
fn accepts_valid_report() {
    let (validated, status) = validate(&report()).expect("valid report");

    assert_eq!(validated.aspect_report.aspect_id, "aspect-1");
    assert!(status.ok);
    assert!(status.issues.is_empty());
}

#[test]
fn rejects_malformed_json() {
    let err = validate_output(
        "{not json",
        &validator_aspect(),
        &evidence(),
        &evidence_policy(),
        &output_policy(),
    )
    .expect_err("malformed JSON must fail");

    assert!(matches!(err, Error::SchemaValidationFailed { .. }));
}

#[test]
fn rejects_wrong_aspect_id() {
    let mut report = report();
    report.aspect_id = "other".to_owned();

    let err = validate(&report).expect_err("wrong aspect id must fail");

    assert!(matches!(err, Error::SchemaValidationFailed { .. }));
}

#[test]
fn rejects_missing_evidence_refs() {
    let mut report = report();
    report.findings[0].evidence_refs.clear();

    let err = validate(&report).expect_err("missing evidence refs must fail");

    assert!(matches!(err, Error::SchemaValidationFailed { .. }));
}

#[test]
fn rejects_unknown_evidence_ref() {
    let mut report = report();
    report.findings[0].evidence_refs = vec!["missing".to_owned()];

    let err = validate(&report).expect_err("unknown evidence ref must fail");

    assert!(matches!(err, Error::SchemaValidationFailed { .. }));
}

/// When the selected evidence diverges from the search-tool candidate,
/// the error message MUST name every mismatched field so operators can
/// see the full diff without re-running. The validator surfaces issues
/// in declaration order, with the first issue feeding the
/// `SchemaValidationFailed.message`.
#[test]
fn rejects_mutated_evidence_provenance_with_field_names() {
    let mut selected = evidence();
    // Rewrite both summary and snippet — the model paraphrasing case
    // we observed in real Layer 2 runs.
    selected[0].summary = "model paraphrased the original markdown".to_owned();
    selected[0].snippet = "shortened snippet".to_owned();

    let err =
        validate_result(&result(report(), selected)).expect_err("mutated provenance must fail");

    let Error::SchemaValidationFailed { message } = err else {
        panic!("expected SchemaValidationFailed, got {err:?}");
    };
    assert!(
        message.contains("mutated_evidence_provenance"),
        "message must carry the issue code, got `{message}`"
    );
    assert!(
        message.contains("snippet") && message.contains("summary"),
        "message must name every mismatched field, got `{message}`"
    );
    assert!(
        message.contains("evidence[0]"),
        "message must include the path to the offending evidence, got `{message}`"
    );
}

#[test]
fn rejects_too_many_findings() {
    let mut report = report();
    report.findings.push(report.findings[0].clone());
    let mut output_policy = output_policy();
    output_policy.max_findings_per_aspect = Some(1);

    let err = validate_output(
        &serde_json::to_string(&result(report, evidence())).expect("serialize result"),
        &validator_aspect(),
        &evidence(),
        &evidence_policy(),
        &output_policy,
    )
    .expect_err("too many findings must fail");

    assert!(matches!(err, Error::SchemaValidationFailed { .. }));
}

#[test]
fn redacts_sensitive_headers() {
    let policy = RedactionPolicy;
    let authorization = policy.redact_header("Authorization", "Bearer secret");
    let api_key = policy.redact_header("x-api-key", "secret");

    assert_ne!(authorization, "Bearer secret");
    assert_ne!(api_key, "secret");
    assert_eq!(
        policy.redact_header("content-type", "application/json"),
        "application/json"
    );
}
