//! Private agent runtime: state machine, live budgets, deadlines, logical tools, and model turns.

mod agent;
pub(crate) mod budget;
pub(crate) mod deadline;
pub(crate) mod model_turn;
pub(crate) mod tools;

pub(crate) use agent::{AgentRuntime, AgentRuntimeFailure, AgentRuntimeOutput};
pub(crate) use budget::{AgentBudgetGuard, ResearchBudgetGuard};
pub(crate) use deadline::{RuntimeDeadline, elapsed_ms};
pub(crate) use model_turn::{add_token_usage, aspect_response_format};
pub(crate) use tools::SUPPORTED_ASPECT_TOOLS;
