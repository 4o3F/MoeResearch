use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use lapis_error::Result;

use crate::log_safe::{SafeHeaderValue, SafeJson, SafeText, SafeUrl};

#[derive(Clone, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct NetworkRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
    pub body: Option<Value>,
    pub timeout_ms: Option<u64>,
}

#[derive(Clone, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct JsonNetworkResponse {
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: Value,
}

#[derive(Clone, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SseEvent {
    pub event: String,
    pub data: String,
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
            .field("name", &self.name)
            .field("value", &SafeHeaderValue::new(&self.name, &self.value))
            .finish()
    }
}

impl fmt::Debug for NetworkRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("NetworkRequest");
        debug
            .field("method", &self.method)
            .field("url", &SafeUrl::new(&self.url))
            .field("headers", &self.headers);

        if let Some(body) = self.body.as_ref() {
            debug.field("body", &SafeJson::new(body));
        } else {
            debug.field("body", &None::<()>);
        }

        debug.field("timeout_ms", &self.timeout_ms).finish()
    }
}

impl fmt::Debug for JsonNetworkResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsonNetworkResponse")
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("body", &SafeJson::new(&self.body))
            .finish()
    }
}

impl fmt::Debug for SseEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SseEvent")
            .field("event", &SafeText::new(&self.event))
            .field("data", &SafeText::new(&self.data))
            .finish()
    }
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
