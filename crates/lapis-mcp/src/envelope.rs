use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use lapis_workflow::AspectFailure;

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

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ToolError {
    pub code: ToolErrorCode,
    pub message: String,
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

impl ToolErrorCode {
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
