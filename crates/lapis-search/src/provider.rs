use async_trait::async_trait;
use lapis_error::Result;

use crate::{SearchRequest, SearchResponse};

pub mod exa;
pub mod grok;

pub use exa::ExaSearchProvider;
pub use grok::{GrokReasoningEffort, GrokSearchProvider};

#[async_trait]
pub trait SearchProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse>;
}
