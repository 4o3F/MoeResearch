use std::fmt::Display;
use std::time::{Duration, Instant};

use eventsource_stream::Eventsource;
use futures::{Stream, StreamExt};
use lapis_error::{Error, Result};
use tokio::sync::mpsc;

use crate::client::NetworkClient;
use crate::policy::RedactionPolicy;
use crate::{Header, JsonNetworkResponse, NetworkRequest, SseEvent, SseNetworkStream};
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
const MAX_SSE_EVENTS: usize = usize::MAX / 2;
const MAX_SSE_DATA_BYTES: usize = MAX_WIRE_BODY_BYTES;
const MAX_SSE_TOTAL_DATA_BYTES: usize = 8 * 1024 * 1024;
const SSE_EVENT_CHANNEL_CAPACITY: usize = 32;

pub struct ReqwestNetworkClient {
    client: reqwest::Client,
    default_timeout_ms: u64,
    max_retries: usize,
    retry_backoff_ms: u64,
}

struct RequestAttempt {
    builder: RequestBuilder,
    attempt: u32,
    host: String,
    path: String,
    timeout_ms: u64,
    correlation_id: Uuid,
}

impl ReqwestNetworkClient {
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

    /// Validates request metadata and builds the reqwest request for one attempt.
    fn prepare_request(&self, request: NetworkRequest, attempt: u32) -> Result<RequestAttempt> {
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
        let correlation_id = Uuid::new_v4();
        let redaction = RedactionPolicy;

        tracing::debug!(
            target: "lapis_core::net::reqwest_client",
            method = %method,
            host = %host,
            path = %path,
            headers = ?redaction.redact_headers(&request.headers),
            timeout_ms,
            "sending outbound request"
        );

        if let Some(body) = request.body.as_ref() {
            emit_outbound_wire_trace(correlation_id, attempt, &method, &host, &path, body);
        }

        let mut builder = self
            .client
            .request(method.clone(), url)
            .timeout(Duration::from_millis(timeout_ms));
        builder = apply_headers(builder, &request.headers)?;

        if let Some(body) = request.body {
            builder = builder.json(&body);
        }

        Ok(RequestAttempt {
            builder,
            attempt,
            host,
            path,
            timeout_ms,
            correlation_id,
        })
    }

    /// Sends one HTTP attempt and reads a complete JSON/text body.
    async fn send_json_once(
        &self,
        request: NetworkRequest,
        attempt: u32,
    ) -> Result<JsonNetworkResponse> {
        let RequestAttempt {
            builder,
            attempt,
            host,
            path,
            timeout_ms,
            correlation_id,
        } = self.prepare_request(request, attempt)?;
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

        if !status.is_success() {
            return handle_non_success_response(
                response,
                &TransportErrorLogContext {
                    phase: "read_response_body",
                    attempt,
                    correlation_id,
                    host: &host,
                    path: &path,
                    timeout_ms,
                },
                status.as_u16(),
                headers,
                started_at,
            )
            .await;
        }

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
        emit_inbound_wire_trace(
            correlation_id,
            attempt,
            &host,
            &path,
            status.as_u16(),
            duration_ms,
            &text,
        );

        let body = serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text));

        Ok(JsonNetworkResponse {
            status: status.as_u16(),
            headers,
            body,
        })
    }

    /// Sends one HTTP attempt and returns a lazy SSE event stream.
    async fn send_sse_once(
        &self,
        request: NetworkRequest,
        attempt: u32,
    ) -> Result<SseNetworkStream> {
        let RequestAttempt {
            builder,
            attempt,
            host,
            path,
            timeout_ms,
            correlation_id,
        } = self.prepare_request(request, attempt)?;
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

        if !status.is_success() {
            return match handle_non_success_response(
                response,
                &TransportErrorLogContext {
                    phase: "read_response_body",
                    attempt,
                    correlation_id,
                    host: &host,
                    path: &path,
                    timeout_ms,
                },
                status.as_u16(),
                headers,
                started_at,
            )
            .await
            {
                Err(error) => Err(error),
                Ok(_) => Err(Error::NetworkFailed {
                    message: "non-success SSE response unexpectedly succeeded".to_owned(),
                }),
            };
        }

        let mut stream = response.bytes_stream().eventsource();
        let mut event_count = 0usize;
        let mut total_data_bytes = 0usize;
        let first_event = match stream.next().await {
            Some(Ok(event)) => event,
            Some(Err(source)) => return Err(sse_stream_error(&source, true)),
            None => {
                return Err(Error::NetworkFailed {
                    message: "SSE stream ended without events".to_owned(),
                });
            }
        };
        enforce_sse_caps(
            first_event.data.len(),
            &mut event_count,
            &mut total_data_bytes,
        )?;

        let (sender, receiver) = mpsc::channel(SSE_EVENT_CHANNEL_CAPACITY);
        sender
            .send(Ok(SseEvent {
                event: first_event.event,
                data: first_event.data,
            }))
            .await
            .map_err(|_| Error::NetworkFailed {
                message: "SSE receiver closed before stream handoff".to_owned(),
            })?;
        let reader = tokio::spawn(pump_sse_events(
            stream,
            sender,
            SsePumpContext {
                attempt,
                correlation_id,
                host,
                path,
                status: status.as_u16(),
            },
            event_count,
            total_data_bytes,
        ));

        Ok(SseNetworkStream::new(
            status.as_u16(),
            headers,
            receiver,
            reader,
        ))
    }

    /// Maps reqwest transport failures into Lapis retry-aware errors.
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

