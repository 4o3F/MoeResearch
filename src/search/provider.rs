use async_trait::async_trait;

use crate::error::Result;
use crate::schema::search::{SearchRequest, SearchResponse};

#[async_trait]
pub trait SearchProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse>;
}
