use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use clap::Args;
use moe_research_config::{MoeResearchConfig, load_config};
use moe_research_error::{Error, Result};
use serde_json::{Value, json};

use crate::compose::{enabled_model_provider_names, enabled_search_provider_names};
use crate::onboarding::config::resolve_config_path;

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct CheckArgs {
    /// Path to the MoeResearch TOML configuration file.
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Include detailed diagnostics in the check output.
    #[arg(long)]
    pub verbose: bool,
    /// Log the check report as JSON.
    #[arg(long)]
    pub json: bool,
    /// Include provider reachability checks when available.
    #[arg(long)]
    pub live: bool,
    /// Skip the local MCP smoke check.
    #[arg(long)]
    pub no_mcp: bool,
    /// List enabled model and search provider names from config (no secrets).
    #[arg(long)]
    pub show_providers: bool,
}

pub fn run(args: CheckArgs) -> Result<()> {
    let path = resolve_config_path(args.config);
    let mut rows = Vec::new();

    let config = match load_config(Some(&path)) {
        Ok(config) => {
            rows.push(CheckRow::pass(
                "config",
                format!("{} is valid", path.display()),
            ));
            Some(config)
        }
        Err(error) => {
            rows.push(CheckRow::fail(
                "config",
                error.to_string(),
                Some(config_failure_fix(&path, &error)),
                Some("docs/configuration.md".to_owned()),
            ));
            None
        }
    };

    if let Some(config) = &config {
        rows.extend(check_provider_environment(config));
        if args.show_providers {
            rows.extend(check_show_providers(config));
        }
        if args.live {
            rows.extend(check_live_probe_support(config));
        }
        if !args.no_mcp {
            if rows.iter().any(|row| row.status == CheckStatus::Fail) {
                rows.push(CheckRow::skipped(
                    "mcp",
                    "skipped because required checks failed",
                ));
            } else {
                rows.push(check_mcp_smoke(&path));
            }
        }
    }

    log_check_rows(&rows, args.json, args.verbose);
    if rows.iter().any(|row| row.status == CheckStatus::Fail) {
        return Err(Error::InvalidInput {
            message: "moeresearch check failed".to_owned(),
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skipped,
}

impl CheckStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warn => "warn",
            Self::Fail => "fail",
            Self::Skipped => "skip",
        }
    }
}

#[derive(Debug)]
struct CheckRow {
    status: CheckStatus,
    target: String,
    summary: String,
    fix: Option<String>,
    detail: Option<String>,
}

fn config_failure_fix(path: &Path, error: &Error) -> String {
    match error {
        Error::ProviderUnavailable { .. } => format!(
            "export the referenced api_key_env variable or disable the provider in {}",
            path.display()
        ),
        _ => format!(
            "edit {} or rerun `moeresearch onboard --force --config {}`",
            path.display(),
            path.display()
        ),
    }
}

impl CheckRow {
    fn pass(target: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Pass,
            target: target.into(),
            summary: summary.into(),
            fix: None,
            detail: None,
        }
    }

    fn warn(target: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Warn,
            target: target.into(),
            summary: summary.into(),
            fix: None,
            detail: None,
        }
    }

    fn fail(
        target: impl Into<String>,
        summary: impl Into<String>,
        fix: Option<String>,
        detail: Option<String>,
    ) -> Self {
        Self {
            status: CheckStatus::Fail,
            target: target.into(),
            summary: summary.into(),
            fix,
            detail,
        }
    }

    fn skipped(target: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Skipped,
            target: target.into(),
            summary: summary.into(),
            fix: None,
            detail: None,
        }
    }
}

fn check_provider_environment(config: &MoeResearchConfig) -> Vec<CheckRow> {
    let mut rows = Vec::new();

    if config.model.enabled_count() == 0 {
        rows.push(CheckRow::fail(
            "model",
            "no model provider is enabled",
            Some("enable [model.providers.openai] in moeresearch.toml".to_owned()),
            Some("docs/configuration.md".to_owned()),
        ));
    }

    if config.search.enabled_count() == 0 {
        rows.push(CheckRow::warn(
            "search",
            "no search provider is enabled; search-enabled aspects will fail",
        ));
    }

    for provider in config.enabled_provider_envs() {
        if let Some(env_name) = provider.api_key_env {
            rows.push(CheckRow::pass(
                format!("{}:{}", provider.kind, provider.name),
                format!("{env_name} is set"),
            ));
        }
    }

    rows
}

