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
        let body = serde_json::to_value(OpenAiResponsesRequest {
            model: request.model.unwrap_or_else(|| "gpt-4o-mini".to_owned()),
            input: request
                .messages
                .into_iter()
                .map(|message| OpenAiInputMessage {
                    role: map_role(message.role),
                    content: message.content,
                })
                .collect::<Vec<_>>(),
            tools: request.tools.into_iter().map(map_tool).collect::<Vec<_>>(),
            temperature: request.temperature,
            max_output_tokens: request.max_tokens,
        })
        .context(JsonSnafu)?;

        Ok(NetworkRequest {
            method: "POST".to_owned(),
            url: format!("{}/responses", self.base_url.trim_end_matches('/')),
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
        let provider_response: OpenAiResponsesResponse =
            serde_json::from_value(body).context(JsonSnafu)?;
        let mut content = Vec::new();
        let mut tool_calls = Vec::new();

        for output in provider_response.output {
            match output {
                OpenAiResponseOutput::Message { content: items, .. } => {
                    content.extend(items.into_iter().map(|item| match item {
                        OpenAiResponseContent::OutputText { text } => text,
                    }));
                }
                OpenAiResponseOutput::FunctionCall {
                    call_id,
                    name,
                    arguments,
                    ..
                } => tool_calls.push(map_tool_call(call_id, name, &arguments)?),
            }
        }

        Ok(ModelResponse {
            provider: self.name().to_owned(),
            model: provider_response.model,
            content: if content.is_empty() {
                None
            } else {
                Some(content.join("\n"))
            },
            tool_calls,
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
        name: tool.name,
        description: tool.description,
        parameters: tool.input_schema,
    }
}

fn map_tool_call(call_id: String, name: String, arguments: &str) -> Result<ModelToolCall> {
    Ok(ModelToolCall {
        id: call_id,
        name,
        arguments: serde_json::from_str(arguments).context(JsonSnafu)?,
    })
}

fn map_usage(usage: &OpenAiUsage) -> TokenUsage {
    TokenUsage {
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
        total_tokens: usage.total_tokens,
    }
}

#[derive(Serialize)]
struct OpenAiResponsesRequest {
    model: String,
    input: Vec<OpenAiInputMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<OpenAiTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Serialize)]
struct OpenAiInputMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct OpenAiTool {
    #[serde(rename = "type")]
    tool_type: &'static str,
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct OpenAiResponsesResponse {
    model: Option<String>,
    #[serde(default)]
    output: Vec<OpenAiResponseOutput>,
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum OpenAiResponseOutput {
    Message {
        #[serde(default)]
        content: Vec<OpenAiResponseContent>,
    },
    FunctionCall {
        call_id: String,
        name: String,
        arguments: String,
    },
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum OpenAiResponseContent {
    OutputText { text: String },
}

#[derive(Deserialize)]
struct OpenAiUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    total_tokens: Option<u64>,
}
