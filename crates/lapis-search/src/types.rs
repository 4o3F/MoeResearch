use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use lapis_error::{Error, Result};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct Freshness {
    pub since: Option<String>,
    pub until: Option<String>,
}

impl Freshness {
    #[must_use]
    pub fn describe_for_prompt(&self) -> Option<String> {
        match (self.since.as_ref(), self.until.as_ref()) {
            (None, None) => None,
            (Some(since), None) => Some(format!("published on or after {since}")),
            (None, Some(until)) => Some(format!("published on or before {until}")),
            (Some(since), Some(until)) => Some(format!("published between {since} and {until}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchDepth {
    LowLatency,
    Balanced,
    HighRecall,
}

impl SearchDepth {
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::LowLatency => 0,
            Self::Balanced => 1,
            Self::HighRecall => 2,
        }
    }

    #[must_use]
    pub fn prompt_hint(self) -> &'static str {
        match self {
            Self::LowLatency => "prefer low latency over exhaustive coverage",
            Self::Balanced => "balance latency, coverage, and source quality",
            Self::HighRecall => "prefer broad source coverage within normal search",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchContentLevel {
    Compact,
    Standard,
    Detailed,
}

impl SearchContentLevel {
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::Compact => 0,
            Self::Standard => 1,
            Self::Detailed => 2,
        }
    }

    #[must_use]
    pub fn prompt_hint(self) -> &'static str {
        match self {
            Self::Compact => "return compact excerpts only",
            Self::Standard => "return enough context for evidence review",
            Self::Detailed => "return richer context for high-value sources",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchRecency {
    Default,
    Live,
    Fresh,
    Recent,
    Cached,
}

impl SearchRecency {
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::Cached => 0,
            Self::Default => 1,
            Self::Recent => 2,
            Self::Fresh => 3,
            Self::Live => 4,
        }
    }

    #[must_use]
    pub fn prompt_hint(self) -> &'static str {
        match self {
            Self::Default => "use the provider default source freshness",
            Self::Live => "prefer the freshest live sources available",
            Self::Fresh => "prefer sources updated very recently",
            Self::Recent => "prefer recent sources without requiring live refresh",
            Self::Cached => "prefer cached or stable sources when suitable",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchCategory {
    Organizations,
    People,
    Academic,
    News,
    PersonalSites,
    FinancialFilings,
    Code,
}

impl SearchCategory {
    #[must_use]
    pub fn prompt_hint(self) -> &'static str {
        match self {
            Self::Organizations => "organizations and company sources",
            Self::People => "people and professional profile sources",
            Self::Academic => "academic and research sources",
            Self::News => "news and current-events sources",
            Self::PersonalSites => "blogs and personal site sources",
            Self::FinancialFilings => "financial filings and earnings sources",
            Self::Code => "code hosting and developer sources",
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchRequest {
    pub provider: String,
    pub query: String,
    pub max_results: usize,
    pub freshness: Option<Freshness>,
    pub depth: Option<SearchDepth>,
    pub content_level: Option<SearchContentLevel>,
    pub recency: Option<SearchRecency>,
    pub category: Option<SearchCategory>,
    pub language: Option<String>,
    pub region: Option<String>,
    pub include_domains: Vec<String>,
    pub exclude_domains: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchResponse {
    pub provider: String,
    pub results: Vec<SearchResult>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: Option<String>,
    pub snippet: String,
    pub summary: Option<String>,
    pub published_at: Option<String>,
}

impl SearchRequest {
    #[must_use]
    pub fn new(provider: impl Into<String>, query: impl Into<String>, max_results: usize) -> Self {
        Self {
            provider: provider.into(),
            query: query.into(),
            max_results,
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

    pub fn validate(&self) -> Result<()> {
        if self.provider.trim().is_empty() || self.provider.trim() != self.provider {
            return Err(Error::InvalidInput {
                message: "search provider must be explicitly selected".to_owned(),
            });
        }

        if self.query.trim().is_empty() {
            return Err(Error::InvalidInput {
                message: "search query must not be empty".to_owned(),
            });
        }

        if self.max_results == 0 {
            return Err(Error::InvalidInput {
                message: "search max_results must be greater than zero".to_owned(),
            });
        }

        Ok(())
    }
}
