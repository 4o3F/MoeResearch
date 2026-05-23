use async_trait::async_trait;
use lapis_core::error::{Error, Result};
use lapis_core::net::client::MockNetworkClient;
use lapis_core::schema::network::NetworkResponse;
use lapis_core::schema::policy::SearchPolicy;
use lapis_core::schema::search::{
    ProviderSearchRequest, SearchRequest, SearchResponse, SearchResult,
};
use lapis_core::search::provider::SearchProvider;
use lapis_core::search::providers::{ExaSearchProvider, GrokSearchProvider};
use lapis_core::search::service::SearchService;
use serde_json::json;
use std::sync::Arc;

struct StaticProvider(&'static str);

#[async_trait]
impl SearchProvider for StaticProvider {
    fn name(&self) -> &'static str {
        self.0
    }

    async fn search(&self, _request: ProviderSearchRequest) -> Result<SearchResponse> {
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

#[tokio::test]
async fn searches_preferred_provider_first() {
    let mut service = SearchService::new();
    service.register(StaticProvider("exa"));
    service.register(StaticProvider("grok"));

    let policy = SearchPolicy {
        preferred_providers: vec!["grok".to_owned()],
        ..SearchPolicy::default()
    };
    let response = service
        .search(SearchRequest::from_query("lapis", 3), &policy)
        .await
        .expect("search response");

    assert_eq!(response.provider, "grok");
}

#[tokio::test]
async fn rejects_invalid_search_requests_before_provider_dispatch() {
    let mut service = SearchService::new();
    service.register(StaticProvider("exa"));

    for request in [
        SearchRequest::from_query(" ", 1),
        SearchRequest::from_query("lapis", 0),
        SearchRequest::from_query("lapis", 6),
    ] {
        let error = service
            .search(request, &SearchPolicy::default())
            .await
            .expect_err("invalid search request");

        assert!(matches!(error, Error::InvalidInput { .. }));
    }
}

#[tokio::test]
async fn rejects_invalid_search_policy_before_provider_dispatch() {
    let mut service = SearchService::new();
    service.register(StaticProvider("exa"));

    let zero_limit = SearchPolicy {
        max_results_per_query: 0,
        ..SearchPolicy::default()
    };
    let error = service
        .search(SearchRequest::from_query("lapis", 1), &zero_limit)
        .await
        .expect_err("invalid search policy");
    assert!(matches!(error, Error::InvalidInput { .. }));

    let overlapping_domains = SearchPolicy {
        include_domains: vec!["example.com".to_owned()],
        exclude_domains: vec!["EXAMPLE.com".to_owned()],
        ..SearchPolicy::default()
    };
    let error = service
        .search(SearchRequest::from_query("lapis", 1), &overlapping_domains)
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
    let policy = SearchPolicy {
        allowed_providers: vec!["exa".to_owned()],
        include_domains: vec!["example.com".to_owned()],
        exclude_domains: vec!["blocked.com".to_owned()],
        ..SearchPolicy::default()
    };

    service
        .search(SearchRequest::from_query("lapis", 1), &policy)
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
        .search(ProviderSearchRequest::from_policy(
            SearchRequest::from_query("lapis", 1),
            &SearchPolicy::default(),
        ))
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
            "results": [{
                "title": "Grok result",
                "url": "https://example.com/grok",
                "snippet": "snippet",
                "summary": "summary",
                "published_at": "2026-01-01"
            }]
        }),
    }]));
    let provider = GrokSearchProvider::new(
        network,
        "https://api.x.ai".to_owned(),
        "key".to_owned(),
        None,
    );

    let response = provider
        .search(ProviderSearchRequest::from_policy(
            SearchRequest::from_query("lapis", 1),
            &SearchPolicy::default(),
        ))
        .await
        .expect("grok response");

    assert_eq!(response.provider, "grok");
    assert_eq!(response.results[0].title, "Grok result");
}
