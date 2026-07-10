//! Private agent runtime support: live budgets, deadlines, search tool, model turn helpers.
//!
//! The agent state machine currently remains in `crate::agent_loop` and is re-exported
//! here for a stable internal import path; Stage D2 moves it into `runtime/agent`.

pub(crate) mod budget;
pub(crate) mod deadline;
pub(crate) mod model_turn;
pub(crate) mod search_tool;

pub(crate) use budget::{AgentBudgetGuard, ResearchBudgetGuard};
pub(crate) use deadline::{RuntimeDeadline, elapsed_ms};
pub(crate) use model_turn::{add_token_usage, aspect_response_format};
pub(crate) use search_tool::SEARCH_TOOL_NAME;
