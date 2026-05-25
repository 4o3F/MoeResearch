use async_trait::async_trait;
use lapis_core::error::{Error, Result};
use lapis_core::net::client::MockNetworkClient;
use lapis_core::schema::network::NetworkResponse;
use lapis_core::schema::policy::SearchPolicy;
use lapis_core::schema::search::{SearchRequest, SearchResponse, SearchResult};
use lapis_core::search::provider::SearchProvider;
use lapis_core::search::provider::{ExaSearchProvider, GrokSearchProvider};
use lapis_core::search::service::SearchService;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct StaticProvider(&'static str);

#[async_trait]
impl SearchProvider for StaticProvider {
    fn name(&self) -> &'static str {
        self.0
    }

    async fn search(&self, _request: SearchRequest) -> Result<SearchResponse> {
        Ok(SearchResponse {
            provider: self.0.to_owned(),
            results: vec![SearchResult {
                title: "title".to_owned(),
                url: None,
                snippet: "snippet".to_owned(),
                summary: None,
                published_at: None,
            }],
        })
    }
}

struct CountingProvider {
    name: &'static str,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl SearchProvider for CountingProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn search(&self, _request: SearchRequest) -> Result<SearchResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(SearchResponse {
            provider: self.name.to_owned(),
            results: Vec::new(),
        })
    }
}

struct FailingProvider(&'static str);

#[async_trait]
impl SearchProvider for FailingProvider {
    fn name(&self) -> &'static str {
        self.0
    }

    async fn search(&self, _request: SearchRequest) -> Result<SearchResponse> {
        Err(Error::ProviderUnavailable {
            provider: self.0.to_owned(),
            message: "selected provider failed".to_owned(),
        })
    }
}

fn search_policy(allowed_providers: &[&str]) -> SearchPolicy {
    SearchPolicy {
        allowed_providers: allowed_providers
            .iter()
            .map(|provider| (*provider).to_owned())
            .collect(),
        max_results_per_query: 5,
        freshness: None,
        language: None,
        region: None,
        include_domains: Vec::new(),
        exclude_domains: Vec::new(),
    }
}

#[tokio::test]
async fn dispatches_only_to_explicit_provider() {
    let mut service = SearchService::new();
    let exa_calls = Arc::new(AtomicUsize::new(0));
    let grok_calls = Arc::new(AtomicUsize::new(0));
    service.register(CountingProvider {
        name: "exa",
        calls: exa_calls.clone(),
    });
    service.register(CountingProvider {
        name: "grok",
        calls: grok_calls.clone(),
    });

    let policy = search_policy(&["exa", "grok"]);
    let response = service
        .search(SearchRequest::new("grok", "lapis", 3), &policy)
        .await
        .expect("search response");

    assert_eq!(response.provider, "grok");
    assert_eq!(exa_calls.load(Ordering::SeqCst), 0);
    assert_eq!(grok_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn does_not_fallback_when_selected_provider_fails() {
    let mut service = SearchService::new();
    service.register(FailingProvider("exa"));
    service.register(StaticProvider("grok"));

    let policy = search_policy(&["exa", "grok"]);
    let error = service
        .search(SearchRequest::new("exa", "lapis", 3), &policy)
        .await
        .expect_err("selected provider error");

    assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "exa"));
}

#[tokio::test]
async fn rejects_missing_explicit_provider() {
    let mut service = SearchService::new();
    service.register(StaticProvider("exa"));

    let error = service
        .search(SearchRequest::new("", "lapis", 1), &search_policy(&["exa"]))
        .await
        .expect_err("missing search provider");

    assert!(matches!(error, Error::InvalidInput { .. }));
}

#[tokio::test]
async fn rejects_empty_allowlist_for_explicit_provider() {
    let mut service = SearchService::new();
    service.register(StaticProvider("exa"));

    let policy = search_policy(&[]);
    let error = service
        .search(SearchRequest::new("exa", "lapis", 1), &policy)
        .await
        .expect_err("empty allowlist rejects provider");

    assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "exa"));
}

#[tokio::test]
async fn rejects_disallowed_explicit_provider() {
    let mut service = SearchService::new();
    service.register(StaticProvider("grok"));

    let policy = search_policy(&["exa"]);
    let error = service
        .search(SearchRequest::new("grok", "lapis", 1), &policy)
        .await
        .expect_err("disallowed search provider");

    assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "grok"));
}

#[tokio::test]
async fn rejects_unconfigured_explicit_provider() {
    let service = SearchService::new();

    let error = service
        .search(
            SearchRequest::new("exa", "lapis", 1),
            &search_policy(&["exa"]),
        )
        .await
        .expect_err("unconfigured search provider");

    assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "exa"));
}

