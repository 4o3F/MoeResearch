use async_trait::async_trait;

use moe_research_error::Result;

use crate::{DocumentNetworkOutcome, JsonNetworkResponse, NetworkRequest, SseNetworkStream};

#[async_trait]
pub trait NetworkClient: Send + Sync {
    async fn send_json(&self, request: NetworkRequest) -> Result<JsonNetworkResponse>;

    async fn send_sse(&self, request: NetworkRequest) -> Result<SseNetworkStream>;

    async fn send_document(&self, request: NetworkRequest) -> Result<DocumentNetworkOutcome>;
}
