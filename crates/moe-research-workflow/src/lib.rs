//! Research workflow boundary for MoeResearch.

mod agent_loop;
pub mod budget;
mod error_log_safe;
pub mod limit;
pub mod policy;
pub mod report;
pub mod research;
mod runtime;
pub mod workflow;

pub use budget::{AgentLimits, BudgetConfig, ResearchLimits};
pub use limit::{CountLimit, DurationLimitMs, Limit, TokenLimit};
pub use policy::{
    EvidencePolicy, ExecutionPolicy, Freshness, ModelPolicy, OutputPolicy, SearchCategory,
    SearchContentLevel, SearchDepth, SearchPolicy, SearchRecency, ToolName,
};
pub use report::{
    AgentBudgetUsage, AspectFailure, AspectReport, AspectResearchResult, Confidence,
    ConfidenceSummary, CoverageSummary, DeepResearchResult, Evidence, Finding, FindingType,
    Importance, OpenQuestion, ResearchBudgetUsage, SourceType, TokenUsage, ValidationIssue,
    ValidationStatus,
};
pub use research::{
    AspectRequest, AspectResearchRequest, DeepResearchRequest, ResearchContext, ResearchPolicy,
    ResearchTask,
};
pub use workflow::{
    AspectResearchFailure, AspectResearchOutput, DeepResearchFailure, aspect_research,
    deep_research,
};
