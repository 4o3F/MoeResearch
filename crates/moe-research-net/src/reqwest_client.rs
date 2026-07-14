use std::fmt::Display;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::{Duration, Instant};

use eventsource_stream::Eventsource;
use futures::{Stream, StreamExt};
use moe_research_error::{Error, Result};
use tokio::sync::mpsc;

use crate::client::NetworkClient;
use crate::log_safe::{SafeText, SafeUrl, SafeWireBody};
use crate::{
    DocumentNetworkOutcome, DocumentNetworkRejection, DocumentNetworkResponse, Header,
    JsonNetworkResponse, NetworkRequest, SseEvent, SseNetworkStream,
};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Method, Proxy, RequestBuilder, Response, Url};
use uuid::Uuid;

/// Maximum byte length captured in a wire body trace event.
///
/// Caps the size of the rendered payload so a single oversized
/// model/search response cannot saturate the tracing stream. When the
/// raw body exceeds this size the trace event sets
/// `body_truncated = true` and emits a JSON marker carrying the
/// `original_bytes` count plus a UTF-8-safe `head` prefix.
pub(crate) const MAX_WIRE_BODY_BYTES: usize = 64 * 1024;

const MAX_SSE_EVENTS: usize = usize::MAX / 2;
const MAX_SSE_DATA_BYTES: usize = 4 * 1024 * 1024;
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
    /// # Errors
    /// - `Error::InvalidInput` if `default_timeout_ms` is zero.
    /// - `Error::ConfigInvalid` if `user_agent` is not a valid HTTP header
    ///   value or `proxy_url` cannot be accepted by Reqwest.
    /// - `Error::HttpTransport` if the Reqwest builder fails.
    pub fn new(
        default_timeout_ms: u64,
        max_retries: usize,
        retry_backoff_ms: u64,
        user_agent: &str,
        proxy_url: Option<&str>,
    ) -> Result<Self> {
        validate_timeout("network.inactivity_timeout_ms", default_timeout_ms)?;
        let header_value =
            HeaderValue::from_str(user_agent).map_err(|source| Error::ConfigInvalid {
                message: format!("invalid network.user_agent header: {source}"),
            })?;
        let mut builder = reqwest::Client::builder()
            .user_agent(header_value)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never());
        if let Some(proxy_url) = proxy_url {
            let proxy = Proxy::all(proxy_url).map_err(|_| Error::ConfigInvalid {
                message: "network.proxy_url is not accepted by the HTTP client".to_owned(),
            })?;
            builder = builder.no_proxy().proxy(proxy);
        }
        let client = builder
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
        let timeout_ms = request
            .inactivity_timeout_ms
            .unwrap_or(self.default_timeout_ms);
        validate_timeout("request.inactivity_timeout_ms", timeout_ms)?;
        let host = url.host_str().unwrap_or("unknown").to_owned();
        let path = url.path().to_owned();
        let correlation_id = Uuid::new_v4();

        let header_names = request
            .headers
            .iter()
            .map(|header| header.name.as_str())
            .collect::<Vec<_>>();
        tracing::debug!(
            event = "outbound_request_sending",
            status = "starting",
            provider_kind = "network",
            method = %method,
            host = %host,
            path = %path,
            header_names = ?header_names,
            timeout_ms,
            "sending outbound request"
        );

        if let Some(body) = request.body.as_ref() {
            emit_outbound_wire_trace(correlation_id, attempt, &method, &host, &path, body);
        }

        let mut builder = self.client.request(method.clone(), url);
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

    /// Sends one HTTP attempt and reads a complete binary body.
    async fn send_bytes_once(&self, request: NetworkRequest, attempt: u32) -> Result<Vec<u8>> {
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

        if !status.is_success() {
            return Err(handle_non_success_binary_response(
                &TransportErrorLogContext {
                    phase: "read_response_body",
                    attempt,
                    correlation_id,
                    host: &host,
                    path: &path,
                    timeout_ms,
                },
                status.as_u16(),
                response_headers(&response),
                started_at,
            ));
        }

        let bytes = read_response_bytes(
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
        emit_inbound_binary_response_metadata(
            correlation_id,
            attempt,
            &host,
            &path,
            status.as_u16(),
            duration_ms,
            bytes.len(),
        );

        Ok(bytes)
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

    async fn send_document_once(
        &self,
        request: NetworkRequest,
        attempt: u32,
    ) -> Result<DocumentNetworkOutcome> {
        let url = match Url::parse(&request.url) {
            Ok(url) => url,
            Err(_) => {
                return Ok(DocumentNetworkOutcome::Rejected(
                    DocumentNetworkRejection::UnsafeScheme,
                ));
            }
        };
        if url.scheme() != "https" {
            return Ok(DocumentNetworkOutcome::Rejected(
                DocumentNetworkRejection::UnsafeScheme,
            ));
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Ok(DocumentNetworkOutcome::Rejected(
                DocumentNetworkRejection::CredentialsPresent,
            ));
        }
        let Some(host) = url.host_str() else {
            return Ok(DocumentNetworkOutcome::Rejected(
                DocumentNetworkRejection::UnsafeHost,
            ));
        };
        if unsafe_document_hostname(host) {
            return Ok(DocumentNetworkOutcome::Rejected(
                DocumentNetworkRejection::UnsafeHost,
            ));
        }
        let resolved_host = host.trim_start_matches('[').trim_end_matches(']');
        let timeout_ms = request
            .inactivity_timeout_ms
            .unwrap_or(self.default_timeout_ms);
        validate_timeout("request.inactivity_timeout_ms", timeout_ms)?;
        let timeout = Duration::from_millis(timeout_ms);
        let port = url.port_or_known_default().unwrap_or(443);
        let addrs = if let Ok(ip) = resolved_host.parse::<IpAddr>() {
            vec![SocketAddr::new(ip, port)]
        } else {
            match tokio::time::timeout(timeout, tokio::net::lookup_host((resolved_host, port)))
                .await
            {
                Ok(Ok(addrs)) => addrs.collect::<Vec<_>>(),
                Ok(Err(_)) | Err(_) => {
                    return Ok(DocumentNetworkOutcome::Rejected(
                        DocumentNetworkRejection::DnsResolutionFailed,
                    ));
                }
            }
        };
        if addrs.is_empty() {
            return Ok(DocumentNetworkOutcome::Rejected(
                DocumentNetworkRejection::DnsResolutionFailed,
            ));
        }
        if addrs.iter().any(|addr| !is_public_document_ip(addr.ip())) {
            return Ok(DocumentNetworkOutcome::Rejected(
                DocumentNetworkRejection::UnsafeResolvedAddress,
            ));
        }

        let RequestAttempt {
            builder,
            attempt,
            host,
            path,
            timeout_ms,
            correlation_id,
        } = self.prepare_request(request, attempt)?;
        let context = TransportErrorLogContext {
            phase: "send_request",
            attempt,
            correlation_id,
            host: &host,
            path: &path,
            timeout_ms,
        };
        let started_at = Instant::now();
        let response = send_request(builder, &context).await?;
        let status = response.status().as_u16();
        let headers = document_response_headers(&response);
        let emit_metadata = |body_bytes| {
            emit_inbound_binary_response_metadata(
                correlation_id,
                attempt,
                &host,
                &path,
                status,
                u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX),
                body_bytes,
            );
        };

        if response
            .headers()
            .get(reqwest::header::CONTENT_ENCODING)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                !value.trim().is_empty() && !value.eq_ignore_ascii_case("identity")
            })
        {
            emit_metadata(0);
            return Ok(DocumentNetworkOutcome::Rejected(
                DocumentNetworkRejection::UnsupportedContentEncoding,
            ));
        }
        if !(200..300).contains(&status) {
            emit_metadata(0);
            return Ok(DocumentNetworkOutcome::Response(DocumentNetworkResponse {
                status,
                headers,
                body: Vec::new(),
            }));
        }

        let body = read_response_bytes(
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
        emit_metadata(body.len());

        Ok(DocumentNetworkOutcome::Response(DocumentNetworkResponse {
            status,
            headers,
            body,
        }))
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
        let first_event =
            match tokio::time::timeout(Duration::from_millis(timeout_ms), stream.next()).await {
                Ok(Some(Ok(event))) => event,
                Ok(Some(Err(source))) => return Err(sse_stream_error(&source, true)),
                Ok(None) => {
                    return Err(Error::NetworkFailed {
                        message: "SSE stream ended without events".to_owned(),
                    });
                }
                Err(_) => {
                    return Err(Error::HttpTransport {
                        message: "SSE stream idle timeout before first event".to_owned(),
                        retryable: true,
                    });
                }
            };
        tracing::trace!(
            event = "sse_event_received",
            status = "streaming",
            provider_kind = "network",
            correlation_id = %correlation_id,
            attempt,
            host = %host,
            path = %path,
            sse_event_type = %first_event.event,
            data_bytes = first_event.data.len(),
            "inbound SSE event metadata"
        );
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
                timeout_ms,
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

    /// Sends a request and returns a complete binary response body.
    ///
    /// This shares request preparation, timeout, retry, and status handling
    /// with JSON/SSE requests while intentionally omitting binary payloads
    /// from wire traces.
    pub async fn send_bytes(&self, request: NetworkRequest) -> Result<Vec<u8>> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let attempt_u32 = u32::try_from(attempt).unwrap_or(u32::MAX);
            match self.send_bytes_once(request.clone(), attempt_u32).await {
                Ok(bytes) => return Ok(bytes),
                Err(error) => {
                    let retryable = is_retryable_error(&error);
                    if !retryable || attempt == self.max_retries {
                        return Err(error);
                    }

                    tracing::warn!(
                        event = "outbound_request_retrying",
                        status = "retrying",
                        provider_kind = "network",
                        response_kind = "binary",
                        attempt = attempt_u32,
                        next_attempt = attempt_u32.saturating_add(1),
                        max_retries = self.max_retries,
                        backoff_ms = self.retry_backoff_ms,
                        error_code = error.code().as_str(),
                        retryable = error.retryable(),
                        "retrying outbound binary request"
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

    /// Maps reqwest transport failures into MoeResearch retry-aware errors.
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

/// Applies logical headers to a reqwest request builder.
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
    timeout_ms: u64,
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

fn validate_document_request(request: &NetworkRequest) -> Result<()> {
    if !request.method.eq_ignore_ascii_case("GET") {
        return Err(Error::InvalidInput {
            message: "document request method must be GET".to_owned(),
        });
    }
    if request.body.is_some() {
        return Err(Error::InvalidInput {
            message: "document request body must be empty".to_owned(),
        });
    }

    let mut accept = false;
    let mut accept_encoding = false;
    for header in &request.headers {
        if header.name.eq_ignore_ascii_case("accept") {
            if accept || header.value != "text/html,text/plain,text/markdown,application/xhtml+xml"
            {
                return Err(Error::InvalidInput {
                    message: "document request Accept header is invalid".to_owned(),
                });
            }
            accept = true;
        } else if header.name.eq_ignore_ascii_case("accept-encoding") {
            if accept_encoding || !header.value.eq_ignore_ascii_case("identity") {
                return Err(Error::InvalidInput {
                    message: "document request Accept-Encoding header must be identity".to_owned(),
                });
            }
            accept_encoding = true;
        } else {
            return Err(Error::InvalidInput {
                message: "document request contains an unsupported header".to_owned(),
            });
        }
    }
    if !accept || !accept_encoding {
        return Err(Error::InvalidInput {
            message: "document request requires Accept and Accept-Encoding headers".to_owned(),
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
    let header_names = headers
        .iter()
        .map(|header| header.name.as_str())
        .collect::<Vec<_>>();
    tracing::debug!(
        event = "outbound_response_non_success",
        status = "failed",
        provider_kind = "network",
        http_status = status,
        host = %context.host,
        path = %context.path,
        header_names = ?header_names,
        body_bytes = text.len(),
        body_excerpt = "[REDACTED]; enable reqwest_client=trace for redacted wire metadata",
        error_code = "http_status",
        retryable = is_retryable_status(status),
        "outbound response returned non-success status"
    );
    Err(Error::HttpStatus {
        status,
        message: "provider returned non-success status".to_owned(),
        retryable: is_retryable_status(status),
    })
}

fn handle_non_success_binary_response(
    context: &TransportErrorLogContext<'_>,
    status: u16,
    headers: Vec<Header>,
    started_at: Instant,
) -> Error {
    let duration_ms = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
    let header_names = headers
        .iter()
        .map(|header| header.name.as_str())
        .collect::<Vec<_>>();
    tracing::debug!(
        event = "outbound_response_non_success",
        status = "failed",
        provider_kind = "network",
        response_kind = "binary",
        http_status = status,
        host = %context.host,
        path = %context.path,
        header_names = ?header_names,
        duration_ms,
        error_code = "http_status",
        retryable = is_retryable_status(status),
        "outbound binary response returned non-success status"
    );
    Error::HttpStatus {
        status,
        message: "provider returned non-success status".to_owned(),
        retryable: is_retryable_status(status),
    }
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
                    event = "sse_receiver_dropped",
                    status = "closed",
                    provider_kind = "network",
                    http_status = context.status,
                    host = %context.host,
                    path = %context.path,
                    event_count,
                    total_data_bytes,
                    "SSE receiver dropped; closing response stream"
                );
                return;
            }
            next = next_sse_event_with_idle_timeout(&mut stream, context.timeout_ms) => {
                let event = match next {
                    Ok(Some(event)) => event,
                    Ok(None) => break,
                    Err(error) => {
                        let _ = sender.send(Err(error)).await;
                        return;
                    }
                };
                tracing::trace!(
                    event = "sse_event_received",
                    status = "streaming",
                    provider_kind = "network",
                    correlation_id = %context.correlation_id,
                    attempt = context.attempt,
                    host = %context.host,
                    path = %context.path,
                    sse_event_type = %event.event,
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
        event = "sse_upstream_ended",
        status = "ok",
        provider_kind = "network",
        http_status = context.status,
        host = %context.host,
        path = %context.path,
        event_count,
        total_data_bytes,
        "SSE upstream stream ended"
    );
}

async fn next_sse_event_with_idle_timeout<S, E>(
    stream: &mut S,
    timeout_ms: u64,
) -> Result<Option<eventsource_stream::Event>>
where
    S: Stream<Item = std::result::Result<eventsource_stream::Event, E>> + Unpin,
    E: Display,
{
    match tokio::time::timeout(Duration::from_millis(timeout_ms), stream.next()).await {
        Ok(Some(Ok(event))) => Ok(Some(event)),
        Ok(Some(Err(source))) => Err(sse_stream_error(&source, false)),
        Ok(None) => Ok(None),
        Err(_) => Err(Error::HttpTransport {
            message: "SSE stream idle timeout waiting for next event".to_owned(),
            retryable: true,
        }),
    }
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
    match tokio::time::timeout(Duration::from_millis(context.timeout_ms), builder.send()).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(source)) => Err(logged_transport_error(&source, context)),
        Err(_) => Err(Error::HttpTransport {
            message: "network inactivity timeout while sending request".to_owned(),
            retryable: true,
        }),
    }
}

/// Reads a successful HTTP response body and logs sanitized read failures.
async fn read_response_body(
    response: Response,
    context: &TransportErrorLogContext<'_>,
) -> Result<String> {
    match tokio::time::timeout(Duration::from_millis(context.timeout_ms), response.text()).await {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(source)) => Err(logged_transport_error(&source, context)),
        Err(_) => Err(Error::HttpTransport {
            message: "network inactivity timeout while reading response body".to_owned(),
            retryable: true,
        }),
    }
}

/// Reads a successful binary response without emitting a body wire trace.
async fn read_response_bytes(
    response: Response,
    context: &TransportErrorLogContext<'_>,
) -> Result<Vec<u8>> {
    match tokio::time::timeout(Duration::from_millis(context.timeout_ms), response.bytes()).await {
        Ok(Ok(bytes)) => Ok(bytes.to_vec()),
        Ok(Err(source)) => Err(logged_transport_error(&source, context)),
        Err(_) => Err(Error::HttpTransport {
            message: "network inactivity timeout while reading response body".to_owned(),
            retryable: true,
        }),
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
    let detail = source.to_string();
    let detail = SafeText::new(&detail).to_string();
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
    let error_detail = transport_error_detail(source);
    tracing::warn!(
        event = "outbound_request_transport_error",
        status = "failed",
        provider_kind = "network",
        phase = context.phase,
        attempt = context.attempt,
        correlation_id = %context.correlation_id,
        host = %context.host,
        path = %context.path,
        timeout_ms = context.timeout_ms,
        error_code = error.code().as_str(),
        retryable = error.retryable(),
        error_message = %SafeText::new(&error_detail),
        "outbound request transport error"
    );
}

fn transport_error_detail(source: &reqwest::Error) -> String {
    let mut detail = source.to_string();
    if let Some(url) = source.url() {
        detail = detail.replace(url.as_str(), &SafeUrl::new(url.as_str()).to_string());
    }
    detail
}

/// Emits the trace-level wire event capturing an outbound request body.
///
/// Internally gated on the compatibility trace target so body rendering
/// and truncation work is skipped when no subscriber is listening at
/// trace level — keeping the cost of normal `RUST_LOG=moe_research_net=debug`
/// runs effectively zero.
fn emit_outbound_wire_trace(
    correlation_id: Uuid,
    attempt: u32,
    method: &Method,
    host: &str,
    path: &str,
    body: &serde_json::Value,
) {
    if !tracing::enabled!(target: "moe_research_net::reqwest_client", tracing::Level::TRACE) {
        return;
    }
    let body_str = body.to_string();
    let body_bytes = body_str.len();
    let truncated = body_bytes > MAX_WIRE_BODY_BYTES;
    tracing::trace!(
        event = "wire_body_recorded",
        status = "ok",
        provider_kind = "network",
        direction = "outbound",
        correlation_id = %correlation_id,
        attempt,
        method = %method,
        host = %host,
        path = %path,
        body_bytes,
        body_truncated = truncated,
        body = %SafeWireBody::new(&body_str, MAX_WIRE_BODY_BYTES),
        "outbound request body"
    );
}

/// Emits metadata for a binary response without rendering its body.
fn emit_inbound_binary_response_metadata(
    correlation_id: Uuid,
    attempt: u32,
    host: &str,
    path: &str,
    status: u16,
    duration_ms: u64,
    body_bytes: usize,
) {
    tracing::debug!(
        event = "outbound_binary_response_received",
        status = "ok",
        provider_kind = "network",
        correlation_id = %correlation_id,
        attempt,
        host = %host,
        path = %path,
        http_status = status,
        duration_ms,
        body_bytes,
        "inbound binary response metadata"
    );
}

/// Emits the trace-level wire event capturing an inbound response body.
///
/// Fires for both success and non-success HTTP statuses so a single
/// trace stream contains the redacted payload of every round trip;
/// gated identically to the outbound helper.
fn emit_inbound_wire_trace(
    correlation_id: Uuid,
    attempt: u32,
    host: &str,
    path: &str,
    status: u16,
    duration_ms: u64,
    text: &str,
) {
    if !tracing::enabled!(target: "moe_research_net::reqwest_client", tracing::Level::TRACE) {
        return;
    }
    let body_bytes = text.len();
    let truncated = body_bytes > MAX_WIRE_BODY_BYTES;
    tracing::trace!(
        event = "wire_body_recorded",
        status = "ok",
        provider_kind = "network",
        direction = "inbound",
        correlation_id = %correlation_id,
        attempt,
        host = %host,
        path = %path,
        status,
        duration_ms,
        body_bytes,
        body_truncated = truncated,
        body = %SafeWireBody::new(text, MAX_WIRE_BODY_BYTES),
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

fn unsafe_document_hostname(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host == "metadata.google.internal"
}

fn is_public_document_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_document_ipv4(ip),
        IpAddr::V6(ip) => is_public_document_ipv6(ip),
    }
}

fn is_public_document_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !matches!(
        (a, b, c),
        (0, _, _)
            | (10, _, _)
            | (100, 64..=127, _)
            | (127, _, _)
            | (169, 254, _)
            | (172, 16..=31, _)
            | (192, 0, _)
            | (192, 168, _)
            | (198, 18..=19, _)
            | (198, 51, 100)
            | (203, 0, 113)
            | (224..=255, _, _)
    )
}

fn is_public_document_ipv6(ip: Ipv6Addr) -> bool {
    if let Some(ipv4) = ip.to_ipv4() {
        return is_public_document_ipv4(ipv4);
    }
    let segments = ip.segments();
    if ip.is_unspecified() || ip.is_loopback() || ip.is_multicast() {
        return false;
    }
    if segments[0] & 0xfe00 == 0xfc00 || segments[0] & 0xffc0 == 0xfe80 {
        return false;
    }
    if segments[0] == 0x0064 && segments[1] == 0xff9b {
        return false;
    }
    if segments[0] == 0x0100 && segments[1..4] == [0, 0, 0] {
        return false;
    }
    if segments[0] == 0x2001 && (segments[1] <= 0x01ff || segments[1] == 0x0db8) {
        return false;
    }
    if segments[0] & 0xfff0 == 0x3ff0 || segments[0] == 0x5f00 {
        return false;
    }
    true
}

fn document_response_headers(response: &Response) -> Vec<Header> {
    [
        reqwest::header::LOCATION,
        reqwest::header::CONTENT_TYPE,
        reqwest::header::CONTENT_LENGTH,
        reqwest::header::CONTENT_ENCODING,
    ]
    .into_iter()
    .filter_map(|name| {
        response.headers().get(&name).and_then(|value| {
            value.to_str().ok().map(|value| Header {
                name: name.as_str().to_owned(),
                value: value.to_owned(),
            })
        })
    })
    .collect()
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
                        event = "outbound_request_retrying",
                        status = "retrying",
                        provider_kind = "network",
                        attempt = attempt_u32,
                        next_attempt = attempt_u32.saturating_add(1),
                        max_retries = self.max_retries,
                        backoff_ms = self.retry_backoff_ms,
                        error_code = error.code().as_str(),
                        retryable = error.retryable(),
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
                        event = "outbound_request_retrying",
                        status = "retrying",
                        provider_kind = "network",
                        attempt = attempt_u32,
                        next_attempt = attempt_u32.saturating_add(1),
                        max_retries = self.max_retries,
                        backoff_ms = self.retry_backoff_ms,
                        error_code = error.code().as_str(),
                        retryable = error.retryable(),
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

    async fn send_document(&self, request: NetworkRequest) -> Result<DocumentNetworkOutcome> {
        validate_document_request(&request)?;
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let attempt_u32 = u32::try_from(attempt).unwrap_or(u32::MAX);
            match self.send_document_once(request.clone(), attempt_u32).await {
                Ok(outcome) => {
                    let retry_code = match &outcome {
                        DocumentNetworkOutcome::Response(response)
                            if is_retryable_status(response.status) =>
                        {
                            Some("http_status")
                        }
                        DocumentNetworkOutcome::Rejected(
                            DocumentNetworkRejection::DnsResolutionFailed,
                        ) => Some("dns_resolution_failed"),
                        _ => None,
                    };
                    let Some(retry_code) = retry_code else {
                        return Ok(outcome);
                    };
                    if attempt == self.max_retries {
                        return Ok(outcome);
                    }

                    tracing::warn!(
                        event = "outbound_request_retrying",
                        status = "retrying",
                        provider_kind = "network",
                        response_kind = "document",
                        attempt = attempt_u32,
                        next_attempt = attempt_u32.saturating_add(1),
                        max_retries = self.max_retries,
                        backoff_ms = self.retry_backoff_ms,
                        error_code = retry_code,
                        retryable = true,
                        "retrying outbound document request"
                    );
                }
                Err(error) => {
                    let retryable = is_retryable_error(&error);
                    if !retryable || attempt == self.max_retries {
                        return Err(error);
                    }

                    tracing::warn!(
                        event = "outbound_request_retrying",
                        status = "retrying",
                        provider_kind = "network",
                        response_kind = "document",
                        attempt = attempt_u32,
                        next_attempt = attempt_u32.saturating_add(1),
                        max_retries = self.max_retries,
                        backoff_ms = self.retry_backoff_ms,
                        error_code = error.code().as_str(),
                        retryable = error.retryable(),
                        "retrying outbound document request"
                    );
                    last_error = Some(error);
                }
            }
            tokio::time::sleep(Duration::from_millis(self.retry_backoff_ms)).await;
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
