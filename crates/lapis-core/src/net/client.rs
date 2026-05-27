use async_trait::async_trait;

use crate::error::Result;
use crate::schema::network::{NetworkRequest, NetworkResponse};

#[async_trait]
pub trait NetworkClient: Send + Sync {
    async fn send(&self, request: NetworkRequest) -> Result<NetworkResponse>;
}
