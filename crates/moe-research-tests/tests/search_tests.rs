mod support;

use async_trait::async_trait;
use moe_research_error::{Error, Result};
use moe_research_net::JsonNetworkResponse;
use moe_research_search::Freshness;
use moe_research_search::SearchProvider;
use moe_research_search::SearchService;
use moe_research_search::{
    ExaSearchProvider, GrokReasoningEffort, GrokSearchProvider, TavilySearchProvider,
};
use moe_research_search::{
    SearchCategory, SearchContentLevel, SearchDepth, SearchRecency, SearchRequest, SearchResponse,
    SearchResult,
};
use moe_research_workflow::SearchPolicy;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use support::network::{MockNetworkClient, mock_completed_sse, sse_json_event, sse_response};

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
        depth: None,
        content_level: None,
        recency: None,
        category: None,
        language: None,
        region: None,
        include_domains: Vec::new(),
        exclude_domains: Vec::new(),
    }
}

fn grok_provider(network: Arc<MockNetworkClient>) -> GrokSearchProvider {
    GrokSearchProvider::new(
        network,
        "https://api.x.ai/v1".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
    )
}

fn tavily_provider(network: Arc<MockNetworkClient>) -> TavilySearchProvider {
    TavilySearchProvider::new(
        network,
        "https://api.tavily.com".to_owned(),
        "key".to_owned(),
        None,
    )
}

async fn search_with_policy(
    service: &SearchService,
    request: SearchRequest,
    policy: &SearchPolicy,
) -> Result<SearchResponse> {
    service.search(policy.apply_to(request)?).await
}

#[tokio::test]
async fn dispatches_only_to_explicit_provider() {
    let mut service = SearchService::new();
    let exa_calls = Arc::new(AtomicUsize::new(0));
    let grok_calls = Arc::new(AtomicUsize::new(0));
    let tavily_calls = Arc::new(AtomicUsize::new(0));
    service.register(CountingProvider {
        name: "exa",
        calls: exa_calls.clone(),
    });
    service.register(CountingProvider {
        name: "grok",
        calls: grok_calls.clone(),
    });
    service.register(CountingProvider {
        name: "tavily",
        calls: tavily_calls.clone(),
    });

    let policy = search_policy(&["exa", "grok", "tavily"]);
    let response = search_with_policy(
        &service,
        SearchRequest::new("tavily", "moeresearch", 3),
        &policy,
    )
    .await
    .expect("search response");

    assert_eq!(response.provider, "tavily");
    assert_eq!(exa_calls.load(Ordering::SeqCst), 0);
    assert_eq!(grok_calls.load(Ordering::SeqCst), 0);
    assert_eq!(tavily_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn does_not_fallback_when_selected_provider_fails() {
    let mut service = SearchService::new();
    service.register(FailingProvider("exa"));
    service.register(StaticProvider("grok"));

    let policy = search_policy(&["exa", "grok"]);
    let error = search_with_policy(
        &service,
        SearchRequest::new("exa", "moeresearch", 3),
        &policy,
    )
    .await
    .expect_err("selected provider error");

    assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "exa"));
}

#[tokio::test]
async fn rejects_missing_explicit_provider() {
    let mut service = SearchService::new();
    service.register(StaticProvider("exa"));

    let error = search_with_policy(
        &service,
        SearchRequest::new("", "moeresearch", 1),
        &search_policy(&["exa"]),
    )
    .await
    .expect_err("missing search provider");

    assert!(matches!(error, Error::InvalidInput { .. }));
}

#[tokio::test]
async fn rejects_empty_allowlist_for_explicit_provider() {
    let mut service = SearchService::new();
    service.register(StaticProvider("exa"));

    let policy = search_policy(&[]);
    let error = search_with_policy(
        &service,
        SearchRequest::new("exa", "moeresearch", 1),
        &policy,
    )
    .await
    .expect_err("empty allowlist rejects provider");

    assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "exa"));
}

#[tokio::test]
async fn rejects_disallowed_explicit_provider() {
    let mut service = SearchService::new();
    service.register(StaticProvider("grok"));

    let policy = search_policy(&["exa"]);
    let error = search_with_policy(
        &service,
        SearchRequest::new("grok", "moeresearch", 1),
        &policy,
    )
    .await
    .expect_err("disallowed search provider");

    assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "grok"));
}

