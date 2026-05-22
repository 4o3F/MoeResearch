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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use super::*;
    use crate::net::client::MockNetworkClient;
    use crate::schema::common::NetworkResponse;
    use crate::schema::model::{ModelMessage, ModelMessageRole};

    #[tokio::test]
    async fn maps_text_response_and_usage() {
        let network = Arc::new(MockNetworkClient::new([NetworkResponse {
            status: 200,
            headers: vec![],
            body: json!({
                "model": "gpt-test",
                "choices": [{
                    "message": {
                        "content": "hello"
                    }
                }],
                "usage": {
                    "prompt_tokens": 3,
                    "completion_tokens": 5,
                    "total_tokens": 8
                }
            }),
        }]));
        let provider = provider(network);

        let response = provider
            .complete(request_with_messages(vec![user_message("hi")]))
            .await
            .expect("model response");

        assert_eq!(response.provider, "openai-compatible");
        assert_eq!(response.model, Some("gpt-test".to_owned()));
        assert_eq!(response.content, Some("hello".to_owned()));
        let usage = response.usage.expect("usage");
        assert_eq!(usage.input_tokens, Some(3));
        assert_eq!(usage.output_tokens, Some(5));
        assert_eq!(usage.total_tokens, Some(8));
    }

    #[tokio::test]
    async fn maps_tool_call_only_response_with_parsed_arguments() {
        let network = Arc::new(MockNetworkClient::new([NetworkResponse {
            status: 200,
            headers: vec![],
            body: json!({
                "choices": [{
                    "message": {
                        "content": null,
                        "tool_calls": [{
                            "id": "call_1",
                            "function": {
                                "name": "search",
                                "arguments": "{\"query\":\"lapis\"}"
                            }
                        }]
                    }
                }]
            }),
        }]));
        let provider = provider(network);

        let response = provider
            .complete(request_with_messages(vec![user_message("search")]))
            .await
            .expect("tool call response");

        assert_eq!(response.content, None);
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].id, "call_1");
        assert_eq!(response.tool_calls[0].name, "search");
        assert_eq!(
            response.tool_calls[0].arguments,
            json!({ "query": "lapis" })
        );
    }

    #[tokio::test]
    async fn missing_usage_maps_to_none() {
        let network = Arc::new(MockNetworkClient::new([NetworkResponse {
            status: 200,
            headers: vec![],
            body: json!({
                "choices": [{
                    "message": {
                        "content": "hello"
                    }
                }]
            }),
        }]));
        let provider = provider(network);

        let response = provider
            .complete(request_with_messages(vec![user_message("hi")]))
            .await
            .expect("model response");

        assert_eq!(response.usage, None);
    }

    #[tokio::test]
    async fn malformed_tool_call_arguments_returns_error() {
        let network = Arc::new(MockNetworkClient::new([NetworkResponse {
            status: 200,
            headers: vec![],
            body: json!({
                "choices": [{
                    "message": {
                        "content": null,
                        "tool_calls": [{
                            "id": "call_1",
                            "function": {
                                "name": "search",
                                "arguments": "{bad"
                            }
                        }]
                    }
                }]
            }),
        }]));
        let provider = provider(network);

        let error = provider
            .complete(request_with_messages(vec![user_message("search")]))
            .await;

        assert!(error.is_err());
    }

    #[tokio::test]
    async fn request_uses_chat_completions_endpoint_and_openai_tool_schema() {
        let network = Arc::new(MockNetworkClient::new([NetworkResponse {
            status: 200,
            headers: vec![],
            body: json!({
                "choices": [{
                    "message": {
                        "content": "ok"
                    }
                }]
            }),
        }]));
        let provider = OpenAiCompatibleProvider::new(
            network.clone(),
            "https://api.example.com/".to_owned(),
            "secret".to_owned(),
            Some(1000),
        );
        let mut request = request_with_messages(vec![user_message("hi")]);
        request.model = None;
        request.tools = vec![ModelTool {
            name: "search".to_owned(),
            description: "Search the web".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }),
        }];
        request.temperature = Some(0.2);
        request.max_tokens = Some(128);

        provider.complete(request).await.expect("model response");

        let requests = network.requests();
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.method, "POST");
        assert_eq!(request.url, "https://api.example.com/chat/completions");
        assert_eq!(request.timeout_ms, Some(1000));
        assert!(
            request.headers.iter().any(|header| {
                header.name == "authorization" && header.value == "Bearer secret"
            })
        );
        assert!(
            request.headers.iter().any(|header| {
                header.name == "content-type" && header.value == "application/json"
            })
        );

        let body = request.body.as_ref().expect("request body");
        assert_eq!(body["model"], "gpt-4o-mini");
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["tools"][0]["type"], "function");
        assert_eq!(body["tools"][0]["function"]["name"], "search");
        assert_eq!(
            body["tools"][0]["function"]["description"],
            "Search the web"
        );
        assert_eq!(body["tools"][0]["function"]["parameters"]["type"], "object");
        let temperature = body["temperature"].as_f64().expect("temperature");
        assert!((temperature - 0.2).abs() < 0.000_001);
        assert_eq!(body["max_tokens"], 128);
    }

    fn provider(network: Arc<MockNetworkClient>) -> OpenAiCompatibleProvider {
        OpenAiCompatibleProvider::new(
            network,
            "https://api.example.com".to_owned(),
            "secret".to_owned(),
            None,
        )
    }

    fn request_with_messages(messages: Vec<ModelMessage>) -> ModelRequest {
        ModelRequest {
            provider: "openai-compatible".to_owned(),
            model: Some("gpt-test".to_owned()),
            messages,
            tools: vec![],
            temperature: None,
            max_tokens: None,
        }
    }

    fn user_message(content: &str) -> ModelMessage {
        ModelMessage {
            role: ModelMessageRole::User,
            content: content.to_owned(),
        }
    }
}
