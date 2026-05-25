//! Per-aspect and cross-aspect budget guards.
//!
//! Two enforcement scopes live here:
//! - [`AgentBudgetGuard`] tracks a single aspect's turn/tool/search counters
//!   and timeout. It is owned by the per-aspect agent loop and is not shared
//!   across tasks.
//! - [`ResearchBudgetGuard`] enforces the deep-research-level ceilings
//!   (model calls, search calls, tokens, timeout) across all aspects of one
//!   `deep_research` invocation. It is shared via `Arc` and uses atomic
//!   counters so concurrent aspect loops can reserve slots without a mutex
//!   on the hot path.
//!
//! Both guards reserve a slot *before* dispatching the provider call so the
//! orchestrator never observes a budget overshoot, and both surface
//! `Error::BudgetExceeded` on rejection so the public failure code stays
//! stable.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::{
    error::{Error, Result},
    schema::{
        budget::{AgentBudget, ResearchBudget},
        report::{AgentBudgetUsage, ResearchBudgetUsage, TokenUsage},
    },
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

/// Cross-aspect budget guard enforcing research-level caps before each
/// model/search dispatch instead of after aggregation.
///
/// Internally uses atomic counters so concurrent aspect loops share the
/// same budget without a mutex on the hot path. Token usage merges under a
/// mutex because [`TokenUsage`] accumulation is not atomic, but the
/// mutex is only acquired on provider replies (not on every tool turn).
pub struct ResearchBudgetGuard {
    budget: ResearchBudget,
    started: Instant,
    model_calls: AtomicU64,
    search_calls: AtomicU64,
    agents_started: AtomicUsize,
    token_usage: Mutex<Option<TokenUsage>>,
}

impl std::fmt::Debug for ResearchBudgetGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResearchBudgetGuard")
            .field("budget", &self.budget)
            .field("model_calls", &self.model_calls.load(Ordering::SeqCst))
            .field("search_calls", &self.search_calls.load(Ordering::SeqCst))
            .field("agents_started", &self.agents_started.load(Ordering::SeqCst))
            .finish_non_exhaustive()
    }
}

impl ResearchBudgetGuard {
    /// Builds a new cross-aspect guard.
    ///
    /// The supplied [`ResearchBudget`] is captured by value so subsequent
    /// reservations evaluate against an immutable snapshot of the requested
    /// caps.
    #[must_use]
    pub fn new(budget: ResearchBudget) -> Arc<Self> {
        Arc::new(Self {
            budget,
            started: Instant::now(),
            model_calls: AtomicU64::new(0),
            search_calls: AtomicU64::new(0),
            agents_started: AtomicUsize::new(0),
            token_usage: Mutex::new(None),
        })
    }

    /// Constructs a guard with no caps on any dimension.
    ///
    /// Intended for the standalone `aspect_research` MCP entry point: the
    /// per-aspect [`AgentBudgetGuard`] still enforces the local turn/tool
    /// caps, and there is no aggregate research budget to apply.
    #[must_use]
    pub fn unlimited() -> Arc<Self> {
        Self::new(ResearchBudget::unlimited())
    }

    /// Records that one aspect has begun execution.
    pub fn record_agent_started(&self) {
        self.agents_started.fetch_add(1, Ordering::SeqCst);
    }

    /// Reserves one model-call slot before dispatch.
    ///
    /// # Errors
    /// Returns [`Error::BudgetExceeded`] if the global `max_total_model_calls`
    /// cap would be exceeded. The counter is rolled back on rejection so the
    /// usage snapshot does not record calls that never ran.
    pub fn try_consume_model_call(&self) -> Result<u64> {
        let next = self.model_calls.fetch_add(1, Ordering::SeqCst) + 1;
        if self
            .budget
            .max_total_model_calls
            .is_exceeded_by(usize_from_u64(next))
        {
            self.model_calls.fetch_sub(1, Ordering::SeqCst);
            return Err(Error::BudgetExceeded {
                message: "research model call budget exhausted".to_owned(),
            });
        }
        Ok(next)
    }

    /// Reserves one search-call slot before dispatch.
    ///
    /// # Errors
    /// Returns [`Error::BudgetExceeded`] if the global `max_total_search_calls`
    /// cap would be exceeded. Rollback semantics mirror
    /// [`Self::try_consume_model_call`].
    pub fn try_consume_search_call(&self) -> Result<u64> {
        let next = self.search_calls.fetch_add(1, Ordering::SeqCst) + 1;
        if self
            .budget
            .max_total_search_calls
            .is_exceeded_by(usize_from_u64(next))
        {
            self.search_calls.fetch_sub(1, Ordering::SeqCst);
            return Err(Error::BudgetExceeded {
                message: "research search call budget exhausted".to_owned(),
            });
        }
        Ok(next)
    }

    /// Merges provider-reported token usage into the running total and
    /// enforces the `max_tokens` cap.
    ///
    /// # Errors
    /// Returns [`Error::BudgetExceeded`] once the merged total exceeds
    /// [`ResearchBudget::max_tokens`]. Providers that omit usage do not
    /// advance the counter, matching the upstream provider contract that
    /// `usage = None` means "untracked", not "zero".
    ///
    /// # Panics
    /// Panics if the internal `Mutex<Option<TokenUsage>>` is poisoned by a
    /// previous panic while the lock was held; under normal operation this
    /// cannot happen because the merge body cannot panic.
    pub fn record_token_usage(&self, usage: Option<TokenUsage>) -> Result<()> {
        let Some(delta) = usage else {
            return Ok(());
        };
        let mut guard = self
            .token_usage
            .lock()
            .expect("research budget guard token usage mutex poisoned");
        let merged = TokenUsage::merge(guard.take(), Some(delta));
        guard.clone_from(&merged);
        if let Some(total) = merged.and_then(|usage| usage.total_or_sum()) {
            self.budget.ensure_tokens_within(total)?;
        }
        Ok(())
    }

    /// Returns a point-in-time snapshot of all cross-aspect counters.
    ///
    /// # Panics
    /// Panics if the internal token-usage mutex is poisoned; see
    /// [`Self::record_token_usage`] for why this should never occur.
    #[must_use]
    pub fn snapshot(&self) -> ResearchBudgetUsage {
        let token_usage = self
            .token_usage
            .lock()
            .expect("research budget guard token usage mutex poisoned")
            .clone();
        ResearchBudgetUsage {
            agents_started: self.agents_started.load(Ordering::SeqCst),
            model_calls_used: usize_from_u64(self.model_calls.load(Ordering::SeqCst)),
            search_calls_used: usize_from_u64(self.search_calls.load(Ordering::SeqCst)),
            elapsed_ms: self
                .started
                .elapsed()
                .as_millis()
                .try_into()
                .unwrap_or(u64::MAX),
            token_usage,
        }
    }
}

fn usize_from_u64(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}