#[tokio::test]
async fn rejects_unconfigured_explicit_provider() {
    let service = SearchService::new();

    let error = search_with_policy(
        &service,
        SearchRequest::new("exa", "moeresearch", 1),
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
        SearchRequest::new("exa", "moeresearch", 0),
        SearchRequest::new("exa", "moeresearch", 6),
    ] {
        let error = search_with_policy(&service, request, &search_policy(&["exa"]))
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
    let error = search_with_policy(
        &service,
        SearchRequest::new("exa", "moeresearch", 1),
        &zero_limit,
    )
    .await
    .expect_err("invalid search policy");
    assert!(matches!(error, Error::InvalidInput { .. }));

    let mut overlapping_domains = search_policy(&["exa"]);
    overlapping_domains.include_domains = vec!["example.com".to_owned()];
    overlapping_domains.exclude_domains = vec!["EXAMPLE.com".to_owned()];
    let error = search_with_policy(
        &service,
        SearchRequest::new("exa", "moeresearch", 1),
        &overlapping_domains,
    )
    .await
    .expect_err("invalid search policy");
    assert!(matches!(error, Error::InvalidInput { .. }));
}

#[test]
fn search_request_defaults_call_time_params_to_none() {
    let request = SearchRequest::new("exa", "moeresearch", 1);

    assert_eq!(request.depth, None);
    assert_eq!(request.content_level, None);
    assert_eq!(request.recency, None);
    assert_eq!(request.category, None);
}

#[tokio::test]
async fn search_policy_applies_call_time_defaults() {
    let mut policy = search_policy(&["exa"]);
    policy.depth = Some(SearchDepth::Balanced);
    policy.content_level = Some(SearchContentLevel::Standard);
    policy.recency = Some(SearchRecency::Recent);
    policy.category = Some(SearchCategory::News);

    let request = policy
        .apply_to(SearchRequest::new("exa", "moeresearch", 1))
        .expect("policy applies");

    assert_eq!(request.depth, Some(SearchDepth::Balanced));
    assert_eq!(request.content_level, Some(SearchContentLevel::Standard));
    assert_eq!(request.recency, Some(SearchRecency::Recent));
    assert_eq!(request.category, Some(SearchCategory::News));
}

#[tokio::test]
async fn search_policy_rejects_call_time_params_above_policy() {
    let mut policy = search_policy(&["exa"]);
    policy.depth = Some(SearchDepth::Balanced);
    policy.content_level = Some(SearchContentLevel::Standard);
    policy.recency = Some(SearchRecency::Recent);

    let mut depth_request = SearchRequest::new("exa", "moeresearch", 1);
    depth_request.depth = Some(SearchDepth::HighRecall);
    assert!(matches!(
        policy.apply_to(depth_request),
        Err(Error::InvalidInput { .. })
    ));

    let mut content_request = SearchRequest::new("exa", "moeresearch", 1);
    content_request.content_level = Some(SearchContentLevel::Detailed);
    assert!(matches!(
        policy.apply_to(content_request),
        Err(Error::InvalidInput { .. })
    ));

    let mut recency_request = SearchRequest::new("exa", "moeresearch", 1);
    recency_request.recency = Some(SearchRecency::Live);
    assert!(matches!(
        policy.apply_to(recency_request),
        Err(Error::InvalidInput { .. })
    ));
}

#[tokio::test]
async fn search_policy_rejects_category_conflict() {
    let mut policy = search_policy(&["exa"]);
    policy.category = Some(SearchCategory::News);
    let mut request = SearchRequest::new("exa", "moeresearch", 1);
    request.category = Some(SearchCategory::Academic);

    assert!(matches!(
        policy.apply_to(request),
        Err(Error::InvalidInput { .. })
    ));
}

#[tokio::test]
async fn forwards_policy_domain_filters_to_exa_provider() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
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

    search_with_policy(
        &service,
        SearchRequest::new("exa", "moeresearch", 1),
        &policy,
    )
    .await
    .expect("search response");

    let request_body = network.requests()[0].body.clone().expect("request body");
    assert_eq!(request_body["includeDomains"], json!(["example.com"]));
    assert_eq!(request_body["excludeDomains"], json!(["blocked.com"]));
}

