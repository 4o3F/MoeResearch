use std::time::{Duration, Instant};

use crate::error::{Error, Result};
use crate::net::client::NetworkClient;
use crate::net::policy::RedactionPolicy;
use crate::schema::config::NetworkConfig;
use crate::schema::network::{Header, NetworkRequest, NetworkResponse};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Method, RequestBuilder, Response, Url};
use uuid::Uuid;

/// Maximum byte length captured in a wire body trace event.
///
/// Caps the size of the rendered payload so a single oversized
/// model/search response cannot saturate the tracing stream. When the
/// raw body exceeds this size the trace event sets
/// `body_truncated = true` and emits a JSON marker carrying the
/// `original_bytes` count plus a UTF-8-safe `head` prefix.
pub(crate) const MAX_WIRE_BODY_BYTES: usize = 64 * 1024;

/// Maximum byte length captured in a non-2xx debug `body_excerpt` field.
///
/// Debug-level events are intended for high-level error signal only; the
/// full provider body still appears in the trace-level wire event when
/// the operator opts in.
const MAX_DEBUG_EXCERPT_BYTES: usize = 256;

pub struct ReqwestNetworkClient {
    client: reqwest::Client,
    default_timeout_ms: u64,
    max_retries: usize,
    retry_backoff_ms: u64,
}

impl ReqwestNetworkClient {
    /// Builds a reqwest-backed network client from a validated `NetworkConfig`.
    ///
    /// The `user_agent` field is injected into the underlying reqwest client
    /// builder so every outbound request advertises a stable identity. The
    /// caller (typically `lapis-cli`) is responsible for choosing a value that
    /// includes the running version (e.g. `"lapis/0.1.0"`).
    ///
    /// # Errors
    /// Returns `Error::ConfigInvalid` if `network.user_agent` cannot be parsed
    /// as a valid HTTP header value, or `Error::HttpTransport` if the reqwest
    /// builder itself fails (rare; usually TLS / runtime issues).
    pub fn from_config(config: &NetworkConfig) -> Result<Self> {
        Self::new(
            config.timeout_ms,
            config.max_retries,
            config.retry_backoff_ms,
            &config.user_agent,
        )
    }

    /// Builds a reqwest-backed network client with explicit knobs.
    ///
    /// Prefer `from_config` over this constructor in production code; this
    /// version exists for tests that need to bypass the full TOML config.
    ///
    /// # Errors
    /// - `Error::InvalidInput` if `default_timeout_ms` is zero.
    /// - `Error::ConfigInvalid` if `user_agent` is not a valid HTTP header
    ///   value.
    /// - `Error::HttpTransport` if the reqwest builder fails.
    pub fn new(
        default_timeout_ms: u64,
        max_retries: usize,
        retry_backoff_ms: u64,
        user_agent: &str,
    ) -> Result<Self> {
        validate_timeout("network.timeout_ms", default_timeout_ms)?;
        let header_value =
            HeaderValue::from_str(user_agent).map_err(|source| Error::ConfigInvalid {
                message: format!("invalid network.user_agent header: {source}"),
            })?;
        let client = reqwest::Client::builder()
            .user_agent(header_value)
            .build()
            .map_err(|source| Self::transport_error(&source))?;

        Ok(Self {
            client,
            default_timeout_ms,
            max_retries,
            retry_backoff_ms,
        })
    }

