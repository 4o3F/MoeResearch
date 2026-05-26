//! Integration tests for the trace-level wire body capture in
//! `lapis_core::net::reqwest_client`.
//!
//! Each test spins up a minimal HTTP mock server on a loopback port,
//! installs a thread-local tracing subscriber that captures emitted
//! events into an in-memory NDJSON buffer, drives a real
//! `ReqwestNetworkClient` against the mock, and then asserts on the
//! captured event stream.
//!
//! The mock server lives in this file (a ~40 line `tokio::net::TcpListener`
//! wrapper) so the test suite does not depend on `wiremock` or any other
//! HTTP mock crate.
//!
//! Each test uses `#[tokio::test(flavor = "current_thread")]` because
//! `tracing::subscriber::set_default` installs a *thread-local* default
//! subscriber. With a single-threaded runtime the test future, its
//! `client.send().await` chain, and the trace emission all run on the
//! same OS thread, so the captured-subscriber guard remains in effect
//! for the entire test body.

use std::io::Write;
use std::sync::{Arc, Mutex};

use lapis_core::net::NetworkClient;
use lapis_core::net::reqwest_client::ReqwestNetworkClient;
use lapis_core::schema::network::{Header, NetworkRequest};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing_subscriber::layer::SubscriberExt;

// ---------------------------------------------------------------------------
// Tracing capture: JSON-line buffer + filter
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

impl SharedBuffer {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(Vec::new())))
    }

    fn snapshot(&self) -> String {
        String::from_utf8(self.0.lock().expect("buffer lock poisoned").clone())
            .expect("captured tracing output must be valid utf-8")
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SharedBuffer {
    type Writer = BufferWriter;

    fn make_writer(&'a self) -> Self::Writer {
        BufferWriter(self.0.clone())
    }
}

struct BufferWriter(Arc<Mutex<Vec<u8>>>);

impl Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .expect("buffer lock poisoned")
            .extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Builds a tracing subscriber that writes every event matching `filter`
/// as one JSON object per line into the returned shared buffer.
///
/// The returned `DefaultGuard` keeps the subscriber active until it is
/// dropped; in single-threaded tests this covers the entire test body.
fn capture_tracing(filter: &str) -> (SharedBuffer, tracing::subscriber::DefaultGuard) {
    let buffer = SharedBuffer::new();
    let layer = tracing_subscriber::fmt::layer()
        .json()
        .with_target(true)
        .with_writer(buffer.clone());
    let env_filter = tracing_subscriber::EnvFilter::new(filter);
    let subscriber = tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(layer);
    let guard = tracing::subscriber::set_default(subscriber);
    (buffer, guard)
}

fn parse_events(buffer: &SharedBuffer) -> Vec<serde_json::Value> {
    buffer
        .snapshot()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line)
                .unwrap_or_else(|err| panic!("non-JSON trace line `{line}`: {err}"))
        })
        .collect()
}

fn message_of(event: &serde_json::Value) -> &str {
    event
        .get("fields")
        .and_then(|f| f.get("message"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
}

fn field_str<'a>(event: &'a serde_json::Value, name: &str) -> Option<&'a str> {
    event.get("fields")?.get(name)?.as_str()
}

fn field_u64(event: &serde_json::Value, name: &str) -> Option<u64> {
    event.get("fields")?.get(name)?.as_u64()
}

fn field_bool(event: &serde_json::Value, name: &str) -> Option<bool> {
    event.get("fields")?.get(name)?.as_bool()
}

// ---------------------------------------------------------------------------
// Minimal HTTP mock server
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct CannedResponse {
    status: u16,
    body: String,
}

impl CannedResponse {
    fn new(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }
}

struct MockServer {
    base_url: String,
}

