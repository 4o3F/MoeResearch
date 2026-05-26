use std::sync::Arc;

use async_trait::async_trait;
use lapis_core::error::{Error, Result};
use lapis_core::model::provider::{ModelProvider, OpenAiProvider};
use lapis_core::model::service::ModelService;
use lapis_core::net::client::MockNetworkClient;
use lapis_core::schema::model::{
    ModelInputItem, ModelMessageRole, ModelRequest, ModelResponse, ModelTool, ModelToolCall,
};
use lapis_core::schema::network::NetworkResponse;
use lapis_core::schema::policy::ModelPolicy;
use serde_json::{Value, json};

struct StaticProvider(&'static str);

struct CapturingProvider {
    seen: Arc<std::sync::Mutex<Option<ModelRequest>>>,
}

#[async_trait]
impl ModelProvider for StaticProvider {
    fn name(&self) -> &'static str {
        self.0
    }

    async fn complete(&self, _request: ModelRequest) -> Result<ModelResponse> {
        Ok(ModelResponse {
            provider: self.0.to_owned(),
            model: None,
            response_id: None,
            content: Some("content".to_owned()),
            tool_calls: vec![],
            output_items: vec![],
            usage: None,
        })
    }
}

#[async_trait]
impl ModelProvider for CapturingProvider {
    fn name(&self) -> &'static str {
        "alpha"
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        *self.seen.lock().expect("request lock") = Some(request.clone());
        Ok(ModelResponse {
            provider: request.provider,
            model: request.model,
            response_id: None,
            content: Some("content".to_owned()),
            tool_calls: vec![],
            output_items: vec![],
            usage: None,
        })
    }
}

fn request(provider: &str) -> ModelRequest {
    ModelRequest {
        provider: provider.to_owned(),
        model: None,
        previous_response_id: None,
        input: vec![user_message("hello")],
        tools: vec![],
        temperature: None,
        max_tokens: None,
    }
}

fn provider(network: Arc<MockNetworkClient>) -> OpenAiProvider {
    OpenAiProvider::new(
        network,
        "https://api.example.com".to_owned(),
        "secret".to_owned(),
        None,
        "configured-model".to_owned(),
    )
}

fn request_with_input(input: Vec<ModelInputItem>) -> ModelRequest {
    ModelRequest {
        provider: "openai".to_owned(),
        model: Some("gpt-test".to_owned()),
        previous_response_id: None,
        input,
        tools: vec![],
        temperature: None,
        max_tokens: None,
    }
}

fn user_message(content: &str) -> ModelInputItem {
    ModelInputItem::message(ModelMessageRole::User, content)
}

fn model_policy(allowed_providers: &[&str]) -> ModelPolicy {
    ModelPolicy {
        allowed_providers: allowed_providers
            .iter()
            .map(|provider| (*provider).to_owned())
            .collect(),
        temperature: Some(0.2),
        max_tokens: None,
        require_tool_call_support: true,
    }
}

#[tokio::test]
async fn routes_requested_allowed_provider() {
    let mut service = ModelService::new();
    service.register(StaticProvider("alpha"));
    service.register(StaticProvider("beta"));
    let policy = model_policy(&["beta"]);

    let response = service
        .complete(request("beta"), &policy)
        .await
        .expect("model response");

    assert_eq!(response.provider, "beta");
}

#[tokio::test]
async fn rejects_empty_request_provider() {
    let mut service = ModelService::new();
    service.register(StaticProvider("alpha"));
    let policy = model_policy(&["alpha"]);

    let error = service
        .complete(request(""), &policy)
        .await
        .expect_err("missing provider error");

    assert!(matches!(error, Error::InvalidInput { .. }));
}

#[tokio::test]
async fn rejects_disallowed_provider() {
    let mut service = ModelService::new();
    service.register(StaticProvider("beta"));
    let policy = model_policy(&["alpha"]);

    let error = service
        .complete(request("beta"), &policy)
        .await
        .expect_err("disallowed provider error");

    assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "beta"));
}

#[tokio::test]
async fn applies_policy_settings_before_dispatch() {
    let seen = Arc::new(std::sync::Mutex::new(None));
    let mut service = ModelService::new();
    service.register(CapturingProvider { seen: seen.clone() });
    let mut policy = model_policy(&["alpha"]);
    policy.temperature = Some(0.7);
    policy.max_tokens = Some(128);

    service
        .complete(request("alpha"), &policy)
        .await
        .expect("model response");
    let request = seen
        .lock()
        .expect("request lock")
        .clone()
        .expect("captured request");

    assert_eq!(request.provider, "alpha");
    assert_eq!(request.model, None);
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(128));
}

