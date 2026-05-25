use std::collections::BTreeMap;

use reqwest::header::HeaderValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::schema::budget::BudgetConfig;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LapisConfig {
    pub logging: LoggingConfig,
    pub network: NetworkConfig,
    pub search: ProviderRegistry,
    pub model: ProviderRegistry,
    pub budget: BudgetConfig,
}

impl LapisConfig {
    pub fn validate(&self) -> Result<()> {
        self.network.validate()?;
        self.budget.validate()?;
        self.search.validate("search")?;
        self.model.validate("model")
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
pub struct ProviderRegistry {
    pub providers: BTreeMap<String, ProviderEndpoint>,
}

impl ProviderRegistry {
    pub fn enabled_count(&self) -> usize {
        self.providers
            .values()
            .filter(|provider| provider.enabled)
            .count()
    }

    pub(crate) fn validate(&self, kind: &str) -> Result<()> {
        for (name, provider) in &self.providers {
            provider.validate(kind, name)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderEndpoint {
    pub enabled: bool,
    pub base_url: String,
    pub api_key_env: Option<String>,
    pub timeout_ms: Option<u64>,
    pub model: Option<String>,
}

impl ProviderEndpoint {
    /// Validates this endpoint within the context of its registry.
    ///
    /// `kind` is the registry name (`"model"` or `"search"`); `name` is the
    /// TOML provider key (e.g. `"openai"`, `"exa"`, `"grok"`). The dispatch
    /// is name-aware: only providers that actually consume a model identifier
    /// require `model` to be set, and only known provider names are accepted.
    ///
    /// # Errors
    /// - `Error::ConfigInvalid` when a structural rule is violated
    ///   (zero timeout, missing required `model`, unknown provider name).
    /// - `Error::ProviderUnavailable` when an enabled provider is missing the
    ///   `api_key_env` field or the referenced environment variable is unset.
    fn validate(&self, kind: &str, name: &str) -> Result<()> {
        if self.timeout_ms == Some(0) {
            return Err(Error::ConfigInvalid {
                message: format!("{kind}.providers.{name}.timeout_ms must not be zero"),
            });
        }

        match (kind, name) {
            ("model", "openai") | ("search", "grok") => {
                self.validate_enabled_common(kind, name)?;
                self.validate_model(kind, name)
            }
            ("search", "exa") => self.validate_enabled_common(kind, name),
            _ => Err(Error::ConfigInvalid {
                message: format!("unknown {kind}.providers.{name} provider"),
            }),
        }
    }

    /// Validates the constraints common to every enabled provider regardless
    /// of name (currently: `api_key_env` resolves to a set environment
    /// variable). Disabled providers skip these checks so a disabled stanza
    /// does not require credentials to be present in the environment.
    fn validate_enabled_common(&self, kind: &str, name: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        self.validate_env_key(kind, name)
    }

    fn validate_env_key(&self, kind: &str, name: &str) -> Result<()> {
        let env_name = self
            .api_key_env
            .as_ref()
            .ok_or_else(|| Error::ProviderUnavailable {
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

    fn validate_model(&self, kind: &str, name: &str) -> Result<()> {
        if !self.enabled {
            // Skip model validation for disabled providers so example configs
            // can leave `model = ""` for stanzas the operator does not use.
            return Ok(());
        }

        let model = self
            .model
            .as_ref()
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
}
