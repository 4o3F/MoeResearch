use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

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
        config.validate_env_keys().map_err(de::Error::custom)?;
        Ok(config)
    }
}

impl LapisConfig {
    pub fn validate_env_keys(&self) -> Result<()> {
        validate_provider_env("search", &self.search.providers)?;
        validate_provider_env("model", &self.model.providers)?;
        Ok(())
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
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            max_retries: 2,
            retry_backoff_ms: 200,
            user_agent: "lapis/0.1.0".to_owned(),
        }
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
            },
        );
        providers.insert(
            "grok".to_owned(),
            ProviderEndpoint {
                enabled: false,
                base_url: "https://api.x.ai".to_owned(),
                api_key_env: Some("XAI_API_KEY".to_owned()),
                timeout_ms: None,
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
}

impl Default for ModelConfig {
    fn default() -> Self {
        let mut providers = BTreeMap::new();
        providers.insert(
            "openai-compatible".to_owned(),
            ProviderEndpoint {
                enabled: false,
                base_url: "https://api.openai.com/v1".to_owned(),
                api_key_env: Some("OPENAI_API_KEY".to_owned()),
                timeout_ms: None,
            },
        );

        Self {
            providers,
            default_provider: "openai-compatible".to_owned(),
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
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BudgetConfig {
    pub max_agents: usize,
    pub max_concurrent_agents: usize,
    pub max_search_calls_per_agent: usize,
    pub max_turns_per_agent: usize,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_agents: 5,
            max_concurrent_agents: 2,
            max_search_calls_per_agent: 4,
            max_turns_per_agent: 6,
        }
    }
}

fn validate_provider_env(kind: &str, providers: &BTreeMap<String, ProviderEndpoint>) -> Result<()> {
    for (name, provider) in providers {
        if !provider.enabled {
            continue;
        }

        let Some(env_name) = provider.api_key_env.as_ref() else {
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
    }

    Ok(())
}
