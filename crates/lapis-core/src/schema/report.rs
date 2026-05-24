use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AspectReport {
    pub aspect_id: String,
    pub aspect_name: String,
    pub question: String,
    pub scope: Vec<String>,
    pub findings: Vec<Finding>,
    pub evidence: Vec<Evidence>,
    pub assumptions: Vec<String>,
    pub risks: Vec<String>,
    pub counterarguments: Vec<String>,
    pub open_questions: Vec<OpenQuestion>,
    pub confidence: Confidence,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Finding {
    pub id: String,
    pub claim: String,
    pub finding_type: FindingType,
    pub importance: Importance,
    pub confidence: Confidence,
    pub evidence_refs: Vec<String>,
    pub contradicted_by: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingType {
    Fact,
    Interpretation,
    Recommendation,
    Risk,
    Assumption,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Importance {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Evidence {
    pub id: String,
    pub source_title: String,
    pub url: Option<String>,
    pub provider: String,
    pub query: String,
    pub snippet: String,
    pub summary: String,
    pub published_at: Option<String>,
    pub retrieved_at: String,
    pub supports_findings: Vec<String>,
    pub source_type: SourceType,
    pub confidence: Confidence,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    #[default]
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Official,
    Documentation,
    News,
    Blog,
    Forum,
    Repository,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct OpenQuestion {
    pub id: String,
    pub question: String,
    pub reason: String,
    pub suggested_follow_up: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct AspectFailure {
    pub aspect_id: String,
    pub error_code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct TraceSummary {
    pub trace_id: String,
    pub root_span: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub model_calls: Vec<ProviderCallSummary>,
    pub search_calls: Vec<ProviderCallSummary>,
    pub termination_reason: Option<TerminationReason>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchSourceTrace {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct SearchToolCallTrace {
    pub provider: String,
    pub query: String,
    pub result_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<SearchSourceTrace>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct SearchQueryTrace {
    pub provider: String,
    pub query: String,
    pub result_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<SearchSourceTrace>,
    pub started_at: String,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ToolCallTrace {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    pub tool_name: String,
    pub input_summary: String,
    pub output_summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<SearchToolCallTrace>,
    pub started_at: String,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct PartialTrace {
    pub trace_summary: TraceSummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub search_queries: Vec<SearchQueryTrace>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCallTrace>,
    pub provider_usage: ProviderUsage,
    pub budget_usage: AgentBudgetUsage,
    pub evidence_count: usize,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ProviderCallSummary {
    pub provider: String,
    pub provider_type: ProviderType,
    pub status: String,
    pub duration_ms: u64,
    pub retry_count: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    Model,
    Search,
    Network,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminationReason {
    Completed,
    PartialCompleted,
    BudgetExceeded,
    Timeout,
    ToolPolicyDenied,
    SchemaValidationFailed,
    ProviderError,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ProviderUsage {
    pub model_calls: usize,
    pub search_calls: usize,
    pub network_calls: usize,
    pub token_usage: Option<TokenUsage>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct AgentBudgetUsage {
    pub turns_used: usize,
    pub tool_calls_used: usize,
    pub search_calls_used: usize,
    pub elapsed_ms: u64,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ResearchBudgetUsage {
    pub agents_started: usize,
    pub model_calls_used: usize,
    pub search_calls_used: usize,
    pub elapsed_ms: u64,
    pub token_usage: Option<TokenUsage>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ValidationStatus {
    pub ok: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct CoverageSummary {
    pub requested_aspects: usize,
    pub completed_aspects: usize,
    pub failed_aspects: usize,
    pub evidence_count: usize,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ConfidenceSummary {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AspectResearchResult {
    pub aspect_report: AspectReport,
    pub evidence: Vec<Evidence>,
    pub search_queries: Vec<SearchQueryTrace>,
    pub tool_calls: Vec<ToolCallTrace>,
    pub provider_usage: ProviderUsage,
    pub budget_usage: AgentBudgetUsage,
    pub validation_status: ValidationStatus,
    pub trace_summary: TraceSummary,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct DeepResearchResult {
    pub run_id: String,
    pub plan_id: String,
    pub completed_aspects: Vec<String>,
    pub failed_aspects: Vec<AspectFailure>,
    pub aspect_reports: Vec<AspectReport>,
    pub evidence_index: Vec<Evidence>,
    pub open_questions: Vec<OpenQuestion>,
    pub coverage_summary: CoverageSummary,
    pub confidence_summary: ConfidenceSummary,
    pub budget_usage: ResearchBudgetUsage,
    pub trace_summary: TraceSummary,
}