#[tokio::test]
async fn maps_exa_response_to_standard_search_response() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "results": [{
                "title": "MoeResearch",
                "url": "https://example.com/moeresearch",
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
        .search(SearchRequest::new("exa", "moeresearch", 1))
        .await
        .expect("exa response");

    assert_eq!(response.provider, "exa");
    assert_eq!(response.results[0].title, "MoeResearch");
}

#[tokio::test]
async fn tavily_search_uses_json_search_request() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "results": [] }),
    }]));
    let provider = TavilySearchProvider::new(
        network.clone(),
        "https://api.tavily.com/".to_owned(),
        "key".to_owned(),
        Some(1000),
    );
    let mut policy = search_policy(&["tavily"]);
    policy.freshness = Some(Freshness {
        since: Some("2026-01-01".to_owned()),
        until: Some("2026-01-31".to_owned()),
    });
    policy.depth = Some(SearchDepth::HighRecall);
    policy.content_level = Some(SearchContentLevel::Detailed);
    policy.recency = Some(SearchRecency::Fresh);
    policy.category = Some(SearchCategory::FinancialFilings);
    policy.include_domains = vec!["example.com".to_owned()];
    policy.exclude_domains = vec!["blocked.com".to_owned()];

    provider
        .search(
            policy
                .apply_to(SearchRequest::new("tavily", "moeresearch", 2))
                .expect("policy applies"),
        )
        .await
        .expect("tavily response");

    let requests = network.requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request.method, "POST");
    assert_eq!(request.url, "https://api.tavily.com/search");
    assert_eq!(request.inactivity_timeout_ms, Some(1000));
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
    assert!(
        request
            .headers
            .iter()
            .any(|header| { header.name == "accept" && header.value == "application/json" })
    );

    let body = request.body.as_ref().expect("request body");
    assert_eq!(body["query"], "moeresearch");
    assert_eq!(body["max_results"], 2);
    assert_eq!(body["search_depth"], "advanced");
    assert_eq!(body["topic"], "finance");
    assert_eq!(body["start_date"], "2026-01-01");
    assert_eq!(body["end_date"], "2026-01-31");
    assert_eq!(body["include_domains"], json!(["example.com"]));
    assert_eq!(body["exclude_domains"], json!(["blocked.com"]));
    assert_eq!(body["include_raw_content"], true);
    assert!(body.get("time_range").is_none());
}

#[tokio::test]
async fn tavily_uses_time_range_only_without_explicit_dates() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "results": [] }),
    }]));
    let provider = tavily_provider(network.clone());
    let mut request = SearchRequest::new("tavily", "moeresearch", 1);
    request.recency = Some(SearchRecency::Fresh);
    request.content_level = Some(SearchContentLevel::Standard);
    request.category = Some(SearchCategory::People);

    provider.search(request).await.expect("tavily response");

    let body = network.requests()[0].body.clone().expect("request body");
    assert_eq!(body["time_range"], "day");
    assert!(body.get("start_date").is_none());
    assert!(body.get("end_date").is_none());
    assert!(body.get("include_raw_content").is_none());
    assert!(body.get("topic").is_none());
}

#[tokio::test]
async fn maps_tavily_response_to_standard_search_response() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "results": [
                {
                    "title": "MoeResearch",
                    "url": "https://example.com/moeresearch",
                    "content": "snippet",
                    "raw_content": "summary",
                    "published_date": "2026-01-01"
                },
                {
                    "title": "Legacy MoeResearch",
                    "link": "https://example.com/legacy",
                    "snippet": "legacy snippet"
                }
            ]
        }),
    }]));
    let provider = tavily_provider(network);

    let response = provider
        .search(SearchRequest::new("tavily", "moeresearch", 5))
        .await
        .expect("tavily response");

    assert_eq!(response.provider, "tavily");
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.results[0].title, "MoeResearch");
    assert_eq!(
        response.results[0].url.as_deref(),
        Some("https://example.com/moeresearch")
    );
    assert_eq!(response.results[0].snippet, "snippet");
    assert_eq!(response.results[0].summary.as_deref(), Some("summary"));
    assert_eq!(
        response.results[0].published_at.as_deref(),
        Some("2026-01-01")
    );
    assert_eq!(
        response.results[1].url.as_deref(),
        Some("https://example.com/legacy")
    );
    assert_eq!(response.results[1].snippet, "legacy snippet");
}

#[tokio::test]
async fn tavily_raw_content_summary_is_bounded() {
    let raw_content = "alpha ".repeat(900);
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({
            "results": [{
                "title": "Long",
                "url": "https://example.com/long",
                "content": "snippet",
                "raw_content": raw_content
            }]
        }),
    }]));
    let provider = tavily_provider(network);

    let response = provider
        .search(SearchRequest::new("tavily", "moeresearch", 1))
        .await
        .expect("tavily response");

    let summary = response.results[0].summary.as_deref().expect("summary");
    assert!(summary.len() <= 4_003);
    assert!(summary.ends_with('…'));
}

