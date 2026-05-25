//! MCP envelope and error contract shared by every Lapis tool.
//!
//! The shapes in this module form the public boundary between Lapis Rust core
//! and external Layer 1 callers (the Claude Code skill or any MCP client).
//! Field names, presence rules, and enum discriminants are part of the contract
//! documented in `docs/research-agent-product.md` §10.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Stable MCP response envelope returned by every Lapis tool.
///
/// The shape is part of the public contract documented in
/// `docs/research-agent-product.md` §10.1. Every field is always serialized:
/// optional fields are emitted as JSON `null` rather than omitted, so external
/// clients can rely on a fixed key set. Golden tests in
/// `crates/lapis-tests/tests/mcp_tests.rs` pin this behavior.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ToolEnvelope<T> {
    /// Schema version negotiated with the caller; echoed verbatim so clients
    /// can route the response based on their own version compatibility table.
    pub schema_version: String,
    /// Caller-supplied correlation id; echoed verbatim for client-side tracing.
    pub request_id: String,
    /// Run id assigned by the orchestrator for multi-aspect workflows.
    ///
    /// `None` for failed envelopes and for single-aspect tools that do not
    /// allocate a run id. Always serialized (as JSON `null` when absent).
    #[serde(default)]
    pub run_id: Option<String>,
    /// Outcome status: `ok`, `partial`, or `failed`.
    pub status: ToolStatus,
    /// Business payload on success or partial success; `None` on `failed`.
    ///
    /// Always serialized (as JSON `null` when absent) to keep the key set
    /// stable across all envelope variants.
    #[serde(default)]
    pub data: Option<T>,
    /// Structured error; `None` on `ok`.
    ///
    /// Always serialized (as JSON `null` when absent) for the same reason as
    /// `data`.
    #[serde(default)]
    pub error: Option<ToolError>,
}

/// Outcome status of an MCP tool invocation.
///
/// Serializes as `snake_case` (`"ok"`, `"partial"`, `"failed"`). The variant
/// set is part of the public contract; adding or renaming a variant is a
/// breaking change for downstream clients.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    /// All aspects completed successfully.
    Ok,
    /// At least one aspect succeeded and at least one failed; `data` is still
    /// populated with the surviving partial result.
    Partial,
    /// The tool call could not be completed; `data` is `None` and `error`
    /// carries the failure reason.
    Failed,
}

/// Public-facing error payload for the MCP envelope.
///
/// `message` is intentionally a stable, redacted summary; detailed context
/// (file paths, raw provider bodies, header values) MUST be emitted through
/// `tracing` instead of into this field, to avoid leaking secrets or host
/// implementation details to external clients.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ToolError {
    /// Stable `snake_case` error code suitable for client-side dispatch.
    pub code: ToolErrorCode,
    /// Generic user-facing summary. Never contains secrets, paths, or raw
    /// provider responses.
    pub message: String,
    /// Aspect identifier when the failure can be attributed to a single
    /// aspect (e.g. `aspect_research` failures). `None` for top-level
    /// deep-research failures that span multiple aspects.
    pub aspect_id: Option<String>,
    /// Whether the caller can retry the same request with a reasonable
    /// expectation of a different outcome (e.g. transient network errors).
    pub retryable: bool,
}

/// Stable, client-visible error code namespace.
///
/// Serializes as `snake_case`. The set is closed: adding a new variant is a
/// breaking change for clients that exhaustively match on the value.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolErrorCode {
    /// Caller supplied a request that failed structural or semantic
    /// validation before any provider was contacted. Not retryable.
    InvalidInput,
    /// Caller supplied a `schema_version` that this Rust core does not accept.
    /// See `crates/lapis-core/src/orchestrator/workflow.rs::SUPPORTED_SCHEMA_VERSIONS`.
    /// Not retryable.
    UnsupportedSchemaVersion,
    /// Server-side configuration is invalid (missing API key, unknown provider,
    /// malformed TOML). Not retryable.
    ConfigInvalid,
    /// Provider rejected the request or is not configured/allowed. Not
    /// retryable under the current public error mapping; transient network
    /// failures are reported as `NetworkFailed` or `Timeout` instead.
    ProviderUnavailable,
    /// Transient network condition (connection reset, DNS failure, etc.).
    /// Retryable.
    NetworkFailed,
    /// Research budget (model calls, search calls, tokens, timeout) was
    /// exhausted before the workflow completed. Not retryable without
    /// raising the budget.
    BudgetExceeded,
    /// Tool policy denied a model-requested tool call (e.g. a tool that was
    /// not in the aspect's `allowed_tools`). Not retryable.
    ToolPolicyDenied,
    /// Provider response failed JSON schema validation. Not retryable.
    SchemaValidationFailed,
    /// Operation exceeded its configured timeout. Retryable.
    Timeout,
    /// Partial-result condition. Reported either inside a `partial` success
    /// envelope (as an embedded aspect failure) or as a `failed` envelope's
    /// top-level code when partial results are disabled by execution policy
    /// or no aspect succeeded.
    PartialResult,
    /// Catch-all internal error. Not retryable; indicates a Lapis bug.
    Internal,
}

impl ToolErrorCode {
    /// Returns the stable `snake_case` identifier for this error code.
    ///
    /// The returned `&'static str` matches the Serde representation produced
    /// by `#[serde(rename_all = "snake_case")]`. Use this helper anywhere an
    /// error code must appear as a plain string (e.g. `AspectFailure.error_code`,
    /// `tracing` fields) to keep `PascalCase` `Debug` output out of public
    /// APIs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::UnsupportedSchemaVersion => "unsupported_schema_version",
            Self::ConfigInvalid => "config_invalid",
            Self::ProviderUnavailable => "provider_unavailable",
            Self::NetworkFailed => "network_failed",
            Self::BudgetExceeded => "budget_exceeded",
            Self::ToolPolicyDenied => "tool_policy_denied",
            Self::SchemaValidationFailed => "schema_validation_failed",
            Self::Timeout => "timeout",
            Self::PartialResult => "partial_result",
            Self::Internal => "internal",
        }
    }
}
