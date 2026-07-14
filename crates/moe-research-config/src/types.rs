use std::collections::BTreeMap;
use std::fmt;

use reqwest::header::HeaderValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use moe_research_error::{Error, Result};

use crate::limit::{CountLimit, DurationLimitMs, TokenLimit};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MoeResearchConfig {
    pub logging: LoggingConfig,
    pub network: NetworkConfig,
    pub search: SearchProviderRegistry,
    pub model: ModelProviderRegistry,
    #[serde(default)]
    pub web_fetch: WebFetchConfig,
    pub limits: LimitsConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnabledProviderEnv<'a> {
    pub kind: &'a str,
    pub name: &'a str,
    pub api_key_env: Option<&'a str>,
}

impl MoeResearchConfig {
    pub fn validate(&self) -> Result<()> {
        self.validate_structure()?;
        self.validate_runtime_environment()
    }

    pub fn enabled_provider_envs(&self) -> Vec<EnabledProviderEnv<'_>> {
        let model_providers = self.model.providers.iter().filter_map(|(name, provider)| {
            provider.enabled.then_some(EnabledProviderEnv {
                kind: "model",
                name,
                api_key_env: provider.api_key_env.as_deref(),
            })
        });
        let search_providers = self.search.providers.iter().filter_map(|(name, provider)| {
            provider.enabled.then_some(EnabledProviderEnv {
                kind: "search",
                name,
                api_key_env: provider.api_key_env.as_deref(),
            })
        });

        let web_fetch = self
            .web_fetch
            .enabled
            .then_some(self.web_fetch.model.as_ref())
            .flatten()
            .map(|model| EnabledProviderEnv {
                kind: "web_fetch",
                name: model.provider.as_str(),
                api_key_env: Some(model.api_key_env.as_str()),
            });

        model_providers
            .chain(search_providers)
            .chain(web_fetch)
            .collect()
    }

    fn validate_structure(&self) -> Result<()> {
        self.network.validate()?;
        self.limits.validate()?;
        self.search.validate_structure()?;
        self.model.validate_structure()?;
        self.web_fetch.validate_structure()
    }

    fn validate_runtime_environment(&self) -> Result<()> {
        for provider in self.enabled_provider_envs() {
            validate_env_key(provider.kind, provider.name, provider.api_key_env)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    pub format: String,
}

#[derive(Clone, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct NetworkProxyUrl(String);

impl NetworkProxyUrl {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self) -> Result<()> {
        let proxy_url = self.as_str();
        if proxy_url.trim().is_empty() {
            return Err(Error::ConfigInvalid {
                message: "network.proxy_url must not be empty".to_owned(),
            });
        }
        if proxy_url != proxy_url.trim() {
            return Err(Error::ConfigInvalid {
                message: "network.proxy_url must not include leading or trailing whitespace"
                    .to_owned(),
            });
        }

        let url = reqwest::Url::parse(proxy_url).map_err(|_| Error::ConfigInvalid {
            message: "network.proxy_url must be an absolute URL with a host".to_owned(),
        })?;
        if url.host().is_none() {
            return Err(Error::ConfigInvalid {
                message: "network.proxy_url must be an absolute URL with a host".to_owned(),
            });
        }
        if !matches!(url.scheme(), "http" | "https" | "socks5" | "socks5h") {
            return Err(Error::ConfigInvalid {
                message: "network.proxy_url must use http, https, socks5, or socks5h".to_owned(),
            });
        }

        reqwest::Proxy::all(proxy_url).map_err(|_| Error::ConfigInvalid {
            message: "network.proxy_url is not accepted by the HTTP client".to_owned(),
        })?;

        Ok(())
    }
}

impl fmt::Debug for NetworkProxyUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("NetworkProxyUrl")
            .field(&"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    pub inactivity_timeout_ms: u64,
    pub max_retries: usize,
    pub retry_backoff_ms: u64,
    pub user_agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<NetworkProxyUrl>,
}

