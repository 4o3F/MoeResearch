use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
pub struct NetworkResponse {
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: Value,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SseEvent {
    pub event: String,
    pub data: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
pub struct SseNetworkResponse {
    pub status: u16,
    pub headers: Vec<Header>,
    pub events: Vec<SseEvent>,
}
