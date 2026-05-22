use crate::error::Result;
use crate::schema::model::{ModelRequest, ModelResponse};

#[async_trait::async_trait]
pub trait ModelProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse>;
}
