use async_trait::async_trait;
use lapis_core::error::Result;
use lapis_core::net::client::MockNetworkClient;
use lapis_core::schema::common::{NetworkResponse, SearchPolicy};
use lapis_core::schema::search::{SearchRequest, SearchResponse, SearchResult};
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
        .search(SearchRequest::from_query("lapis", 1))
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
        .search(SearchRequest::from_query("lapis", 1))
        .await
        .expect("grok response");

    assert_eq!(response.provider, "grok");
    assert_eq!(response.results[0].title, "Grok result");
}