#[tokio::test]
async fn maps_grok_response_to_standard_search_response() {
    let network = mock_completed_sse(json!({
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
    }));
    let provider = grok_provider(network);

    let response = provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
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
    let network = mock_completed_sse(json!({ "output": [] }));
    let provider = GrokSearchProvider::new(
        network.clone(),
        "https://api.x.ai/v1/".to_owned(),
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
        .search(
            policy
                .apply_to(SearchRequest::new("grok", "moeresearch", 2))
                .expect("policy applies"),
        )
        .await
        .expect("grok response");

    let requests = network.requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request.method, "POST");
    assert_eq!(request.url, "https://api.x.ai/v1/responses");
    assert_eq!(request.inactivity_timeout_ms, Some(1000));
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
    assert!(
        request
            .headers
            .iter()
            .any(|header| { header.name == "accept" && header.value == "text/event-stream" })
    );

    let body = request.body.as_ref().expect("request body");
    assert_eq!(body["model"], "configured-grok-model");
    assert_eq!(body["stream"], true);
    assert_eq!(body["input"][0]["role"], "user");
    assert_eq!(body["tools"][0]["type"], "web_search");
    assert!(body["tools"][0].get("search_context_size").is_none());
    assert_eq!(
        body["tools"][0]["filters"]["allowed_domains"],
        json!(["example.com"])
    );
    let prompt = body["input"][0]["content"].as_str().expect("prompt");
    assert!(prompt.contains("Search the web for: moeresearch"));
    assert!(prompt.contains("Maximum results: 2"));
    assert!(prompt.contains("Language: en"));
    assert!(prompt.contains("Region: US"));
    assert!(prompt.contains("Exclude domains: blocked.com"));
}

#[tokio::test]
async fn grok_search_uses_annotation_local_text_for_snippets() {
    let network = mock_completed_sse(json!({
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
    }));
    let provider = grok_provider(network);

    let response = provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
        .await
        .expect("grok response");

    // Snippet is the Tight excerpt around the citation indices.
    assert_eq!(response.results[0].snippet, "Second");
    // Summary is the per-source Wide excerpt of the LOCAL output_text
    // that owns the citation, not the accumulated full_text — so two
    // evidence rows in the same response no longer share one 1 KiB blob.
    assert_eq!(
        response.results[0].summary.as_deref(),
        Some("Second block cited")
    );
}

/// Two citations anchored at different positions inside the same
/// `output_text` MUST yield different per-source summaries when the
/// text exceeds the summary budget. This pins the regression where
/// every evidence row used to share an identical `full_text` blob,
/// wasting Layer 2's prompt budget on duplicated context.
#[tokio::test]
async fn grok_search_emits_distinct_per_source_summaries() {
    // Build a text well over SUMMARY_MAX_BYTES (600) so the summary
    // window cannot span both citations. The two anchors sit at
    // opposite ends so their local windows have no overlap.
    let prefix: String = "alpha ".repeat(80);
    let middle = "GAMMA ";
    let suffix: String = "omega ".repeat(80);
    let text = format!("{prefix}{middle}{suffix}");
    let suffix_start = prefix.len() + middle.len();
    let tail_start = text.len() - 6;

    let network = mock_completed_sse(json!({
        "output": [{
            "type": "message",
            "content": [{
                "type": "output_text",
                "text": text,
                "annotations": [
                    {
                        "type": "url_citation",
                        "url": "https://example.com/start",
                        "title": "Start",
                        "start_index": 0,
                        "end_index": 5
                    },
                    {
                        "type": "url_citation",
                        "url": "https://example.com/end",
                        "title": "End",
                        "start_index": tail_start,
                        "end_index": text.len()
                    }
                ]
            }]
        }]
    }));
    let provider = grok_provider(network);

    let response = provider
        .search(SearchRequest::new("grok", "moeresearch", 2))
        .await
        .expect("grok response");

    assert_eq!(response.results.len(), 2);
    let s0 = response.results[0]
        .summary
        .as_deref()
        .expect("first summary");
    let s1 = response.results[1]
        .summary
        .as_deref()
        .expect("second summary");
    assert_ne!(s0, s1, "per-source summaries must diverge");
    assert!(
        s0.contains("alpha"),
        "first citation's window covers the prefix"
    );
    assert!(
        !s0.contains("omega"),
        "first citation's window must not reach the suffix at offset {suffix_start}"
    );
    assert!(
        s1.contains("omega"),
        "second citation's window covers the suffix"
    );
    assert!(
        !s1.contains("alpha"),
        "second citation's window must not reach back to the prefix"
    );
}

