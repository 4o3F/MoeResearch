use std::sync::Arc;

use async_trait::async_trait;
use lapis_core::error::{Error, Result};
use lapis_core::model::provider::ModelProvider;
use lapis_core::model::providers::OpenAiCompatibleProvider;
use lapis_core::model::service::ModelService;
use lapis_core::net::client::MockNetworkClient;
use lapis_core::schema::model::{
    ModelMessage, ModelMessageRole, ModelRequest, ModelResponse, ModelTool,
};
use lapis_core::schema::network::NetworkResponse;
use lapis_core::schema::policy::ModelPolicy;
use serde_json::json;

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
            content: Some("content".to_owned()),
            tool_calls: vec![],
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
            content: Some("content".to_owned()),
            tool_calls: vec![],
            usage: None,
        })
    }
}

fn request(provider: &str) -> ModelRequest {
    ModelRequest {
        provider: provider.to_owned(),
        model: None,
        messages: vec![user_message("hello")],
        tools: vec![],
        temperature: None,
        max_tokens: None,
    }
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

#[tokio::test]
async fn routes_requested_allowed_provider() {
    let mut service = ModelService::new();
    service.register(StaticProvider("alpha"));
    service.register(StaticProvider("beta"));
    let policy = ModelPolicy {
        allowed_providers: vec!["beta".to_owned()],
        ..ModelPolicy::default()
    };

    let response = service
        .complete(request("beta"), &policy)
        .await
        .expect("model response");

    assert_eq!(response.provider, "beta");
}

#[tokio::test]
async fn uses_default_provider_when_request_provider_is_empty() {
    let mut service = ModelService::new();
    service.register(StaticProvider("alpha"));
    let policy = ModelPolicy {
        default_provider: "alpha".to_owned(),
        allowed_providers: vec!["alpha".to_owned()],
        ..ModelPolicy::default()
    };

    let response = service
        .complete(request(""), &policy)
        .await
        .expect("model response");

    assert_eq!(response.provider, "alpha");
}

#[tokio::test]
async fn rejects_disallowed_provider() {
    let mut service = ModelService::new();
    service.register(StaticProvider("beta"));
    let policy = ModelPolicy {
        allowed_providers: vec!["alpha".to_owned()],
        ..ModelPolicy::default()
    };

    let error = service
        .complete(request("beta"), &policy)
        .await
        .expect_err("disallowed provider error");

    assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "beta"));
}

#[tokio::test]
async fn applies_policy_defaults_before_dispatch() {
    let seen = Arc::new(std::sync::Mutex::new(None));
    let mut service = ModelService::new();
    service.register(CapturingProvider { seen: seen.clone() });
    let policy = ModelPolicy {
        default_provider: "alpha".to_owned(),
        default_model: Some("model-a".to_owned()),
        allowed_providers: vec!["alpha".to_owned()],
        temperature: Some(0.7),
        max_tokens: Some(128),
        ..ModelPolicy::default()
    };

    service
        .complete(request(""), &policy)
        .await
        .expect("model response");
    let request = seen
        .lock()
        .expect("request lock")
        .clone()
        .expect("captured request");

    assert_eq!(request.provider, "alpha");
    assert_eq!(request.model.as_deref(), Some("model-a"));
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(128));
}

#[tokio::test]
async fn validates_request_after_policy_defaults() {
    let seen = Arc::new(std::sync::Mutex::new(None));
    let mut service = ModelService::new();
    service.register(CapturingProvider { seen: seen.clone() });
    let policy = ModelPolicy {
        default_provider: "alpha".to_owned(),
        allowed_providers: vec!["alpha".to_owned()],
        temperature: Some(3.0),
        max_tokens: Some(128),
        ..ModelPolicy::default()
    };

    let error = service
        .complete(request(""), &policy)
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
    let policy = ModelPolicy {
        default_provider: "alpha".to_owned(),
        allowed_providers: vec!["alpha".to_owned()],
        max_tokens: Some(0),
        ..ModelPolicy::default()
    };

    let error = service
        .complete(request(""), &policy)
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
    let policy = ModelPolicy {
        default_provider: "alpha".to_owned(),
        allowed_providers: vec!["alpha".to_owned()],
        ..ModelPolicy::default()
    };
    let mut invalid = request("");
    invalid.messages = vec![];

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
        .complete(request_with_messages(vec![user_message("search")]))
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
    assert_eq!(body["model"], "gpt-4o-mini");
    assert_eq!(body["input"][0]["role"], "user");
    assert_eq!(body["input"][0]["content"], "hi");
    assert_eq!(body["tools"][0]["type"], "function");
    assert_eq!(body["tools"][0]["name"], "search");
    assert_eq!(body["tools"][0]["description"], "Search the web");
    assert_eq!(body["tools"][0]["parameters"]["type"], "object");
    let temperature = body["temperature"].as_f64().expect("temperature");
    assert!((temperature - 0.2).abs() < 0.000_001);
    assert_eq!(body["max_output_tokens"], 128);
}
