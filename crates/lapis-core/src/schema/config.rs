use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::schema::limit::{CountLimit, DurationLimitMs, Limit};

#[derive(Clone, Debug, Default, JsonSchema, PartialEq, Eq, Serialize)]
pub struct LapisConfig {
    pub logging: LoggingConfig,
    pub network: NetworkConfig,
    pub search: SearchConfig,
    pub model: ModelConfig,
    pub budget: BudgetConfig,
}

impl<'de> Deserialize<'de> for LapisConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Default, Deserialize)]
        #[serde(default, deny_unknown_fields)]
        struct RawLapisConfig {
            logging: LoggingConfig,
            network: NetworkConfig,
            search: SearchConfig,
            model: ModelConfig,
            budget: BudgetConfig,
        }

        let raw = RawLapisConfig::deserialize(deserializer)?;
        let config = Self {
            logging: raw.logging,
            network: raw.network,
            search: raw.search,
            model: raw.model,
            budget: raw.budget,
        };
        config.validate().map_err(de::Error::custom)?;
        Ok(config)
    }
}

impl LapisConfig {
    pub fn validate(&self) -> Result<()> {
        self.network.validate()?;
        self.budget.validate()?;
        self.search.validate_with(&ProviderValidationContext {
            kind: "search",
            network_limits: &self.network.limits,
            validate_env: true,
        })?;
        self.model.validate_with(&ProviderValidationContext {
            kind: "model",
            network_limits: &self.network.limits,
            validate_env: true,
        })
    }

