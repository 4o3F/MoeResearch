use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use moe_research_error::ErrorCode;
use moe_research_workflow::{AspectFailure, FailureDiagnostic};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ToolEnvelope<T> {
    pub schema_version: String,
    pub request_id: String,
    #[serde(default)]
    pub run_id: Option<String>,
    pub status: ToolStatus,
    #[serde(default)]
    pub data: Option<T>,
    #[serde(default)]
    pub error: Option<ToolError>,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    Ok,
    Partial,
    Failed,
}

/// Public-facing error payload for the MCP envelope.
///
/// `message` is intentionally public-safe. Most variants use a stable,
/// redacted summary; `SchemaValidationFailed` may include curated validator
/// diagnostics such as issue code, JSON path, and human-readable message.
/// `diagnostic` supplies host-owned execution stage and safe operation
/// ordinals. Raw provider bodies, host file paths, header values, and secrets
/// stay in `tracing`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ToolError {
    pub code: ToolErrorCode,
    /// User-facing message. Never contains secrets, host paths, or raw provider
    /// responses; schema validation failures may include JSON paths.
    pub message: String,
    /// Host-owned stage and safe execution ordinals for this failure.
    pub diagnostic: FailureDiagnostic,
    pub aspect_id: Option<String>,
    pub retryable: bool,
    pub failed_aspects: Vec<AspectFailure>,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolErrorCode {
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

impl From<ErrorCode> for ToolErrorCode {
    fn from(code: ErrorCode) -> Self {
        match code {
            ErrorCode::InvalidInput => Self::InvalidInput,
            ErrorCode::UnsupportedSchemaVersion => Self::UnsupportedSchemaVersion,
            ErrorCode::ConfigInvalid => Self::ConfigInvalid,
            ErrorCode::ProviderUnavailable => Self::ProviderUnavailable,
            ErrorCode::NetworkFailed => Self::NetworkFailed,
            ErrorCode::BudgetExceeded => Self::BudgetExceeded,
            ErrorCode::ToolPolicyDenied => Self::ToolPolicyDenied,
            ErrorCode::SchemaValidationFailed => Self::SchemaValidationFailed,
            ErrorCode::Timeout => Self::Timeout,
            ErrorCode::PartialResult => Self::PartialResult,
            ErrorCode::Internal => Self::Internal,
        }
    }
}

impl ToolErrorCode {
    /// Public snake_case identifier; string literals live only on `ErrorCode::as_str`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => ErrorCode::InvalidInput.as_str(),
            Self::UnsupportedSchemaVersion => ErrorCode::UnsupportedSchemaVersion.as_str(),
            Self::ConfigInvalid => ErrorCode::ConfigInvalid.as_str(),
            Self::ProviderUnavailable => ErrorCode::ProviderUnavailable.as_str(),
            Self::NetworkFailed => ErrorCode::NetworkFailed.as_str(),
            Self::BudgetExceeded => ErrorCode::BudgetExceeded.as_str(),
            Self::ToolPolicyDenied => ErrorCode::ToolPolicyDenied.as_str(),
            Self::SchemaValidationFailed => ErrorCode::SchemaValidationFailed.as_str(),
            Self::Timeout => ErrorCode::Timeout.as_str(),
            Self::PartialResult => ErrorCode::PartialResult.as_str(),
            Self::Internal => ErrorCode::Internal.as_str(),
        }
    }
}
