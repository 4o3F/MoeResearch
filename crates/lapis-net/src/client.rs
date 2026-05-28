use async_trait::async_trait;

use lapis_error::Result;

use crate::{JsonNetworkResponse, NetworkRequest, SseNetworkStream};

#[async_trait]
pub trait NetworkClient: Send + Sync {
    async fn send_json(&self, request: NetworkRequest) -> Result<JsonNetworkResponse>;

    async fn send_sse(&self, request: NetworkRequest) -> Result<SseNetworkStream>;
}