    async fn send_once(&self, request: NetworkRequest, attempt: u32) -> Result<NetworkResponse> {
        let method = request
            .method
            .parse::<Method>()
            .map_err(|source| Error::InvalidInput {
                message: format!("invalid HTTP method `{}`: {source}", request.method),
            })?;
        let url = Url::parse(&request.url).map_err(|_| Error::InvalidInput {
            message: "invalid outbound URL".to_owned(),
        })?;
        let timeout_ms = request.timeout_ms.unwrap_or(self.default_timeout_ms);
        validate_timeout("request.timeout_ms", timeout_ms)?;
        let host = url.host_str().unwrap_or("unknown").to_owned();
        let path = url.path().to_owned();
        let redaction = RedactionPolicy;
        // A fresh correlation id per attempt — operators can grep both the
        // outbound and inbound wire events for the same UUID to reconstruct
        // a single round trip, and retried attempts get distinct ids so
        // they cannot be conflated.
        let correlation_id = Uuid::new_v4();

        tracing::debug!(
            method = %method,
            host = %host,
            path = %path,
            headers = ?redaction.redact_headers(&request.headers),
            timeout_ms,
            "sending outbound request"
        );

        // Trace-level wire body capture. The helper internally re-checks
        // `enabled!()` so the rendering / truncation work is skipped
        // entirely when the operator has not opted in via
        // `RUST_LOG=...reqwest_client=trace`.
        if let Some(body) = request.body.as_ref() {
            emit_outbound_wire_trace(correlation_id, attempt, &method, &host, &path, body);
        }

        let mut builder = self
            .client
            .request(method, url)
            .timeout(Duration::from_millis(timeout_ms));
        builder = apply_headers(builder, &request.headers)?;

        if let Some(body) = request.body {
            builder = builder.json(&body);
        }

        let started_at = Instant::now();
        let response = send_request(
            builder,
            &TransportErrorLogContext {
                phase: "send_request",
                attempt,
                correlation_id,
                host: &host,
                path: &path,
                timeout_ms,
            },
        )
        .await?;
        let status = response.status();
        let headers = response_headers(&response);
        let text = read_response_body(
            response,
            &TransportErrorLogContext {
                phase: "read_response_body",
                attempt,
                correlation_id,
                host: &host,
                path: &path,
                timeout_ms,
            },
        )
        .await?;
        let duration_ms = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);

        // Trace-level wire body capture for the inbound side. Fires for
        // both success and non-success statuses so a single trace stream
        // contains the complete plaintext payload; the debug-level event
        // below only carries a short excerpt to keep error logs compact.
        emit_inbound_wire_trace(
            correlation_id,
            attempt,
            &host,
            &path,
            status.as_u16(),
            duration_ms,
            &text,
        );

        if !status.is_success() {
            let redacted = redaction.redact_body_text(&text);
            let excerpt = excerpt_for_debug(&redacted, MAX_DEBUG_EXCERPT_BYTES);
            tracing::debug!(
                status = status.as_u16(),
                host = %host,
                path = %path,
                headers = ?redaction.redact_headers(&headers),
                body_excerpt = %excerpt,
                "outbound response returned non-success status"
            );
            return Err(Error::HttpStatus {
                status: status.as_u16(),
                message: "provider returned non-success status".to_owned(),
                retryable: is_retryable_status(status.as_u16()),
            });
        }

        let body = serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text));

        Ok(NetworkResponse {
            status: status.as_u16(),
            headers,
            body,
        })
    }

    fn transport_error(source: &reqwest::Error) -> Error {
        let retryable = is_retryable_transport_error(source);
        let message = if source.is_timeout() {
            "request timed out"
        } else if source.is_connect() {
            "connection failed"
        } else if source.is_body() || source.is_decode() {
            "response body handling failed"
        } else {
            "HTTP transport failed"
        };

        Error::HttpTransport {
            message: message.to_owned(),
            retryable,
        }
    }
}

/// Metadata emitted with operator-only transport failure diagnostics.
struct TransportErrorLogContext<'a> {
    /// Request phase where reqwest reported the transport error.
    phase: &'static str,
    /// Retry attempt index for the failing outbound request.
    attempt: u32,
    /// Correlation id shared with wire trace events for this attempt.
    correlation_id: Uuid,
    /// Redacted request host, without scheme, query, or credentials.
    host: &'a str,
    /// Request path without query parameters.
    path: &'a str,
    /// Effective per-request timeout in milliseconds.
    timeout_ms: u64,
}

/// Applies already-redacted logical headers to a reqwest request builder.
fn apply_headers(mut builder: RequestBuilder, headers: &[Header]) -> Result<RequestBuilder> {
    for header in headers {
        let name = HeaderName::from_bytes(header.name.as_bytes()).map_err(|source| {
            Error::InvalidInput {
                message: format!("invalid HTTP header `{}`: {source}", header.name),
            }
        })?;
        let value = HeaderValue::from_str(&header.value).map_err(|source| Error::InvalidInput {
            message: format!("invalid value for HTTP header `{}`: {source}", header.name),
        })?;
        builder = builder.header(name, value);
    }
    Ok(builder)
}

