#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use lapis_core::error::{Error, Result};
use lapis_core::net::client::NetworkClient;
use lapis_core::schema::network::{NetworkRequest, NetworkResponse};

#[derive(Clone, Default)]
pub struct MockNetworkClient {
    responses: Arc<Mutex<VecDeque<NetworkResponse>>>,
    requests: Arc<Mutex<Vec<NetworkRequest>>>,
}

impl MockNetworkClient {
    pub fn new(responses: impl IntoIterator<Item = NetworkResponse>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses.into_iter().collect())),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn requests(&self) -> Vec<NetworkRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

#[async_trait]
impl NetworkClient for MockNetworkClient {
    async fn send(&self, request: NetworkRequest) -> Result<NetworkResponse> {
        self.requests.lock().expect("requests lock").push(request);
        self.responses
            .lock()
            .expect("responses lock")
            .pop_front()
            .ok_or_else(|| Error::NetworkFailed {
                message: "mock network response queue is empty".to_owned(),
            })
    }
}
