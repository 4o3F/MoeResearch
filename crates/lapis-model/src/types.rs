use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use lapis_error::{Error, Result};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ModelRequest {
    pub provider: String,
    pub model: Option<String>,
    pub previous_response_id: Option<String>,
    pub input: Vec<ModelInputItem>,
    pub tools: Vec<ModelTool>,
    pub response_format: Option<ModelResponseFormat>,
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

        if let Some(ModelResponseFormat::JsonSchema(format)) = &self.response_format {
            ensure_non_empty_str(&format.name, "model response format name")?;
            if !format.schema.is_object() {
                return Err(schema_error("model response JSON schema must be an object"));
            }
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

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

impl TokenUsage {
    #[must_use]
    pub fn zero() -> Self {
        Self {
            input_tokens: None,
            output_tokens: None,
            total_tokens: None,
        }
    }

    #[must_use]
    pub fn total_or_sum(&self) -> Option<u64> {
        self.total_tokens.or_else(|| {
            self.input_tokens
                .zip(self.output_tokens)
                .map(|(i, o)| i.saturating_add(o))
        })
    }

    #[must_use]
    pub fn merge(left: Option<Self>, right: Option<Self>) -> Option<Self> {
        match (left, right) {
            (None, None) => None,
            (Some(usage), None) | (None, Some(usage)) => Some(usage),
            (Some(left), Some(right)) => {
                let total_tokens = sum_optional_tokens(left.total_or_sum(), right.total_or_sum());
                Some(Self {
                    input_tokens: sum_optional_tokens(left.input_tokens, right.input_tokens),
                    output_tokens: sum_optional_tokens(left.output_tokens, right.output_tokens),
                    total_tokens,
                })
            }
        }
    }
}

fn sum_optional_tokens(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (None, None) => None,
        (Some(value), None) | (None, Some(value)) => Some(value),
        (Some(left), Some(right)) => Some(left.saturating_add(right)),
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelResponseFormat {
    Text,
    JsonSchema(JsonSchemaFormat),
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct JsonSchemaFormat {
    pub name: String,
    pub strict: bool,
    pub schema: Value,
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
    pub usage: Option<TokenUsage>,
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
