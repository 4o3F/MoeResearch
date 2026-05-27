//! Lapis-core error type and conversions to the public MCP error envelope.
//!
//! `Error` is the single error type returned by all internal APIs. The mapping
//! into the public `ToolError` payload is intentionally narrow: detailed
//! context (paths, raw bodies, header values) stays in `tracing` records, while
//! the `ToolError.message` field carries a stable, redacted summary suitable
//! for external clients.

use std::path::PathBuf;

use snafu::Snafu;

use crate::schema::mcp::{ToolError, ToolErrorCode};
use crate::schema::report::AspectFailure;

/// Alias for `Result<T, Error>` used throughout lapis-core.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Catch-all error type for lapis-core.
///
/// Every variant maps to exactly one `ToolErrorCode` via `Error::code`. New
/// variants MUST update both `code()` and `public_message()`. Variants are
/// not exhaustive in the public contract; only their `ToolErrorCode` is
/// observable from outside.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// Caller supplied input that failed structural or semantic validation
    /// before any provider was contacted. Maps to `ToolErrorCode::InvalidInput`.
    /// Not retryable.
    #[snafu(display("invalid input: {message}"))]
    InvalidInput { message: String },

    /// Configuration is invalid (missing keys, malformed value, etc.).
    /// Maps to `ToolErrorCode::ConfigInvalid`. Not retryable.
    #[snafu(display("configuration is invalid: {message}"))]
    ConfigInvalid { message: String },

    /// I/O failure while reading a configuration file. Maps to
    /// `ToolErrorCode::ConfigInvalid`. Not retryable.
    #[snafu(display("configuration I/O failed for {}: {source}", path.display()))]
    ConfigIo {
        path: PathBuf,
        source: std::io::Error,
    },

    /// TOML parse failure while loading a configuration file. Maps to
    /// `ToolErrorCode::ConfigInvalid`. Not retryable.
    #[snafu(display("configuration parse failed for {}: {source}", path.display()))]
    ConfigParse {
        path: PathBuf,
        source: toml::de::Error,
    },

    /// Provider rejected the request or is otherwise unreachable.
    /// Maps to `ToolErrorCode::ProviderUnavailable`. Not retryable by default.
    #[snafu(display("provider unavailable: {provider}: {message}"))]
    ProviderUnavailable { provider: String, message: String },

    /// Transient network failure unrelated to a specific HTTP status code.
    /// Maps to `ToolErrorCode::NetworkFailed`. Retryable.
    #[snafu(display("network failed: {message}"))]
    NetworkFailed { message: String },

    /// HTTP transport error reported by the network client. Maps to
    /// `ToolErrorCode::NetworkFailed`. Retryability is determined by the
    /// `retryable` field (e.g. timeouts and connection resets are retryable;
    /// TLS handshake failures usually are not).
    #[snafu(display("HTTP transport failed: {message}"))]
    HttpTransport { message: String, retryable: bool },

    /// Non-2xx HTTP status returned by a provider. Maps to
    /// `ToolErrorCode::NetworkFailed`. Retryability is determined by the
    /// `retryable` field (typically 5xx and 429 are retryable).
    #[snafu(display("HTTP status {status}: {message}"))]
    HttpStatus {
        status: u16,
        message: String,
        retryable: bool,
    },

    /// Research budget (model calls, search calls, tokens, timeout) exhausted.
    /// Maps to `ToolErrorCode::BudgetExceeded`. Not retryable without raising
    /// the budget.
    #[snafu(display("budget exceeded: {message}"))]
    BudgetExceeded { message: String },

    /// Tool policy denied a model-requested tool call (unknown tool, duplicate
    /// tool call id, denied by policy). Maps to `ToolErrorCode::ToolPolicyDenied`.
    /// Not retryable.
    #[snafu(display("tool policy denied: {message}"))]
    ToolPolicyDenied { message: String },

    /// Caller supplied a `schema_version` value not in the supported list
    /// (see `orchestrator::workflow::SUPPORTED_SCHEMA_VERSIONS`).
    /// Maps to `ToolErrorCode::UnsupportedSchemaVersion`. Not retryable.
    #[snafu(display("unsupported schema version: {version}"))]
    UnsupportedSchemaVersion { version: String },

    /// Schema validation failed for a provider response or for an internal
    /// payload. Maps to `ToolErrorCode::SchemaValidationFailed`. Not retryable.
    #[snafu(display("schema validation failed: {message}"))]
    SchemaValidationFailed { message: String },

    /// Operation exceeded its configured timeout. Maps to
    /// `ToolErrorCode::Timeout`. Retryable.
    #[snafu(display("operation timed out: {message}"))]
    Timeout { message: String },

    /// One or more aspects failed but at least one succeeded. Maps to
    /// `ToolErrorCode::PartialResult`. Not retryable as an error path; this
    /// variant is mainly an internal signal.
    #[snafu(display("partial result: {message}"))]
    PartialResult { message: String },

    /// JSON (de)serialization failed. Maps to
    /// `ToolErrorCode::SchemaValidationFailed`. Not retryable.
    #[snafu(display("JSON conversion failed: {source}"))]
    Json { source: serde_json::Error },

    /// Logging subsystem failed to initialize. Maps to
    /// `ToolErrorCode::Internal`. Not retryable.
    #[snafu(display("logging initialization failed: {message}"))]
    LoggingInit { message: String },

    /// Catch-all internal error indicating a Lapis bug. Maps to
    /// `ToolErrorCode::Internal`. Not retryable.
    #[snafu(display("internal error: {message}"))]
    Internal { message: String },
}

