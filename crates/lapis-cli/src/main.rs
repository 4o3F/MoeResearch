#![warn(clippy::pedantic)]

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand, ValueEnum};
use lapis_config::{BudgetConfig as ConfigBudgetConfig, ConfigLimit, LapisConfig, load_config};
use lapis_error::{Error, Result};
use lapis_model::{ModelService, OpenAiProvider};
use lapis_net::NetworkClient;
use lapis_net::reqwest_client::ReqwestNetworkClient;
use lapis_search::{ExaSearchProvider, GrokSearchProvider, SearchService};
use lapis_workflow::{AgentBudget, BudgetConfig, Limit, ResearchBudget};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "lapis")]
#[command(about = "Lapis Rust Core")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Serve {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = LogFormat::Json)]
        log_format: LogFormat,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum LogFormat {
    Compact,
    Pretty,
    Json,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Serve { config, log_format } => {
            init_logging(log_format)?;
            let config = load_config(config.as_deref())?;
            tracing::info!(
                search_providers = config.search.enabled_count(),
                model_providers = config.model.enabled_count(),
                "lapis initialized"
            );

            let network: Arc<dyn NetworkClient> = Arc::new(ReqwestNetworkClient::new(
                config.network.timeout_ms,
                config.network.max_retries,
                config.network.retry_backoff_ms,
                &config.network.user_agent,
            )?);
            let model_service = build_model_service(&config, &network)?;
            let search_service = build_search_service(&config, &network)?;
            let budget_config = build_workflow_budget(&config.budget);

            lapis_mcp::serve_stdio(model_service, search_service, budget_config).await
        }
    }
}

fn init_logging(format: LogFormat) -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("lapis=info"));

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
    config: &LapisConfig,
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
                    provider.timeout_ms.or(Some(config.network.timeout_ms)),
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
    config: &LapisConfig,
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
                provider.timeout_ms.or(Some(config.network.timeout_ms)),
            )),
            "grok" => service.register(GrokSearchProvider::with_max_output_tokens(
                network.clone(),
                provider.base_url.clone(),
                api_key,
                provider.timeout_ms.or(Some(config.network.timeout_ms)),
                provider_model("search", name, provider.model.as_ref())?,
                provider.max_output_tokens,
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

fn provider_api_key(kind: &str, name: &str, api_key_env: Option<&String>) -> Result<String> {
    let api_key_env = api_key_env.ok_or_else(|| Error::ProviderUnavailable {
        provider: format!("{kind}:{name}"),
        message: "enabled provider must set api_key_env".to_owned(),
    })?;

    std::env::var(api_key_env).map_err(|_| Error::ProviderUnavailable {
        provider: format!("{kind}:{name}"),
        message: format!("environment variable {api_key_env} is not set"),
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