fn check_show_providers(config: &MoeResearchConfig) -> Vec<CheckRow> {
    let models = enabled_model_provider_names(config);
    let searches = enabled_search_provider_names(config);
    let mut rows = Vec::new();

    rows.push(if models.is_empty() {
        CheckRow::fail(
            "providers:model",
            "no model providers enabled",
            Some("enable [model.providers.openai] in moeresearch.toml".to_owned()),
            Some("docs/configuration.md".to_owned()),
        )
    } else {
        CheckRow::pass("providers:model", format!("enabled: {}", models.join(", ")))
    });
    rows.push(if searches.is_empty() {
        CheckRow::warn(
            "providers:search",
            "no search providers enabled; search-enabled aspects will fail",
        )
    } else {
        CheckRow::pass(
            "providers:search",
            format!("enabled: {}", searches.join(", ")),
        )
    });
    rows
}

fn check_live_probe_support(config: &MoeResearchConfig) -> Vec<CheckRow> {
    config
        .enabled_provider_envs()
        .into_iter()
        .map(|provider| {
            CheckRow::warn(
                format!("{}:{}", provider.kind, provider.name),
                "provider reachability probe is deferred in v1; local checks verify config and environment variables only",
            )
        })
        .collect()
}

fn check_mcp_smoke(config_path: &Path) -> CheckRow {
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(source) => {
            return CheckRow::fail(
                "mcp",
                format!("failed to locate current executable: {source}"),
                None,
                None,
            );
        }
    };

    let mut child = match Command::new(executable)
        .arg("serve")
        .arg("--config")
        .arg(config_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(source) => {
            return CheckRow::fail(
                "mcp",
                format!("failed to start MCP server: {source}"),
                Some("run `moeresearch serve --config <path>` for details".to_owned()),
                None,
            );
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let messages = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"moe-research-check\",\"version\":\"0.1.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n"
        );
        if let Err(source) = stdin.write_all(messages.as_bytes()) {
            let _ = child.kill();
            return CheckRow::fail(
                "mcp",
                format!("failed to write MCP request: {source}"),
                None,
                None,
            );
        }
    }

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(source) => {
                let _ = child.kill();
                return CheckRow::fail(
                    "mcp",
                    format!("failed to poll MCP server: {source}"),
                    None,
                    None,
                );
            }
        }
    }
    if child.try_wait().ok().flatten().is_none() {
        let _ = child.kill();
    }

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(source) => {
            return CheckRow::fail(
                "mcp",
                format!("failed to collect MCP output: {source}"),
                None,
                None,
            );
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout);

    if tools_list_contains(&stdout, &["aspect_research", "deep_research"]) {
        CheckRow::pass(
            "mcp",
            "tools/list exposes aspect_research and deep_research",
        )
    } else {
        CheckRow::fail(
            "mcp",
            "tools/list did not expose expected MoeResearch tools",
            Some("run `moeresearch serve --config <path>` and inspect stderr".to_owned()),
            Some(String::from_utf8_lossy(&output.stderr).into_owned()),
        )
    }
}

fn tools_list_contains(stdout: &str, expected_tools: &[&str]) -> bool {
    stdout.lines().any(|line| {
        let Ok(message) = serde_json::from_str::<Value>(line) else {
            return false;
        };
        if message.get("id") != Some(&Value::from(2)) {
            return false;
        }
        let Some(tools) = message
            .get("result")
            .and_then(|result| result.get("tools"))
            .and_then(Value::as_array)
        else {
            return false;
        };

        expected_tools.iter().all(|expected| {
            tools.iter().any(|tool| {
                tool.get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name == *expected)
            })
        })
    })
}

fn log_check_rows(rows: &[CheckRow], json: bool, verbose: bool) {
    if json {
        let rows = check_rows_json(rows, verbose);
        tracing::info!(report = %serde_json::to_string_pretty(&rows).expect("check rows must serialize"), "moeresearch check report");
        return;
    }

    tracing::info!("moeresearch check");
    for row in rows {
        tracing::info!(
            status = row.status.as_str(),
            target = %row.target,
            summary = %row.summary,
            fix = row.fix.as_deref(),
            detail = if verbose { row.detail.as_deref() } else { None },
            "check result"
        );
    }
}

fn check_rows_json(rows: &[CheckRow], verbose: bool) -> Vec<Value> {
    rows.iter()
        .map(|row| {
            json!({
                "status": row.status.as_str(),
                "target": row.target,
                "summary": row.summary,
                "fix": row.fix,
                "detail": if verbose { row.detail.clone() } else { None },
            })
        })
        .collect()
}
