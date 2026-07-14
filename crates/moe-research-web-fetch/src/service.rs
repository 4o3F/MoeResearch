use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use moe_research_error::{Error, ErrorCode, Result};
use moe_research_model::{ModelService, TokenUsage};
use moe_research_net::{
    DocumentNetworkOutcome, DocumentNetworkRejection, DocumentNetworkResponse, Header,
    NetworkClient, NetworkRequest,
};
use time::OffsetDateTime;
use url::Url;

use crate::cache::DocumentCache;
use crate::document::convert_document;
use crate::flight::FetchCoordinator;
use crate::model::{self, WebFetchAnswerOutcome};

#[derive(Clone, Debug)]
pub struct WebFetchRuntimeConfig {
    pub cache_ttl_ms: u64,
    pub max_cache_entries: usize,
    pub max_redirects: usize,
    pub inactivity_timeout_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct WebFetchDocument {
    pub requested_url: String,
    pub final_url: String,
    pub title: String,
    pub content_type: String,
    pub markdown: Arc<str>,
    pub retrieved_at: OffsetDateTime,
}

#[derive(Clone, Debug)]
pub enum WebFetchDocumentOutcome {
    Document(Arc<WebFetchDocument>),
    Redirect { redirect_url: String },
    SoftError(WebFetchSoftError),
}

#[derive(Clone, Debug)]
pub struct WebFetchSoftError {
    pub code: &'static str,
    pub retryable: bool,
    pub message: &'static str,
    pub token_usage: Option<TokenUsage>,
}

impl WebFetchSoftError {
    #[must_use]
    pub const fn new(code: &'static str, retryable: bool, message: &'static str) -> Self {
        Self {
            code,
            retryable,
            message,
            token_usage: None,
        }
    }

    #[must_use]
    pub fn with_token_usage(mut self, token_usage: Option<TokenUsage>) -> Self {
        self.token_usage = token_usage;
        self
    }
}

enum PromptProcessor {
    Disabled,
    Enabled {
        model_service: Arc<ModelService>,
        model_provider: String,
    },
}

pub struct WebFetchService {
    network: Arc<dyn NetworkClient>,
    prompt_processor: PromptProcessor,
    config: WebFetchRuntimeConfig,
    cache: DocumentCache,
    fetch_coordinator: FetchCoordinator,
}

impl WebFetchService {
    pub fn new(
        network: Arc<dyn NetworkClient>,
        model_service: ModelService,
        model_provider: impl Into<String>,
        config: WebFetchRuntimeConfig,
    ) -> Result<Self> {
        validate_config(&config)?;
        let model_provider = model_provider.into();
        if model_provider.trim().is_empty() || model_provider.trim() != model_provider {
            return Err(Error::ConfigInvalid {
                message: "web_fetch model provider must be non-empty".to_owned(),
            });
        }
        Ok(Self {
            network,
            prompt_processor: PromptProcessor::Enabled {
                model_service: Arc::new(model_service),
                model_provider,
            },
            cache: DocumentCache::new(
                Duration::from_millis(config.cache_ttl_ms),
                config.max_cache_entries,
            ),
            fetch_coordinator: FetchCoordinator::default(),
            config,
        })
    }

    pub fn disabled(
        network: Arc<dyn NetworkClient>,
        config: WebFetchRuntimeConfig,
    ) -> Result<Self> {
        validate_config(&config)?;
        Ok(Self {
            network,
            prompt_processor: PromptProcessor::Disabled,
            cache: DocumentCache::new(
                Duration::from_millis(config.cache_ttl_ms),
                config.max_cache_entries,
            ),
            fetch_coordinator: FetchCoordinator::default(),
            config,
        })
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        matches!(&self.prompt_processor, PromptProcessor::Enabled { .. })
    }

