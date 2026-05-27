use async_trait::async_trait;

use lapis_error::Result;

use crate::{NetworkRequest, NetworkResponse};

#[async_trait]
pub trait NetworkClient: Send + Sync {
    async fn send(&self, request: NetworkRequest) -> Result<NetworkResponse>;
}
