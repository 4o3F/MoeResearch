use std::future::Future;
use std::time::{Duration, Instant};

use crate::limit::{DurationLimitMs, Limit};
use moe_research_error::{Error, Result};

pub(crate) fn elapsed_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn timeout_budget_exceeded(timeout_ms: DurationLimitMs) -> Error {
    let cap = match timeout_ms {
        Limit::Unlimited => "unlimited".to_owned(),
        Limit::Limited(value) => value.to_string(),
    };
    Error::BudgetExceeded {
        message: format!("budget exceeded: timeout_ms exhausted (effective cap {cap})"),
    }
}

pub(crate) struct RuntimeDeadline {
    started: Instant,
    timeout_ms: DurationLimitMs,
}

impl RuntimeDeadline {
    pub(crate) fn new(timeout_ms: DurationLimitMs) -> Self {
        Self {
            started: Instant::now(),
            timeout_ms,
        }
    }

    pub(crate) fn remaining(&self) -> Result<Option<Duration>> {
        match self.timeout_ms {
            Limit::Unlimited => Ok(None),
            Limit::Limited(limit_ms) => {
                let elapsed = elapsed_ms(self.started.elapsed());
                if elapsed >= limit_ms {
                    return Err(timeout_budget_exceeded(self.timeout_ms));
                }
                Ok(Some(Duration::from_millis(limit_ms - elapsed)))
            }
        }
    }

    pub(crate) async fn run<F, T>(&self, future: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match self.remaining()? {
            None => future.await,
            Some(remaining) => tokio::time::timeout(remaining, future)
                .await
                .map_err(|_| timeout_budget_exceeded(self.timeout_ms))?,
        }
    }
}
