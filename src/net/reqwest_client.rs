use std::time::Duration;

use crate::error::{Error, Result};
use crate::net::client::NetworkClient;
use crate::net::policy::redact_headers;
use crate::schema::common::{Header, NetworkRequest, NetworkResponse};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Method, Url};

pub struct ReqwestNetworkClient {
    client: reqwest::Client,
    default_timeout_ms: u64,
    max_retries: usize,
    retry_backoff_ms: u64,
}

impl ReqwestNetworkClient {
    pub fn new(default_timeout_ms: u64, max_retries: usize, retry_backoff_ms: u64) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|source| Self::transport_error(&source))?;

        Ok(Self {
            client,
            default_timeout_ms,
            max_retries,
            retry_backoff_ms,
        })
    }

    async fn send_once(&self, request: NetworkRequest) -> Result<NetworkResponse> {
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
        let host = url.host_str().unwrap_or("unknown").to_owned();
        let path = url.path().to_owned();

        tracing::debug!(
            method = %method,
            host = %host,
            path = %path,
            headers = ?redact_headers(&request.headers),
            timeout_ms,
            "sending outbound request"
        );

        let mut builder = self
            .client
            .request(method, url)
            .timeout(Duration::from_millis(timeout_ms));

        for header in &request.headers {
            let name = HeaderName::from_bytes(header.name.as_bytes()).map_err(|source| {
                Error::InvalidInput {
                    message: format!("invalid HTTP header `{}`: {source}", header.name),
                }
            })?;
            let value =
                HeaderValue::from_str(&header.value).map_err(|source| Error::InvalidInput {
                    message: format!("invalid value for HTTP header `{}`: {source}", header.name),
                })?;
            builder = builder.header(name, value);
        }

        if let Some(body) = request.body {
            builder = builder.json(&body);
        }

        let response = builder
            .send()
            .await
            .map_err(|source| Self::transport_error(&source))?;
        let status = response.status();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| Header {
                name: name.to_string(),
                value: value.to_str().unwrap_or_default().to_owned(),
            })
            .collect();
        let text = response
            .text()
            .await
            .map_err(|source| Self::transport_error(&source))?;
        let body = serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text));

        if !status.is_success() {
            return Err(Error::HttpStatus {
                status: status.as_u16(),
                message: "provider returned non-success status".to_owned(),
                retryable: is_retryable_status(status.as_u16()),
            });
        }

        Ok(NetworkResponse {
            status: status.as_u16(),
            headers,
            body,
        })
    }

    fn transport_error(source: &reqwest::Error) -> Error {
        let retryable = source.is_timeout() || source.is_connect();
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

fn is_retryable_status(status: u16) -> bool {
    matches!(status, 408 | 429 | 500..=599)
}

#[async_trait::async_trait]
impl NetworkClient for ReqwestNetworkClient {
    async fn send(&self, request: NetworkRequest) -> Result<NetworkResponse> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.send_once(request.clone()).await {
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

                    tracing::warn!(attempt, error = %error, "retrying outbound request");
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