impl MockServer {
    /// Starts a loopback HTTP server that returns the supplied responses
    /// in order, one per incoming request. After the queue is drained
    /// any further request receives `503 server queue empty`.
    async fn start(responses: Vec<CannedResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind loopback");
        let port = listener.local_addr().expect("local_addr").port();
        let base_url = format!("http://127.0.0.1:{port}");
        let queue = Arc::new(Mutex::new(responses));

        tokio::spawn(async move {
            loop {
                let (mut socket, _) = match listener.accept().await {
                    Ok(pair) => pair,
                    Err(_) => return,
                };
                let queue = queue.clone();
                tokio::spawn(async move {
                    // 1. Read until end-of-headers.
                    let mut head = Vec::with_capacity(512);
                    let mut byte = [0u8; 1];
                    while socket.read_exact(&mut byte).await.is_ok() {
                        head.push(byte[0]);
                        if head.ends_with(b"\r\n\r\n") {
                            break;
                        }
                    }

                    // 2. Parse Content-Length so we drain the request body
                    //    before responding (avoids `connection reset` on the
                    //    client side when reqwest pipelines).
                    let head_str = String::from_utf8_lossy(&head);
                    let content_length: usize = head_str
                        .lines()
                        .find_map(|line| {
                            let lower = line.to_ascii_lowercase();
                            lower
                                .strip_prefix("content-length:")
                                .map(|rest| rest.trim().parse::<usize>().ok())
                                .unwrap_or(None)
                        })
                        .unwrap_or(0);
                    if content_length > 0 {
                        let mut body = vec![0u8; content_length];
                        let _ = socket.read_exact(&mut body).await;
                    }

                    // 3. Pop the next canned response.
                    let canned = {
                        let mut q = queue.lock().expect("queue lock poisoned");
                        if q.is_empty() {
                            CannedResponse::new(503, "server queue empty")
                        } else {
                            q.remove(0)
                        }
                    };

                    // 4. Write a minimal HTTP/1.1 response. `application/json`
                    //    matches what real providers use; reqwest still
                    //    succeeds on text bodies because lapis only parses
                    //    the body string after read.
                    let response = format!(
                        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
                        status = canned.status,
                        len = canned.body.len(),
                        body = canned.body
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.shutdown().await;
                });
            }
        });

        // Yield once so the spawn definitely runs before tests start.
        tokio::task::yield_now().await;
        Self { base_url }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }
}

fn build_client() -> ReqwestNetworkClient {
    // `max_retries = 0` for tests that do not exercise retry; the retry
    // test overrides this via a dedicated constructor call.
    ReqwestNetworkClient::new(5_000, 0, 50, "lapis-tests/0.0.0").expect("ReqwestNetworkClient::new")
}

fn build_client_with_retries(max_retries: usize) -> ReqwestNetworkClient {
    ReqwestNetworkClient::new(5_000, max_retries, 50, "lapis-tests/0.0.0")
        .expect("ReqwestNetworkClient::new")
}

fn request_to(url: String) -> NetworkRequest {
    NetworkRequest {
        method: "POST".to_owned(),
        url,
        headers: vec![Header {
            name: "content-type".to_owned(),
            value: "application/json".to_owned(),
        }],
        body: Some(serde_json::json!({ "input": "hello" })),
        timeout_ms: Some(5_000),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// When the operator runs with `RUST_LOG=lapis_core=debug` (the default
/// production case), no `direction` field — i.e. no trace-level wire
/// body event — must appear in the captured stream.
#[tokio::test(flavor = "current_thread")]
async fn wire_trace_disabled_emits_no_body_events() {
    let server =
        MockServer::start(vec![CannedResponse::new(200, r#"{"ok":true}"#.to_owned())]).await;

    let (buffer, _guard) = capture_tracing("lapis_core=debug");
    let client = build_client();
    let _ = client
        .send(request_to(server.url("/responses")))
        .await
        .expect("request succeeds");

    let events = parse_events(&buffer);
    let wire_events: Vec<_> = events
        .iter()
        .filter(|e| field_str(e, "direction").is_some())
        .collect();
    assert!(
        wire_events.is_empty(),
        "wire trace events must be silent at debug level, got {wire_events:?}"
    );

    // The metadata-only debug event still fires so the operator retains a
    // basic record of every outbound call.
    let debug_metadata = events
        .iter()
        .any(|e| message_of(e) == "sending outbound request");
    assert!(debug_metadata, "debug-level metadata event must still fire");
}

/// At trace level, a single round trip emits exactly one outbound and
/// one inbound wire event, sharing the same `correlation_id` and an
/// `attempt` index of 0.
#[tokio::test(flavor = "current_thread")]
async fn wire_trace_enabled_emits_paired_outbound_and_inbound() {
    let server =
        MockServer::start(vec![CannedResponse::new(200, r#"{"ok":true}"#.to_owned())]).await;

    let (buffer, _guard) = capture_tracing("lapis_core::net::reqwest_client=trace");
    let client = build_client();
    let _ = client
        .send(request_to(server.url("/responses")))
        .await
        .expect("request succeeds");

    let events = parse_events(&buffer);
    let outbound: Vec<_> = events
        .iter()
        .filter(|e| field_str(e, "direction") == Some("outbound"))
        .collect();
    let inbound: Vec<_> = events
        .iter()
        .filter(|e| field_str(e, "direction") == Some("inbound"))
        .collect();

    assert_eq!(outbound.len(), 1, "exactly one outbound wire event");
    assert_eq!(inbound.len(), 1, "exactly one inbound wire event");

    let outbound_id = field_str(outbound[0], "correlation_id").expect("outbound correlation_id");
    let inbound_id = field_str(inbound[0], "correlation_id").expect("inbound correlation_id");
    assert_eq!(
        outbound_id, inbound_id,
        "outbound and inbound must share correlation_id"
    );

    assert_eq!(field_u64(outbound[0], "attempt"), Some(0));
    assert_eq!(field_u64(inbound[0], "attempt"), Some(0));
    assert_eq!(field_u64(inbound[0], "status"), Some(200));

    // Body field round-trips through the trace formatter as a string;
    // the inbound event must carry the literal response payload.
    let inbound_body = field_str(inbound[0], "body").expect("inbound body");
    assert!(
        inbound_body.contains("\"ok\":true"),
        "inbound body must carry the upstream response, got `{inbound_body}`"
    );
}

/// Each `send_once` attempt generates a fresh `correlation_id` and a
/// monotonically increasing `attempt` index; a retry therefore produces
/// two outbound and two inbound wire events, none of which share an id.
#[tokio::test(flavor = "current_thread")]
async fn wire_trace_retry_emits_new_correlation_per_attempt() {
    let server = MockServer::start(vec![
        CannedResponse::new(503, r#"{"err":"transient"}"#.to_owned()),
        CannedResponse::new(200, r#"{"ok":true}"#.to_owned()),
    ])
    .await;

    let (buffer, _guard) = capture_tracing("lapis_core::net::reqwest_client=trace");
    let client = build_client_with_retries(1);
    let _ = client
        .send(request_to(server.url("/responses")))
        .await
        .expect("retry succeeds");

    let events = parse_events(&buffer);
    let outbound: Vec<_> = events
        .iter()
        .filter(|e| field_str(e, "direction") == Some("outbound"))
        .collect();
    let inbound: Vec<_> = events
        .iter()
        .filter(|e| field_str(e, "direction") == Some("inbound"))
        .collect();

    assert_eq!(
        outbound.len(),
        2,
        "two attempts produce two outbound events"
    );
    assert_eq!(inbound.len(), 2, "two attempts produce two inbound events");

    let id0 = field_str(outbound[0], "correlation_id").unwrap();
    let id1 = field_str(outbound[1], "correlation_id").unwrap();
    assert_ne!(id0, id1, "each attempt must mint a fresh correlation_id");

    assert_eq!(field_u64(outbound[0], "attempt"), Some(0));
    assert_eq!(field_u64(outbound[1], "attempt"), Some(1));
    assert_eq!(field_u64(inbound[0], "status"), Some(503));
    assert_eq!(field_u64(inbound[1], "status"), Some(200));
}

/// Inbound bodies larger than `MAX_WIRE_BODY_BYTES` (64 KiB) must be
/// truncated with `body_truncated=true` and an `original_bytes` field
/// reporting the full upstream payload size.
#[tokio::test(flavor = "current_thread")]
async fn wire_trace_truncates_oversized_inbound_body() {
    // 70 KiB payload — comfortably above the 64 KiB cap.
    let huge_body = format!("{{\"blob\":\"{}\"}}", "x".repeat(70 * 1024));
    let original_size = huge_body.len();
    let server = MockServer::start(vec![CannedResponse::new(200, huge_body)]).await;

    let (buffer, _guard) = capture_tracing("lapis_core::net::reqwest_client=trace");
    let client = build_client();
    let _ = client
        .send(request_to(server.url("/responses")))
        .await
        .expect("oversized response succeeds");

    let events = parse_events(&buffer);
    let inbound = events
        .iter()
        .find(|e| field_str(e, "direction") == Some("inbound"))
        .expect("inbound wire event present");
    assert_eq!(field_bool(inbound, "body_truncated"), Some(true));
    assert_eq!(
        field_u64(inbound, "body_bytes").map(|n| n as usize),
        Some(original_size),
        "body_bytes must report the pre-truncation byte count"
    );
    let body = field_str(inbound, "body").expect("body field");
    let parsed: serde_json::Value =
        serde_json::from_str(body).expect("truncation marker is valid JSON");
    assert_eq!(parsed["__truncated"], serde_json::Value::Bool(true));
    assert_eq!(parsed["original_bytes"], serde_json::json!(original_size));
}

/// Non-2xx responses must drop the full body from the debug event and
/// expose only a short `body_excerpt`. The complete payload remains
/// available in the trace-level inbound event.
#[tokio::test(flavor = "current_thread")]
async fn wire_non_success_debug_event_uses_excerpt_not_full_body() {
    // Make the body large enough to trigger excerpt truncation.
    let big_error = format!("{{\"err\":\"{}\"}}", "z".repeat(400));
    let server = MockServer::start(vec![CannedResponse::new(400, big_error.clone())]).await;

    let (buffer, _guard) = capture_tracing("lapis_core::net::reqwest_client=trace");
    let client = build_client();
    let _ = client
        .send(request_to(server.url("/responses")))
        .await
        .expect_err("4xx must surface an error to the caller");

    let events = parse_events(&buffer);
    let non_success = events
        .iter()
        .find(|e| message_of(e) == "outbound response returned non-success status")
        .expect("non-success debug event present");

    assert!(
        non_success
            .get("fields")
            .and_then(|f| f.get("body"))
            .is_none(),
        "non-success debug event must NOT carry the full `body` field"
    );
    let excerpt = field_str(non_success, "body_excerpt").expect("body_excerpt field");
    assert!(
        excerpt.contains("reqwest_client=trace"),
        "excerpt must point operators at the trace knob for the full body"
    );
    assert!(
        excerpt.len() < big_error.len(),
        "excerpt must be shorter than the original body"
    );

    // The full body is still captured at trace level.
    let inbound = events
        .iter()
        .find(|e| field_str(e, "direction") == Some("inbound"))
        .expect("trace-level inbound event present");
    assert_eq!(field_u64(inbound, "status"), Some(400));
    let inbound_body = field_str(inbound, "body").expect("inbound body");
    assert!(
        inbound_body.contains("zzzz"),
        "trace inbound event must carry the full upstream body"
    );
}
