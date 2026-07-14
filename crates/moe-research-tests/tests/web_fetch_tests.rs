mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use moe_research_error::{Error, Result};
use moe_research_model::{
    ModelInputItem, ModelProvider, ModelRequest, ModelResponse, ModelService, ModelToolCall,
};
use moe_research_net::{
    DocumentNetworkOutcome, DocumentNetworkResponse, Header, JsonNetworkResponse,
};
use moe_research_search::SearchService;
use moe_research_web_fetch::{
    WebFetchAnswerOutcome, WebFetchDocumentOutcome, WebFetchRuntimeConfig, WebFetchService,
};
use moe_research_workflow::{FailureStage, ToolName};
use serde_json::json;
use support::network::MockNetworkClient;
use support::research::{
    aspect_request, aspect_research, first_evidence_from_tool_output, medium_result_json,
    unlimited_budget_config,
};

struct StaticAnswerProvider {
    calls: Arc<AtomicUsize>,
    content: String,
}

#[async_trait]
impl ModelProvider for StaticAnswerProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(request.tools.is_empty());
        assert!(request.previous_response_id.is_none());
        assert_eq!(request.temperature, Some(0.0));
        assert!(request.max_tokens.is_none());
        Ok(ModelResponse {
            provider: "openai".to_owned(),
            model: Some("web-fetch-test".to_owned()),
            response_id: None,
            content: Some(self.content.clone()),
            tool_calls: Vec::new(),
            output_items: Vec::new(),
            usage: None,
        })
    }
}

struct AspectWebFetchProvider;
struct MixedRetrievalProvider;
struct UnavailableAnswerProvider;

#[async_trait]
impl ModelProvider for AspectWebFetchProvider {
    fn name(&self) -> &'static str {
        "model"
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        if request
            .input
            .iter()
            .any(|item| matches!(item, ModelInputItem::ToolOutput(_)))
        {
            let evidence = first_evidence_from_tool_output(&request.input);
            return Ok(ModelResponse {
                provider: "model".to_owned(),
                model: None,
                response_id: None,
                content: Some(medium_result_json("aspect-1", "Aspect 1", evidence)),
                tool_calls: Vec::new(),
                output_items: Vec::new(),
                usage: None,
            });
        }
        let call = ModelToolCall {
            id: "fetch-1".to_owned(),
            name: "web_fetch".to_owned(),
            arguments: json!({
                "url": "https://example.com/docs",
                "prompt": "What is the schema version?"
            }),
        };
        Ok(ModelResponse {
            provider: "model".to_owned(),
            model: None,
            response_id: None,
            content: None,
            tool_calls: vec![call.clone()],
            output_items: vec![ModelInputItem::tool_call(call)],
            usage: None,
        })
    }
}

#[async_trait]
impl ModelProvider for UnavailableAnswerProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    async fn complete(&self, _request: ModelRequest) -> Result<ModelResponse> {
        Err(Error::ProviderUnavailable {
            provider: "openai".to_owned(),
            message: "test endpoint unavailable".to_owned(),
            retryable: false,
        })
    }
}