struct SsePumpContext {
    attempt: u32,
    correlation_id: Uuid,
    host: String,
    path: String,
    status: u16,
}

fn validate_accept_header(headers: &[Header], expected: &str) -> Result<()> {
    let mut saw_accept = false;
    let mut saw_expected = false;

    for header in headers
        .iter()
        .filter(|header| header.name.eq_ignore_ascii_case("accept"))
    {
        saw_accept = true;
        for value in header.value.split(',') {
            let media_type = value.split(';').next().unwrap_or_default().trim();
            if media_type.is_empty() {
                continue;
            }
            if media_type.eq_ignore_ascii_case(expected) {
                saw_expected = true;
            } else if media_type.eq_ignore_ascii_case("application/json")
                || media_type.eq_ignore_ascii_case("text/event-stream")
            {
                return Err(Error::InvalidInput {
                    message: format!(
                        "request expected Accept {expected}, got incompatible {media_type}"
                    ),
                });
            } else {
                return Err(Error::InvalidInput {
                    message: format!("unsupported Accept header response type {media_type}"),
                });
            }
        }
    }

    if !saw_accept {
        return Err(Error::InvalidInput {
            message: "missing Accept header".to_owned(),
        });
    }
    if !saw_expected {
        return Err(Error::InvalidInput {
            message: format!("request missing expected Accept {expected}"),
        });
    }
    Ok(())
}

/// Reads and logs a non-success provider response before returning `HttpStatus`.
async fn handle_non_success_response(
    response: Response,
    context: &TransportErrorLogContext<'_>,
    status: u16,
    headers: Vec<Header>,
    started_at: Instant,
) -> Result<JsonNetworkResponse> {
    let text = read_response_body(response, context).await?;
    let duration_ms = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
    emit_inbound_wire_trace(
        context.correlation_id,
        context.attempt,
        context.host,
        context.path,
        status,
        duration_ms,
        &text,
    );
    let redaction = RedactionPolicy;
    let redacted = redaction.redact_body_text(&text);
    let excerpt = excerpt_for_debug(&redacted, MAX_DEBUG_EXCERPT_BYTES);
    tracing::debug!(
        target: "lapis_core::net::reqwest_client",
        status,
        host = %context.host,
        path = %context.path,
        headers = ?redaction.redact_headers(&headers),
        body_excerpt = %excerpt,
        "outbound response returned non-success status"
    );
    Err(Error::HttpStatus {
        status,
        message: "provider returned non-success status".to_owned(),
        retryable: is_retryable_status(status),
    })
}

async fn pump_sse_events<S, E>(
    mut stream: S,
    sender: mpsc::Sender<Result<SseEvent>>,
    context: SsePumpContext,
    mut event_count: usize,
    mut total_data_bytes: usize,
) where
    S: Stream<Item = std::result::Result<eventsource_stream::Event, E>> + Unpin,
    E: Display,
{
    loop {
        tokio::select! {
            _ = sender.closed() => {
                tracing::debug!(
                    target: "lapis_core::net::reqwest_client",
                    status = context.status,
                    host = %context.host,
                    path = %context.path,
                    event_count,
                    total_data_bytes,
                    "SSE receiver dropped; closing response stream"
                );
                return;
            }
            next = stream.next() => {
                let Some(next) = next else {
                    break;
                };
                let event = match next {
                    Ok(event) => event,
                    Err(source) => {
                        let _ = sender.send(Err(sse_stream_error(&source, false))).await;
                        return;
                    }
                };
                tracing::trace!(
                    target: "lapis_core::net::reqwest_client",
                    correlation_id = %context.correlation_id,
                    attempt = context.attempt,
                    host = %context.host,
                    path = %context.path,
                    event = %event.event,
                    data_bytes = event.data.len(),
                    "inbound SSE event metadata"
                );
                if let Err(error) = enforce_sse_caps(
                    event.data.len(),
                    &mut event_count,
                    &mut total_data_bytes,
                ) {
                    let _ = sender.send(Err(error)).await;
                    return;
                }
                if sender
                    .send(Ok(SseEvent {
                        event: event.event,
                        data: event.data,
                    }))
                    .await
                    .is_err()
                {
                    return;
                }
            }
        }
    }

    tracing::debug!(
        target: "lapis_core::net::reqwest_client",
        status = context.status,
        host = %context.host,
        path = %context.path,
        event_count,
        total_data_bytes,
        "SSE upstream stream ended"
    );
}