#[tokio::test]
async fn rejects_invalid_search_requests_before_provider_dispatch() {
    let mut service = SearchService::new();
    service.register(StaticProvider("exa"));

    for request in [
        SearchRequest::new("exa", " ", 1),
        SearchRequest::new("exa", "lapis", 0),
        SearchRequest::new("exa", "lapis", 6),
    ] {
        let error = service
            .search(request, &search_policy(&["exa"]))
            .await
            .expect_err("invalid search request");

        assert!(matches!(error, Error::InvalidInput { .. }));
    }
}

#[tokio::test]
async fn rejects_invalid_search_policy_before_provider_dispatch() {
    let mut service = SearchService::new();
    service.register(StaticProvider("exa"));

    let mut zero_limit = search_policy(&["exa"]);
    zero_limit.max_results_per_query = 0;
    let error = service
        .search(SearchRequest::new("exa", "lapis", 1), &zero_limit)
        .await
        .expect_err("invalid search policy");
    assert!(matches!(error, Error::InvalidInput { .. }));

    let mut overlapping_domains = search_policy(&["exa"]);
    overlapping_domains.include_domains = vec!["example.com".to_owned()];
    overlapping_domains.exclude_domains = vec!["EXAMPLE.com".to_owned()];
    let error = service
        .search(SearchRequest::new("exa", "lapis", 1), &overlapping_domains)
        .await
        .expect_err("invalid search policy");
    assert!(matches!(error, Error::InvalidInput { .. }));
}

#[tokio::test]
async fn forwards_policy_domain_filters_to_exa_provider() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "results": [] }),
    }]));
    let mut service = SearchService::new();
    service.register(ExaSearchProvider::new(
        network.clone(),
        "https://api.exa.ai".to_owned(),
        "key".to_owned(),
        None,
    ));
    let mut policy = search_policy(&["exa"]);
    policy.include_domains = vec!["example.com".to_owned()];
    policy.exclude_domains = vec!["blocked.com".to_owned()];

    service
        .search(SearchRequest::new("exa", "lapis", 1), &policy)
        .await
        .expect("search response");

    let request_body = network.requests()[0].body.clone().expect("request body");
    assert_eq!(request_body["include_domains"], json!(["example.com"]));
    assert_eq!(request_body["exclude_domains"], json!(["blocked.com"]));
}

#[tokio::test]
async fn maps_exa_response_to_standard_search_response() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "results": [{
                "title": "Lapis",
                "url": "https://example.com/lapis",
                "text": "snippet",
                "summary": "summary",
                "publishedDate": "2026-01-01"
            }]
        }),
    }]));
    let provider = ExaSearchProvider::new(
        network,
        "https://api.exa.ai".to_owned(),
        "key".to_owned(),
        None,
    );

    let response = provider
        .search(SearchRequest::new("exa", "lapis", 1))
        .await
        .expect("exa response");

    assert_eq!(response.provider, "exa");
    assert_eq!(response.results[0].title, "Lapis");
}

#[tokio::test]
async fn maps_grok_response_to_standard_search_response() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [
                { "type": "web_search_call", "status": "completed" },
                {
                    "type": "message",
                    "content": [{
                        "type": "output_text",
                        "text": "Result from source",
                        "annotations": [{
                            "type": "url_citation",
                            "url": "https://example.com/grok",
                            "title": "Grok result",
                            "start_index": 0,
                            "end_index": 6
                        }]
                    }]
                }
            ]
        }),
    }]));
    let provider = GrokSearchProvider::new(
        network,
        "https://api.x.ai".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
    );

    let response = provider
        .search(SearchRequest::new("grok", "lapis", 1))
        .await
        .expect("grok response");

    assert_eq!(response.provider, "grok");
    assert_eq!(response.results[0].title, "Grok result");
    assert_eq!(
        response.results[0].url.as_deref(),
        Some("https://example.com/grok")
    );
    assert_eq!(response.results[0].snippet, "Result");
    assert_eq!(
        response.results[0].summary.as_deref(),
        Some("Result from source")
    );
    assert_eq!(response.results[0].published_at, None);
}