#[tokio::test]
async fn validates_request_after_policy_settings() {
    let seen = Arc::new(std::sync::Mutex::new(None));
    let mut service = ModelService::new();
    service.register(CapturingProvider { seen: seen.clone() });
    let mut policy = model_policy(&["alpha"]);
    policy.temperature = Some(3.0);
    policy.max_tokens = Some(128);

    let error = service
        .complete(request("alpha"), &policy)
        .await
        .expect_err("invalid model request");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
    assert!(seen.lock().expect("request lock").is_none());
}

#[tokio::test]
async fn rejects_zero_policy_max_tokens_before_dispatch() {
    let seen = Arc::new(std::sync::Mutex::new(None));
    let mut service = ModelService::new();
    service.register(CapturingProvider { seen: seen.clone() });
    let mut policy = model_policy(&["alpha"]);
    policy.max_tokens = Some(0);

    let error = service
        .complete(request("alpha"), &policy)
        .await
        .expect_err("invalid model request");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
    assert!(seen.lock().expect("request lock").is_none());
}

#[tokio::test]
async fn rejects_empty_model_messages_before_dispatch() {
    let seen = Arc::new(std::sync::Mutex::new(None));
    let mut service = ModelService::new();
    service.register(CapturingProvider { seen: seen.clone() });
    let policy = model_policy(&["alpha"]);
    let mut invalid = request("alpha");
    invalid.input = vec![];

    let error = service
        .complete(invalid, &policy)
        .await
        .expect_err("invalid model request");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
    assert!(seen.lock().expect("request lock").is_none());
}

#[test]
fn provider_names_returns_registered_names() {
    let mut service = ModelService::new();
    service.register(StaticProvider("beta"));
    service.register(StaticProvider("alpha"));

    assert_eq!(
        service.provider_names(),
        vec!["alpha".to_owned(), "beta".to_owned()]
    );
}

#[tokio::test]
async fn maps_text_response_and_usage() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "id": "resp_1",
            "model": "gpt-test",
            "output": [{
                "type": "message",
                "content": [{
                    "type": "output_text",
                    "text": "hello"
                }]
            }],
            "usage": {
                "input_tokens": 3,
                "output_tokens": 5,
                "total_tokens": 8
            }
        }),
    }]));
    let provider = provider(network);

    let response = provider
        .complete(request_with_input(vec![user_message("hi")]))
        .await
        .expect("model response");

    assert_eq!(response.provider, "openai");
    assert_eq!(response.model, Some("gpt-test".to_owned()));
    assert_eq!(response.response_id, Some("resp_1".to_owned()));
    assert_eq!(response.content, Some("hello".to_owned()));
    let usage = response.usage.expect("usage");
    assert_eq!(usage.input_tokens, Some(3));
    assert_eq!(usage.output_tokens, Some(5));
    assert_eq!(usage.total_tokens, Some(8));
}

#[tokio::test]
async fn unstored_response_id_maps_to_none_for_stateless_replay() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "id": "resp_1",
            "store": false,
            "output": [{
                "type": "message",
                "content": [{
                    "type": "output_text",
                    "text": "hello"
                }]
            }]
        }),
    }]));
    let provider = provider(network);

    let response = provider
        .complete(request_with_input(vec![user_message("hi")]))
        .await
        .expect("model response");

    assert_eq!(response.response_id, None);
    assert_eq!(response.content, Some("hello".to_owned()));
}

#[tokio::test]
async fn maps_tool_call_only_response_with_parsed_arguments() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [{
                "type": "function_call",
                "call_id": "call_1",
                "name": "search",
                "arguments": "{\"query\":\"lapis\"}"
            }]
        }),
    }]));
    let provider = provider(network);

    let response = provider
        .complete(request_with_input(vec![user_message("search")]))
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
    assert_eq!(
        response.output_items,
        vec![ModelInputItem::tool_call(response.tool_calls[0].clone())]
    );
}

#[tokio::test]
async fn maps_mixed_message_and_tool_call_output_items_in_order() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [
                {
                    "type": "message",
                    "content": [{
                        "type": "output_text",
                        "text": "I will search."
                    }]
                },
                {
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "search",
                    "arguments": "{\"query\":\"lapis\"}"
                }
            ]
        }),
    }]));
    let provider = provider(network);

    let response = provider
        .complete(request_with_input(vec![user_message("search")]))
        .await
        .expect("mixed response");

    assert_eq!(response.content, Some("I will search.".to_owned()));
    assert_eq!(response.tool_calls.len(), 1);
    assert_eq!(response.output_items.len(), 2);
    assert!(matches!(
        &response.output_items[0],
        ModelInputItem::Message(message)
            if message.role == ModelMessageRole::Assistant && message.content == "I will search."
    ));
    assert!(matches!(
        &response.output_items[1],
        ModelInputItem::ToolCall(call) if call.id == "call_1" && call.name == "search"
    ));
}