#[async_trait]
impl ModelProvider for MixedRetrievalProvider {
    fn name(&self) -> &'static str {
        "model"
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        let evidence = request
            .input
            .iter()
            .filter_map(|item| match item {
                ModelInputItem::ToolOutput(output) => {
                    let value = serde_json::from_str::<serde_json::Value>(&output.output).ok()?;
                    value["results"].as_array()?.first().cloned()
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let response = match evidence.len() {
            0 => tool_call_response(
                "fetch-1",
                "web_fetch",
                json!({
                    "url": "https://example.com/docs",
                    "prompt": "What is the schema version?"
                }),
            ),
            1 => tool_call_response(
                "search-1",
                "search",
                json!({
                    "query": "schema version",
                    "max_results": 1,
                    "intent": support::research::default_search_intent()
                }),
            ),
            _ => {
                let ids = evidence
                    .iter()
                    .filter_map(|item| item["id"].as_str().map(str::to_owned))
                    .collect::<Vec<_>>();
                ModelResponse {
                    provider: "model".to_owned(),
                    model: None,
                    response_id: None,
                    content: Some(
                        json!({
                            "aspect_report": {
                                "aspect_id": "aspect-1",
                                "aspect_name": "Aspect 1",
                                "question": "What is true?",
                                "scope": ["scope"],
                                "findings": [{
                                    "id": "finding-1",
                                    "claim": "The sources support the result.",
                                    "finding_type": "fact",
                                    "importance": "high",
                                    "confidence": "medium",
                                    "evidence_refs": ids,
                                    "contradicted_by": []
                                }],
                                "assumptions": [],
                                "risks": [],
                                "counterarguments": [],
                                "open_questions": [],
                                "confidence": "medium",
                                "limitations": []
                            },
                            "selected_evidence": ids
                        })
                        .to_string(),
                    ),
                    tool_calls: Vec::new(),
                    output_items: Vec::new(),
                    usage: None,
                }
            }
        };
        Ok(response)
    }
}

fn tool_call_response(id: &str, name: &str, arguments: serde_json::Value) -> ModelResponse {
    let call = ModelToolCall {
        id: id.to_owned(),
        name: name.to_owned(),
        arguments,
    };
    ModelResponse {
        provider: "model".to_owned(),
        model: None,
        response_id: None,
        content: None,
        tool_calls: vec![call.clone()],
        output_items: vec![ModelInputItem::tool_call(call)],
        usage: None,
    }
}

fn document_response(status: u16, headers: Vec<Header>, body: &[u8]) -> DocumentNetworkOutcome {
    DocumentNetworkOutcome::Response(DocumentNetworkResponse {
        status,
        headers,
        body: body.to_vec(),
    })
}

fn service(
    outcomes: impl IntoIterator<Item = DocumentNetworkOutcome>,
) -> (WebFetchService, Arc<MockNetworkClient>, Arc<AtomicUsize>) {
    let network = Arc::new(
        MockNetworkClient::new(std::iter::empty::<JsonNetworkResponse>()).with_documents(outcomes),
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let mut model = ModelService::new();
    model.register(StaticAnswerProvider {
        calls: calls.clone(),
        content: json!({
            "found": true,
            "answer": "The schema version is 0.2.",
            "supporting_excerpt": "The schema version is 0\\.2\\."
        })
        .to_string(),
    });
    let service = WebFetchService::new(
        network.clone(),
        model,
        "openai",
        WebFetchRuntimeConfig {
            cache_ttl_ms: 900_000,
            max_cache_entries: 8,
            max_redirects: 3,
            inactivity_timeout_ms: Some(5_000),
        },
    )
    .expect("service");
    (service, network, calls)
}

#[tokio::test]
async fn fetch_normalizes_http_and_answers_from_verified_excerpt() {
    let (service, network, calls) = service([document_response(
        200,
        vec![Header {
            name: "content-type".to_owned(),
            value: "text/html; charset=utf-8".to_owned(),
        }],
        b"<html><head><title>Docs</title></head><body><article><p>The schema version is 0.2.</p></article></body></html>",
    )]);
    let deadline = Instant::now() + Duration::from_secs(5);
    let document = match service
        .fetch_document("http://example.com/docs#section", Some(deadline))
        .await
        .expect("fetch")
    {
        WebFetchDocumentOutcome::Document(document) => document,
        other => panic!("expected document, got {other:?}"),
    };
    assert_eq!(network.document_requests().len(), 1);
    assert_eq!(
        network.document_requests()[0].url,
        "https://example.com/docs"
    );

    assert!(
        document.markdown.contains("schema version"),
        "unexpected markdown: {}",
        document.markdown
    );
    let answer = service
        .answer_document(&document, "What is the schema version?", Some(deadline))
        .await
        .expect("answer");
    match answer {
        WebFetchAnswerOutcome::Answer(answer) => assert!(answer.found),
        other => panic!("expected answer, got {other:?}"),
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn document_cache_reuses_fetch_but_not_model_answer() {
    let (service, network, calls) = service([document_response(
        200,
        vec![Header {
            name: "content-type".to_owned(),
            value: "text/plain; charset=utf-8".to_owned(),
        }],
        b"The schema version is 0.2.",
    )]);
    let deadline = Instant::now() + Duration::from_secs(5);
    let first = match service
        .fetch_document("https://example.com/docs", Some(deadline))
        .await
        .expect("first")
    {
        WebFetchDocumentOutcome::Document(document) => document,
        other => panic!("expected document, got {other:?}"),
    };
    let second = match service
        .fetch_document("https://example.com/docs#other", Some(deadline))
        .await
        .expect("second")
    {
        WebFetchDocumentOutcome::Document(document) => document,
        other => panic!("expected document, got {other:?}"),
    };
    assert_eq!(network.document_requests().len(), 1);
    service
        .answer_document(&first, "First prompt", Some(deadline))
        .await
        .expect("first answer");
    service
        .answer_document(&second, "Second prompt", Some(deadline))
        .await
        .expect("second answer");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn concurrent_cache_misses_share_one_document_fetch() {
    let network = Arc::new(
        MockNetworkClient::new(std::iter::empty::<JsonNetworkResponse>())
            .with_documents([document_response(
                200,
                vec![Header {
                    name: "content-type".to_owned(),
                    value: "text/plain; charset=utf-8".to_owned(),
                }],
                b"The schema version is 0.2.",
            )])
            .with_document_delay(Duration::from_millis(50)),
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let mut model = ModelService::new();
    model.register(StaticAnswerProvider {
        calls,
        content: String::new(),
    });
    let service = Arc::new(
        WebFetchService::new(
            network.clone(),
            model,
            "openai",
            WebFetchRuntimeConfig {
                cache_ttl_ms: 900_000,
                max_cache_entries: 8,
                max_redirects: 3,
                inactivity_timeout_ms: Some(5_000),
            },
        )
        .expect("service"),
    );
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..8 {
        let service = service.clone();
        tasks.spawn(async move {
            service
                .fetch_document(
                    "https://example.com/docs",
                    Some(Instant::now() + Duration::from_secs(5)),
                )
                .await
        });
    }
    while let Some(result) = tasks.join_next().await {
        match result.expect("fetch task").expect("fetch") {
            WebFetchDocumentOutcome::Document(document) => {
                assert_eq!(document.final_url, "https://example.com/docs");
            }
            other => panic!("expected document, got {other:?}"),
        }
    }
    assert_eq!(network.document_requests().len(), 1);
}

#[tokio::test]
async fn cross_host_redirect_is_returned_without_following() {
    let (service, network, _) = service([document_response(
        302,
        vec![Header {
            name: "location".to_owned(),
            value: "https://other.example/path".to_owned(),
        }],
        b"ignored",
    )]);
    let outcome = service
        .fetch_document(
            "https://example.com/start",
            Some(Instant::now() + Duration::from_secs(5)),
        )
        .await
        .expect("fetch");
    match outcome {
        WebFetchDocumentOutcome::Redirect { redirect_url } => {
            assert_eq!(redirect_url, "https://other.example/path");
        }
        other => panic!("expected redirect, got {other:?}"),
    }
    assert_eq!(network.document_requests().len(), 1);
}

#[tokio::test]
async fn sensitive_query_is_rejected_before_network_dispatch() {
    let (service, network, _) = service([]);
    let outcome = service
        .fetch_document(
            "https://example.com/docs?access_token=secret",
            Some(Instant::now() + Duration::from_secs(5)),
        )
        .await
        .expect("fetch");
    match outcome {
        WebFetchDocumentOutcome::SoftError(error) => {
            assert_eq!(error.code, "sensitive_query");
        }
        other => panic!("expected soft error, got {other:?}"),
    }
    assert!(network.document_requests().is_empty());
}

#[tokio::test]
async fn document_transport_failure_is_returned_as_soft_error() {
    let (service, network, _) = service([]);
    let outcome = service
        .fetch_document(
            "https://example.com/docs",
            Some(Instant::now() + Duration::from_secs(5)),
        )
        .await
        .expect("fetch");
    match outcome {
        WebFetchDocumentOutcome::SoftError(error) => {
            assert_eq!(error.code, "network_failed");
            assert!(error.retryable);
        }
        other => panic!("expected soft error, got {other:?}"),
    }
    assert_eq!(network.document_requests().len(), 1);
}

#[tokio::test]
async fn unavailable_prompt_model_is_not_masked_as_document_soft_error() {
    let network = Arc::new(
        MockNetworkClient::new(std::iter::empty::<JsonNetworkResponse>()).with_documents([
            document_response(
                200,
                vec![Header {
                    name: "content-type".to_owned(),
                    value: "text/plain; charset=utf-8".to_owned(),
                }],
                b"The schema version is 0.2.",
            ),
        ]),
    );
    let mut model = ModelService::new();
    model.register(UnavailableAnswerProvider);
    let service = WebFetchService::new(
        network,
        model,
        "openai",
        WebFetchRuntimeConfig {
            cache_ttl_ms: 900_000,
            max_cache_entries: 8,
            max_redirects: 3,
            inactivity_timeout_ms: Some(5_000),
        },
    )
    .expect("service");
    let deadline = Some(Instant::now() + Duration::from_secs(5));
    let document = match service
        .fetch_document("https://example.com/docs", deadline)
        .await
        .expect("fetch")
    {
        WebFetchDocumentOutcome::Document(document) => document,
        other => panic!("expected document, got {other:?}"),
    };

    let error = service
        .answer_document(&document, "What is the schema version?", deadline)
        .await
        .expect_err("provider unavailability must remain a hard typed error");
    assert_eq!(error.code().as_str(), "provider_unavailable");
}

#[tokio::test]
async fn workflow_dispatches_web_fetch_and_creates_host_owned_evidence() {
    let (web_fetch, _, _) = service([document_response(
        200,
        vec![Header {
            name: "content-type".to_owned(),
            value: "text/html; charset=utf-8".to_owned(),
        }],
        b"<html><body><article><p>The schema version is 0.2.</p></article></body></html>",
    )]);
    let mut model = ModelService::new();
    model.register(AspectWebFetchProvider);
    let search = SearchService::new();
    let mut request = aspect_request();
    request.task.tools = vec![ToolName("web_fetch".to_owned())];
    request.task.search_provider = None;

    let output = aspect_research(
        request,
        &model,
        &search,
        Some(&web_fetch),
        &unlimited_budget_config(),
    )
    .await
    .expect("aspect result");
    assert_eq!(output.result.evidence.len(), 1);
    assert_eq!(output.result.evidence[0].provider, "web_fetch");
    assert_eq!(output.budget_usage.search_calls_used, 0);
}

#[tokio::test]
async fn redirect_to_different_port_is_returned_without_following() {
    let (service, network, _) = service([document_response(
        302,
        vec![Header {
            name: "location".to_owned(),
            value: "https://example.com/next".to_owned(),
        }],
        b"ignored",
    )]);
    let outcome = service
        .fetch_document(
            "https://example.com:8443/start",
            Some(Instant::now() + Duration::from_secs(5)),
        )
        .await
        .expect("fetch");
    match outcome {
        WebFetchDocumentOutcome::Redirect { redirect_url } => {
            assert_eq!(redirect_url, "https://example.com/next");
        }
        other => panic!("expected redirect, got {other:?}"),
    }
    assert_eq!(network.document_requests().len(), 1);
}

#[tokio::test]
async fn workflow_rejects_web_fetch_at_dispatch_when_service_is_disabled() {
    let mut model = ModelService::new();
    model.register(AspectWebFetchProvider);
    let search = SearchService::new();
    let mut request = aspect_request();
    request.task.tools = vec![ToolName("web_fetch".to_owned())];
    request.task.search_provider = None;

    let failure = aspect_research(request, &model, &search, None, &unlimited_budget_config())
        .await
        .expect_err("disabled web_fetch must be rejected");
    assert_eq!(failure.error.code().as_str(), "tool_policy_denied");
    assert_eq!(failure.diagnostic.stage, FailureStage::ToolValidation);
}

#[tokio::test]
async fn mixed_fetch_then_search_uses_unique_retrieval_evidence_ids() {
    let (web_fetch, _, _) = service([document_response(
        200,
        vec![Header {
            name: "content-type".to_owned(),
            value: "text/html; charset=utf-8".to_owned(),
        }],
        b"<html><body><article><p>The schema version is 0.2.</p></article></body></html>",
    )]);
    let mut model = ModelService::new();
    model.register(MixedRetrievalProvider);
    let search_calls = Arc::new(AtomicUsize::new(0));
    let search = support::research::static_search_service(search_calls.clone());
    let mut request = aspect_request();
    request.task.tools = vec![
        ToolName("search".to_owned()),
        ToolName("web_fetch".to_owned()),
    ];

    let output = aspect_research(
        request,
        &model,
        &search,
        Some(&web_fetch),
        &unlimited_budget_config(),
    )
    .await
    .expect("aspect result");
    let ids = output
        .result
        .evidence
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["ev-1-1", "ev-2-2"]);
    assert_eq!(search_calls.load(Ordering::SeqCst), 1);
    assert_eq!(output.budget_usage.search_calls_used, 1);
    assert_eq!(output.budget_usage.tool_calls_used, 2);
}
