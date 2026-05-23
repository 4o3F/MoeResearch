use lapis_core::schema::budget::{AgentBudget, ResearchBudget};
use lapis_core::schema::model::{ModelMessage, ModelMessageRole, ModelRequest};
use lapis_core::schema::policy::{
    EvidencePolicy, EvidenceRequirement, ExecutionPolicy, ModelPolicy, OutputPolicy, SearchPolicy,
    ToolName,
};
use lapis_core::schema::research::{
    AspectResearchRequest, AspectSpec, DeliverableSpec, PromptAssets, ResearchConstraint,
    ResearchContext, ResearchPlan,
};

fn aspect() -> AspectSpec {
    AspectSpec {
        aspect_id: "market".to_owned(),
        name: "Market".to_owned(),
        role: "researcher".to_owned(),
        research_question: "What changed?".to_owned(),
        scope: vec!["market sizing".to_owned()],
        boundaries: vec!["no private data".to_owned()],
        success_criteria: vec!["evidence-backed findings".to_owned()],
        prompt_assets: prompt_assets(),
        required_evidence: EvidenceRequirement::default(),
        allowed_tools: vec![ToolName("search".to_owned())],
        model_override: None,
        search_override: None,
        budget_override: None,
    }
}

fn minimal_request() -> ModelRequest {
    ModelRequest {
        provider: String::new(),
        model: None,
        messages: vec![ModelMessage {
            role: ModelMessageRole::User,
            content: "hello".to_string(),
        }],
        tools: Vec::new(),
        temperature: None,
        max_tokens: None,
    }
}

fn prompt_assets() -> PromptAssets {
    PromptAssets {
        aspect_agent_prompt_path: "prompts/layer2/aspect-agent.md".to_owned(),
    }
}

#[test]
fn research_plan_roundtrips_json() {
    let plan = ResearchPlan {
        plan_id: "plan-1".to_owned(),
        user_question: "What should Lapis build first?".to_owned(),
        deliverable: DeliverableSpec {
            kind: "implementation_plan".to_owned(),
            language: "zh-CN".to_owned(),
            expected_sections: vec!["summary".to_owned()],
            notes: vec![],
        },
        constraints: vec![ResearchConstraint {
            key: "scope".to_owned(),
            value: "mvp".to_owned(),
        }],
        aspects: vec![AspectSpec {
            aspect_id: "schema".to_owned(),
            name: "Schema".to_owned(),
            role: "contract reviewer".to_owned(),
            research_question: "Are contracts stable?".to_owned(),
            scope: vec!["schema".to_owned()],
            boundaries: vec![],
            success_criteria: vec!["roundtrip".to_owned()],
            prompt_assets: prompt_assets(),
            required_evidence: EvidenceRequirement::default(),
            allowed_tools: vec![ToolName("search".to_owned())],
            model_override: None,
            search_override: None,
            budget_override: None,
        }],
        budget: ResearchBudget::default(),
        model_policy: ModelPolicy::default(),
        search_policy: SearchPolicy::default(),
        evidence_policy: EvidencePolicy::default(),
        output_policy: OutputPolicy::default(),
    };

    let value = serde_json::to_string(&plan).expect("serialize plan");
    let decoded: ResearchPlan = serde_json::from_str(&value).expect("deserialize plan");

    assert_eq!(decoded.plan_id, plan.plan_id);
    assert_eq!(decoded.aspects[0].role, "contract reviewer");
}

#[test]
fn aspect_research_request_roundtrips_json() {
    let request = AspectResearchRequest {
        schema_version: "m4".to_owned(),
        request_id: "req-1".to_owned(),
        aspect: aspect(),
        shared_context: ResearchContext {
            summary: "shared context".to_owned(),
            ..ResearchContext::default()
        },
        model_policy: ModelPolicy::default(),
        search_policy: SearchPolicy::default(),
        evidence_policy: EvidencePolicy::default(),
        output_policy: OutputPolicy::default(),
        budget: AgentBudget::default(),
        execution_policy: ExecutionPolicy::default(),
    };

    let value = serde_json::to_string(&request).expect("serialize request");
    let decoded: AspectResearchRequest = serde_json::from_str(&value).expect("deserialize request");

    assert_eq!(decoded, request);
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
    let research = ResearchBudget::default();
    assert!(research.max_agents.is_unlimited());
    assert!(research.max_concurrent_agents.is_unlimited());
    assert!(research.max_total_model_calls.is_unlimited());
    assert!(research.max_total_search_calls.is_unlimited());
    assert!(research.total_timeout_ms.is_unlimited());
    assert!(research.max_tokens.is_unlimited());

    let agent = AgentBudget::default();
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