#[tokio::test]
async fn grok_search_ignores_unknown_content_and_annotations() {
    let network = mock_completed_sse(json!({
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
    }));
    let provider = grok_provider(network);

    let response = provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
        .await
        .expect("grok response");

    assert_eq!(response.results[0].title, "https://example.com/known");
    assert_eq!(response.results[0].snippet, "Known");
}

#[tokio::test]
async fn grok_search_dedupes_citations_and_limits_results() {
    let network = mock_completed_sse(json!({
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
    }));
    let provider = grok_provider(network);

    let response = provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
        .await
        .expect("grok response");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].url.as_deref(),
        Some("https://example.com/alpha")
    );
}

/// Grok's `search_sources` array discloses every URL the model consulted,
/// including pages it did not cite inline via `url_citation`. When citations
/// underfill `max_results`, these sources MUST be appended so high-value
/// references (e.g. reddit/substack threads) are not silently dropped.
#[tokio::test]
async fn grok_search_appends_search_sources_when_citations_underfill() {
    let network = mock_completed_sse(json!({
        "output": [{
            "type": "message",
            "content": [{
                "type": "output_text",
                "text": "Cited summary",
                "annotations": [{
                    "type": "url_citation",
                    "url": "https://example.com/cited",
                    "title": "Cited",
                    "start_index": 0,
                    "end_index": 5
                }]
            }],
            "search_sources": [
                { "url": "https://example.com/cited",  "title": "Duplicate",  "type": "web" },
                { "url": "https://example.com/reddit", "title": "Reddit",     "type": "web" },
                { "url": "https://example.com/sub",    "title": "Substack",   "type": "web" }
            ]
        }]
    }));
    let provider = grok_provider(network);

    let response = provider
        .search(SearchRequest::new("grok", "moeresearch", 5))
        .await
        .expect("grok response");

    // 1 cited + 2 fallback sources (third source is a URL duplicate of the
    // citation and is dropped by dedupe).
    assert_eq!(response.results.len(), 3);
    assert_eq!(
        response.results[0].url.as_deref(),
        Some("https://example.com/cited"),
        "citation must rank first"
    );
    assert_eq!(
        response.results[1].url.as_deref(),
        Some("https://example.com/reddit"),
        "first non-duplicate fallback source is appended"
    );
    assert_eq!(response.results[1].title, "Reddit");
    assert_eq!(response.results[1].snippet, "Reddit");
    assert_eq!(
        response.results[1].summary.as_deref(),
        Some("Cited summary"),
        "fallback inherits the message-level narrative"
    );
    assert_eq!(
        response.results[2].url.as_deref(),
        Some("https://example.com/sub")
    );
}

/// `max_results` is the hard cap on the total returned vector — citations
/// fill first, and fallback sources MUST NOT cause an overflow.
#[tokio::test]
async fn grok_search_search_sources_respect_max_results_cap() {
    let network = mock_completed_sse(json!({
        "output": [{
            "type": "message",
            "content": [{
                "type": "output_text",
                "text": "Alpha cited",
                "annotations": [{
                    "type": "url_citation",
                    "url": "https://example.com/alpha",
                    "title": "Alpha",
                    "start_index": 0,
                    "end_index": 5
                }]
            }],
            "search_sources": [
                { "url": "https://example.com/beta",  "title": "Beta",  "type": "web" },
                { "url": "https://example.com/gamma", "title": "Gamma", "type": "web" }
            ]
        }]
    }));
    let provider = grok_provider(network);

    let response = provider
        .search(SearchRequest::new("grok", "moeresearch", 2))
        .await
        .expect("grok response");

    assert_eq!(response.results.len(), 2);
    assert_eq!(
        response.results[0].url.as_deref(),
        Some("https://example.com/alpha")
    );
    assert_eq!(
        response.results[1].url.as_deref(),
        Some("https://example.com/beta"),
        "gamma is dropped because cap was reached"
    );
}

#[tokio::test]
async fn grok_search_rejects_non_success_status() {
    let network = Arc::new(MockNetworkClient::new_sse([sse_response(429, vec![])]));
    let provider = grok_provider(network);

    let error = provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
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

/// When `freshness.since` and `freshness.until` are both supplied, the Exa
/// request body MUST include `startPublishedDate` and `endPublishedDate`
/// so Exa applies the date window server-side.
#[tokio::test]
async fn grok_sse_terminal_failure_returns_provider_error() {
    let network = Arc::new(MockNetworkClient::new_sse([sse_response(
        200,
        vec![sse_json_event(
            "response.incomplete",
            json!({ "type": "response.incomplete" }),
        )],
    )]));
    let provider = grok_provider(network);

    let error = provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
        .await
        .expect_err("terminal failure errors");

    assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "grok"));
}

