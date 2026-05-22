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

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_request() -> ModelRequest {
        ModelRequest {
            provider: String::new(),
            model: None,
            messages: vec![ModelMessage {
                role: ModelMessageRole::User,
                content: "hello".to_string(),
            }],
            tools: Vec::new(),
            temperature: None,
            max_tokens: None,
        }
    }

    #[test]
    fn model_message_role_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&ModelMessageRole::System).unwrap(),
            "\"system\""
        );
        assert_eq!(
            serde_json::to_string(&ModelMessageRole::User).unwrap(),
            "\"user\""
        );
        assert_eq!(
            serde_json::from_str::<ModelMessageRole>("\"assistant\"").unwrap(),
            ModelMessageRole::Assistant
        );
    }

    #[test]
    fn validate_accepts_valid_minimal_request() {
        assert!(minimal_request().validate().is_ok());
    }

    #[test]
    fn validate_rejects_invalid_temperature_and_zero_max_tokens() {
        for temperature in [f32::NAN, -0.1, 2.1] {
            let mut request = minimal_request();
            request.temperature = Some(temperature);

            assert!(request.validate().is_err());
        }

        let mut request = minimal_request();
        request.max_tokens = Some(0);

        assert!(request.validate().is_err());
    }
}