impl Error {
    /// Maps this error to its stable public `ToolErrorCode`.
    ///
    /// The returned code is part of the MCP contract; never derive it from
    /// `Debug` output. Every new `Error` variant MUST be covered here.
    #[must_use]
    pub fn code(&self) -> ToolErrorCode {
        match self {
            Self::InvalidInput { .. } => ToolErrorCode::InvalidInput,
            Self::ConfigInvalid { .. } | Self::ConfigIo { .. } | Self::ConfigParse { .. } => {
                ToolErrorCode::ConfigInvalid
            }
            Self::ProviderUnavailable { .. } => ToolErrorCode::ProviderUnavailable,
            Self::NetworkFailed { .. } | Self::HttpTransport { .. } | Self::HttpStatus { .. } => {
                ToolErrorCode::NetworkFailed
            }
            Self::BudgetExceeded { .. } => ToolErrorCode::BudgetExceeded,
            Self::ToolPolicyDenied { .. } => ToolErrorCode::ToolPolicyDenied,
            Self::UnsupportedSchemaVersion { .. } => ToolErrorCode::UnsupportedSchemaVersion,
            Self::SchemaValidationFailed { .. } | Self::Json { .. } => {
                ToolErrorCode::SchemaValidationFailed
            }
            Self::Timeout { .. } => ToolErrorCode::Timeout,
            Self::PartialResult { .. } => ToolErrorCode::PartialResult,
            Self::LoggingInit { .. } | Self::Internal { .. } => ToolErrorCode::Internal,
        }
    }

    /// Returns whether a caller can retry the same request and reasonably
    /// expect a different outcome.
    ///
    /// HTTP-level errors honor their explicit `retryable` field; all other
    /// variants are derived from `ToolErrorCode` (network and timeout are
    /// retryable; everything else is not).
    #[must_use]
    pub fn retryable(&self) -> bool {
        match self {
            Self::HttpTransport { retryable, .. } | Self::HttpStatus { retryable, .. } => {
                *retryable
            }
            _ => matches!(
                self.code(),
                ToolErrorCode::NetworkFailed | ToolErrorCode::Timeout
            ),
        }
    }

    /// Returns a stable, redacted, user-facing summary suitable for the
    /// `ToolError.message` field of an MCP envelope.
    ///
    /// Detailed context (file paths, raw provider bodies, header values, etc.)
    /// MUST be emitted through `tracing` instead, never through this string.
    /// Adding a new variant requires extending this match arm so the public
    /// summary stays curated.
    #[must_use]
    pub fn public_message(&self) -> String {
        match self {
            Self::InvalidInput { .. } => "invalid input".to_owned(),
            Self::ConfigInvalid { .. } | Self::ConfigIo { .. } | Self::ConfigParse { .. } => {
                "configuration is invalid".to_owned()
            }
            Self::ProviderUnavailable { .. } => "provider unavailable".to_owned(),
            Self::NetworkFailed { .. } | Self::HttpTransport { .. } => {
                "network request failed".to_owned()
            }
            Self::HttpStatus { status, .. } => {
                format!("provider returned HTTP status {status}")
            }
            Self::BudgetExceeded { message } => message.clone(),
            Self::ToolPolicyDenied { .. } => "tool policy denied request".to_owned(),
            Self::UnsupportedSchemaVersion { .. } => "unsupported schema version".to_owned(),
            Self::SchemaValidationFailed { .. } | Self::Json { .. } => {
                "schema validation failed".to_owned()
            }
            Self::Timeout { .. } => "operation timed out".to_owned(),
            Self::PartialResult { .. } => "partial result".to_owned(),
            Self::LoggingInit { .. } | Self::Internal { .. } => "internal error".to_owned(),
        }
    }

    /// Builds the public `ToolError` payload with optional aspect attribution
    /// and terminal deep-research aspect diagnostics.
    #[must_use]
    pub fn to_tool_error(
        &self,
        aspect_id: Option<String>,
        failed_aspects: Vec<AspectFailure>,
    ) -> ToolError {
        ToolError {
            code: self.code(),
            message: self.public_message(),
            aspect_id,
            retryable: self.tool_error_retryable(&failed_aspects),
            failed_aspects,
        }
    }

    fn tool_error_retryable(&self, failed_aspects: &[AspectFailure]) -> bool {
        if self.code() == ToolErrorCode::PartialResult && !failed_aspects.is_empty() {
            failed_aspects.iter().all(|failure| failure.retryable)
        } else {
            self.retryable()
        }
    }
}
