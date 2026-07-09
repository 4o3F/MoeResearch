use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use moe_research_error::{Error, Result};

use super::limit::{CountLimit, DurationLimitMs, Limit, TokenLimit};
use super::report::ResearchBudgetUsage;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetConfig {
    pub research: ResearchLimits,
    pub per_agent: AgentLimits,
}

impl BudgetConfig {
    pub fn validate(&self) -> Result<()> {
        self.research
            .max_agents
            .require_non_zero("budget.research.max_agents")?;
        self.research
            .max_concurrent_agents
            .require_non_zero("budget.research.max_concurrent_agents")?;
        self.research
            .total_timeout_ms
            .require_non_zero("budget.research.total_timeout_ms")?;
        self.per_agent
            .max_turns
            .require_non_zero("budget.per_agent.max_turns")?;
        self.per_agent
            .timeout_ms
            .require_non_zero("budget.per_agent.timeout_ms")?;

        if self
            .research
            .max_concurrent_agents
            .exceeds(self.research.max_agents)
        {
            return Err(Error::ConfigInvalid {
                message: "budget.research.max_concurrent_agents must not exceed \
                          budget.research.max_agents"
                    .to_owned(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchLimits {
    pub max_agents: CountLimit,
    pub max_concurrent_agents: CountLimit,
    pub max_total_model_calls: CountLimit,
    pub max_total_search_calls: CountLimit,
    pub total_timeout_ms: DurationLimitMs,
    pub max_tokens: TokenLimit,
}

impl ResearchLimits {
    /// Explicitly construct research limits with no caps on any dimension.
    /// Intended for tests and controlled environments only; production callers
    /// must declare each dimension to prevent unbounded resource use.
    #[must_use]
    pub fn unlimited() -> Self {
        Self {
            max_agents: Limit::unlimited(),
            max_concurrent_agents: Limit::unlimited(),
            max_total_model_calls: Limit::unlimited(),
            max_total_search_calls: Limit::unlimited(),
            total_timeout_ms: Limit::unlimited(),
            max_tokens: Limit::unlimited(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentLimits {
    pub max_turns: CountLimit,
    pub max_tool_calls: CountLimit,
    pub max_search_calls: CountLimit,
    pub timeout_ms: DurationLimitMs,
}

impl AgentLimits {
    /// Explicitly construct agent limits with no caps on any dimension.
    /// Intended for tests and controlled environments only.
    #[must_use]
    pub fn unlimited() -> Self {
        Self {
            max_turns: Limit::unlimited(),
            max_tool_calls: Limit::unlimited(),
            max_search_calls: Limit::unlimited(),
            timeout_ms: Limit::unlimited(),
        }
    }
}

impl ResearchLimits {
    pub(crate) fn validate_against_config(&self, limits: &BudgetConfig) -> Result<()> {
        ensure_budget_non_zero(
            "research budget requires at least one agent",
            self.max_agents,
        )?;
        ensure_budget_non_zero(
            "research budget requires non-zero concurrency",
            self.max_concurrent_agents,
        )?;
        ensure_budget_non_zero(
            "research budget requires a non-zero timeout",
            self.total_timeout_ms,
        )?;

        if self.max_concurrent_agents.exceeds(self.max_agents) {
            return Err(Error::BudgetExceeded {
                message: "research concurrency must not exceed max_agents".to_owned(),
            });
        }

        ensure_within(
            "research max_agents",
            self.max_agents,
            limits.research.max_agents,
        )?;
        ensure_within(
            "research concurrency",
            self.max_concurrent_agents,
            limits.research.max_concurrent_agents,
        )?;
        ensure_within(
            "research model calls",
            self.max_total_model_calls,
            limits.research.max_total_model_calls,
        )?;
        ensure_within(
            "research search calls",
            self.max_total_search_calls,
            limits.research.max_total_search_calls,
        )?;
        ensure_within(
            "research timeout",
            self.total_timeout_ms,
            limits.research.total_timeout_ms,
        )?;
        ensure_within(
            "research tokens",
            self.max_tokens,
            limits.research.max_tokens,
        )?;

        Ok(())
    }

    pub(crate) fn ensure_usage_within(&self, usage: &ResearchBudgetUsage) -> Result<()> {
        ensure_usage_within(
            "research model call",
            Limit::limited(usage.model_calls_used),
            self.max_total_model_calls,
        )?;
        ensure_usage_within(
            "research search call",
            Limit::limited(usage.search_calls_used),
            self.max_total_search_calls,
        )?;
        ensure_usage_within(
            "research timeout",
            Limit::limited(usage.elapsed_ms),
            self.total_timeout_ms,
        )?;
        if let Some(total_tokens) = usage
            .token_usage
            .as_ref()
            .and_then(super::report::TokenUsage::total_or_sum)
        {
            self.ensure_tokens_within(total_tokens)?;
        }
        Ok(())
    }

    /// Asserts that the supplied cumulative token total stays within
    /// `max_tokens`.
    ///
    /// # Errors
    /// Returns `Error::BudgetExceeded` when `total_tokens` exceeds the cap.
    /// Untracked usage (provider reported nothing) is reported as `None` by
    /// the caller and therefore does not advance the counter.
    pub(crate) fn ensure_tokens_within(&self, total_tokens: u64) -> Result<()> {
        ensure_usage_within(
            "research token",
            Limit::limited(total_tokens),
            self.max_tokens,
        )
    }
}

impl AgentLimits {
    pub(crate) fn ensure_runnable(&self) -> Result<()> {
        ensure_budget_non_zero(
            "agent budget requires at least one model turn",
            self.max_turns,
        )?;
        ensure_budget_non_zero("agent budget requires a non-zero timeout", self.timeout_ms)?;
        Ok(())
    }

    pub(crate) fn validate_against_config(&self, limits: &BudgetConfig) -> Result<()> {
        self.ensure_runnable()?;
        ensure_within("agent turns", self.max_turns, limits.per_agent.max_turns)?;
        ensure_within(
            "agent tool calls",
            self.max_tool_calls,
            limits.per_agent.max_tool_calls,
        )?;
        ensure_within(
            "agent search calls",
            self.max_search_calls,
            limits.per_agent.max_search_calls,
        )?;
        ensure_within(
            "agent timeout",
            self.timeout_ms,
            limits.per_agent.timeout_ms,
        )?;
        Ok(())
    }
}

fn ensure_within<T>(label: &str, value: Limit<T>, max: Limit<T>) -> Result<()>
where
    T: Copy + Ord,
{
    if value.exceeds(max) {
        return Err(Error::BudgetExceeded {
            message: format!("{label} exceeds configured budget limit"),
        });
    }
    Ok(())
}

fn ensure_usage_within<T>(label: &str, used: Limit<T>, max: Limit<T>) -> Result<()>
where
    T: Copy + Ord,
{
    if used.exceeds(max) {
        return Err(Error::BudgetExceeded {
            message: format!("{label} budget exhausted"),
        });
    }
    Ok(())
}

fn ensure_budget_non_zero<T>(message: &str, value: Limit<T>) -> Result<()>
where
    T: Copy + PartialEq + Default,
{
    if value.is_zero() {
        return Err(Error::BudgetExceeded {
            message: message.to_owned(),
        });
    }
    Ok(())
}