fn enforce_sse_caps(
    data_len: usize,
    event_count: &mut usize,
    total_data_bytes: &mut usize,
) -> Result<()> {
    if *event_count >= MAX_SSE_EVENTS {
        return Err(Error::HttpTransport {
            message: "SSE stream exceeded event limit".to_owned(),
            retryable: false,
        });
    }
    if data_len > MAX_SSE_DATA_BYTES {
        return Err(Error::HttpTransport {
            message: "SSE event exceeded data limit".to_owned(),
            retryable: false,
        });
    }
    *total_data_bytes = total_data_bytes.saturating_add(data_len);
    if *total_data_bytes > MAX_SSE_TOTAL_DATA_BYTES {
        return Err(Error::HttpTransport {
            message: "SSE stream exceeded total data limit".to_owned(),
            retryable: false,
        });
    }
    *event_count += 1;
    Ok(())
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
fn logged_transport_error(
    source: &reqwest::Error,
    context: &TransportErrorLogContext<'_>,
) -> Error {
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

/// Converts SSE parser failures into public-safe retry-aware transport errors.
fn sse_stream_error(source: &impl Display, retryable: bool) -> Error {
    let detail = redact_sensitive_fragments(&source.to_string());
    Error::HttpTransport {
        message: format!("SSE stream handling failed: {detail}"),
        retryable,
    }
}

/// Emits operator-only transport diagnostics without request or response bodies.
fn emit_transport_error_detail(
    source: &reqwest::Error,
    error: &Error,
    context: &TransportErrorLogContext<'_>,
) {
    tracing::warn!(
        target: "lapis_core::net::reqwest_client",
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
/// Internally gated on the compatibility trace target so body rendering
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
    if !tracing::enabled!(target: "lapis_core::net::reqwest_client", tracing::Level::TRACE) {
        return;
    }
    let body_str = body.to_string();
    let (rendered, truncated, body_bytes) = render_body_for_trace(&body_str, MAX_WIRE_BODY_BYTES);
    tracing::trace!(
        target: "lapis_core::net::reqwest_client",
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
    if !tracing::enabled!(target: "lapis_core::net::reqwest_client", tracing::Level::TRACE) {
        return;
    }
    let (rendered, truncated, body_bytes) = render_body_for_trace(text, MAX_WIRE_BODY_BYTES);
    tracing::trace!(
        target: "lapis_core::net::reqwest_client",
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
    async fn send_json(&self, request: NetworkRequest) -> Result<JsonNetworkResponse> {
        validate_accept_header(&request.headers, "application/json")?;
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let attempt_u32 = u32::try_from(attempt).unwrap_or(u32::MAX);
            match self.send_json_once(request.clone(), attempt_u32).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    let retryable = is_retryable_error(&error);
                    if !retryable || attempt == self.max_retries {
                        return Err(error);
                    }

                    tracing::warn!(
                        target: "lapis_core::net::reqwest_client",
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

    async fn send_sse(&self, request: NetworkRequest) -> Result<SseNetworkStream> {
        validate_accept_header(&request.headers, "text/event-stream")?;
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let attempt_u32 = u32::try_from(attempt).unwrap_or(u32::MAX);
            match self.send_sse_once(request.clone(), attempt_u32).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    let retryable = is_retryable_error(&error);
                    if !retryable || attempt == self.max_retries {
                        return Err(error);
                    }

                    tracing::warn!(
                        target: "lapis_core::net::reqwest_client",
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

fn is_retryable_error(error: &Error) -> bool {
    matches!(
        error,
        Error::HttpTransport {
            retryable: true,
            ..
        } | Error::HttpStatus {
            retryable: true,
            ..
        }
    )
}