/// Sends the HTTP request and logs sanitized reqwest details on transport failure.
async fn send_request(
    builder: RequestBuilder,
    context: &TransportErrorLogContext<'_>,
) -> Result<Response> {
    match builder.send().await {
        Ok(response) => Ok(response),
        Err(source) => Err(logged_transport_error(&source, context)),
    }
}

/// Reads a successful HTTP response body and logs sanitized read failures.
async fn read_response_body(
    response: Response,
    context: &TransportErrorLogContext<'_>,
) -> Result<String> {
    match response.text().await {
        Ok(text) => Ok(text),
        Err(source) => Err(logged_transport_error(&source, context)),
    }
}

/// Converts a reqwest transport error after emitting operator diagnostics.
fn logged_transport_error(source: &reqwest::Error, context: &TransportErrorLogContext<'_>) -> Error {
    let error = ReqwestNetworkClient::transport_error(source);
    emit_transport_error_detail(source, &error, context);
    error
}

/// Copies response headers into the provider-neutral network schema.
fn response_headers(response: &Response) -> Vec<Header> {
    response
        .headers()
        .iter()
        .map(|(name, value)| Header {
            name: name.to_string(),
            value: value.to_str().unwrap_or_default().to_owned(),
        })
        .collect()
}

/// Returns whether a reqwest transport failure is worth retrying.
fn is_retryable_transport_error(source: &reqwest::Error) -> bool {
    source.is_timeout() || source.is_connect() || source.is_body() || source.is_decode()
}

/// Emits operator-only transport diagnostics without request or response bodies.
fn emit_transport_error_detail(
    source: &reqwest::Error,
    error: &Error,
    context: &TransportErrorLogContext<'_>,
) {
    tracing::warn!(
        phase = context.phase,
        attempt = context.attempt,
        correlation_id = %context.correlation_id,
        host = %context.host,
        path = %context.path,
        timeout_ms = context.timeout_ms,
        retryable = error.retryable(),
        error_detail = %safe_transport_error_detail(source),
        "outbound request transport error"
    );
}

/// Renders reqwest's error text after stripping URL credentials and queries.
fn safe_transport_error_detail(source: &reqwest::Error) -> String {
    let mut detail = source.to_string();
    if let Some(url) = source.url() {
        let mut redacted_url = url.clone();
        let _ = redacted_url.set_username("");
        let _ = redacted_url.set_password(None);
        redacted_url.set_query(None);
        redacted_url.set_fragment(None);
        detail = detail.replace(url.as_str(), redacted_url.as_str());
    }
    redact_sensitive_fragments(&detail)
}

/// Redacts common key-value secret fragments from diagnostic strings.
fn redact_sensitive_fragments(input: &str) -> String {
    let mut output = input.to_owned();
    for key in ["api_key=", "token=", "key=", "Authorization="] {
        output = redact_value_after_key(&output, key);
    }
    output
}

/// Replaces one key's value with `[REDACTED]` until a safe delimiter.
fn redact_value_after_key(input: &str, key: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;
    while let Some(index) = remaining.find(key) {
        let (before, after_before) = remaining.split_at(index);
        output.push_str(before);
        output.push_str(key);
        output.push_str("[REDACTED]");

        let after_key = &after_before[key.len()..];
        let value_end = after_key
            .find(|ch: char| ch.is_whitespace() || matches!(ch, '&' | '"' | '\'' | ')' | ','))
            .unwrap_or(after_key.len());
        remaining = &after_key[value_end..];
    }
    output.push_str(remaining);
    output
}

