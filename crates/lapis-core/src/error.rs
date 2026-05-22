use std::path::PathBuf;

use snafu::Snafu;

use crate::schema::mcp::{ToolError, ToolErrorCode};

pub type Result<T, E = Error> = std::result::Result<T, E>;

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
            Self::SchemaValidationFailed { .. } | Self::Json { .. } => {
                ToolErrorCode::SchemaValidationFailed
            }
            Self::Timeout { .. } => ToolErrorCode::Timeout,
            Self::PartialResult { .. } => ToolErrorCode::PartialResult,
            Self::LoggingInit { .. } | Self::Internal { .. } => ToolErrorCode::Internal,
        }
    }

    pub fn to_tool_error(&self) -> ToolError {
        ToolError {
            code: self.code(),
            message: self.to_string(),
            aspect_id: None,
            retryable: match self {
                Self::HttpTransport { retryable, .. } | Self::HttpStatus { retryable, .. } => {
                    *retryable
                }
                _ => matches!(
                    self.code(),
                    ToolErrorCode::NetworkFailed | ToolErrorCode::Timeout
                ),
            },
        }
    }
}