    pub async fn fetch_document(
        &self,
        url: &str,
        deadline: Option<Instant>,
    ) -> Result<WebFetchDocumentOutcome> {
        let requested = match normalize_url(url) {
            Ok(url) => url,
            Err(error) => return Ok(WebFetchDocumentOutcome::SoftError(error)),
        };
        let cache_key = requested.as_str().to_owned();
        if let Some(document) = self.cache.get(&cache_key) {
            return Ok(WebFetchDocumentOutcome::Document(document));
        }
        let _fetch_permit = if self.cache.is_enabled() {
            let permit = self.fetch_coordinator.acquire(&cache_key).await;
            if let Some(document) = self.cache.get(&cache_key) {
                return Ok(WebFetchDocumentOutcome::Document(document));
            }
            Some(permit)
        } else {
            None
        };

        let mut current = requested.clone();
        let mut seen = HashSet::new();
        let mut redirects = 0usize;
        loop {
            if !seen.insert(current.as_str().to_owned()) {
                return Ok(soft_error(
                    "redirect_loop",
                    false,
                    "web_fetch redirect loop detected",
                ));
            }
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                return Err(Error::Timeout {
                    message: "web_fetch deadline exceeded".to_owned(),
                });
            }
            let outcome = match self
                .network
                .send_document(NetworkRequest {
                    method: "GET".to_owned(),
                    url: current.as_str().to_owned(),
                    headers: vec![
                        Header {
                            name: "accept".to_owned(),
                            value: "text/html,text/plain,text/markdown,application/xhtml+xml"
                                .to_owned(),
                        },
                        Header {
                            name: "accept-encoding".to_owned(),
                            value: "identity".to_owned(),
                        },
                    ],
                    body: None,
                    inactivity_timeout_ms: self.config.inactivity_timeout_ms,
                })
                .await
            {
                Ok(outcome) => outcome,
                Err(error) if error.code() == ErrorCode::NetworkFailed => {
                    return Ok(soft_error(
                        "network_failed",
                        error.retryable(),
                        "the document request failed",
                    ));
                }
                Err(error) => return Err(error),
            };
            let response = match outcome {
                DocumentNetworkOutcome::Rejected(rejection) => {
                    return Ok(WebFetchDocumentOutcome::SoftError(rejection_error(
                        rejection,
                    )));
                }
                DocumentNetworkOutcome::Response(response) => response,
            };

            if (300..400).contains(&response.status) {
                let Some(location) = header_value(&response.headers, "location") else {
                    return Ok(soft_error(
                        "redirect_failed",
                        false,
                        "web_fetch redirect did not include a valid location",
                    ));
                };
                let target = match current
                    .join(location)
                    .ok()
                    .and_then(|url| normalize_url(url.as_str()).ok())
                {
                    Some(target) => target,
                    None => {
                        return Ok(soft_error(
                            "redirect_failed",
                            false,
                            "web_fetch redirect target was invalid",
                        ));
                    }
                };
                let same_origin = current.host_str().zip(target.host_str()).is_some_and(
                    |(current_host, target_host)| {
                        current_host.eq_ignore_ascii_case(target_host)
                            && current.port_or_known_default() == target.port_or_known_default()
                    },
                );
                if !same_origin {
                    return Ok(WebFetchDocumentOutcome::Redirect {
                        redirect_url: target.to_string(),
                    });
                }
                if redirects >= self.config.max_redirects {
                    return Ok(soft_error(
                        "redirect_limit_exceeded",
                        false,
                        "web_fetch redirect limit was exceeded",
                    ));
                }
                redirects = redirects.saturating_add(1);
                current = target;
                continue;
            }

            return self.convert_response(
                requested.as_str(),
                current.as_str(),
                response,
                cache_key,
            );
        }
    }