#[tokio::test]
async fn missing_usage_maps_to_none() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [{
                "type": "message",
                "content": [{
                    "type": "output_text",
                    "text": "hello"
                }]
            }]
        }),
    }]));
    let provider = provider(network);

    let response = provider
        .complete(request_with_input(vec![user_message("hi")]))
        .await
        .expect("model response");

    assert_eq!(response.usage, None);
}

#[tokio::test]
async fn ignores_reasoning_output_items() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [
                {
                    "type": "reasoning",
                    "id": "rs_1",
                    "summary": []
                },
                {
                    "type": "message",
                    "content": [{
                        "type": "output_text",
                        "text": "hello"
                    }]
                }
            ]
        }),
    }]));
    let provider = provider(network);

    let response = provider
        .complete(request_with_input(vec![user_message("hi")]))
        .await
        .expect("model response");

    assert_eq!(response.content, Some("hello".to_owned()));
    assert!(response.tool_calls.is_empty());
}

#[tokio::test]
async fn structured_output_refusal_returns_schema_validation_error() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [{
                "type": "message",
                "content": [{
                    "type": "refusal",
                    "refusal": "I cannot comply."
                }]
            }]
        }),
    }]));
    let provider = provider(network);

    let error = provider
        .complete(request_with_input(vec![user_message("hi")]))
        .await
        .expect_err("refusal error");

    assert!(matches!(
        error,
        Error::SchemaValidationFailed { ref message } if message == "openai structured output was refused"
    ));
    assert!(!error.retryable());
}

#[tokio::test]
async fn malformed_tool_call_arguments_returns_error() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [{
                "type": "function_call",
                "call_id": "call_1",
                "name": "search",
                "arguments": "{bad"
            }]
        }),
    }]));
    let provider = provider(network);

    let error = provider
        .complete(request_with_input(vec![user_message("search")]))
        .await;

    assert!(error.is_err());
}

#[tokio::test]
async fn request_uses_responses_endpoint_and_openai_tool_schema() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [{
                "type": "message",
                "content": [{
                    "type": "output_text",
                    "text": "ok"
                }]
            }]
        }),
    }]));
    let provider = OpenAiProvider::new(
        network.clone(),
        "https://api.example.com/".to_owned(),
        "secret".to_owned(),
        Some(1000),
        "configured-model".to_owned(),
    );
    let mut request = request_with_input(vec![user_message("hi")]);
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
    assert_eq!(request.url, "https://api.example.com/responses");
    assert_eq!(request.timeout_ms, Some(1000));
    assert!(
        request
            .headers
            .iter()
            .any(|header| { header.name == "authorization" && header.value == "Bearer secret" })
    );
    assert!(
        request
            .headers
            .iter()
            .any(|header| { header.name == "content-type" && header.value == "application/json" })
    );

    let body = request.body.as_ref().expect("request body");
    assert_eq!(body["model"], "configured-model");
    assert_eq!(body["stream"], false);
    assert_eq!(body["parallel_tool_calls"], false);
    assert_eq!(body["input"][0]["role"], "user");
    assert_eq!(body["input"][0]["content"], "hi");
    assert_eq!(body["tools"][0]["type"], "function");
    assert_eq!(body["tools"][0]["name"], "search");
    assert_eq!(body["tools"][0]["description"], "Search the web");
    assert_eq!(body["tools"][0]["parameters"]["type"], "object");
    assert_eq!(body["text"]["format"]["type"], "json_schema");
    assert_eq!(body["text"]["format"]["name"], "aspect_research_result_v1");
    assert_eq!(body["text"]["format"]["strict"], true);
    assert!(
        body["tools"]
            .as_array()
            .is_some_and(|tools| !tools.is_empty()),
        "tools and text.format must coexist in OpenAI request body"
    );
    let temperature = body["temperature"].as_f64().expect("temperature");
    assert!((temperature - 0.2).abs() < 0.000_001);
    assert_eq!(body["max_output_tokens"], 128);
}

#[tokio::test]
async fn openai_structured_output_schema_restricts_source_type_enum() {
    let body = captured_openai_request_body().await;
    let schema = openai_text_schema(&body);
    let source_type = schema_def(schema, "SourceType");
    let enum_values = source_type["enum"]
        .as_array()
        .expect("SourceType enum")
        .iter()
        .map(|value| value.as_str().expect("enum string"))
        .collect::<Vec<_>>();

    assert_eq!(
        enum_values,
        vec![
            "official",
            "documentation",
            "news",
            "blog",
            "forum",
            "repository",
            "unknown"
        ]
    );
    assert!(!enum_values.contains(&"discussion"));
}

