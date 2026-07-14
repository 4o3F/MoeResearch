use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Args, ValueEnum};
use moe_research_config::load_config;
use moe_research_error::{Error, Result};
use tracing_subscriber::EnvFilter;

use crate::compose::{
    build_model_service, build_network_client, build_search_service, build_web_fetch_service,
    build_workflow_budget, enabled_model_provider_names, enabled_search_provider_names,
};

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
        network_proxy_configured = config.network.proxy_url.is_some(),
        operator_limits_research = ?workflow_budget.research,
        operator_limits_per_agent = ?workflow_budget.per_agent,
        "moeresearch initialized"
    );

    let network = build_network_client(&config)?;
    let model_service = build_model_service(&config, &network)?;
    let search_service = build_search_service(&config, &network)?;
    let web_fetch_service = build_web_fetch_service(&config, &network)?;

    moe_research_mcp::serve_stdio(
        model_service,
        search_service,
        web_fetch_service,
        workflow_budget,
    )
    .await
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
