#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use lapis_error::{Error, Result};
use lapis_net::client::NetworkClient;
use lapis_net::{NetworkRequest, NetworkResponse, SseEvent, SseNetworkResponse};

#[derive(Clone, Default)]
pub struct MockNetworkClient {
    responses: Arc<Mutex<VecDeque<NetworkResponse>>>,
    sse_responses: Arc<Mutex<VecDeque<SseNetworkResponse>>>,
    requests: Arc<Mutex<Vec<NetworkRequest>>>,
}

pub fn completed_sse_response(body: serde_json::Value) -> SseNetworkResponse {
    SseNetworkResponse {
        status: 200,
        headers: vec![],
        events: vec![SseEvent {
            event: "response.completed".to_owned(),
            data: serde_json::json!({
                "type": "response.completed",
                "response": body,
            })
            .to_string(),
        }],
    }
}

pub fn sse_response(status: u16, events: Vec<SseEvent>) -> SseNetworkResponse {
    SseNetworkResponse {
        status,
        headers: vec![],
        events,
    }
}

pub fn sse_json_event(event: &str, data: serde_json::Value) -> SseEvent {
    SseEvent {
        event: event.to_owned(),
        data: data.to_string(),
    }
}

impl MockNetworkClient {
    pub fn new(responses: impl IntoIterator<Item = NetworkResponse>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses.into_iter().collect())),
            sse_responses: Arc::new(Mutex::new(VecDeque::new())),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn new_sse(responses: impl IntoIterator<Item = SseNetworkResponse>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::new())),
            sse_responses: Arc::new(Mutex::new(responses.into_iter().collect())),
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

    async fn send_sse(&self, request: NetworkRequest) -> Result<SseNetworkResponse> {
        self.requests.lock().expect("requests lock").push(request);
        self.sse_responses
            .lock()
            .expect("sse responses lock")
            .pop_front()
            .ok_or_else(|| Error::NetworkFailed {
                message: "mock SSE network response queue is empty".to_owned(),
            })
    }
}
