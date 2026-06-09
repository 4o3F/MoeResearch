use std::collections::BTreeMap;

use reqwest::header::HeaderValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use lapis_error::{Error, Result};

use crate::limit::{CountLimit, DurationLimitMs, TokenLimit};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LapisConfig {
    pub logging: LoggingConfig,
    pub network: NetworkConfig,
    pub search: SearchProviderRegistry,
    pub model: ModelProviderRegistry,
    pub budget: BudgetConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnabledProviderEnv<'a> {
    pub kind: &'a str,
    pub name: &'a str,
    pub api_key_env: Option<&'a str>,
}

impl LapisConfig {
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

        model_providers.chain(search_providers).collect()
    }

    fn validate_structure(&self) -> Result<()> {
        self.network.validate()?;
        self.budget.validate()?;
        self.search.validate_structure()?;
        self.model.validate_structure()
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

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    pub timeout_ms: u64,
    pub max_retries: usize,
    pub retry_backoff_ms: u64,
    pub user_agent: String,
}

impl NetworkConfig {
    fn validate(&self) -> Result<()> {
        if self.timeout_ms == 0 {
            return Err(Error::ConfigInvalid {
                message: "network.timeout_ms must not be zero".to_owned(),
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
    pub timeout_ms: Option<u64>,
    pub model: Option<String>,
}

impl ModelProviderEndpoint {
    fn validate_structure(&self, name: &str) -> Result<()> {
        if name != "openai" {
            return Err(Error::ConfigInvalid {
                message: format!("unknown model.providers.{name} provider"),
            });
        }

        validate_timeout("model", name, self.timeout_ms)?;
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
    pub timeout_ms: Option<u64>,
    pub model: Option<String>,
    pub reasoning_effort: Option<GrokReasoningEffort>,
    #[serde(default)]
    pub max_output_tokens: Option<u32>,
}

impl SearchProviderEndpoint {
    fn validate_structure(&self, name: &str) -> Result<()> {
        validate_timeout("search", name, self.timeout_ms)?;
        validate_api_key_env_name("search", name, self.api_key_env.as_ref())?;

        match name {
            "exa" => validate_exa_knobs(
                name,
                self.reasoning_effort.is_some(),
                self.max_output_tokens.is_some(),
            ),
            "grok" => {
                validate_model("search", name, self.enabled, self.model.as_ref())?;
                validate_grok_knobs(name, self.max_output_tokens)
            }
            _ => Err(Error::ConfigInvalid {
                message: format!("unknown search.providers.{name} provider"),
            }),
        }
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
pub struct BudgetConfig {
    pub research: ResearchBudgetConfig,
    pub per_agent: AgentBudgetConfig,
}

impl BudgetConfig {
    pub fn validate(&self) -> Result<()> {
        self.research
            .max_agents
            .require_non_zero("budget.research.max_agents")?;
        self.research
            .max_concurrent_agents
            .require_non_zero("budget.research.max_concurrent_agents")?;
        self.research
            .total_timeout_ms
            .require_non_zero("budget.research.total_timeout_ms")?;
        self.per_agent
            .max_turns
            .require_non_zero("budget.per_agent.max_turns")?;
        self.per_agent
            .timeout_ms
            .require_non_zero("budget.per_agent.timeout_ms")?;

        if self
            .research
            .max_concurrent_agents
            .exceeds(self.research.max_agents)
        {
            return Err(Error::ConfigInvalid {
                message: "budget.research.max_concurrent_agents must not exceed \
                          budget.research.max_agents"
                    .to_owned(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchBudgetConfig {
    pub max_agents: CountLimit,
    pub max_concurrent_agents: CountLimit,
    pub max_total_model_calls: CountLimit,
    pub max_total_search_calls: CountLimit,
    pub total_timeout_ms: DurationLimitMs,
    pub max_tokens: TokenLimit,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentBudgetConfig {
    pub max_turns: CountLimit,
    pub max_tool_calls: CountLimit,
    pub max_search_calls: CountLimit,
    pub timeout_ms: DurationLimitMs,
}

fn validate_timeout(kind: &str, name: &str, timeout_ms: Option<u64>) -> Result<()> {
    if timeout_ms == Some(0) {
        return Err(Error::ConfigInvalid {
            message: format!("{kind}.providers.{name}.timeout_ms must not be zero"),
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
    })?;

    if std::env::var_os(env_name).is_none() {
        return Err(Error::ProviderUnavailable {
            provider: format!("{kind}:{name}"),
            message: format!("environment variable {env_name} is not set"),
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

fn validate_grok_knobs(name: &str, max_output_tokens: Option<u32>) -> Result<()> {
    if max_output_tokens == Some(0) {
        return Err(Error::ConfigInvalid {
            message: format!("search.providers.{name}.max_output_tokens must be greater than zero"),
        });
    }
    Ok(())
}

fn validate_exa_knobs(
    name: &str,
    has_reasoning_effort: bool,
    has_max_output_tokens: bool,
) -> Result<()> {
    if has_reasoning_effort {
        return Err(Error::ConfigInvalid {
            message: format!("search.providers.{name}.reasoning_effort is only supported by grok"),
        });
    }
    if has_max_output_tokens {
        return Err(Error::ConfigInvalid {
            message: format!("search.providers.{name}.max_output_tokens is only supported by grok"),
        });
    }
    Ok(())
}
