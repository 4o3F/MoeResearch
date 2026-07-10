use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

use clap::{Args, ValueEnum};
use moe_research_config::{
    ConfigLimit, GrokReasoningEffort, LimitsConfig, MoeResearchConfig, load_config,
};
use moe_research_error::{Error, Result};
use moe_research_model::{ModelService, OpenAiProvider};
use moe_research_net::NetworkClient;
use moe_research_net::reqwest_client::ReqwestNetworkClient;
use moe_research_search::{
    ExaSearchProvider, GrokSearchProvider, SearchService, TavilySearchProvider,
};
use moe_research_workflow::{AgentLimits, BudgetConfig, Limit, ResearchLimits};
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
    let log_format = args.log_format;
    let config_source = if args.config.is_some() {
        "explicit"
    } else {
        "default"
    };
    let config_filename = args
        .config
        .as_ref()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned);
    let (effective_filter, filter_source) = init_logging(log_format)?;
    let ppid = parent_pid_best_effort();
    let parent_process_name = ppid.as_deref().and_then(parent_process_name_best_effort);
    let launcher_hint = parent_process_name.as_deref().unwrap_or("unknown");

    tracing::info!(
        event = "serve_starting",
        status = "starting",
        binary = "moeresearch",
        version = env!("CARGO_PKG_VERSION"),
        os = env::consts::OS,
        arch = env::consts::ARCH,
        pid = process::id(),
        ppid = ?ppid,
        parent_process_name = ?parent_process_name,
        launcher_hint,
        config_source,
        config_filename = ?config_filename,
        log_format = ?log_format,
        rust_log_present = env::var_os("RUST_LOG").is_some(),
        effective_filter = %effective_filter,
        filter_source,
        "moeresearch serve starting"
    );

    let config = load_config(args.config.as_deref())?;
    let workflow_budget = build_workflow_budget(&config.limits);
    let enabled_model_providers = enabled_model_provider_names(&config);
    let enabled_search_providers = enabled_search_provider_names(&config);
    tracing::info!(
        event = "serve_initialized",
        status = "ok",
        model_provider_count = enabled_model_providers.len(),
        model_providers = ?enabled_model_providers,
        search_provider_count = enabled_search_providers.len(),
        search_providers = ?enabled_search_providers,
        network_timeout_ms = config.network.inactivity_timeout_ms,
        network_max_retries = config.network.max_retries,
        network_retry_backoff_ms = config.network.retry_backoff_ms,
        operator_limits_research = ?workflow_budget.research,
        operator_limits_per_agent = ?workflow_budget.per_agent,
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

    moe_research_mcp::serve_stdio(model_service, search_service, workflow_budget).await
}

fn init_logging(format: LogFormat) -> Result<(String, &'static str)> {
    let default_filter = "moeresearch=info,moe_research=info";
    let rust_log = env::var("RUST_LOG").ok();
    let (filter, effective_filter, filter_source) = match rust_log.as_deref() {
        Some(value) => match EnvFilter::try_new(value) {
            Ok(filter) => (filter, value.to_owned(), "env"),
            Err(_) => (
                EnvFilter::new(default_filter),
                default_filter.to_owned(),
                "default_invalid_env",
            ),
        },
        None => (
            EnvFilter::new(default_filter),
            default_filter.to_owned(),
            "default",
        ),
    };

    match format {
        LogFormat::Compact => tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .try_init(),
        LogFormat::Pretty => tracing_subscriber::fmt()
            .pretty()
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .try_init(),
        LogFormat::Json => tracing_subscriber::fmt()
            .json()
            .flatten_event(true)
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .try_init(),
    }
    .map_err(|source| Error::LoggingInit {
        message: source.to_string(),
    })?;

    Ok((effective_filter, filter_source))
}

fn parent_pid_best_effort() -> Option<String> {
    let stat = fs::read_to_string("/proc/self/stat").ok()?;
    let after_process_name = stat.rsplit_once(") ")?.1;
    after_process_name
        .split_whitespace()
        .nth(1)
        .map(ToOwned::to_owned)
}

fn parent_process_name_best_effort(ppid: &str) -> Option<String> {
    let name = fs::read_to_string(format!("/proc/{ppid}/comm")).ok()?;
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

fn enabled_model_provider_names(config: &MoeResearchConfig) -> Vec<&str> {
    config
        .model
        .providers
        .iter()
        .filter_map(|(name, provider)| provider.enabled.then_some(name.as_str()))
        .collect()
}

fn enabled_search_provider_names(config: &MoeResearchConfig) -> Vec<&str> {
    config
        .search
        .providers
        .iter()
        .filter_map(|(name, provider)| provider.enabled.then_some(name.as_str()))
        .collect()
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

fn map_grok_reasoning_effort(
    effort: GrokReasoningEffort,
) -> moe_research_search::GrokReasoningEffort {
    match effort {
        GrokReasoningEffort::None => moe_research_search::GrokReasoningEffort::None,
        GrokReasoningEffort::Low => moe_research_search::GrokReasoningEffort::Low,
        GrokReasoningEffort::Medium => moe_research_search::GrokReasoningEffort::Medium,
        GrokReasoningEffort::High => moe_research_search::GrokReasoningEffort::High,
    }
}

fn provider_api_key(kind: &str, name: &str, api_key_env: Option<&String>) -> Result<String> {
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

fn build_workflow_budget(config: &LimitsConfig) -> BudgetConfig {
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

fn map_limit<T>(limit: ConfigLimit<T>) -> Limit<T> {
    match limit {
        ConfigLimit::Limited(value) => Limit::limited(value),
        ConfigLimit::Unlimited => Limit::unlimited(),
    }
}
