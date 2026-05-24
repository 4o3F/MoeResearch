use std::time::Instant;

use crate::{
    error::{Error, Result},
    schema::{budget::AgentBudget, report::AgentBudgetUsage},
};

#[derive(Clone, Debug)]
pub struct AgentBudgetGuard {
    budget: AgentBudget,
    start_time: Instant,
    turns_used: usize,
    tool_calls_used: usize,
    search_calls_used: usize,
}

impl AgentBudgetGuard {
    pub fn new(budget: AgentBudget) -> Result<Self> {
        budget.ensure_runnable()?;
        Ok(Self {
            budget,
            start_time: Instant::now(),
            turns_used: 0,
            tool_calls_used: 0,
            search_calls_used: 0,
        })
    }

    pub fn consume_model_turn(&mut self) -> Result<()> {
        self.check_timeout()?;

        if !self.budget.max_turns.permits_next(self.turns_used) {
            return Err(Error::BudgetExceeded {
                message: "agent model turn budget exhausted".to_owned(),
            });
        }

        self.turns_used += 1;
        Ok(())
    }

    pub fn consume_tool_call(&mut self) -> Result<()> {
        self.check_timeout()?;

        if !self
            .budget
            .max_tool_calls
            .permits_next(self.tool_calls_used)
        {
            return Err(Error::BudgetExceeded {
                message: "agent tool call budget exhausted".to_owned(),
            });
        }

        self.tool_calls_used += 1;
        Ok(())
    }

    pub fn consume_search_call(&mut self) -> Result<()> {
        self.check_timeout()?;

        if !self
            .budget
            .max_search_calls
            .permits_next(self.search_calls_used)
        {
            return Err(Error::BudgetExceeded {
                message: "agent search call budget exhausted".to_owned(),
            });
        }

        self.search_calls_used += 1;
        Ok(())
    }

    pub fn consume_search_tool_call(&mut self) -> Result<()> {
        self.check_timeout()?;

        if !self
            .budget
            .max_tool_calls
            .permits_next(self.tool_calls_used)
        {
            return Err(Error::BudgetExceeded {
                message: "agent tool call budget exhausted".to_owned(),
            });
        }

        if !self
            .budget
            .max_search_calls
            .permits_next(self.search_calls_used)
        {
            return Err(Error::BudgetExceeded {
                message: "agent search call budget exhausted".to_owned(),
            });
        }

        self.tool_calls_used += 1;
        self.search_calls_used += 1;
        Ok(())
    }

    pub fn usage(&self) -> AgentBudgetUsage {
        AgentBudgetUsage {
            turns_used: self.turns_used,
            tool_calls_used: self.tool_calls_used,
            search_calls_used: self.search_calls_used,
            elapsed_ms: self.elapsed_ms(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX)
    }

    fn check_timeout(&self) -> Result<()> {
        if self.budget.timeout_ms.is_elapsed(self.elapsed_ms()) {
            return Err(Error::BudgetExceeded {
                message: "agent timeout budget exhausted".to_owned(),
            });
        }

        Ok(())
    }
}
