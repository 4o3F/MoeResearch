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

pub fn mock_completed_sse(body: serde_json::Value) -> Arc<MockNetworkClient> {
    Arc::new(MockNetworkClient::new_sse([completed_sse_response(body)]))
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
        let accept_sse = request.headers.iter().any(|header| {
            header.name.eq_ignore_ascii_case("accept")
                && header
                    .value
                    .split(',')
                    .any(|value| value.trim().eq_ignore_ascii_case("text/event-stream"))
        });
        self.requests.lock().expect("requests lock").push(request);

        if accept_sse {
            let response = self
                .sse_responses
                .lock()
                .expect("sse responses lock")
                .pop_front()
                .ok_or_else(|| Error::NetworkFailed {
                    message: "mock SSE network response queue is empty".to_owned(),
                })?;
            let status = response.status;
            let headers = response.headers.clone();
            let body = serde_json::to_value(response).map_err(|source| Error::Json { source })?;
            return Ok(NetworkResponse {
                status,
                headers,
                body,
            });
        }

        self.responses
            .lock()
            .expect("responses lock")
            .pop_front()
            .ok_or_else(|| Error::NetworkFailed {
                message: "mock network response queue is empty".to_owned(),
            })
    }
}
