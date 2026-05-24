use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.input.is_empty() {
            return Err(crate::error::Error::SchemaValidationFailed {
                message: "model input must not be empty".to_string(),
            });
        }

        if self
            .previous_response_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(crate::error::Error::SchemaValidationFailed {
                message: "model previous_response_id must not be empty".to_string(),
            });
        }

        for item in &self.input {
            match item {
                ModelInputItem::Message(message) if message.content.trim().is_empty() => {
                    return Err(crate::error::Error::SchemaValidationFailed {
                        message: "model message content must not be empty".to_string(),
                    });
                }
                ModelInputItem::ToolCall(call) if call.id.trim().is_empty() => {
                    return Err(crate::error::Error::SchemaValidationFailed {
                        message: "model tool call id must not be empty".to_string(),
                    });
                }
                ModelInputItem::ToolCall(call) if call.name.trim().is_empty() => {
                    return Err(crate::error::Error::SchemaValidationFailed {
                        message: "model tool call name must not be empty".to_string(),
                    });
                }
                ModelInputItem::ToolOutput(output) if output.call_id.trim().is_empty() => {
                    return Err(crate::error::Error::SchemaValidationFailed {
                        message: "model tool output call_id must not be empty".to_string(),
                    });
                }
                _ => {}
            }
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
