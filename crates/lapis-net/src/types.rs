use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use lapis_error::Result;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct NetworkRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
    pub body: Option<Value>,
    pub timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct JsonNetworkResponse {
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: Value,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SseEvent {
    pub event: String,
    pub data: String,
}

pub struct SseNetworkStream {
    pub status: u16,
    pub headers: Vec<Header>,
    receiver: mpsc::Receiver<Result<SseEvent>>,
    reader: JoinHandle<()>,
}

impl SseNetworkStream {
    pub fn new(
        status: u16,
        headers: Vec<Header>,
        receiver: mpsc::Receiver<Result<SseEvent>>,
        reader: JoinHandle<()>,
    ) -> Self {
        Self {
            status,
            headers,
            receiver,
            reader,
        }
    }

    pub fn from_events(status: u16, headers: Vec<Header>, events: Vec<SseEvent>) -> Self {
        Self::from_results(status, headers, events.into_iter().map(Ok).collect())
    }

    pub fn from_results(status: u16, headers: Vec<Header>, events: Vec<Result<SseEvent>>) -> Self {
        let capacity = events.len().max(1);
        let (sender, receiver) = mpsc::channel(capacity);
        let reader = tokio::spawn(async move {
            for event in events {
                if sender.send(event).await.is_err() {
                    break;
                }
            }
        });
        Self::new(status, headers, receiver, reader)
    }

    pub async fn next_event(&mut self) -> Result<Option<SseEvent>> {
        match self.receiver.recv().await {
            Some(Ok(event)) => Ok(Some(event)),
            Some(Err(error)) => Err(error),
            None => Ok(None),
        }
    }
}

impl Drop for SseNetworkStream {
    fn drop(&mut self) {
        self.reader.abort();
    }
}
