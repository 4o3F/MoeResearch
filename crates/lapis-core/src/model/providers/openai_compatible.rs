use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::error::{Error, JsonSnafu, Result};
use crate::model::provider::ModelProvider;
use crate::net::NetworkClient;
use crate::schema::common::{Header, NetworkRequest};
use crate::schema::model::{
    ModelMessageRole, ModelRequest, ModelResponse, ModelTool, ModelToolCall,
};
use crate::schema::report::TokenUsage;

pub struct OpenAiCompatibleProvider {
    network: Arc<dyn NetworkClient>,
    base_url: String,
    api_key: String,
    timeout_ms: Option<u64>,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        network: Arc<dyn NetworkClient>,
        base_url: String,
        api_key: String,
        timeout_ms: Option<u64>,
    ) -> Self {
        Self {
            network,
            base_url,
            api_key,
            timeout_ms,
        }
    }

    fn validate_request(&self, request: &ModelRequest) -> Result<()> {
        if self.base_url.trim().is_empty() {
            return Err(Error::ConfigInvalid {
                message: "openai-compatible base_url is empty".to_owned(),
            });
        }

        if request.provider != self.name() {
            return Err(Error::InvalidInput {
                message: format!(
                    "model request provider must be {}, got {}",
                    self.name(),
                    request.provider
                ),
            });
        }

        if request.messages.is_empty() {
            return Err(Error::InvalidInput {
                message: "model request must include at least one message".to_owned(),
            });
        }

        Ok(())
    }

    fn build_network_request(&self, request: ModelRequest) -> Result<NetworkRequest> {
        let body = serde_json::to_value(OpenAiChatCompletionRequest {
            model: request.model.unwrap_or_else(|| "gpt-4o-mini".to_owned()),
            messages: request
                .messages
                .into_iter()
                .map(|message| OpenAiMessage {
                    role: map_role(message.role),
                    content: message.content,
                })
                .collect::<Vec<_>>(),
            tools: request.tools.into_iter().map(map_tool).collect::<Vec<_>>(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        })
        .context(JsonSnafu)?;

        Ok(NetworkRequest {
            method: "POST".to_owned(),
            url: format!("{}/chat/completions", self.base_url.trim_end_matches('/')),
            headers: vec![
                Header {
                    name: "authorization".to_owned(),
                    value: format!("Bearer {}", self.api_key),
                },
                Header {
                    name: "content-type".to_owned(),
                    value: "application/json".to_owned(),
                },
            ],
            body: Some(body),
            timeout_ms: self.timeout_ms,
        })
    }

    fn map_response(&self, body: Value) -> Result<ModelResponse> {
        let provider_response: OpenAiChatCompletionResponse =
            serde_json::from_value(body).context(JsonSnafu)?;
        let choice = provider_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| Error::SchemaValidationFailed {
                message: "openai-compatible response did not include choices".to_owned(),
            })?;

        Ok(ModelResponse {
            provider: self.name().to_owned(),
            model: provider_response.model,
            content: choice.message.content,
            tool_calls: choice
                .message
                .tool_calls
                .into_iter()
                .map(map_tool_call)
                .collect::<Result<Vec<_>>>()?,
            usage: provider_response.usage.as_ref().map(map_usage),
        })
    }
}

#[async_trait]
impl ModelProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &'static str {
        "openai-compatible"
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        self.validate_request(&request)?;
        let network_request = self.build_network_request(request)?;
        let response = self.network.send(network_request).await?;

        if !(200..300).contains(&response.status) {
            return Err(Error::HttpStatus {
                status: response.status,
                message: "openai-compatible model provider returned non-success status".to_owned(),
                retryable: response.status == 429 || response.status >= 500,
            });
        }

        self.map_response(response.body)
    }
}

fn map_role(role: ModelMessageRole) -> &'static str {
    match role {
        ModelMessageRole::System => "system",
        ModelMessageRole::User => "user",
        ModelMessageRole::Assistant => "assistant",
    }
}

fn map_tool(tool: ModelTool) -> OpenAiTool {
    OpenAiTool {
        tool_type: "function",
        function: OpenAiToolFunction {
            name: tool.name,
            description: tool.description,
            parameters: tool.input_schema,
        },
    }
}

fn map_tool_call(tool_call: OpenAiResponseToolCall) -> Result<ModelToolCall> {
    Ok(ModelToolCall {
        id: tool_call.id,
        name: tool_call.function.name,
        arguments: serde_json::from_str(&tool_call.function.arguments).context(JsonSnafu)?,
    })
}

fn map_usage(usage: &OpenAiUsage) -> TokenUsage {
    TokenUsage {
        input_tokens: usage.prompt_tokens,
        output_tokens: usage.completion_tokens,
        total_tokens: usage.total_tokens,
    }
}

#[derive(Serialize)]
struct OpenAiChatCompletionRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<OpenAiTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct OpenAiTool {
    #[serde(rename = "type")]
    tool_type: &'static str,
    function: OpenAiToolFunction,
}

#[derive(Serialize)]
struct OpenAiToolFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct OpenAiChatCompletionResponse {
    model: Option<String>,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}

#[derive(Deserialize)]
struct OpenAiResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<OpenAiResponseToolCall>,
}

#[derive(Deserialize)]
struct OpenAiResponseToolCall {
    id: String,
    function: OpenAiResponseFunctionCall,
}

#[derive(Deserialize)]
struct OpenAiResponseFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}
