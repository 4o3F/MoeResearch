//! CLI composition root: pure config → runtime mapping and service wiring.
//!
//! Domain crates stay free of host wiring. All ConfigLimit / Grok / provider
//! registration mapping lives here so `commands/serve` stays a thin host.

use moe_research_config::{ConfigLimit, LimitsConfig};
use moe_research_workflow::{AgentLimits, BudgetConfig, Limit, ResearchLimits};

/// Map operator config limit into workflow budget limit (dual types intentionally kept).
#[must_use]
pub(crate) fn map_limit<T>(limit: ConfigLimit<T>) -> Limit<T> {
    match limit {
        ConfigLimit::Limited(value) => Limit::limited(value),
        ConfigLimit::Unlimited => Limit::unlimited(),
    }
}

/// Build workflow `BudgetConfig` from operator `LimitsConfig`.
#[must_use]
pub(crate) fn build_workflow_budget(config: &LimitsConfig) -> BudgetConfig {
    BudgetConfig {
        research: ResearchLimits {
            max_agents: map_limit(config.research.max_agents),
            max_concurrent_agents: map_limit(config.research.max_concurrent_agents),
            max_total_model_calls: map_limit(config.research.max_total_model_calls),
            max_total_search_calls: map_limit(config.research.max_total_search_calls),
            total_timeout_ms: map_limit(config.research.total_timeout_ms),
            max_tokens: map_limit(config.research.max_tokens),
        },
        per_agent: AgentLimits {
            max_turns: map_limit(config.per_agent.max_turns),
            max_tool_calls: map_limit(config.per_agent.max_tool_calls),
            max_search_calls: map_limit(config.per_agent.max_search_calls),
            timeout_ms: map_limit(config.per_agent.timeout_ms),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moe_research_config::{AgentLimitsConfig, ResearchLimitsConfig};

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
}