#[tokio::test]
async fn grok_sse_error_event_returns_provider_error() {
    let network = Arc::new(MockNetworkClient::new_sse([sse_response(
        200,
        vec![sse_json_event("error", json!({ "type": "error" }))],
    )]));
    let provider = grok_provider(network);

    let error = provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
        .await
        .expect_err("error event errors");

    assert!(matches!(error, Error::ProviderUnavailable { provider, .. } if provider == "grok"));
}

#[tokio::test]
async fn grok_sse_missing_terminal_event_errors() {
    let network = Arc::new(MockNetworkClient::new_sse([sse_response(
        200,
        vec![sse_json_event(
            "response.created",
            json!({ "type": "response.created" }),
        )],
    )]));
    let provider = grok_provider(network);

    let error = provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
        .await
        .expect_err("missing terminal errors");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
}

#[tokio::test]
async fn grok_sse_completed_missing_response_errors() {
    let network = Arc::new(MockNetworkClient::new_sse([sse_response(
        200,
        vec![sse_json_event(
            "response.completed",
            json!({ "type": "response.completed" }),
        )],
    )]));
    let provider = grok_provider(network);

    let error = provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
        .await
        .expect_err("missing response errors");

    assert!(matches!(error, Error::SchemaValidationFailed { .. }));
}

#[tokio::test]
async fn exa_request_includes_start_and_end_published_date_from_freshness_since_until() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "results": [] }),
    }]));
    let provider = ExaSearchProvider::new(
        network.clone(),
        "https://api.exa.ai".to_owned(),
        "key".to_owned(),
        None,
    );
    let mut request = SearchRequest::new("exa", "moeresearch", 1);
    request.freshness = Some(Freshness {
        since: Some("2024-01-01".to_owned()),
        until: Some("2024-12-31".to_owned()),
    });

    provider.search(request).await.expect("exa response");

    let body = network.requests()[0].body.clone().expect("request body");
    assert_eq!(body["startPublishedDate"], json!("2024-01-01"));
    assert_eq!(body["endPublishedDate"], json!("2024-12-31"));
}

/// When no freshness is supplied, the Exa request body MUST omit
/// `startPublishedDate` and `endPublishedDate` entirely (not send
/// `null`), so Exa applies its default unbounded window.
#[tokio::test]
async fn exa_request_omits_dates_when_freshness_is_none() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "results": [] }),
    }]));
    let provider = ExaSearchProvider::new(
        network.clone(),
        "https://api.exa.ai".to_owned(),
        "key".to_owned(),
        None,
    );

    provider
        .search(SearchRequest::new("exa", "moeresearch", 1))
        .await
        .expect("exa response");

    let body = network.requests()[0].body.clone().expect("request body");
    assert!(body.get("startPublishedDate").is_none());
    assert!(body.get("endPublishedDate").is_none());
}

/// Half-open freshness windows (since-only) must still be forwarded to Exa,
/// pinning the contract that providers honor either bound independently.
#[tokio::test]
async fn exa_request_forwards_since_only_freshness_window() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "results": [] }),
    }]));
    let provider = ExaSearchProvider::new(
        network.clone(),
        "https://api.exa.ai".to_owned(),
        "key".to_owned(),
        None,
    );
    let mut request = SearchRequest::new("exa", "moeresearch", 1);
    request.freshness = Some(Freshness {
        since: Some("2025-06-01".to_owned()),
        until: None,
    });

    provider.search(request).await.expect("exa response");

    let body = network.requests()[0].body.clone().expect("request body");
    assert_eq!(body["startPublishedDate"], json!("2025-06-01"));
    assert!(body.get("endPublishedDate").is_none());
}

