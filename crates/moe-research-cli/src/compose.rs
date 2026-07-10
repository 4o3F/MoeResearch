//! CLI composition root: pure config → runtime mapping and service wiring.
//!
//! Domain crates stay free of host wiring. All ConfigLimit / Grok / provider
//! registration mapping lives here so `commands/serve` stays a thin host.
//!
//! Tests for pure mappers live in `moe-research-tests` (not in this file).

use std::env;
use std::sync::Arc;

use moe_research_config::{
    ConfigLimit, GrokReasoningEffort as ConfigGrokReasoningEffort, LimitsConfig, MoeResearchConfig,
};
use moe_research_error::{Error, Result};
use moe_research_model::{ModelService, OpenAiProvider};
use moe_research_net::NetworkClient;
use moe_research_net::reqwest_client::ReqwestNetworkClient;
use moe_research_search::{
    ExaSearchProvider, GrokReasoningEffort as SearchGrokReasoningEffort, GrokSearchProvider,
    SearchService, TavilySearchProvider,
};
use moe_research_workflow::{AgentLimits, BudgetConfig, Limit, ResearchLimits};

/// Map operator config limit into workflow budget limit (dual types intentionally kept).
#[must_use]
pub fn map_limit<T>(limit: ConfigLimit<T>) -> Limit<T> {
    match limit {
        ConfigLimit::Limited(value) => Limit::limited(value),
        ConfigLimit::Unlimited => Limit::unlimited(),
    }
}

/// Build workflow `BudgetConfig` from operator `LimitsConfig`.
#[must_use]
pub fn build_workflow_budget(config: &LimitsConfig) -> BudgetConfig {
    BudgetConfig {
        research: ResearchLimits {
            max_agents: map_limit(config.research.max_agents),
            max_concurrent_agents: map_limit(config.research.max_concurrent_agents),
            max_total_model_calls: map_limit(config.research.max_total_model_calls),
            max_total_search_calls: map_limit(config.research.max_total_search_calls),
            total_timeout_ms: map_limit(config.research.total_timeout_ms),
            max_tokens: map_limit(config.research.max_tokens),
        },
        per_agent: AgentLimits {
            max_turns: map_limit(config.per_agent.max_turns),
            max_tool_calls: map_limit(config.per_agent.max_tool_calls),
            max_search_calls: map_limit(config.per_agent.max_search_calls),
            timeout_ms: map_limit(config.per_agent.timeout_ms),
        },
    }
}

/// Map config Grok reasoning effort into the search-provider enum (dual types kept).
#[must_use]
pub fn map_grok_reasoning_effort(effort: ConfigGrokReasoningEffort) -> SearchGrokReasoningEffort {
    match effort {
        ConfigGrokReasoningEffort::None => SearchGrokReasoningEffort::None,
        ConfigGrokReasoningEffort::Low => SearchGrokReasoningEffort::Low,
        ConfigGrokReasoningEffort::Medium => SearchGrokReasoningEffort::Medium,
        ConfigGrokReasoningEffort::High => SearchGrokReasoningEffort::High,
    }
}

pub fn provider_api_key(kind: &str, name: &str, api_key_env: Option<&String>) -> Result<String> {
    let api_key_env = api_key_env.ok_or_else(|| Error::ProviderUnavailable {
        provider: format!("{kind}:{name}"),
        message: "enabled provider must set api_key_env".to_owned(),
        retryable: false,
    })?;

    env::var(api_key_env).map_err(|_| Error::ProviderUnavailable {
        provider: format!("{kind}:{name}"),
        message: format!("environment variable {api_key_env} is not set"),
        retryable: false,
    })
}

pub fn provider_model(kind: &str, name: &str, model: Option<&String>) -> Result<String> {
    let Some(model) = model
        .as_ref()
        .map(|value| value.trim())
        .filter(|model| !model.is_empty())
    else {
        return Err(Error::ConfigInvalid {
            message: format!("{kind}.providers.{name}.model must be set"),
        });
    };
    Ok(model.to_owned())
}

#[must_use]
pub fn enabled_model_provider_names(config: &MoeResearchConfig) -> Vec<&str> {
    config
        .model
        .providers
        .iter()
        .filter_map(|(name, provider)| provider.enabled.then_some(name.as_str()))
        .collect()
}

#[must_use]
pub fn enabled_search_provider_names(config: &MoeResearchConfig) -> Vec<&str> {
    config
        .search
        .providers
        .iter()
        .filter_map(|(name, provider)| provider.enabled.then_some(name.as_str()))
        .collect()
}

pub fn build_network_client(config: &MoeResearchConfig) -> Result<Arc<dyn NetworkClient>> {
    Ok(Arc::new(ReqwestNetworkClient::new(
        config.network.inactivity_timeout_ms,
        config.network.max_retries,
        config.network.retry_backoff_ms,
        &config.network.user_agent,
    )?))
}

pub fn build_model_service(
    config: &MoeResearchConfig,
    network: &Arc<dyn NetworkClient>,
) -> Result<ModelService> {
    let mut service = ModelService::new();

    for (name, provider) in &config.model.providers {
        if !provider.enabled {
            continue;
        }

        match name.as_str() {
            "openai" => {
                let api_key = provider_api_key("model", name, provider.api_key_env.as_ref())?;
                let model = provider_model("model", name, provider.model.as_ref())?;
                service.register(OpenAiProvider::new(
                    network.clone(),
                    provider.base_url.clone(),
                    api_key,
                    provider
                        .inactivity_timeout_ms
                        .or(Some(config.network.inactivity_timeout_ms)),
                    model,
                ));
            }
            other => {
                return Err(Error::ConfigInvalid {
                    message: format!("unknown model provider `{other}`"),
                });
            }
        }
    }

    Ok(service)
}

pub fn build_search_service(
    config: &MoeResearchConfig,
    network: &Arc<dyn NetworkClient>,
) -> Result<SearchService> {
    let mut service = SearchService::new();

    for (name, provider) in &config.search.providers {
        if !provider.enabled {
            continue;
        }

        let api_key = provider_api_key("search", name, provider.api_key_env.as_ref())?;
        match name.as_str() {
            "exa" => service.register(ExaSearchProvider::new(
                network.clone(),
                provider.base_url.clone(),
                api_key,
                provider
                    .inactivity_timeout_ms
                    .or(Some(config.network.inactivity_timeout_ms)),
            )),
            "grok" => service.register(GrokSearchProvider::with_request_options(
                network.clone(),
                provider.base_url.clone(),
                api_key,
                provider
                    .inactivity_timeout_ms
                    .or(Some(config.network.inactivity_timeout_ms)),
                provider_model("search", name, provider.model.as_ref())?,
                provider.max_output_tokens,
                provider.reasoning_effort.map(map_grok_reasoning_effort),
            )),
            "tavily" => service.register(TavilySearchProvider::new(
                network.clone(),
                provider.base_url.clone(),
                api_key,
                provider
                    .inactivity_timeout_ms
                    .or(Some(config.network.inactivity_timeout_ms)),
            )),
            other => {
                return Err(Error::ConfigInvalid {
                    message: format!("unknown search provider `{other}`"),
                });
            }
        }
    }

    Ok(service)
}