/// Emits the trace-level wire event capturing an outbound request body.
///
/// Internally gated on `tracing::enabled!(TRACE)` so the body rendering
/// and truncation work is skipped when no subscriber is listening at
/// trace level — keeping the cost of normal `RUST_LOG=lapis_core=debug`
/// runs effectively zero.
fn emit_outbound_wire_trace(
    correlation_id: Uuid,
    attempt: u32,
    method: &Method,
    host: &str,
    path: &str,
    body: &serde_json::Value,
) {
    if !tracing::enabled!(tracing::Level::TRACE) {
        return;
    }
    let body_str = body.to_string();
    let (rendered, truncated, body_bytes) = render_body_for_trace(&body_str, MAX_WIRE_BODY_BYTES);
    tracing::trace!(
        direction = "outbound",
        correlation_id = %correlation_id,
        attempt,
        method = %method,
        host = %host,
        path = %path,
        body_bytes,
        body_truncated = truncated,
        body = %rendered,
        "outbound request body"
    );
}

/// Emits the trace-level wire event capturing an inbound response body.
///
/// Fires for both success and non-success HTTP statuses so a single
/// trace stream contains the complete plaintext payload of every round
/// trip; gated identically to the outbound helper.
fn emit_inbound_wire_trace(
    correlation_id: Uuid,
    attempt: u32,
    host: &str,
    path: &str,
    status: u16,
    duration_ms: u64,
    text: &str,
) {
    if !tracing::enabled!(tracing::Level::TRACE) {
        return;
    }
    let (rendered, truncated, body_bytes) = render_body_for_trace(text, MAX_WIRE_BODY_BYTES);
    tracing::trace!(
        direction = "inbound",
        correlation_id = %correlation_id,
        attempt,
        host = %host,
        path = %path,
        status,
        duration_ms,
        body_bytes,
        body_truncated = truncated,
        body = %rendered,
        "inbound response body"
    );
}

fn is_retryable_status(status: u16) -> bool {
    matches!(status, 408 | 429 | 500..=599)
}

fn validate_timeout(field: &str, timeout_ms: u64) -> Result<()> {
    if timeout_ms == 0 {
        return Err(Error::InvalidInput {
            message: format!("{field} must not be zero"),
        });
    }
    Ok(())
}

/// Renders a wire body for inclusion in a trace event.
///
/// Returns a tuple of `(rendered, truncated, original_bytes)`:
/// - `rendered` is the string to emit in the `body` trace field. When the
///   raw payload fits inside `cap` it is returned verbatim; otherwise the
///   function emits a compact JSON marker of the form
///   `{"__truncated":true,"original_bytes":N,"head":"<utf8-safe prefix>"}`
///   so downstream log consumers can detect and recover from the cut.
/// - `truncated` mirrors the `body_truncated` field on the trace event.
/// - `original_bytes` is always the raw byte length of the input.
///
/// `cap` is the maximum number of bytes from `raw` that may appear in the
/// rendered output. The cut point is rounded down to the nearest UTF-8
/// char boundary so the prefix remains valid UTF-8 and the embedded
/// JSON marker is always parseable.
pub(crate) fn render_body_for_trace(raw: &str, cap: usize) -> (String, bool, usize) {
    let body_bytes = raw.len();
    if body_bytes <= cap {
        return (raw.to_owned(), false, body_bytes);
    }

    let mut cut = cap;
    while cut > 0 && !raw.is_char_boundary(cut) {
        cut -= 1;
    }

    let marker = serde_json::json!({
        "__truncated": true,
        "original_bytes": body_bytes,
        "head": &raw[..cut],
    });
    (marker.to_string(), true, body_bytes)
}

/// Trims a redacted body to at most `cap` bytes for inclusion in a
/// debug-level error event. Adds an ellipsis + byte-count suffix when
/// truncation occurs so operators can tell that the full payload is
/// available at trace level.
fn excerpt_for_debug(raw: &str, cap: usize) -> String {
    let body_bytes = raw.len();
    if body_bytes <= cap {
        return raw.to_owned();
    }

    let mut cut = cap;
    while cut > 0 && !raw.is_char_boundary(cut) {
        cut -= 1;
    }
    format!(
        "{}… ({} of {} bytes; enable reqwest_client=trace for full body)",
        &raw[..cut],
        cut,
        body_bytes
    )
}

