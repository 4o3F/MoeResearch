pub mod common;
pub mod config;
pub mod mcp;
pub mod model;
pub mod report;
pub mod search;

#[cfg(test)]
mod tests {
    use super::common::{
        AspectSpec, DeliverableSpec, EvidencePolicy, EvidenceRequirement, ModelPolicy,
        OutputPolicy, ResearchBudget, ResearchConstraint, ResearchPlan, SearchPolicy, ToolName,
    };

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
}
