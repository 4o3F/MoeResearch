use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ModelRequest {
    pub provider: String,
    pub model: Option<String>,
    pub messages: Vec<ModelMessage>,
    pub tools: Vec<ModelTool>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl ModelRequest {
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.messages.is_empty() {
            return Err(crate::error::Error::SchemaValidationFailed {
                message: "model messages must not be empty".to_string(),
            });
        }

        if self.tools.iter().any(|tool| tool.name.trim().is_empty()) {
            return Err(crate::error::Error::SchemaValidationFailed {
                message: "model tool names must not be empty".to_string(),
            });
        }

        if let Some(temperature) = self.temperature
            && (!temperature.is_finite() || !(0.0..=2.0).contains(&temperature))
        {
            return Err(crate::error::Error::SchemaValidationFailed {
                message: "model temperature must be finite and between 0.0 and 2.0".to_string(),
            });
        }

        if self.max_tokens == Some(0) {
            return Err(crate::error::Error::SchemaValidationFailed {
                message: "model max_tokens must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelMessageRole {
    System,
    User,
    Assistant,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ModelMessage {
    pub role: ModelMessageRole,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ModelTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ModelResponse {
    pub provider: String,
    pub model: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Vec<ModelToolCall>,
    pub usage: Option<super::report::TokenUsage>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ModelToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}