impl NetworkConfig {
    fn validate(&self) -> Result<()> {
        if self.inactivity_timeout_ms == 0 {
            return Err(Error::ConfigInvalid {
                message: "network.inactivity_timeout_ms must not be zero".to_owned(),
            });
        }

        let user_agent = self.user_agent.trim();
        if user_agent.is_empty() {
            return Err(Error::ConfigInvalid {
                message: "network.user_agent must not be empty".to_owned(),
            });
        }

        HeaderValue::from_str(user_agent).map_err(|source| Error::ConfigInvalid {
            message: format!("network.user_agent must be a valid HTTP header value: {source}"),
        })?;

        if let Some(proxy_url) = &self.proxy_url {
            proxy_url.validate()?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct WebFetchConfig {
    pub enabled: bool,
    pub cache_ttl_ms: u64,
    pub max_cache_entries: usize,
    pub max_redirects: usize,
    pub model: Option<WebFetchModelEndpoint>,
}

impl Default for WebFetchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cache_ttl_ms: 900_000,
            max_cache_entries: 128,
            max_redirects: 5,
            model: None,
        }
    }
}

impl WebFetchConfig {
    fn validate_structure(&self) -> Result<()> {
        if self.max_cache_entries == 0 {
            return Err(Error::ConfigInvalid {
                message: "web_fetch max_cache_entries must be greater than zero".to_owned(),
            });
        }
        match (&self.model, self.enabled) {
            (Some(model), _) => model.validate_structure(),
            (None, true) => Err(Error::ConfigInvalid {
                message: "web_fetch.model must be configured when web_fetch is enabled".to_owned(),
            }),
            (None, false) => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WebFetchModelEndpoint {
    pub provider: String,
    pub base_url: String,
    pub api_key_env: String,
    pub inactivity_timeout_ms: Option<u64>,
    pub model: String,
}

impl WebFetchModelEndpoint {
    fn validate_structure(&self) -> Result<()> {
        if self.provider != "openai" {
            return Err(Error::ConfigInvalid {
                message: "web_fetch.model.provider must be `openai`".to_owned(),
            });
        }
        let url = reqwest::Url::parse(&self.base_url).map_err(|_| Error::ConfigInvalid {
            message: "web_fetch.model.base_url must be an absolute HTTPS URL".to_owned(),
        })?;
        if url.scheme() != "https"
            || url.host().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(Error::ConfigInvalid {
                message:
                    "web_fetch.model.base_url must be an absolute HTTPS URL without credentials"
                        .to_owned(),
            });
        }
        if !is_valid_env_var_name(&self.api_key_env) {
            return Err(Error::ConfigInvalid {
                message: "web_fetch.model.api_key_env must be a valid environment variable name"
                    .to_owned(),
            });
        }
        if self.inactivity_timeout_ms == Some(0) {
            return Err(Error::ConfigInvalid {
                message: "web_fetch.model.inactivity_timeout_ms must not be zero".to_owned(),
            });
        }
        if self.model.trim().is_empty() || self.model.trim() != self.model {
            return Err(Error::ConfigInvalid {
                message: "web_fetch.model.model must be a non-empty model name".to_owned(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelProviderRegistry {
    pub providers: BTreeMap<String, ModelProviderEndpoint>,
}

impl ModelProviderRegistry {
    pub fn enabled_count(&self) -> usize {
        self.providers
            .values()
            .filter(|provider| provider.enabled)
            .count()
    }

    fn validate_structure(&self) -> Result<()> {
        for (name, provider) in &self.providers {
            provider.validate_structure(name)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SearchProviderRegistry {
    pub providers: BTreeMap<String, SearchProviderEndpoint>,
}

impl SearchProviderRegistry {
    pub fn enabled_count(&self) -> usize {
        self.providers
            .values()
            .filter(|provider| provider.enabled)
            .count()
    }

    fn validate_structure(&self) -> Result<()> {
        for (name, provider) in &self.providers {
            provider.validate_structure(name)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelProviderEndpoint {
    pub enabled: bool,
    pub base_url: String,
    pub api_key_env: Option<String>,
    pub inactivity_timeout_ms: Option<u64>,
    pub model: Option<String>,
}

impl ModelProviderEndpoint {
    fn validate_structure(&self, name: &str) -> Result<()> {
        if name != "openai" {
            return Err(Error::ConfigInvalid {
                message: format!("unknown model.providers.{name} provider"),
            });
        }

        validate_inactivity_timeout("model", name, self.inactivity_timeout_ms)?;
        validate_api_key_env_name("model", name, self.api_key_env.as_ref())?;
        validate_model("model", name, self.enabled, self.model.as_ref())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SearchProviderEndpoint {
    pub enabled: bool,
    pub base_url: String,
    pub api_key_env: Option<String>,
    pub inactivity_timeout_ms: Option<u64>,
    pub model: Option<String>,
    pub reasoning_effort: Option<GrokReasoningEffort>,
    #[serde(default)]
    pub max_output_tokens: Option<u32>,
}

impl SearchProviderEndpoint {
    fn validate_structure(&self, name: &str) -> Result<()> {
        validate_inactivity_timeout("search", name, self.inactivity_timeout_ms)?;
        validate_api_key_env_name("search", name, self.api_key_env.as_ref())?;
        search_provider_spec(name)?.validate(name, self)
    }
}

#[derive(Clone, Copy)]
struct SearchProviderSpec {
    model: SearchModelPolicy,
    reasoning_effort: SearchFieldPolicy,
    max_output_tokens: SearchTokenPolicy,
}

impl SearchProviderSpec {
    fn validate(self, name: &str, endpoint: &SearchProviderEndpoint) -> Result<()> {
        self.model
            .validate(name, endpoint.enabled, endpoint.model.as_ref())?;
        self.reasoning_effort.validate(
            name,
            "reasoning_effort",
            endpoint.reasoning_effort.is_some(),
        )?;
        self.max_output_tokens
            .validate(name, endpoint.max_output_tokens)
    }
}

#[derive(Clone, Copy)]
enum SearchModelPolicy {
    Unsupported,
    RequiredWhenEnabled,
}

impl SearchModelPolicy {
    fn validate(self, name: &str, enabled: bool, model: Option<&String>) -> Result<()> {
        match self {
            Self::Unsupported => validate_unsupported_search_field(name, "model", model.is_some()),
            Self::RequiredWhenEnabled => validate_model("search", name, enabled, model),
        }
    }
}

#[derive(Clone, Copy)]
enum SearchFieldPolicy {
    GrokOnly,
    Supported,
}

impl SearchFieldPolicy {
    fn validate(self, name: &str, field: &str, is_set: bool) -> Result<()> {
        match self {
            Self::GrokOnly => validate_grok_only_search_field(name, field, is_set),
            Self::Supported => Ok(()),
        }
    }
}

#[derive(Clone, Copy)]
enum SearchTokenPolicy {
    GrokOnly,
    PositiveOptional,
}

impl SearchTokenPolicy {
    fn validate(self, name: &str, max_output_tokens: Option<u32>) -> Result<()> {
        match self {
            Self::GrokOnly => validate_grok_only_search_field(
                name,
                "max_output_tokens",
                max_output_tokens.is_some(),
            ),
            Self::PositiveOptional => validate_positive_search_tokens(name, max_output_tokens),
        }
    }
}

fn search_provider_spec(name: &str) -> Result<SearchProviderSpec> {
    match name {
        "exa" | "tavily" => Ok(SearchProviderSpec {
            model: SearchModelPolicy::Unsupported,
            reasoning_effort: SearchFieldPolicy::GrokOnly,
            max_output_tokens: SearchTokenPolicy::GrokOnly,
        }),
        "grok" => Ok(SearchProviderSpec {
            model: SearchModelPolicy::RequiredWhenEnabled,
            reasoning_effort: SearchFieldPolicy::Supported,
            max_output_tokens: SearchTokenPolicy::PositiveOptional,
        }),
        _ => Err(Error::ConfigInvalid {
            message: format!("unknown search.providers.{name} provider"),
        }),
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GrokReasoningEffort {
    None,
    Low,
    Medium,
    High,
}

impl GrokReasoningEffort {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LimitsConfig {
    pub research: ResearchLimitsConfig,
    pub per_agent: AgentLimitsConfig,
}

impl LimitsConfig {
    pub fn validate(&self) -> Result<()> {
        self.research
            .max_agents
            .require_non_zero("limits.research.max_agents")?;
        self.research
            .max_concurrent_agents
            .require_non_zero("limits.research.max_concurrent_agents")?;
        self.research
            .total_timeout_ms
            .require_non_zero("limits.research.total_timeout_ms")?;
        self.per_agent
            .max_turns
            .require_non_zero("limits.per_agent.max_turns")?;
        self.per_agent
            .timeout_ms
            .require_non_zero("limits.per_agent.timeout_ms")?;

        if self
            .research
            .max_concurrent_agents
            .exceeds(self.research.max_agents)
        {
            return Err(Error::ConfigInvalid {
                message: "limits.research.max_concurrent_agents must not exceed \
                          limits.research.max_agents"
                    .to_owned(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchLimitsConfig {
    pub max_agents: CountLimit,
    pub max_concurrent_agents: CountLimit,
    pub max_total_model_calls: CountLimit,
    pub max_total_search_calls: CountLimit,
    pub total_timeout_ms: DurationLimitMs,
    pub max_tokens: TokenLimit,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentLimitsConfig {
    pub max_turns: CountLimit,
    pub max_tool_calls: CountLimit,
    pub max_search_calls: CountLimit,
    pub timeout_ms: DurationLimitMs,
}

fn validate_inactivity_timeout(
    kind: &str,
    name: &str,
    inactivity_timeout_ms: Option<u64>,
) -> Result<()> {
    if inactivity_timeout_ms == Some(0) {
        return Err(Error::ConfigInvalid {
            message: format!("{kind}.providers.{name}.inactivity_timeout_ms must not be zero"),
        });
    }
    Ok(())
}

fn validate_api_key_env_name(kind: &str, name: &str, api_key_env: Option<&String>) -> Result<()> {
    let Some(env_name) = api_key_env else {
        return Ok(());
    };

    if !is_valid_env_var_name(env_name) {
        return Err(Error::ConfigInvalid {
            message: format!(
                "{kind}.providers.{name}.api_key_env must be a valid environment variable name"
            ),
        });
    }

    Ok(())
}

fn is_valid_env_var_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn validate_env_key(kind: &str, name: &str, api_key_env: Option<&str>) -> Result<()> {
    let env_name = api_key_env.ok_or_else(|| Error::ProviderUnavailable {
        provider: format!("{kind}:{name}"),
        message: "enabled provider must set api_key_env".to_owned(),
        retryable: false,
    })?;

    if std::env::var_os(env_name).is_none() {
        return Err(Error::ProviderUnavailable {
            provider: format!("{kind}:{name}"),
            message: format!("environment variable {env_name} is not set"),
            retryable: false,
        });
    }

    Ok(())
}

fn validate_model(kind: &str, name: &str, enabled: bool, model: Option<&String>) -> Result<()> {
    if !enabled {
        return Ok(());
    }

    let model = model
        .map(|model| model.trim())
        .ok_or_else(|| Error::ConfigInvalid {
            message: format!("{kind}.providers.{name}.model must be set"),
        })?;

    if model.is_empty() {
        return Err(Error::ConfigInvalid {
            message: format!("{kind}.providers.{name}.model must not be empty"),
        });
    }

    Ok(())
}

fn validate_positive_search_tokens(name: &str, max_output_tokens: Option<u32>) -> Result<()> {
    if max_output_tokens == Some(0) {
        return Err(Error::ConfigInvalid {
            message: format!("search.providers.{name}.max_output_tokens must be greater than zero"),
        });
    }
    Ok(())
}

fn validate_unsupported_search_field(name: &str, field: &str, is_set: bool) -> Result<()> {
    if is_set {
        return Err(Error::ConfigInvalid {
            message: format!("search.providers.{name}.{field} is not supported"),
        });
    }
    Ok(())
}

fn validate_grok_only_search_field(name: &str, field: &str, is_set: bool) -> Result<()> {
    if is_set {
        return Err(Error::ConfigInvalid {
            message: format!("search.providers.{name}.{field} is only supported by grok"),
        });
    }
    Ok(())
}
