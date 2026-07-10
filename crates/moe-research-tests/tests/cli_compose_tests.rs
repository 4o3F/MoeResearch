//! Pure CLI composition mapping tests.
//!
//! Lives in the dedicated test crate (not `#[cfg(test)]` inside CLI sources).
//! Covers ConfigLimit→Limit, Grok effort dual-enum mapping, and provider
//! credential helpers owned by `moe_research_cli::compose`.

use moe_research_cli::compose::{
    build_workflow_budget, map_grok_reasoning_effort, map_limit, provider_api_key, provider_model,
};
use moe_research_config::{
    AgentLimitsConfig, ConfigLimit, GrokReasoningEffort as ConfigGrokEffort, LimitsConfig,
    ResearchLimitsConfig,
};
use moe_research_search::GrokReasoningEffort as SearchGrokEffort;
use moe_research_workflow::Limit;

#[test]
fn map_limit_maps_limited_and_unlimited() {
    assert_eq!(map_limit(ConfigLimit::limited(7_usize)), Limit::limited(7));
    assert_eq!(
        map_limit::<usize>(ConfigLimit::unlimited()),
        Limit::unlimited()
    );
    assert_eq!(map_limit(ConfigLimit::limited(0_u64)), Limit::limited(0));
}

#[test]
fn build_workflow_budget_maps_every_field() {
    let config = LimitsConfig {
        research: ResearchLimitsConfig {
            max_agents: ConfigLimit::limited(3),
            max_concurrent_agents: ConfigLimit::limited(2),
            max_total_model_calls: ConfigLimit::limited(10),
            max_total_search_calls: ConfigLimit::unlimited(),
            total_timeout_ms: ConfigLimit::limited(60_000),
            max_tokens: ConfigLimit::limited(100_000),
        },
        per_agent: AgentLimitsConfig {
            max_turns: ConfigLimit::limited(5),
            max_tool_calls: ConfigLimit::limited(8),
            max_search_calls: ConfigLimit::limited(4),
            timeout_ms: ConfigLimit::unlimited(),
        },
    };

    let budget = build_workflow_budget(&config);

    assert_eq!(budget.research.max_agents, Limit::limited(3));
    assert_eq!(budget.research.max_concurrent_agents, Limit::limited(2));
    assert_eq!(budget.research.max_total_model_calls, Limit::limited(10));
    assert_eq!(budget.research.max_total_search_calls, Limit::unlimited());
    assert_eq!(budget.research.total_timeout_ms, Limit::limited(60_000));
    assert_eq!(budget.research.max_tokens, Limit::limited(100_000));
    assert_eq!(budget.per_agent.max_turns, Limit::limited(5));
    assert_eq!(budget.per_agent.max_tool_calls, Limit::limited(8));
    assert_eq!(budget.per_agent.max_search_calls, Limit::limited(4));
    assert_eq!(budget.per_agent.timeout_ms, Limit::unlimited());
}

#[test]
fn map_grok_reasoning_effort_is_exhaustive_and_1to1() {
    let cases = [
        (ConfigGrokEffort::None, SearchGrokEffort::None),
        (ConfigGrokEffort::Low, SearchGrokEffort::Low),
        (ConfigGrokEffort::Medium, SearchGrokEffort::Medium),
        (ConfigGrokEffort::High, SearchGrokEffort::High),
    ];
    for (from, expected) in cases {
        assert_eq!(map_grok_reasoning_effort(from), expected);
        assert_eq!(from.as_str(), expected.as_str());
    }
}

#[test]
fn provider_model_rejects_missing_and_blank() {
    assert!(provider_model("model", "openai", None).is_err());
    assert!(provider_model("model", "openai", Some(&"  ".to_owned())).is_err());
    assert_eq!(
        provider_model("model", "openai", Some(&" gpt-test ".to_owned())).unwrap(),
        "gpt-test"
    );
}

#[test]
fn provider_api_key_requires_env_name() {
    let err = provider_api_key("search", "exa", None).unwrap_err();
    assert_eq!(err.code().as_str(), "provider_unavailable");
}