#[tokio::test]
async fn exa_request_maps_call_time_params_to_private_fields() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "results": [] }),
    }]));
    let provider = ExaSearchProvider::new(
        network.clone(),
        "https://api.exa.ai".to_owned(),
        "key".to_owned(),
        None,
    );
    let mut request = SearchRequest::new("exa", "moeresearch", 1);
    request.depth = Some(SearchDepth::LowLatency);
    request.content_level = Some(SearchContentLevel::Detailed);
    request.recency = Some(SearchRecency::Live);
    request.category = Some(SearchCategory::News);

    provider.search(request).await.expect("exa response");

    let body = network.requests()[0].body.clone().expect("request body");
    assert_eq!(body["type"], json!("instant"));
    assert_eq!(body["contents"]["highlights"], json!(true));
    assert_eq!(body["contents"]["text"]["maxCharacters"], json!(4000));
    assert_eq!(body["contents"]["maxAgeHours"], json!(0));
    assert_eq!(body["category"], json!("news"));
}

#[tokio::test]
async fn exa_request_maps_code_category_to_private_hint() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "results": [] }),
    }]));
    let provider = ExaSearchProvider::new(
        network.clone(),
        "https://api.exa.ai".to_owned(),
        "key".to_owned(),
        None,
    );
    let mut request = SearchRequest::new("exa", "moeresearch", 1);
    request.category = Some(SearchCategory::Code);

    provider.search(request).await.expect("exa response");

    let body = network.requests()[0].body.clone().expect("request body");
    assert_eq!(body["category"], json!("code"));
}

#[tokio::test]
async fn exa_people_category_rejects_non_linkedin_include_domains() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "results": [] }),
    }]));
    let provider = ExaSearchProvider::new(
        network,
        "https://api.exa.ai".to_owned(),
        "key".to_owned(),
        None,
    );
    let mut request = SearchRequest::new("exa", "moeresearch", 1);
    request.category = Some(SearchCategory::People);
    request.include_domains = vec!["notlinkedin.com".to_owned()];

    assert!(matches!(
        provider.search(request).await,
        Err(Error::InvalidInput { .. })
    ));
}

#[tokio::test]
async fn exa_request_maps_high_recall_to_normal_search_type() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "results": [] }),
    }]));
    let provider = ExaSearchProvider::new(
        network.clone(),
        "https://api.exa.ai".to_owned(),
        "key".to_owned(),
        None,
    );
    let mut request = SearchRequest::new("exa", "moeresearch", 1);
    request.depth = Some(SearchDepth::HighRecall);

    provider.search(request).await.expect("exa response");

    let body = network.requests()[0].body.clone().expect("request body");
    assert_eq!(body["type"], json!("auto"));
    assert_ne!(body["type"], json!("deep"));
    assert_ne!(body["type"], json!("deep-lite"));
    assert_ne!(body["type"], json!("deep-reasoning"));
}

#[tokio::test]
async fn exa_request_keeps_recency_distinct_from_freshness_dates() {
    let network = Arc::new(MockNetworkClient::new([JsonNetworkResponse {
        status: 200,
        headers: vec![],
        body: json!({ "results": [] }),
    }]));
    let provider = ExaSearchProvider::new(
        network.clone(),
        "https://api.exa.ai".to_owned(),
        "key".to_owned(),
        None,
    );
    let mut request = SearchRequest::new("exa", "moeresearch", 1);
    request.recency = Some(SearchRecency::Fresh);
    request.freshness = Some(Freshness {
        since: Some("2026-01-01".to_owned()),
        until: Some("2026-01-31".to_owned()),
    });

    provider.search(request).await.expect("exa response");

    let body = network.requests()[0].body.clone().expect("request body");
    assert_eq!(body["contents"]["maxAgeHours"], json!(24));
    assert_eq!(body["startPublishedDate"], json!("2026-01-01"));
    assert_eq!(body["endPublishedDate"], json!("2026-01-31"));
}

/// Grok's prompt-based search MUST include a freshness window phrase when
/// the request carries one, so the model's downstream search respects the
/// date range.
#[tokio::test]
async fn grok_search_prompt_includes_freshness_window_when_present() {
    let network = mock_completed_sse(json!({ "output": [] }));
    let provider = GrokSearchProvider::new(
        network.clone(),
        "https://api.x.ai/v1".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
    );
    let mut request = SearchRequest::new("grok", "moeresearch", 1);
    request.freshness = Some(Freshness {
        since: Some("2024-01-01".to_owned()),
        until: Some("2024-12-31".to_owned()),
    });

    provider.search(request).await.expect("grok response");

    let body = network.requests()[0].body.clone().expect("request body");
    let prompt = body["input"][0]["content"].as_str().expect("prompt");
    assert!(prompt.contains("Freshness:"));
    assert!(prompt.contains("between 2024-01-01 and 2024-12-31"));
}

