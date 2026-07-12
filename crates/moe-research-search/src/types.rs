use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use moe_research_error::{Error, Result};

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
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LowLatency => "low_latency",
            Self::Balanced => "balanced",
            Self::HighRecall => "high_recall",
        }
    }

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
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Standard => "standard",
            Self::Detailed => "detailed",
        }
    }

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
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Live => "live",
            Self::Fresh => "fresh",
            Self::Recent => "recent",
            Self::Cached => "cached",
        }
    }

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
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Organizations => "organizations",
            Self::People => "people",
            Self::Academic => "academic",
            Self::News => "news",
            Self::PersonalSites => "personal_sites",
            Self::FinancialFilings => "financial_filings",
            Self::Code => "code",
        }
    }

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
#[serde(deny_unknown_fields)]
pub struct SearchIntent {
    pub source_focus: SourceFocus,
    pub timeliness: Timeliness,
    pub coverage: Coverage,
    pub detail: Detail,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFocus {
    General,
    Organizations,
    People,
    Academic,
    News,
    PersonalSites,
    FinancialFilings,
    Code,
}

impl SourceFocus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Organizations => "organizations",
            Self::People => "people",
            Self::Academic => "academic",
            Self::News => "news",
            Self::PersonalSites => "personal_sites",
            Self::FinancialFilings => "financial_filings",
            Self::Code => "code",
        }
    }

    #[must_use]
    pub const fn as_category(self) -> Option<SearchCategory> {
        match self {
            Self::General => None,
            Self::Organizations => Some(SearchCategory::Organizations),
            Self::People => Some(SearchCategory::People),
            Self::Academic => Some(SearchCategory::Academic),
            Self::News => Some(SearchCategory::News),
            Self::PersonalSites => Some(SearchCategory::PersonalSites),
            Self::FinancialFilings => Some(SearchCategory::FinancialFilings),
            Self::Code => Some(SearchCategory::Code),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Timeliness {
    Any,
    Stable,
    Recent,
    Fresh,
    Live,
}

impl Timeliness {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Stable => "stable",
            Self::Recent => "recent",
            Self::Fresh => "fresh",
            Self::Live => "live",
        }
    }

    #[must_use]
    pub const fn as_recency(self) -> Option<SearchRecency> {
        match self {
            Self::Any => None,
            Self::Stable => Some(SearchRecency::Cached),
            Self::Recent => Some(SearchRecency::Recent),
            Self::Fresh => Some(SearchRecency::Fresh),
            Self::Live => Some(SearchRecency::Live),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Coverage {
    Focused,
    Balanced,
    Broad,
}

impl Coverage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Focused => "focused",
            Self::Balanced => "balanced",
            Self::Broad => "broad",
        }
    }

    #[must_use]
    pub const fn as_depth(self) -> SearchDepth {
        match self {
            Self::Focused => SearchDepth::LowLatency,
            Self::Balanced => SearchDepth::Balanced,
            Self::Broad => SearchDepth::HighRecall,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Detail {
    Compact,
    Standard,
    Detailed,
}

impl Detail {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Standard => "standard",
            Self::Detailed => "detailed",
        }
    }

    #[must_use]
    pub const fn as_content_level(self) -> SearchContentLevel {
        match self {
            Self::Compact => SearchContentLevel::Compact,
            Self::Standard => SearchContentLevel::Standard,
            Self::Detailed => SearchContentLevel::Detailed,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentDimension {
    SourceFocus,
    Timeliness,
    Coverage,
    Detail,
}

impl IntentDimension {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceFocus => "source_focus",
            Self::Timeliness => "timeliness",
            Self::Coverage => "coverage",
            Self::Detail => "detail",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentEnforcement {
    Enforced,
    BestEffort,
    Unsupported,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IntentDimensionResolution {
    pub dimension: IntentDimension,
    pub requested: String,
    pub enforcement: IntentEnforcement,
    pub reason_key: String,
}

impl IntentDimensionResolution {
    #[must_use]
    pub fn new(
        dimension: IntentDimension,
        requested: impl Into<String>,
        enforcement: IntentEnforcement,
        reason_key: impl Into<String>,
    ) -> Self {
        Self {
            dimension,
            requested: requested.into(),
            enforcement,
            reason_key: reason_key.into(),
        }
    }
}

impl SearchIntent {
    #[must_use]
    pub fn default_resolution(&self) -> Vec<IntentDimensionResolution> {
        vec![
            IntentDimensionResolution::new(
                IntentDimension::SourceFocus,
                self.source_focus.as_str(),
                IntentEnforcement::BestEffort,
                "generic_provider_mapping",
            ),
            IntentDimensionResolution::new(
                IntentDimension::Timeliness,
                self.timeliness.as_str(),
                IntentEnforcement::BestEffort,
                "generic_provider_mapping",
            ),
            IntentDimensionResolution::new(
                IntentDimension::Coverage,
                self.coverage.as_str(),
                IntentEnforcement::BestEffort,
                "generic_provider_mapping",
            ),
            IntentDimensionResolution::new(
                IntentDimension::Detail,
                self.detail.as_str(),
                IntentEnforcement::BestEffort,
                "generic_provider_mapping",
            ),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchIntentConstraints {
    pub max_results_per_query: usize,
    pub fixed_category: Option<SearchCategory>,
    pub max_depth: Option<SearchDepth>,
    pub max_content_level: Option<SearchContentLevel>,
    pub max_recency: Option<SearchRecency>,
    pub freshness: Option<Freshness>,
    pub language: Option<String>,
    pub region: Option<String>,
    pub include_domains: Vec<String>,
    pub exclude_domains: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedSearchIntent {
    pub request: SearchRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSearchIntent {
    pub request: SearchRequest,
    pub resolution: Vec<IntentDimensionResolution>,
}

impl SearchIntentConstraints {
    #[must_use]
    pub fn policy_constrains_source_focus(&self, intent: &SearchIntent) -> bool {
        self.fixed_category.is_some() && intent.source_focus.as_category().is_none()
    }

    #[must_use]
    pub fn policy_constrains_timeliness(&self, intent: &SearchIntent) -> bool {
        let has_freshness_window = self
            .freshness
            .as_ref()
            .is_some_and(|freshness| freshness.since.is_some() || freshness.until.is_some());
        let policy_applies_recency = matches!(
            (intent.timeliness, self.max_recency),
            (Timeliness::Any, Some(recency)) if recency != SearchRecency::Default
        );

        has_freshness_window || policy_applies_recency
    }

    pub fn prepare(
        &self,
        mut request: SearchRequest,
        intent: &SearchIntent,
    ) -> Result<PreparedSearchIntent> {
        request.max_results = request.max_results.min(self.max_results_per_query);
        request.freshness.clone_from(&self.freshness);
        request.language.clone_from(&self.language);
        request.region.clone_from(&self.region);
        request.include_domains.clone_from(&self.include_domains);
        request.exclude_domains.clone_from(&self.exclude_domains);

        let requested_category = intent.source_focus.as_category();
        if let (Some(requested), Some(fixed)) = (requested_category, self.fixed_category)
            && requested != fixed
        {
            return Err(intent_policy_conflict("source_focus"));
        }
        request.category = self.fixed_category.or(requested_category);

        let requested_recency = intent.timeliness.as_recency();
        if let (Some(requested), Some(maximum)) = (requested_recency, self.max_recency)
            && requested.rank() > maximum.rank()
        {
            return Err(intent_policy_conflict("timeliness"));
        }
        request.recency = requested_recency.or(self.max_recency);

        let requested_depth = intent.coverage.as_depth();
        if let Some(maximum) = self.max_depth
            && requested_depth.rank() > maximum.rank()
        {
            return Err(intent_policy_conflict("coverage"));
        }
        request.depth = Some(requested_depth);

        let requested_detail = intent.detail.as_content_level();
        if let Some(maximum) = self.max_content_level
            && requested_detail.rank() > maximum.rank()
        {
            return Err(intent_policy_conflict("detail"));
        }
        request.content_level = Some(requested_detail);

        Ok(PreparedSearchIntent { request })
    }
}

fn intent_policy_conflict(key: &'static str) -> Error {
    Error::ToolPolicyDenied {
        message: format!("search intent conflicts with policy [branch=intent_policy key={key}]"),
        public: true,
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