#[tokio::test]
async fn grok_search_uses_responses_web_search_request() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "output": [] }),
    }]));
    let provider = GrokSearchProvider::new(
        network.clone(),
        "https://api.x.ai/".to_owned(),
        "key".to_owned(),
        Some(1000),
        "configured-grok-model".to_owned(),
    );
    let mut policy = search_policy(&["grok"]);
    policy.include_domains = vec!["example.com".to_owned()];
    policy.exclude_domains = vec!["blocked.com".to_owned()];
    policy.language = Some("en".to_owned());
    policy.region = Some("US".to_owned());

    provider
        .search(SearchRequest::new("grok", "lapis", 2).with_policy(&policy))
        .await
        .expect("grok response");

    let requests = network.requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request.method, "POST");
    assert_eq!(request.url, "https://api.x.ai/responses");
    assert_eq!(request.timeout_ms, Some(1000));
    assert!(
        request
            .headers
            .iter()
            .any(|header| { header.name == "authorization" && header.value == "Bearer key" })
    );
    assert!(
        request
            .headers
            .iter()
            .any(|header| { header.name == "content-type" && header.value == "application/json" })
    );

    let body = request.body.as_ref().expect("request body");
    assert_eq!(body["model"], "configured-grok-model");
    assert_eq!(body["stream"], false);
    assert_eq!(body["input"][0]["role"], "user");
    assert_eq!(body["tools"][0]["type"], "web_search");
    assert_eq!(body["tools"][0]["search_context_size"], "low");
    assert_eq!(
        body["tools"][0]["filters"]["allowed_domains"],
        json!(["example.com"])
    );
    let prompt = body["input"][0]["content"].as_str().expect("prompt");
    assert!(prompt.contains("Search the web for: lapis"));
    assert!(prompt.contains("Maximum results: 2"));
    assert!(prompt.contains("Language: en"));
    assert!(prompt.contains("Region: US"));
    assert!(prompt.contains("Exclude domains: blocked.com"));
}

#[tokio::test]
async fn grok_search_uses_annotation_local_text_for_snippets() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [{
                "type": "message",
                "content": [
                    {
                        "type": "output_text",
                        "text": "First block without citation",
                        "annotations": []
                    },
                    {
                        "type": "output_text",
                        "text": "Second block cited",
                        "annotations": [{
                            "type": "url_citation",
                            "url": "https://example.com/second",
                            "title": "Second",
                            "start_index": 0,
                            "end_index": 6
                        }]
                    }
                ]
            }]
        }),
    }]));
    let provider = GrokSearchProvider::new(
        network,
        "https://api.x.ai".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
    );

    let response = provider
        .search(SearchRequest::new("grok", "lapis", 1))
        .await
        .expect("grok response");

    assert_eq!(response.results[0].snippet, "Second");
    assert_eq!(
        response.results[0].summary.as_deref(),
        Some("First block without citation\nSecond block cited")
    );
}

#[tokio::test]
async fn grok_search_ignores_unknown_content_and_annotations() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [{
                "type": "message",
                "content": [
                    { "type": "input_text", "text": "ignored" },
                    {
                        "type": "output_text",
                        "text": "Known text",
                        "annotations": [
                            { "type": "file_citation", "file_id": "file_1" },
                            {
                                "type": "url_citation",
                                "url": "https://example.com/known",
                                "start_index": 0,
                                "end_index": 5
                            }
                        ]
                    }
                ]
            }]
        }),
    }]));
    let provider = GrokSearchProvider::new(
        network,
        "https://api.x.ai".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
    );

    let response = provider
        .search(SearchRequest::new("grok", "lapis", 1))
        .await
        .expect("grok response");

    assert_eq!(response.results[0].title, "https://example.com/known");
    assert_eq!(response.results[0].snippet, "Known");
}

#[tokio::test]
async fn grok_search_dedupes_citations_and_limits_results() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "output": [{
                "type": "message",
                "content": [{
                    "type": "output_text",
                    "text": "Alpha Beta Gamma",
                    "annotations": [
                        {
                            "type": "url_citation",
                            "url": "https://example.com/alpha",
                            "title": "Alpha",
                            "start_index": 0,
                            "end_index": 5
                        },
                        {
                            "type": "url_citation",
                            "url": "https://example.com/alpha",
                            "title": "Alpha duplicate",
                            "start_index": 0,
                            "end_index": 5
                        },
                        {
                            "type": "url_citation",
                            "url": "https://example.com/beta",
                            "title": "Beta",
                            "start_index": 6,
                            "end_index": 10
                        }
                    ]
                }]
            }]
        }),
    }]));
    let provider = GrokSearchProvider::new(
        network,
        "https://api.x.ai".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
    );

    let response = provider
        .search(SearchRequest::new("grok", "lapis", 1))
        .await
        .expect("grok response");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].url.as_deref(),
        Some("https://example.com/alpha")
    );
}

#[tokio::test]
async fn grok_search_rejects_non_success_status() {
    let network = Arc::new(MockNetworkClient::new([NetworkResponse {
        status: 429,
        headers: vec![],
        body: json!({ "error": "rate limit" }),
    }]));
    let provider = GrokSearchProvider::new(
        network,
        "https://api.x.ai".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
    );

    let error = provider
        .search(SearchRequest::new("grok", "lapis", 1))
        .await
        .expect_err("grok status error");

    assert!(matches!(
        error,
        Error::HttpStatus {
            status: 429,
            retryable: true,
            ..
        }
    ));
}
