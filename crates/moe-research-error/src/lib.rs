//! Transport-neutral MoeResearch error API.
//!
//! `Error::public_message` is intentionally narrow: raw provider bodies, host
//! paths, header values, and secrets stay in `tracing` records.
//! `SchemaValidationFailed.message` is curated validator output and may carry
//! the offending validation code, JSON path, and human-readable diagnostic.

use std::path::PathBuf;

use snafu::Snafu;

/// Alias for `Result<T, Error>` used throughout MoeResearch crates.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Transport-neutral error classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidInput,
    UnsupportedSchemaVersion,
    ConfigInvalid,
    ProviderUnavailable,
    NetworkFailed,
    BudgetExceeded,
    ToolPolicyDenied,
    SchemaValidationFailed,
    Timeout,
    PartialResult,
    Internal,
}

impl ErrorCode {
    /// Returns the stable `snake_case` identifier for this error code.
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

/// Catch-all error type for internal MoeResearch APIs.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("invalid input: {message}"))]
    InvalidInput { message: String },

    #[snafu(display("configuration is invalid: {message}"))]
    ConfigInvalid { message: String },

    #[snafu(display("configuration I/O failed for {}: {source}", path.display()))]
    ConfigIo {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("configuration parse failed for {}: {source}", path.display()))]
    ConfigParse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[snafu(display("provider unavailable: {provider}: {message}"))]
    ProviderUnavailable { provider: String, message: String },

    #[snafu(display("network failed: {message}"))]
    NetworkFailed { message: String },

    #[snafu(display("HTTP transport failed: {message}"))]
    HttpTransport { message: String, retryable: bool },

    #[snafu(display("HTTP status {status}: {message}"))]
    HttpStatus {
        status: u16,
        message: String,
        retryable: bool,
    },

    #[snafu(display("budget exceeded: {message}"))]
    BudgetExceeded { message: String },

    #[snafu(display("tool policy denied: {message}"))]
    ToolPolicyDenied { message: String },

    #[snafu(display("unsupported schema version: {version}"))]
    UnsupportedSchemaVersion { version: String },

    /// The message is exposed in public transport envelopes, so keep it public-safe.
    #[snafu(display("schema validation failed: {message}"))]
    SchemaValidationFailed { message: String },

    #[snafu(display("operation timed out: {message}"))]
    Timeout { message: String },

    #[snafu(display("partial result: {message}"))]
    PartialResult { message: String },

    #[snafu(display("JSON conversion failed: {source}"))]
    Json { source: serde_json::Error },

    #[snafu(display("logging initialization failed: {message}"))]
    LoggingInit { message: String },

    #[snafu(display("internal error: {message}"))]
    Internal { message: String },
}

impl Error {
    /// Maps this error to its stable transport-neutral `ErrorCode`.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidInput { .. } => ErrorCode::InvalidInput,
            Self::ConfigInvalid { .. } | Self::ConfigIo { .. } | Self::ConfigParse { .. } => {
                ErrorCode::ConfigInvalid
            }
            Self::ProviderUnavailable { .. } => ErrorCode::ProviderUnavailable,
            Self::NetworkFailed { .. } | Self::HttpTransport { .. } | Self::HttpStatus { .. } => {
                ErrorCode::NetworkFailed
            }
            Self::BudgetExceeded { .. } => ErrorCode::BudgetExceeded,
            Self::ToolPolicyDenied { .. } => ErrorCode::ToolPolicyDenied,
            Self::UnsupportedSchemaVersion { .. } => ErrorCode::UnsupportedSchemaVersion,
            Self::SchemaValidationFailed { .. } | Self::Json { .. } => {
                ErrorCode::SchemaValidationFailed
            }
            Self::Timeout { .. } => ErrorCode::Timeout,
            Self::PartialResult { .. } => ErrorCode::PartialResult,
            Self::LoggingInit { .. } | Self::Internal { .. } => ErrorCode::Internal,
        }
    }

    /// Returns whether a caller can retry the same request and reasonably expect a different outcome.
    #[must_use]
    pub fn retryable(&self) -> bool {
        match self {
            Self::HttpTransport { retryable, .. } | Self::HttpStatus { retryable, .. } => {
                *retryable
            }
            _ => matches!(self.code(), ErrorCode::NetworkFailed | ErrorCode::Timeout),
        }
    }

    /// Returns a public-safe, user-facing message suitable for public transport envelopes.
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
            Self::HttpStatus { status, .. } => format!("provider returned HTTP status {status}"),
            Self::BudgetExceeded { message } => message.clone(),
            Self::ToolPolicyDenied { .. } => "tool policy denied request".to_owned(),
            Self::UnsupportedSchemaVersion { .. } => "unsupported schema version".to_owned(),
            Self::SchemaValidationFailed { message } => message.clone(),
            Self::Json { .. } => "schema validation failed".to_owned(),
            Self::Timeout { .. } => "operation timed out".to_owned(),
            Self::PartialResult { .. } => "partial result".to_owned(),
            Self::LoggingInit { .. } | Self::Internal { .. } => "internal error".to_owned(),
        }
    }
}
