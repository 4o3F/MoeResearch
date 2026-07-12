//! Search provider boundary for MoeResearch.

pub mod provider;
pub mod service;
pub mod types;

pub use provider::{
    ExaSearchProvider, GrokReasoningEffort, GrokSearchProvider, SearchProvider,
    TavilySearchProvider,
};
pub use service::SearchService;
pub use types::{
    Coverage, Detail, Freshness, IntentDimension, IntentDimensionResolution, IntentEnforcement,
    PreparedSearchIntent, ResolvedSearchIntent, SearchCategory, SearchContentLevel, SearchDepth,
    SearchIntent, SearchIntentConstraints, SearchRecency, SearchRequest, SearchResponse,
    SearchResult, SourceFocus, Timeliness,
};
