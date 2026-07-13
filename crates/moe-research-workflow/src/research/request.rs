use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::budget::{AgentLimits, BudgetConfig, ResearchLimits};
use super::super::policy::{
    EvidencePolicy, ExecutionPolicy, ModelPolicy, OutputPolicy, SearchPolicy, ToolName,
};
use super::super::report::Evidence;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchContext {
    pub summary: String,
    pub known_facts: Vec<String>,
    pub excluded_assumptions: Vec<String>,
    pub prior_sources: Vec<Evidence>,
}

impl ResearchContext {
    /// Explicitly construct an empty research context that carries no prior
    /// knowledge.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            summary: String::new(),
            known_facts: Vec::new(),
            excluded_assumptions: Vec::new(),
            prior_sources: Vec::new(),
        }
    }
}

/// Policies that shape one MoeResearch request without carrying resource limits.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchPolicy {
    pub model: ModelPolicy,
    pub search: SearchPolicy,
    pub evidence: EvidencePolicy,
    pub output: OutputPolicy,
    pub execution: ExecutionPolicy,
}

/// One runnable research aspect.
///
/// Aspects are the unit of parallelism for deep research: each aspect runs
/// inside its own agent loop with its own limits, tool allowlist, and provider
/// selection. Layer 1 (the Claude Code skill) constructs one `AspectRequest`
/// per dimension of the user's question.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AspectRequest {
    /// Stable identifier used by the orchestrator to track per-aspect state
    /// (limits, evidence namespacing, failure records).
    pub id: String,
    /// Human-readable aspect name surfaced to the model and final report.
    pub name: String,
    /// Short role description (e.g. "competitive landscape analyst") used in
    /// the aspect agent's system prompt.
    pub role: String,
    /// Concrete research question this aspect must answer.
    pub question: String,
    /// Topical scope guides for the aspect agent (in-scope topics).
    pub scope: Vec<String>,
    /// Explicit out-of-scope boundaries.
    pub boundaries: Vec<String>,
    /// Acceptance criteria the aspect's findings must meet before completion.
    pub success_criteria: Vec<String>,
    /// Layer 2 aspect-agent system prompt content, provided inline by Layer 1.
    ///
    /// Rust core never performs prompt file IO; Layer 1 (the Claude Code skill)
    /// owns prompt selection, version pinning, and substitution, then passes
    /// the resolved Markdown verbatim as this field. Validation requires a
    /// non-empty string under `ASPECT_PROMPT_MAX_BYTES` (64 KiB) to guard
    /// against accidental payload bloat.
    pub instructions: String,
    /// Tools the aspect agent is allowed to call (currently only `search`).
    pub tools: Vec<ToolName>,
    /// Explicit model provider selection; must satisfy `ResearchPolicy.model`.
    pub model_provider: String,
    /// Explicit search provider selection (exactly one when `tools` includes
    /// `search`); must satisfy `ResearchPolicy.search`.
    pub search_provider: Option<String>,
    /// Per-aspect resource and timeout limits.
    pub limits: AgentLimits,
}

pub(crate) const SUPPORTED_SCHEMA_VERSIONS: &[&str] = &["0.2"];

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchTask {
    pub question: String,
    pub aspects: Vec<AspectRequest>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AspectResearchRequest {
    pub schema_version: String,
    pub request_id: String,
    pub task: AspectRequest,
    pub policy: ResearchPolicy,
    pub context: ResearchContext,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeepResearchRequest {
    pub schema_version: String,
    pub request_id: String,
    pub task: ResearchTask,
    pub limits: ResearchLimits,
    pub policy: ResearchPolicy,
    pub context: ResearchContext,
}

/// Read-only request identity for `get_runtime_capabilities`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeCapabilitiesRequest {
    pub schema_version: String,
    pub request_id: String,
}

/// Live provider names and operator limit ceilings exposed by the MCP server.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeCapabilities {
    pub model_providers: Vec<String>,
    pub search_providers: Vec<String>,
    pub operator_limits: BudgetConfig,
}
