//! Research workflow boundary for Lapis.

pub mod agent_loop;
pub mod budget;
pub mod limit;
pub mod policy;
pub mod report;
pub mod research;
pub mod runtime_budget;
pub mod tool_policy;
pub mod validator;
pub mod workflow;

pub use agent_loop::{AgentRuntime, AgentRuntimeFailure, AgentRuntimeOutput};
pub use budget::{AgentBudget, BudgetConfig, ResearchBudget};
pub use limit::{CountLimit, DurationLimitMs, Limit, TokenLimit};
pub use policy::{
    EvidencePolicy, ExecutionPolicy, Freshness, ModelPolicy, OutputPolicy, SearchPolicy, ToolName,
};
pub use report::{
    AgentBudgetUsage, AspectFailure, AspectReport, AspectResearchResult, Confidence,
    ConfidenceSummary, CoverageSummary, DeepResearchResult, Evidence, Finding, FindingType,
    Importance, OpenQuestion, ResearchBudgetUsage, SourceType, TokenUsage, ValidationIssue,
    ValidationStatus,
};
pub use research::{
    AspectResearchRequest, AspectResearchTask, AspectSpec, DeepResearchRequest, ResearchContext,
};
pub use runtime_budget::{AgentBudgetGuard, ResearchBudgetGuard};
pub use tool_policy::{SEARCH_TOOL_NAME, SearchToolArgs, ToolPolicyGuard, search_model_tool};
pub use validator::{OutputValidator, validate_output};
pub use workflow::{DeepResearchFailure, aspect_research, deep_research};
