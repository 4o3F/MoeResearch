use std::path::PathBuf;
use std::sync::Arc;

use clap::{Args, ValueEnum};
use moe_research_config::{
    BudgetConfig as ConfigBudgetConfig, ConfigLimit,
    GrokReasoningEffort as ConfigGrokReasoningEffort, MoeResearchConfig, load_config,
};
use moe_research_error::{Error, Result};
use moe_research_model::{ModelService, OpenAiProvider};
use moe_research_net::NetworkClient;
use moe_research_net::reqwest_client::ReqwestNetworkClient;
use moe_research_search::{
    ExaSearchProvider, GrokReasoningEffort as SearchGrokReasoningEffort, GrokSearchProvider,
    SearchService, TavilySearchProvider,
};
use moe_research_workflow::{AgentBudget, BudgetConfig, Limit, ResearchBudget};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Path to the MoeResearch TOML configuration file.
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Format used for server logs written to stderr.
    #[arg(long, value_enum, default_value_t = LogFormat::Json)]
    pub log_format: LogFormat,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum LogFormat {
    /// Compact single-line text logs.
    Compact,
    /// Pretty multi-line text logs for local debugging.
    Pretty,
    /// Structured JSON logs.
    Json,
}

pub async fn run(args: ServeArgs) -> Result<()> {
    init_logging(args.log_format)?;
    let config = load_config(args.config.as_deref())?;
    tracing::info!(
        search_providers = config.search.enabled_count(),
        model_providers = config.model.enabled_count(),
        "moeresearch initialized"
    );

    let network: Arc<dyn NetworkClient> = Arc::new(ReqwestNetworkClient::new(
        config.network.inactivity_timeout_ms,
        config.network.max_retries,
        config.network.retry_backoff_ms,
        &config.network.user_agent,
    )?);
    let model_service = build_model_service(&config, &network)?;
    let search_service = build_search_service(&config, &network)?;
    let budget_config = build_workflow_budget(&config.budget);

    moe_research_mcp::serve_stdio(model_service, search_service, budget_config).await
}

fn init_logging(format: LogFormat) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("moeresearch=info,moe_research=info"));

    match format {
        LogFormat::Compact => tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .try_init(),
        LogFormat::Pretty => tracing_subscriber::fmt()
            .pretty()
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .try_init(),
        LogFormat::Json => tracing_subscriber::fmt()
            .json()
            .flatten_event(true)
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .try_init(),
    }
    .map_err(|source| Error::LoggingInit {
        message: source.to_string(),
    })
}

fn build_model_service(
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

fn build_search_service(
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

fn map_grok_reasoning_effort(effort: ConfigGrokReasoningEffort) -> SearchGrokReasoningEffort {
    match effort {
        ConfigGrokReasoningEffort::None => SearchGrokReasoningEffort::None,
        ConfigGrokReasoningEffort::Low => SearchGrokReasoningEffort::Low,
        ConfigGrokReasoningEffort::Medium => SearchGrokReasoningEffort::Medium,
        ConfigGrokReasoningEffort::High => SearchGrokReasoningEffort::High,
    }
}

fn provider_api_key(kind: &str, name: &str, api_key_env: Option<&String>) -> Result<String> {
    let api_key_env = api_key_env.ok_or_else(|| Error::ProviderUnavailable {
        provider: format!("{kind}:{name}"),
        message: "enabled provider must set api_key_env".to_owned(),
        retryable: false,
    })?;

    std::env::var(api_key_env).map_err(|_| Error::ProviderUnavailable {
        provider: format!("{kind}:{name}"),
        message: format!("environment variable {api_key_env} is not set"),
        retryable: false,
    })
}

fn provider_model(kind: &str, name: &str, model: Option<&String>) -> Result<String> {
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

fn build_workflow_budget(config: &ConfigBudgetConfig) -> BudgetConfig {
    BudgetConfig {
        research: ResearchBudget {
            max_agents: map_limit(config.research.max_agents),
            max_concurrent_agents: map_limit(config.research.max_concurrent_agents),
            max_total_model_calls: map_limit(config.research.max_total_model_calls),
            max_total_search_calls: map_limit(config.research.max_total_search_calls),
            total_timeout_ms: map_limit(config.research.total_timeout_ms),
            max_tokens: map_limit(config.research.max_tokens),
        },
        per_agent: AgentBudget {
            max_turns: map_limit(config.per_agent.max_turns),
            max_tool_calls: map_limit(config.per_agent.max_tool_calls),
            max_search_calls: map_limit(config.per_agent.max_search_calls),
            timeout_ms: map_limit(config.per_agent.timeout_ms),
        },
    }
}

fn map_limit<T>(limit: ConfigLimit<T>) -> Limit<T> {
    match limit {
        ConfigLimit::Limited(value) => Limit::limited(value),
        ConfigLimit::Unlimited => Limit::unlimited(),
    }
}
