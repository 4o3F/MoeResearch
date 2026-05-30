use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use lapis_error::{Error, JsonSnafu, Result};
use lapis_net::NetworkClient;
use lapis_net::provider_http::bearer_json_post;

use crate::{
    SearchCategory, SearchContentLevel, SearchDepth, SearchProvider, SearchRecency, SearchRequest,
    SearchResponse, SearchResult,
};

pub struct ExaSearchProvider {
    network: Arc<dyn NetworkClient>,
    base_url: String,
    api_key: String,
    timeout_ms: Option<u64>,
}

impl ExaSearchProvider {
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
}

#[async_trait]
impl SearchProvider for ExaSearchProvider {
    fn name(&self) -> &'static str {
        "exa"
    }

    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        validate_exa_category_conflicts(&request)?;

        let (start_published_date, end_published_date) = match request.freshness.as_ref() {
            Some(freshness) => (freshness.since.clone(), freshness.until.clone()),
            None => (None, None),
        };
        let body = serde_json::to_value(ExaSearchRequest {
            query: request.query,
            num_results: request.max_results,
            include_domains: request.include_domains,
            exclude_domains: request.exclude_domains,
            search_type: exa_search_type(request.depth),
            contents: exa_contents(request.content_level, request.recency),
            category: request.category.and_then(exa_category),
            start_published_date,
            end_published_date,
        })
        .context(JsonSnafu)?;

        let response = self
            .network
            .send_json(bearer_json_post(
                &self.base_url,
                "search",
                &self.api_key,
                body,
                self.timeout_ms,
            ))
            .await?;

        let provider_response: ExaSearchResponse =
            serde_json::from_value(response.body).context(JsonSnafu)?;

        Ok(SearchResponse {
            provider: self.name().to_owned(),
            results: provider_response
                .results
                .into_iter()
                .map(|result| {
                    let snippet = result
                        .text
                        .clone()
                        .or_else(|| result.highlights.first().cloned())
                        .unwrap_or_default();
                    SearchResult {
                        title: result
                            .title
                            .unwrap_or_else(|| result.url.clone().unwrap_or_default()),
                        url: result.url,
                        snippet,
                        summary: result.summary,
                        published_at: result.published_date,
                    }
                })
                .collect(),
        })
    }
}

fn validate_exa_category_conflicts(request: &SearchRequest) -> Result<()> {
    match request.category {
        Some(SearchCategory::Organizations) => {
            if request.freshness.is_some() || !request.exclude_domains.is_empty() {
                return Err(Error::InvalidInput {
                    message: "search category organizations cannot be combined with publish-date or exclude-domain filters for Exa".to_owned(),
                });
            }
        }
        Some(SearchCategory::People) => {
            if request.freshness.is_some() || !request.exclude_domains.is_empty() {
                return Err(Error::InvalidInput {
                    message: "search category people cannot be combined with publish-date or exclude-domain filters for Exa".to_owned(),
                });
            }
            if request
                .include_domains
                .iter()
                .any(|domain| !is_linkedin_domain_filter(domain))
            {
                return Err(Error::InvalidInput {
                    message:
                        "search category people only supports LinkedIn include_domains for Exa"
                            .to_owned(),
                });
            }
        }
        _ => {}
    }

    Ok(())
}

fn is_linkedin_domain_filter(domain: &str) -> bool {
    let host = domain
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("*.")
        .split('/')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();

    host == "linkedin.com" || host.ends_with(".linkedin.com")
}

fn exa_search_type(depth: Option<SearchDepth>) -> Option<&'static str> {
    match depth {
        None => None,
        Some(SearchDepth::LowLatency) => Some("instant"),
        Some(SearchDepth::Balanced | SearchDepth::HighRecall) => Some("auto"),
    }
}

fn exa_category(category: SearchCategory) -> Option<String> {
    match category {
        SearchCategory::Organizations => Some("company".to_owned()),
        SearchCategory::People => Some("people".to_owned()),
        SearchCategory::Academic => Some("research paper".to_owned()),
        SearchCategory::News => Some("news".to_owned()),
        SearchCategory::PersonalSites => Some("personal site".to_owned()),
        SearchCategory::FinancialFilings => Some("financial report".to_owned()),
        SearchCategory::Code => Some("code".to_owned()),
    }
}

fn exa_max_age_hours(recency: Option<SearchRecency>) -> Option<i32> {
    match recency {
        None | Some(SearchRecency::Default) => None,
        Some(SearchRecency::Live) => Some(0),
        Some(SearchRecency::Fresh) => Some(24),
        Some(SearchRecency::Recent) => Some(24 * 7),
        Some(SearchRecency::Cached) => Some(-1),
    }
}

fn exa_contents(
    level: Option<SearchContentLevel>,
    recency: Option<SearchRecency>,
) -> Option<ExaContents> {
    let max_age_hours = exa_max_age_hours(recency);
    match (level, max_age_hours) {
        (None, None) => None,
        (None, Some(max_age_hours)) => Some(ExaContents {
            highlights: None,
            text: None,
            max_age_hours: Some(max_age_hours),
        }),
        (Some(SearchContentLevel::Compact | SearchContentLevel::Standard), max_age_hours) => {
            Some(ExaContents {
                highlights: Some(true),
                text: None,
                max_age_hours,
            })
        }
        (Some(SearchContentLevel::Detailed), max_age_hours) => Some(ExaContents {
            highlights: Some(true),
            text: Some(ExaTextContent {
                max_characters: 4_000,
            }),
            max_age_hours,
        }),
    }
}

#[derive(Serialize)]
struct ExaSearchRequest {
    query: String,
    #[serde(rename = "numResults")]
    num_results: usize,
    #[serde(rename = "includeDomains")]
    include_domains: Vec<String>,
    #[serde(rename = "excludeDomains")]
    exclude_domains: Vec<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    search_type: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contents: Option<ExaContents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    #[serde(rename = "startPublishedDate", skip_serializing_if = "Option::is_none")]
    start_published_date: Option<String>,
    #[serde(rename = "endPublishedDate", skip_serializing_if = "Option::is_none")]
    end_published_date: Option<String>,
}

#[derive(Serialize)]
struct ExaContents {
    #[serde(skip_serializing_if = "Option::is_none")]
    highlights: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<ExaTextContent>,
    #[serde(rename = "maxAgeHours", skip_serializing_if = "Option::is_none")]
    max_age_hours: Option<i32>,
}

#[derive(Serialize)]
struct ExaTextContent {
    #[serde(rename = "maxCharacters")]
    max_characters: u32,
}

#[derive(Deserialize)]
struct ExaSearchResponse {
    #[serde(default)]
    results: Vec<ExaResult>,
}

#[derive(Deserialize)]
struct ExaResult {
    title: Option<String>,
    url: Option<String>,
    text: Option<String>,
    summary: Option<String>,
    #[serde(default)]
    highlights: Vec<String>,
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
}