    pub fn validate_env_keys(&self) -> Result<()> {
        self.search.validate_env("search")?;
        self.model.validate_env("model")
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LoggingConfig {
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: "json".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConfig {
    pub timeout_ms: u64,
    pub max_retries: usize,
    pub retry_backoff_ms: u64,
    pub user_agent: String,
    pub limits: NetworkLimits,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            max_retries: 2,
            retry_backoff_ms: 200,
            user_agent: "lapis/0.1.0".to_owned(),
            limits: NetworkLimits::default(),
        }
    }
}

impl NetworkConfig {
    fn validate(&self) -> Result<()> {
        self.limits.validate()?;
        self.limits
            .require_timeout_within("network.timeout_ms", self.timeout_ms)?;

        if self.max_retries > self.limits.max_retries {
            return Err(Error::ConfigInvalid {
                message: "network.max_retries exceeds network.limits.max_retries".to_owned(),
            });
        }

        if self.retry_backoff_ms > self.limits.max_retry_backoff_ms {
            return Err(Error::ConfigInvalid {
                message: "network.retry_backoff_ms exceeds network.limits.max_retry_backoff_ms"
                    .to_owned(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkLimits {
    pub max_timeout_ms: u64,
    pub max_retries: usize,
    pub max_retry_backoff_ms: u64,
}

impl Default for NetworkLimits {
    fn default() -> Self {
        Self {
            max_timeout_ms: 120_000,
            max_retries: 5,
            max_retry_backoff_ms: 5_000,
        }
    }
}

impl NetworkLimits {
    fn validate(&self) -> Result<()> {
        Self::require_positive("network.limits.max_timeout_ms", self.max_timeout_ms)?;
        Self::require_positive(
            "network.limits.max_retry_backoff_ms",
            self.max_retry_backoff_ms,
        )
    }

    fn require_timeout_within(&self, field: &str, timeout_ms: u64) -> Result<()> {
        Self::require_positive(field, timeout_ms)?;
        if timeout_ms > self.max_timeout_ms {
            return Err(Error::ConfigInvalid {
                message: format!("{field} exceeds network.limits.max_timeout_ms"),
            });
        }
        Ok(())
    }

    pub(crate) fn validate_runtime_settings(
        &self,
        timeout_field: &str,
        timeout_ms: u64,
        max_retries: usize,
        retry_backoff_ms: u64,
    ) -> Result<()> {
        if timeout_ms == 0 {
            return Err(Error::InvalidInput {
                message: format!("{timeout_field} must be greater than zero"),
            });
        }

        if timeout_ms > self.max_timeout_ms {
            return Err(Error::InvalidInput {
                message: format!("{timeout_field} exceeds configured timeout limit"),
            });
        }

        if max_retries > self.max_retries {
            return Err(Error::InvalidInput {
                message: "network.max_retries exceeds configured limit".to_owned(),
            });
        }

        if retry_backoff_ms > self.max_retry_backoff_ms {
            return Err(Error::InvalidInput {
                message: "network.retry_backoff_ms exceeds configured limit".to_owned(),
            });
        }

        Ok(())
    }

    fn require_positive(field: &str, value: u64) -> Result<()> {
        if value == 0 {
            return Err(Error::ConfigInvalid {
                message: format!("{field} must be greater than zero"),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SearchConfig {
    pub providers: BTreeMap<String, ProviderEndpoint>,
    pub preferred_providers: Vec<String>,
}

impl SearchConfig {
    pub fn enabled_count(&self) -> usize {
        self.providers
            .values()
            .filter(|provider| provider.enabled)
            .count()
    }

    fn validate_with(&self, context: &ProviderValidationContext<'_>) -> Result<()> {
        for (name, provider) in &self.providers {
            provider.validate_with(name, context)?;
            match name.as_str() {
                "grok" => provider.validate_model(context.kind, name)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn validate_env(&self, kind: &str) -> Result<()> {
        for (name, provider) in &self.providers {
            provider.validate_env(kind, name)?;
        }
        Ok(())
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        let mut providers = BTreeMap::new();
        providers.insert(
            "exa".to_owned(),
            ProviderEndpoint {
                enabled: false,
                base_url: "https://api.exa.ai".to_owned(),
                api_key_env: Some("EXA_API_KEY".to_owned()),
                timeout_ms: None,
                model: None,
            },
        );
        providers.insert(
            "grok".to_owned(),
            ProviderEndpoint {
                enabled: false,
                base_url: "https://api.x.ai".to_owned(),
                api_key_env: Some("XAI_API_KEY".to_owned()),
                timeout_ms: None,
                model: None,
            },
        );

        Self {
            providers,
            preferred_providers: vec!["exa".to_owned(), "grok".to_owned()],
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ModelConfig {
    pub providers: BTreeMap<String, ProviderEndpoint>,
    pub default_provider: String,
}

impl ModelConfig {
    pub fn enabled_count(&self) -> usize {
        self.providers
            .values()
            .filter(|provider| provider.enabled)
            .count()
    }

    fn validate_with(&self, context: &ProviderValidationContext<'_>) -> Result<()> {
        for (name, provider) in &self.providers {
            provider.validate_with(name, context)?;
            match name.as_str() {
                "openai" => provider.validate_model(context.kind, name)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn validate_env(&self, kind: &str) -> Result<()> {
        for (name, provider) in &self.providers {
            provider.validate_env(kind, name)?;
        }
        Ok(())
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        let mut providers = BTreeMap::new();
        providers.insert(
            "openai".to_owned(),
            ProviderEndpoint {
                enabled: false,
                base_url: "https://api.openai.com/v1".to_owned(),
                api_key_env: Some("OPENAI_API_KEY".to_owned()),
                timeout_ms: None,
                model: None,
            },
        );

        Self {
            providers,
            default_provider: "openai".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProviderEndpoint {
    pub enabled: bool,
    pub base_url: String,
    pub api_key_env: Option<String>,
    pub timeout_ms: Option<u64>,
    pub model: Option<String>,
}

struct ProviderValidationContext<'a> {
    kind: &'a str,
    network_limits: &'a NetworkLimits,
    validate_env: bool,
}

impl ProviderEndpoint {
    fn validate_with(&self, name: &str, context: &ProviderValidationContext<'_>) -> Result<()> {
        self.validate_timeout(name, context)?;
        if context.validate_env {
            self.validate_env(context.kind, name)?;
        }
        Ok(())
    }

    fn validate_timeout(&self, name: &str, context: &ProviderValidationContext<'_>) -> Result<()> {
        let Some(timeout_ms) = self.timeout_ms else {
            return Ok(());
        };

        context.network_limits.require_timeout_within(
            &format!("{}.providers.{name}.timeout_ms", context.kind),
            timeout_ms,
        )
    }

    fn validate_env(&self, kind: &str, name: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let Some(env_name) = self.api_key_env.as_ref() else {
            return Err(Error::ProviderUnavailable {
                provider: format!("{kind}:{name}"),
                message: "enabled provider must set api_key_env".to_owned(),
            });
        };

        if std::env::var_os(env_name).is_none() {
            return Err(Error::ProviderUnavailable {
                provider: format!("{kind}:{name}"),
                message: format!("environment variable {env_name} is not set"),
            });
        }

        Ok(())
    }

    fn validate_model(&self, kind: &str, name: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let Some(model) = self.model.as_ref().map(|model| model.trim()) else {
            return Err(Error::ConfigInvalid {
                message: format!("{kind}.providers.{name}.model must be set"),
            });
        };

        if model.is_empty() {
            return Err(Error::ConfigInvalid {
                message: format!("{kind}.providers.{name}.model must not be empty"),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BudgetConfig {
    pub max_agents: CountLimit,
    pub max_concurrent_agents: CountLimit,
    pub max_search_calls_per_agent: CountLimit,
    pub max_tool_calls_per_agent: CountLimit,
    pub max_turns_per_agent: CountLimit,
    pub max_total_model_calls: CountLimit,
    pub max_total_search_calls: CountLimit,
    pub max_agent_timeout_ms: DurationLimitMs,
    pub max_total_timeout_ms: DurationLimitMs,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_agents: Limit::unlimited(),
            max_concurrent_agents: Limit::unlimited(),
            max_search_calls_per_agent: Limit::unlimited(),
            max_tool_calls_per_agent: Limit::unlimited(),
            max_turns_per_agent: Limit::unlimited(),
            max_total_model_calls: Limit::unlimited(),
            max_total_search_calls: Limit::unlimited(),
            max_agent_timeout_ms: Limit::unlimited(),
            max_total_timeout_ms: Limit::unlimited(),
        }
    }
}

impl BudgetConfig {
    fn validate(&self) -> Result<()> {
        self.max_agents.require_non_zero("budget.max_agents")?;
        self.max_concurrent_agents
            .require_non_zero("budget.max_concurrent_agents")?;
        self.max_turns_per_agent
            .require_non_zero("budget.max_turns_per_agent")?;
        self.max_agent_timeout_ms
            .require_non_zero("budget.max_agent_timeout_ms")?;
        self.max_total_timeout_ms
            .require_non_zero("budget.max_total_timeout_ms")?;

        if self.max_concurrent_agents.exceeds(self.max_agents) {
            return Err(Error::ConfigInvalid {
                message: "budget.max_concurrent_agents must not exceed budget.max_agents"
                    .to_owned(),
            });
        }

        Ok(())
    }
}
