use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ModelRequest {
    pub provider: String,
    pub model: Option<String>,
    pub previous_response_id: Option<String>,
    pub input: Vec<ModelInputItem>,
    pub tools: Vec<ModelTool>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl ModelRequest {
    pub fn validate(&self) -> Result<()> {
        if self.input.is_empty() {
            return Err(schema_error("model input must not be empty"));
        }

        if self
            .previous_response_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(schema_error("model previous_response_id must not be empty"));
        }

        for item in &self.input {
            match item {
                ModelInputItem::Message(message) => {
                    ensure_non_empty_str(&message.content, "model message content")?;
                }
                ModelInputItem::ToolCall(call) => {
                    ensure_non_empty_str(&call.id, "model tool call id")?;
                    ensure_non_empty_str(&call.name, "model tool call name")?;
                }
                ModelInputItem::ToolOutput(output) => {
                    ensure_non_empty_str(&output.call_id, "model tool output call_id")?;
                }
            }
        }

        for tool in &self.tools {
            ensure_non_empty_str(&tool.name, "model tool names")?;
        }

        if let Some(temperature) = self.temperature
            && (!temperature.is_finite() || !(0.0..=2.0).contains(&temperature))
        {
            return Err(schema_error(
                "model temperature must be finite and between 0.0 and 2.0",
            ));
        }

        if self.max_tokens == Some(0) {
            return Err(schema_error("model max_tokens must be greater than 0"));
        }

        Ok(())
    }
}

fn ensure_non_empty_str(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(schema_error(&format!("{label} must not be empty")));
    }
    Ok(())
}

fn schema_error(message: &str) -> Error {
    Error::SchemaValidationFailed {
        message: message.to_owned(),
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelInputItem {
    Message(ModelMessage),
    ToolCall(ModelToolCall),
    ToolOutput(ModelToolOutput),
}

impl ModelInputItem {
    #[must_use]
    pub fn message(role: ModelMessageRole, content: impl Into<String>) -> Self {
        Self::Message(ModelMessage {
            role,
            content: content.into(),
        })
    }

    #[must_use]
    pub fn tool_call(call: ModelToolCall) -> Self {
        Self::ToolCall(call)
    }

    #[must_use]
    pub fn tool_output(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self::ToolOutput(ModelToolOutput::new(call_id, output))
    }
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
    pub response_id: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Vec<ModelToolCall>,
    pub output_items: Vec<ModelInputItem>,
    pub usage: Option<super::report::TokenUsage>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ModelToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct ModelToolOutput {
    pub call_id: String,
    pub output: String,
}

impl ModelToolOutput {
    #[must_use]
    pub fn new(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            output: output.into(),
        }
    }
}
