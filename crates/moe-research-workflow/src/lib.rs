//! Research workflow boundary for MoeResearch.

pub mod budget;
pub mod limit;
pub mod policy;
pub mod report;
pub mod research;
pub mod workflow;

mod error_log_safe;
mod runtime;

pub use budget::{AgentLimits, BudgetConfig, ResearchLimits};
pub use limit::{CountLimit, DurationLimitMs, Limit, TokenLimit};
pub use policy::{
    EvidencePolicy, ExecutionPolicy, Freshness, ModelPolicy, OutputPolicy, SearchCategory,
    SearchContentLevel, SearchDepth, SearchPolicy, SearchRecency, ToolName,
};
pub use report::{
    AgentBudgetUsage, AspectFailure, AspectReport, AspectResearchResult, Confidence,
    ConfidenceSummary, CoverageSummary, DeepResearchResult, Evidence, FailureDiagnostic,
    FailureStage, Finding, FindingType, Importance, OpenQuestion, ResearchBudgetUsage, SourceType,
    TokenUsage, ValidationIssue, ValidationStatus, provenance_mismatch_fields,
};
pub use research::{
    AspectRequest, AspectResearchRequest, DeepResearchRequest, ResearchContext, ResearchPolicy,
    ResearchTask, effective_research_limits,
};
pub use workflow::{
    AspectResearchFailure, AspectResearchOutput, DeepResearchFailure, aspect_research,
    deep_research,
};
