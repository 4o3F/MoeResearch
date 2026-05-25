use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AspectReport {
    pub aspect_id: String,
    pub aspect_name: String,
    pub question: String,
    pub scope: Vec<String>,
    pub findings: Vec<Finding>,
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

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
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

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

impl TokenUsage {
    /// Zero baseline for cumulative statistics: every dimension is unobserved.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            input_tokens: None,
            output_tokens: None,
            total_tokens: None,
        }
    }

    /// Returns the best available total-token count for this usage record.
    ///
    /// Prefers the provider-reported `total_tokens`; falls back to
    /// `input_tokens + output_tokens` (saturating) when only the components
    /// are present. Returns `None` when no usage field is reported, so the
    /// caller can treat the call as untracked rather than zero.
    #[must_use]
    pub fn total_or_sum(&self) -> Option<u64> {
        self.total_tokens.or_else(|| {
            self.input_tokens
                .zip(self.output_tokens)
                .map(|(i, o)| i.saturating_add(o))
        })
    }

    /// Combines two optional usage records dimension by dimension.
    ///
    /// Saturating addition prevents overflow on long-running runs; `None`
    /// dimensions in either operand pass through unchanged so providers that
    /// report only `total_tokens` are not forced to synthesize component
    /// counters.
    #[must_use]
    pub fn merge(left: Option<Self>, right: Option<Self>) -> Option<Self> {
        match (left, right) {
            (None, None) => None,
            (Some(usage), None) | (None, Some(usage)) => Some(usage),
            (Some(left), Some(right)) => Some(Self {
                input_tokens: sum_optional_tokens(left.input_tokens, right.input_tokens),
                output_tokens: sum_optional_tokens(left.output_tokens, right.output_tokens),
                total_tokens: sum_optional_tokens(left.total_tokens, right.total_tokens),
            }),
        }
    }
}

fn sum_optional_tokens(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (None, None) => None,
        (Some(value), None) | (None, Some(value)) => Some(value),
        (Some(left), Some(right)) => Some(left.saturating_add(right)),
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct AgentBudgetUsage {
    pub turns_used: usize,
    pub tool_calls_used: usize,
    pub search_calls_used: usize,
    pub elapsed_ms: u64,
}

impl AgentBudgetUsage {
    /// Zero baseline for cumulative statistics.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            turns_used: 0,
            tool_calls_used: 0,
            search_calls_used: 0,
            elapsed_ms: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ResearchBudgetUsage {
    pub agents_started: usize,
    pub model_calls_used: usize,
    pub search_calls_used: usize,
    pub elapsed_ms: u64,
    pub token_usage: Option<TokenUsage>,
}

impl ResearchBudgetUsage {
    /// Zero baseline for cumulative statistics.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            agents_started: 0,
            model_calls_used: 0,
            search_calls_used: 0,
            elapsed_ms: 0,
            token_usage: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ValidationStatus {
    pub ok: bool,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationStatus {
    /// Explicitly construct a "passed" validation status.
    #[must_use]
    pub fn passed() -> Self {
        Self {
            ok: true,
            issues: Vec::new(),
        }
    }

    /// Explicitly construct a "failed" validation status with the given issues.
    #[must_use]
    pub fn failed(issues: Vec<ValidationIssue>) -> Self {
        Self { ok: false, issues }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct CoverageSummary {
    pub requested_aspects: usize,
    pub completed_aspects: usize,
    pub failed_aspects: usize,
    pub evidence_count: usize,
}

impl CoverageSummary {
    /// Zero baseline for cumulative statistics.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            requested_aspects: 0,
            completed_aspects: 0,
            failed_aspects: 0,
            evidence_count: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ConfidenceSummary {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

impl ConfidenceSummary {
    /// Zero baseline for cumulative statistics.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            high: 0,
            medium: 0,
            low: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AspectResearchResult {
    pub aspect_report: AspectReport,
    pub evidence: Vec<Evidence>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct DeepResearchResult {
    pub run_id: String,
    pub completed_aspects: Vec<String>,
    pub failed_aspects: Vec<AspectFailure>,
    pub aspect_reports: Vec<AspectReport>,
    pub evidence_index: Vec<Evidence>,
    pub open_questions: Vec<OpenQuestion>,
    pub coverage_summary: CoverageSummary,
    pub confidence_summary: ConfidenceSummary,
    pub budget_usage: ResearchBudgetUsage,
}
