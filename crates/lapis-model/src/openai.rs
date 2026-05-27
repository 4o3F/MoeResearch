use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use lapis_error::{Error, JsonSnafu, Result};
use lapis_net::NetworkClient;
use lapis_net::NetworkRequest;
use lapis_net::provider_http::{bearer_json_post, provider_status_retryable};

use crate::{
    JsonSchemaFormat, ModelInputItem, ModelMessageRole, ModelProvider, ModelRequest, ModelResponse,
    ModelResponseFormat, ModelTool, ModelToolCall, TokenUsage,
};

pub struct OpenAiProvider {
    network: Arc<dyn NetworkClient>,
    base_url: String,
    api_key: String,
    timeout_ms: Option<u64>,
    model: String,
}

impl OpenAiProvider {
    pub fn new(
        network: Arc<dyn NetworkClient>,
        base_url: String,
        api_key: String,
        timeout_ms: Option<u64>,
        model: String,
    ) -> Self {
        Self {
            network,
            base_url,
            api_key,
            timeout_ms,
            model,
        }
    }

    fn validate_request(&self, request: &ModelRequest) -> Result<()> {
        if self.base_url.trim().is_empty() {
            return Err(Error::ConfigInvalid {
                message: "openai base_url is empty".to_owned(),
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

        if request.input.is_empty() {
            return Err(Error::InvalidInput {
                message: "model request must include at least one input item".to_owned(),
            });
        }

        Ok(())
    }

    fn build_network_request(&self, request: ModelRequest) -> Result<NetworkRequest> {
        let input = request
            .input
            .into_iter()
            .map(map_input_item)
            .collect::<Result<Vec<_>>>()?;
        let body = serde_json::to_value(OpenAiRequest {
            model: request.model.unwrap_or_else(|| self.model.clone()),
            previous_response_id: request.previous_response_id,
            input,
            tools: request.tools.into_iter().map(map_tool).collect::<Vec<_>>(),
            text: text_config(request.response_format),
            parallel_tool_calls: false,
            temperature: request.temperature,
            max_output_tokens: request.max_tokens,
            stream: false,
        })
        .context(JsonSnafu)?;

        Ok(bearer_json_post(
            &self.base_url,
            "responses",
            &self.api_key,
            body,
            self.timeout_ms,
        ))
    }

    fn map_response(&self, body: Value) -> Result<ModelResponse> {
        let provider_response: OpenAiResponse = serde_json::from_value(body).context(JsonSnafu)?;
        let mut content = Vec::new();
        let mut tool_calls = Vec::new();
        let mut output_items = Vec::new();

        for output in provider_response.output {
            match output {
                OpenAiResponseOutput::Message { content: items, .. } => {
                    let message = items
                        .into_iter()
                        .map(|item| match item {
                            OpenAiResponseContent::OutputText { text } => Ok(text),
                            OpenAiResponseContent::Refusal { .. } => {
                                Err(Error::SchemaValidationFailed {
                                    message: "openai structured output was refused".to_owned(),
                                })
                            }
                        })
                        .collect::<Result<Vec<_>>>()?
                        .join("\n");
                    if !message.is_empty() {
                        content.push(message.clone());
                        output_items.push(ModelInputItem::message(
                            ModelMessageRole::Assistant,
                            message,
                        ));
                    }
                }
                OpenAiResponseOutput::FunctionCall {
                    call_id,
                    name,
                    arguments,
                    ..
                } => {
                    let tool_call = map_tool_call(call_id, name, &arguments)?;
                    output_items.push(ModelInputItem::tool_call(tool_call.clone()));
                    tool_calls.push(tool_call);
                }
                OpenAiResponseOutput::Reasoning {} => {}
                OpenAiResponseOutput::Unknown => {
                    tracing::debug!(
                        provider = "openai",
                        "ignoring unknown OpenAI response output kind"
                    );
                }
            }
        }

        let response_id = if provider_response.store == Some(false) {
            None
        } else {
            provider_response.id
        };

        Ok(ModelResponse {
            provider: self.name().to_owned(),
            model: provider_response.model,
            response_id,
            content: if content.is_empty() {
                None
            } else {
                Some(content.join("\n"))
            },
            tool_calls,
            output_items,
            usage: provider_response.usage.as_ref().map(map_usage),
        })
    }
}

#[async_trait]
impl ModelProvider for OpenAiProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        self.validate_request(&request)?;
        let network_request = self.build_network_request(request)?;
        let response = self.network.send(network_request).await?;

        if !(200..300).contains(&response.status) {
            return Err(Error::HttpStatus {
                status: response.status,
                message: "openai model provider returned non-success status".to_owned(),
                retryable: provider_status_retryable(response.status),
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

fn map_input_item(item: ModelInputItem) -> Result<OpenAiInputItem> {
    match item {
        ModelInputItem::Message(message) => Ok(OpenAiInputItem::Message(OpenAiInputMessage {
            role: map_role(message.role),
            content: message.content,
        })),
        ModelInputItem::ToolCall(call) => {
            let arguments = serde_json::to_string(&call.arguments).context(JsonSnafu)?;
            Ok(OpenAiInputItem::FunctionCall(OpenAiFunctionCallInput {
                item_type: "function_call",
                call_id: call.id,
                name: call.name,
                arguments,
            }))
        }
        ModelInputItem::ToolOutput(output) => Ok(OpenAiInputItem::FunctionCallOutput(
            OpenAiFunctionCallOutputInput {
                item_type: "function_call_output",
                call_id: output.call_id,
                output: output.output,
            },
        )),
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

fn text_config(response_format: Option<ModelResponseFormat>) -> Option<OpenAiTextConfig> {
    match response_format {
        Some(ModelResponseFormat::JsonSchema(format)) => Some(json_schema_text_config(format)),
        Some(ModelResponseFormat::Text) | None => None,
    }
}

fn json_schema_text_config(format: JsonSchemaFormat) -> OpenAiTextConfig {
    let mut schema = format.schema;
    normalize_openai_strict_schema(&mut schema);

    OpenAiTextConfig {
        format: OpenAiTextFormat {
            format_type: "json_schema",
            name: format.name,
            strict: format.strict,
            schema,
        },
    }
}

fn normalize_openai_strict_schema(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if map.get("type").is_some_and(is_object_type) {
                map.insert("additionalProperties".to_owned(), Value::Bool(false));
                let required = map
                    .get("properties")
                    .and_then(Value::as_object)
                    .map(|properties| properties.keys().cloned().map(Value::String).collect())
                    .unwrap_or_default();
                map.insert("required".to_owned(), Value::Array(required));
            }

            for child in map.values_mut() {
                normalize_openai_strict_schema(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                normalize_openai_strict_schema(item);
            }
        }
        _ => {}
    }
}

fn is_object_type(value: &Value) -> bool {
    match value {
        Value::String(schema_type) => schema_type == "object",
        Value::Array(schema_types) => schema_types
            .iter()
            .any(|schema_type| schema_type.as_str() == Some("object")),
        _ => false,
    }
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_response_id: Option<String>,
    input: Vec<OpenAiInputItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<OpenAiTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<OpenAiTextConfig>,
    parallel_tool_calls: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    stream: bool,
}

#[derive(Serialize)]
struct OpenAiTextConfig {
    format: OpenAiTextFormat,
}

#[derive(Serialize)]
struct OpenAiTextFormat {
    #[serde(rename = "type")]
    format_type: &'static str,
    name: String,
    strict: bool,
    schema: Value,
}

#[derive(Serialize)]
#[serde(untagged)]
enum OpenAiInputItem {
    Message(OpenAiInputMessage),
    FunctionCall(OpenAiFunctionCallInput),
    FunctionCallOutput(OpenAiFunctionCallOutputInput),
}

#[derive(Serialize)]
struct OpenAiInputMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct OpenAiFunctionCallInput {
    #[serde(rename = "type")]
    item_type: &'static str,
    call_id: String,
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct OpenAiFunctionCallOutputInput {
    #[serde(rename = "type")]
    item_type: &'static str,
    call_id: String,
    output: String,
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
struct OpenAiResponse {
    id: Option<String>,
    model: Option<String>,
    store: Option<bool>,
    #[serde(default)]
    output: Vec<OpenAiResponseOutput>,
    usage: Option<OpenAiUsage>,
}

/// Wire form of a single `output` entry in the `OpenAI` Responses API.
///
/// The variant set is intentionally open via `#[serde(other)]` so that any
/// future provider-side output kind (e.g. new reasoning modes, tool result
/// envelopes) deserializes into `Unknown` and is silently ignored by the
/// parser, mirroring the tolerant strategy in
/// `crates/lapis-core/src/search/provider/grok.rs`. This keeps Lapis robust
/// against provider additions without requiring a client-side update.
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
    Reasoning {},
    /// Catch-all for output kinds Lapis does not currently understand. The
    /// parser ignores `Unknown` entries instead of failing the whole response.
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum OpenAiResponseContent {
    OutputText {
        text: String,
    },
    Refusal {
        #[serde(rename = "refusal")]
        _refusal: String,
    },
}

#[derive(Deserialize)]
struct OpenAiUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    total_tokens: Option<u64>,
}
