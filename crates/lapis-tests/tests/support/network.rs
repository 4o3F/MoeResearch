#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use lapis_error::{Error, Result};
use lapis_net::client::NetworkClient;
use lapis_net::{JsonNetworkResponse, NetworkRequest, SseEvent, SseNetworkStream};

#[derive(Clone, Default)]
pub struct MockNetworkClient {
    responses: Arc<Mutex<VecDeque<JsonNetworkResponse>>>,
    sse_responses: Arc<Mutex<VecDeque<MockSseResponse>>>,
    requests: Arc<Mutex<Vec<NetworkRequest>>>,
}

pub fn mock_completed_sse(body: serde_json::Value) -> Arc<MockNetworkClient> {
    Arc::new(MockNetworkClient::new_sse([completed_sse_response(body)]))
}

pub fn completed_sse_response(body: serde_json::Value) -> MockSseResponse {
    MockSseResponse {
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

pub fn sse_response(status: u16, events: Vec<SseEvent>) -> MockSseResponse {
    MockSseResponse {
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

pub struct MockSseResponse {
    pub status: u16,
    pub headers: Vec<lapis_net::Header>,
    pub events: Vec<SseEvent>,
}

fn has_accept(request: &NetworkRequest, expected: &str) -> bool {
    request.headers.iter().any(|header| {
        header.name.eq_ignore_ascii_case("accept")
            && header
                .value
                .split(',')
                .any(|value| value.split(';').next().unwrap_or_default().trim() == expected)
    })
}

impl MockNetworkClient {
    pub fn new(responses: impl IntoIterator<Item = JsonNetworkResponse>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses.into_iter().collect())),
            sse_responses: Arc::new(Mutex::new(VecDeque::new())),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn new_sse(responses: impl IntoIterator<Item = MockSseResponse>) -> Self {
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
    async fn send_json(&self, request: NetworkRequest) -> Result<JsonNetworkResponse> {
        if !has_accept(&request, "application/json") {
            return Err(Error::InvalidInput {
                message: "mock JSON request missing application/json Accept".to_owned(),
            });
        }
        self.requests.lock().expect("requests lock").push(request);
        self.responses
            .lock()
            .expect("responses lock")
            .pop_front()
            .ok_or_else(|| Error::NetworkFailed {
                message: "mock network response queue is empty".to_owned(),
            })
    }

    async fn send_sse(&self, request: NetworkRequest) -> Result<SseNetworkStream> {
        if !has_accept(&request, "text/event-stream") {
            return Err(Error::InvalidInput {
                message: "mock SSE request missing text/event-stream Accept".to_owned(),
            });
        }
        self.requests.lock().expect("requests lock").push(request);
        let response = self
            .sse_responses
            .lock()
            .expect("sse responses lock")
            .pop_front()
            .ok_or_else(|| Error::NetworkFailed {
                message: "mock SSE network response queue is empty".to_owned(),
            })?;
        Ok(SseNetworkStream::from_events(
            response.status,
            response.headers,
            response.events,
        ))
    }
}