/// When no freshness is supplied, the Grok prompt MUST omit the freshness
/// line entirely so the model does not invent constraints.
#[tokio::test]
async fn grok_search_prompt_omits_freshness_when_none() {
    let network = mock_completed_sse(json!({ "output": [] }));
    let provider = GrokSearchProvider::new(
        network.clone(),
        "https://api.x.ai/v1".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
    );

    provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
        .await
        .expect("grok response");

    let body = network.requests()[0].body.clone().expect("request body");
    let prompt = body["input"][0]["content"].as_str().expect("prompt");
    assert!(!prompt.contains("Freshness:"));
}

/// Grok's `max_output_tokens` from config MUST appear in the request body
/// so operators can cap response cost.
#[tokio::test]
async fn grok_search_request_uses_configured_max_output_tokens() {
    let network = mock_completed_sse(json!({ "output": [] }));
    let provider = GrokSearchProvider::with_max_output_tokens(
        network.clone(),
        "https://api.x.ai/v1".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
        Some(2048),
    );

    provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
        .await
        .expect("grok response");

    let body = network.requests()[0].body.clone().expect("request body");
    assert_eq!(body["max_output_tokens"], json!(2048));
}

#[tokio::test]
async fn grok_search_request_uses_configured_reasoning_effort() {
    for (effort, expected) in [
        (GrokReasoningEffort::None, "none"),
        (GrokReasoningEffort::Low, "low"),
        (GrokReasoningEffort::Medium, "medium"),
        (GrokReasoningEffort::High, "high"),
    ] {
        let network = mock_completed_sse(json!({ "output": [] }));
        let provider = GrokSearchProvider::with_request_options(
            network.clone(),
            "https://api.x.ai/v1".to_owned(),
            "key".to_owned(),
            None,
            "configured-grok-model".to_owned(),
            None,
            Some(effort),
        );

        provider
            .search(SearchRequest::new("grok", "moeresearch", 1))
            .await
            .expect("grok response");

        let body = network.requests()[0].body.clone().expect("request body");
        assert_eq!(body["reasoning"]["effort"], json!(expected));
    }
}

/// When `max_output_tokens` is not configured, the field MUST be omitted
/// from the wire request (not sent as `null`), so the upstream provider
/// applies its own default.
#[tokio::test]
async fn grok_search_request_omits_max_output_tokens_when_unconfigured() {
    let network = mock_completed_sse(json!({ "output": [] }));
    let provider = GrokSearchProvider::new(
        network.clone(),
        "https://api.x.ai/v1".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
    );

    provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
        .await
        .expect("grok response");

    let body = network.requests()[0].body.clone().expect("request body");
    assert!(body.get("max_output_tokens").is_none());
    assert!(body.get("reasoning").is_none());
    assert!(body["tools"][0].get("search_context_size").is_none());
    assert!(body["tools"][0].get("contents").is_none());
    assert!(body["tools"][0].get("maxAgeHours").is_none());
    assert_eq!(body["tools"][0]["type"], json!("web_search"));
}

#[tokio::test]
async fn grok_search_request_omits_reasoning_when_unconfigured() {
    let network = mock_completed_sse(json!({ "output": [] }));
    let provider = GrokSearchProvider::new(
        network.clone(),
        "https://api.x.ai/v1".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
    );

    provider
        .search(SearchRequest::new("grok", "moeresearch", 1))
        .await
        .expect("grok response");

    let body = network.requests()[0].body.clone().expect("request body");
    assert!(body.get("reasoning").is_none());
}

#[tokio::test]
async fn grok_search_prompt_uses_neutral_call_time_hints() {
    let network = mock_completed_sse(json!({ "output": [] }));
    let provider = GrokSearchProvider::new(
        network.clone(),
        "https://api.x.ai/v1".to_owned(),
        "key".to_owned(),
        None,
        "configured-grok-model".to_owned(),
    );
    let mut request = SearchRequest::new("grok", "moeresearch", 1);
    request.depth = Some(SearchDepth::Balanced);
    request.content_level = Some(SearchContentLevel::Compact);
    request.recency = Some(SearchRecency::Recent);
    request.category = Some(SearchCategory::News);

    provider.search(request).await.expect("grok response");

    let body = network.requests()[0].body.clone().expect("request body");
    let prompt = body["input"][0]["content"].as_str().expect("prompt");
    assert!(prompt.contains("Search depth preference:"));
    assert!(prompt.contains("Content detail preference:"));
    assert!(prompt.contains("Source recency preference:"));
    assert!(prompt.contains("Category focus: news and current-events sources"));
    assert!(!prompt.contains("maxAgeHours"));
}
