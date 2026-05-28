use serde_json::Value;

use crate::{Header, NetworkRequest};

pub fn bearer_json_post(
    base_url: &str,
    path: &str,
    api_key: &str,
    body: Value,
    timeout_ms: Option<u64>,
) -> NetworkRequest {
    NetworkRequest {
        method: "POST".to_owned(),
        url: format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        ),
        headers: vec![
            Header {
                name: "authorization".to_owned(),
                value: format!("Bearer {api_key}"),
            },
            Header {
                name: "content-type".to_owned(),
                value: "application/json".to_owned(),
            },
        ],
        body: Some(body),
        timeout_ms,
    }
}

pub fn bearer_json_sse_post(
    base_url: &str,
    path: &str,
    api_key: &str,
    body: Value,
    timeout_ms: Option<u64>,
) -> NetworkRequest {
    let mut request = bearer_json_post(base_url, path, api_key, body, timeout_ms);
    request.headers.push(Header {
        name: "accept".to_owned(),
        value: "text/event-stream".to_owned(),
    });
    request
}

pub fn provider_status_retryable(status: u16) -> bool {
    status == 429 || status >= 500
}
