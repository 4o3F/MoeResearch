use async_trait::async_trait;

use crate::error::Result;
use crate::schema::config::NetworkConfig;
use crate::schema::network::{NetworkRequest, NetworkResponse};

pub struct ReqwestNetworkClient {
    inner: lapis_net::reqwest_client::ReqwestNetworkClient,
}

impl ReqwestNetworkClient {
    pub fn from_config(config: &NetworkConfig) -> Result<Self> {
        Self::new(
            config.timeout_ms,
            config.max_retries,
            config.retry_backoff_ms,
            &config.user_agent,
        )
    }

    pub fn new(
        default_timeout_ms: u64,
        max_retries: usize,
        retry_backoff_ms: u64,
        user_agent: &str,
    ) -> Result<Self> {
        Ok(Self {
            inner: lapis_net::reqwest_client::ReqwestNetworkClient::new(
                default_timeout_ms,
                max_retries,
                retry_backoff_ms,
                user_agent,
            )?,
        })
    }
}

#[async_trait]
impl crate::net::NetworkClient for ReqwestNetworkClient {
    async fn send(&self, request: NetworkRequest) -> Result<NetworkResponse> {
        self.inner.send(request).await
    }
}
