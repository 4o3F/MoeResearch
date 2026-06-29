use async_trait::async_trait;
use moe_research_error::Result;

use crate::{SearchRequest, SearchResponse};

pub mod exa;
pub mod grok;
pub mod tavily;

pub use exa::ExaSearchProvider;
pub use grok::{GrokReasoningEffort, GrokSearchProvider};
pub use tavily::TavilySearchProvider;

#[async_trait]
pub trait SearchProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse>;
}