    pub async fn answer_document(
        &self,
        document: &WebFetchDocument,
        prompt: &str,
        deadline: Option<Instant>,
    ) -> Result<WebFetchAnswerOutcome> {
        let PromptProcessor::Enabled {
            model_service,
            model_provider,
        } = &self.prompt_processor
        else {
            return Err(Error::ProviderUnavailable {
                provider: "web_fetch".to_owned(),
                message: "web_fetch prompt processor is disabled".to_owned(),
                retryable: false,
            });
        };
        let operation = model::answer_document(model_service, model_provider, document, prompt);
        let Some(deadline) = deadline else {
            return operation.await;
        };
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            return Err(Error::Timeout {
                message: "web_fetch deadline exceeded".to_owned(),
            });
        };
        tokio::time::timeout(remaining, operation)
            .await
            .map_err(|_| Error::Timeout {
                message: "web_fetch prompt processing timed out".to_owned(),
            })?
    }

    fn convert_response(
        &self,
        requested_url: &str,
        final_url: &str,
        response: DocumentNetworkResponse,
        cache_key: String,
    ) -> Result<WebFetchDocumentOutcome> {
        if response.status == 401 || response.status == 403 {
            return Ok(soft_error(
                "auth_required",
                false,
                "the document requires authentication",
            ));
        }
        if !(200..300).contains(&response.status) {
            return Ok(soft_error(
                "http_status",
                matches!(response.status, 408 | 429 | 500..=599),
                "the document request returned an unsuccessful status",
            ));
        }
        let content_type = header_value(&response.headers, "content-type")
            .unwrap_or("text/html; charset=utf-8")
            .to_owned();
        let converted = match convert_document(&response.body, &content_type, final_url) {
            Ok(converted) => converted,
            Err(error) => return Ok(WebFetchDocumentOutcome::SoftError(error)),
        };
        let document = Arc::new(WebFetchDocument {
            requested_url: requested_url.to_owned(),
            final_url: final_url.to_owned(),
            title: converted.title,
            content_type,
            markdown: Arc::from(converted.markdown),
            retrieved_at: OffsetDateTime::now_utc(),
        });
        self.cache.insert(cache_key, document.clone());
        Ok(WebFetchDocumentOutcome::Document(document))
    }
}

fn validate_config(config: &WebFetchRuntimeConfig) -> Result<()> {
    if config.max_cache_entries == 0 || config.inactivity_timeout_ms == Some(0) {
        return Err(Error::ConfigInvalid {
            message: "web_fetch runtime limits must be greater than zero".to_owned(),
        });
    }
    Ok(())
}

fn normalize_url(input: &str) -> std::result::Result<Url, WebFetchSoftError> {
    let mut url = Url::parse(input.trim())
        .map_err(|_| WebFetchSoftError::new("invalid_url", false, "web_fetch URL is invalid"))?;
    if url.scheme() == "http" {
        url.set_scheme("https").map_err(|_| {
            WebFetchSoftError::new("invalid_url", false, "web_fetch URL is invalid")
        })?;
    }
    if url.scheme() != "https" || url.host_str().is_none() {
        return Err(WebFetchSoftError::new(
            "unsafe_url",
            false,
            "web_fetch only accepts public HTTP or HTTPS URLs",
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(WebFetchSoftError::new(
            "credentials_present",
            false,
            "web_fetch URL credentials are not allowed",
        ));
    }
    if url
        .query_pairs()
        .any(|(name, _)| sensitive_query_name(&name))
    {
        return Err(WebFetchSoftError::new(
            "sensitive_query",
            false,
            "web_fetch URLs must not include authentication secrets",
        ));
    }
    url.set_fragment(None);
    if url.port() == Some(443) {
        let _ = url.set_port(None);
    }
    Ok(url)
}

fn sensitive_query_name(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase().replace(['-', '.'], "_");
    [
        "access_token",
        "api_key",
        "apikey",
        "auth",
        "authorization",
        "credential",
        "jwt",
        "key",
        "password",
        "secret",
        "session",
        "signature",
        "sig",
        "token",
    ]
    .iter()
    .any(|candidate| normalized == *candidate || normalized.ends_with(&format!("_{candidate}")))
}

fn header_value<'a>(headers: &'a [Header], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(name))
        .map(|header| header.value.as_str())
}

fn rejection_error(rejection: DocumentNetworkRejection) -> WebFetchSoftError {
    match rejection {
        DocumentNetworkRejection::UnsupportedContentEncoding => WebFetchSoftError::new(
            "unsupported_content_encoding",
            false,
            "the document used an unsupported content encoding",
        ),
        DocumentNetworkRejection::DnsResolutionFailed => WebFetchSoftError::new(
            "dns_resolution_failed",
            true,
            "the document host could not be resolved",
        ),
        _ => WebFetchSoftError::new(
            "unsafe_target",
            false,
            "the document target was rejected by network safety policy",
        ),
    }
}

fn soft_error(
    code: &'static str,
    retryable: bool,
    message: &'static str,
) -> WebFetchDocumentOutcome {
    WebFetchDocumentOutcome::SoftError(WebFetchSoftError::new(code, retryable, message))
}
