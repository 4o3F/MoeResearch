use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::report::PartialTrace;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ToolEnvelope<T> {
    pub schema_version: String,
    pub request_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub status: ToolStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ToolError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_trace: Option<PartialTrace>,
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
