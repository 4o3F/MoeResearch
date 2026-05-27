//! Search provider boundary for Lapis.

pub mod provider;
pub mod service;
pub mod types;

pub use provider::{ExaSearchProvider, GrokSearchProvider, SearchProvider};
pub use service::SearchService;
pub use types::{Freshness, SearchRequest, SearchResponse, SearchResult};
