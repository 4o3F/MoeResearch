use async_trait::async_trait;

use crate::error::Result;
use crate::schema::search::{SearchRequest, SearchResponse};

pub mod exa;
pub mod grok;

pub use exa::ExaSearchProvider;
pub use grok::GrokSearchProvider;

#[async_trait]
pub trait SearchProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse>;
}