#[tokio::test]
async fn openai_structured_output_schema_closes_objects_for_strict_mode() {
    let body = captured_openai_request_body().await;
    let schema = openai_text_schema(&body);

    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema_def(schema, "Evidence")["additionalProperties"],
        false
    );
    assert_eq!(
        schema_def(schema, "AspectReport")["additionalProperties"],
        false
    );
}

#[tokio::test]
async fn openai_structured_output_schema_requires_nullable_evidence_fields() {
    let body = captured_openai_request_body().await;
    let schema = openai_text_schema(&body);
    let evidence = schema_def(schema, "Evidence");
    let required = evidence["required"]
        .as_array()
        .expect("Evidence required fields")
        .iter()
        .map(|value| value.as_str().expect("required string"))
        .collect::<Vec<_>>();

    assert!(required.contains(&"url"));
    assert!(required.contains(&"published_at"));
    assert!(schema_allows_null(&evidence["properties"]["url"]));
    assert!(schema_allows_null(&evidence["properties"]["published_at"]));
}

async fn captured_openai_request_body() -> Value {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [{
                "type": "message",
                "content": [{
                    "type": "output_text",
                    "text": "ok"
                }]
            }]
        }),
    }]));
    let provider = provider(network.clone());

    provider
        .complete(request_with_input(vec![user_message("hi")]))
        .await
        .expect("model response");

    network.requests()[0].body.clone().expect("request body")
}

fn openai_text_schema(body: &Value) -> &Value {
    &body["text"]["format"]["schema"]
}

fn schema_def<'a>(schema: &'a Value, name: &str) -> &'a Value {
    schema["$defs"]
        .get(name)
        .unwrap_or_else(|| panic!("missing schema definition {name}"))
}

fn schema_allows_null(schema: &Value) -> bool {
    schema_type_allows_null(schema)
        || schema_keyword_allows_null(schema, "anyOf")
        || schema_keyword_allows_null(schema, "oneOf")
}

fn schema_keyword_allows_null(schema: &Value, keyword: &str) -> bool {
    schema
        .get(keyword)
        .and_then(Value::as_array)
        .is_some_and(|items| items.iter().any(schema_allows_null))
}

fn schema_type_allows_null(schema: &Value) -> bool {
    match schema.get("type") {
        Some(Value::String(schema_type)) => schema_type == "null",
        Some(Value::Array(schema_types)) => schema_types
            .iter()
            .any(|schema_type| schema_type.as_str() == Some("null")),
        _ => false,
    }
}

#[tokio::test]
async fn request_serializes_function_call_output_items() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [{
                "type": "message",
                "content": [{
                    "type": "output_text",
                    "text": "ok"
                }]
            }]
        }),
    }]));
    let provider = provider(network.clone());
    let tool_call = ModelToolCall {
        id: "call_1".to_owned(),
        name: "search".to_owned(),
        arguments: json!({ "query": "lapis" }),
    };
    let mut request = request_with_input(vec![
        user_message("search"),
        ModelInputItem::tool_call(tool_call),
        ModelInputItem::tool_output("call_1", r#"{"result_count":1}"#),
    ]);
    request.previous_response_id = Some("resp_1".to_owned());

    provider.complete(request).await.expect("model response");

    let requests = network.requests();
    let body = requests[0].body.as_ref().expect("request body");
    assert_eq!(body["previous_response_id"], "resp_1");
    assert_eq!(body["input"][0]["role"], "user");
    assert_eq!(body["input"][0]["content"], "search");
    assert_eq!(body["input"][1]["type"], "function_call");
    assert_eq!(body["input"][1]["call_id"], "call_1");
    assert_eq!(body["input"][1]["name"], "search");
    assert_eq!(
        body["input"][1]["arguments"],
        serde_json::to_string(&json!({ "query": "lapis" })).expect("arguments json")
    );
    assert_eq!(body["input"][2]["type"], "function_call_output");
    assert_eq!(body["input"][2]["call_id"], "call_1");
    assert_eq!(body["input"][2]["output"], r#"{"result_count":1}"#);
}

/// The OpenAI Responses parser MUST silently tolerate unknown `output` kinds
/// (forward-compatible behavior driven by `#[serde(other)]` on
/// `OpenAiResponseOutput::Unknown`), so a provider-side addition does not
/// fail the whole response.
#[tokio::test]
async fn openai_provider_tolerates_unknown_response_output() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [
                {
                    "type": "some_future_kind",
                    "details": {"foo": "bar"}
                },
                {
                    "type": "message",
                    "content": [{
                        "type": "output_text",
                        "text": "hello"
                    }]
                }
            ]
        }),
    }]));
    let provider = provider(network);

    let response = provider
        .complete(request_with_input(vec![user_message("hi")]))
        .await
        .expect("model response");

    assert_eq!(response.content, Some("hello".to_owned()));
    assert!(response.tool_calls.is_empty());
}