#[async_trait::async_trait]
impl NetworkClient for ReqwestNetworkClient {
    async fn send(&self, request: NetworkRequest) -> Result<NetworkResponse> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let attempt_u32 = u32::try_from(attempt).unwrap_or(u32::MAX);
            match self.send_once(request.clone(), attempt_u32).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    let retryable = matches!(
                        error,
                        Error::HttpTransport {
                            retryable: true,
                            ..
                        } | Error::HttpStatus {
                            retryable: true,
                            ..
                        }
                    );
                    if !retryable || attempt == self.max_retries {
                        return Err(error);
                    }

                    tracing::warn!(
                        attempt = attempt_u32,
                        error = %error,
                        "retrying outbound request"
                    );
                    last_error = Some(error);
                    tokio::time::sleep(Duration::from_millis(self.retry_backoff_ms)).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::NetworkFailed {
            message: "request failed without an error".to_owned(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::{excerpt_for_debug, render_body_for_trace};

    #[test]
    fn render_body_for_trace_passes_through_short_bodies() {
        let (rendered, truncated, original) = render_body_for_trace("hello", 100);
        assert_eq!(rendered, "hello");
        assert!(!truncated);
        assert_eq!(original, 5);
    }

    #[test]
    fn render_body_for_trace_passes_through_at_exact_cap() {
        let (rendered, truncated, original) = render_body_for_trace("hello", 5);
        assert_eq!(rendered, "hello");
        assert!(!truncated);
        assert_eq!(original, 5);
    }

    #[test]
    fn render_body_for_trace_truncates_oversized_bodies() {
        let raw = "abcdefghij";
        let (rendered, truncated, original) = render_body_for_trace(raw, 3);
        assert!(truncated);
        assert_eq!(original, 10);
        let parsed: serde_json::Value =
            serde_json::from_str(&rendered).expect("truncation marker is valid JSON");
        assert_eq!(parsed["__truncated"], serde_json::Value::Bool(true));
        assert_eq!(parsed["original_bytes"], serde_json::json!(10));
        assert_eq!(parsed["head"], serde_json::Value::String("abc".to_owned()));
    }

    #[test]
    fn render_body_for_trace_respects_utf8_char_boundary() {
        // "héllo" — `é` occupies bytes 1..=2; cap=2 lands mid-character so
        // the cut must back off to byte 1 to preserve UTF-8 validity.
        let raw = "héllo";
        let (rendered, truncated, original) = render_body_for_trace(raw, 2);
        assert!(truncated);
        assert_eq!(original, 6);
        let parsed: serde_json::Value =
            serde_json::from_str(&rendered).expect("truncation marker is valid JSON");
        assert_eq!(parsed["head"], serde_json::Value::String("h".to_owned()));
    }

    #[test]
    fn render_body_for_trace_handles_zero_cap() {
        let (rendered, truncated, original) = render_body_for_trace("x", 0);
        assert!(truncated);
        assert_eq!(original, 1);
        let parsed: serde_json::Value =
            serde_json::from_str(&rendered).expect("truncation marker is valid JSON");
        assert_eq!(parsed["head"], serde_json::Value::String(String::new()));
    }

    #[test]
    fn excerpt_for_debug_passes_through_short_bodies() {
        assert_eq!(excerpt_for_debug("short", 100), "short");
    }

    #[test]
    fn excerpt_for_debug_trims_with_marker_and_byte_counts() {
        let raw = "x".repeat(500);
        let excerpt = excerpt_for_debug(&raw, 16);
        assert!(excerpt.starts_with(&"x".repeat(16)));
        assert!(excerpt.contains("16 of 500 bytes"));
        assert!(excerpt.contains("reqwest_client=trace"));
    }

    #[test]
    fn excerpt_for_debug_respects_utf8_char_boundary() {
        // 100 copies of "é" (2 bytes each = 200 bytes total). A cap of 5
        // lands mid-`é`; cut must back off to 4 (= 2 full `é` chars).
        let raw = "é".repeat(100);
        let excerpt = excerpt_for_debug(&raw, 5);
        assert!(excerpt.starts_with("éé"));
        assert!(excerpt.contains("4 of 200 bytes"));
    }
}
