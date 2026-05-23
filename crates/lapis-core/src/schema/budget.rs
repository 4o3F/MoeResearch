use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::config::BudgetConfig;
use super::limit::{CountLimit, DurationLimitMs, Limit, TokenLimit};
use super::report::ResearchBudgetUsage;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ResearchBudget {
    pub max_agents: CountLimit,
    pub max_concurrent_agents: CountLimit,
    pub max_total_model_calls: CountLimit,
    pub max_total_search_calls: CountLimit,
    pub total_timeout_ms: DurationLimitMs,
    pub max_tokens: TokenLimit,
}

impl Default for ResearchBudget {
    fn default() -> Self {
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
pub struct AgentBudget {
    pub max_turns: CountLimit,
    pub max_tool_calls: CountLimit,
    pub max_search_calls: CountLimit,
    pub timeout_ms: DurationLimitMs,
}

impl Default for AgentBudget {
    fn default() -> Self {
        Self {
            max_turns: Limit::unlimited(),
            max_tool_calls: Limit::unlimited(),
            max_search_calls: Limit::unlimited(),
            timeout_ms: Limit::unlimited(),
        }
    }
}

impl ResearchBudget {
    pub(crate) fn validate_against_config(&self, limits: &BudgetConfig) -> Result<()> {
        if self.max_agents.is_zero() {
            return Err(Error::BudgetExceeded {
                message: "research budget requires at least one agent".to_owned(),
            });
        }

        if self.max_concurrent_agents.is_zero() {
            return Err(Error::BudgetExceeded {
                message: "research budget requires non-zero concurrency".to_owned(),
            });
        }

        if self.total_timeout_ms.is_zero() {
            return Err(Error::BudgetExceeded {
                message: "research budget requires a non-zero timeout".to_owned(),
            });
        }

        if self.max_concurrent_agents.exceeds(self.max_agents) {
            return Err(Error::BudgetExceeded {
                message: "research concurrency must not exceed max_agents".to_owned(),
            });
        }

        if self.max_agents.exceeds(limits.max_agents) {
            return Err(Error::BudgetExceeded {
                message: "research max_agents exceeds configured budget limit".to_owned(),
            });
        }

        if self
            .max_concurrent_agents
            .exceeds(limits.max_concurrent_agents)
        {
            return Err(Error::BudgetExceeded {
                message: "research concurrency exceeds configured budget limit".to_owned(),
            });
        }

        if self
            .max_total_model_calls
            .exceeds(limits.max_total_model_calls)
        {
            return Err(Error::BudgetExceeded {
                message: "research model calls exceed configured budget limit".to_owned(),
            });
        }

        if self
            .max_total_search_calls
            .exceeds(limits.max_total_search_calls)
        {
            return Err(Error::BudgetExceeded {
                message: "research search calls exceed configured budget limit".to_owned(),
            });
        }

        if self.total_timeout_ms.exceeds(limits.max_total_timeout_ms) {
            return Err(Error::BudgetExceeded {
                message: "research timeout exceeds configured budget limit".to_owned(),
            });
        }

        Ok(())
    }

    pub(crate) fn ensure_usage_within(&self, usage: &ResearchBudgetUsage) -> Result<()> {
        if Limit::limited(usage.model_calls_used).exceeds(self.max_total_model_calls) {
            return Err(Error::BudgetExceeded {
                message: "research model call budget exhausted".to_owned(),
            });
        }

        if Limit::limited(usage.search_calls_used).exceeds(self.max_total_search_calls) {
            return Err(Error::BudgetExceeded {
                message: "research search call budget exhausted".to_owned(),
            });
        }

        if Limit::limited(usage.elapsed_ms).exceeds(self.total_timeout_ms) {
            return Err(Error::BudgetExceeded {
                message: "research timeout budget exhausted".to_owned(),
            });
        }

        Ok(())
    }
}

impl AgentBudget {
    pub(crate) fn ensure_runnable(&self) -> Result<()> {
        if self.max_turns.is_zero() {
            return Err(Error::BudgetExceeded {
                message: "agent budget requires at least one model turn".to_owned(),
            });
        }

        if self.timeout_ms.is_zero() {
            return Err(Error::BudgetExceeded {
                message: "agent budget requires a non-zero timeout".to_owned(),
            });
        }

        Ok(())
    }

    pub(crate) fn validate_against_config(&self, limits: &BudgetConfig) -> Result<()> {
        self.ensure_runnable()?;

        if self.max_turns.exceeds(limits.max_turns_per_agent) {
            return Err(Error::BudgetExceeded {
                message: "agent turns exceed configured budget limit".to_owned(),
            });
        }

        if self.max_tool_calls.exceeds(limits.max_tool_calls_per_agent) {
            return Err(Error::BudgetExceeded {
                message: "agent tool calls exceed configured budget limit".to_owned(),
            });
        }

        if self
            .max_search_calls
            .exceeds(limits.max_search_calls_per_agent)
        {
            return Err(Error::BudgetExceeded {
                message: "agent search calls exceed configured budget limit".to_owned(),
            });
        }

        if self.timeout_ms.exceeds(limits.max_agent_timeout_ms) {
            return Err(Error::BudgetExceeded {
                message: "agent timeout exceeds configured budget limit".to_owned(),
            });
        }

        Ok(())
    }
}
