use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use clap::{Args, Subcommand};
use moe_research_config::{MoeResearchConfig, load_config};
use moe_research_error::{Error, Result};

use crate::onboarding::claude::{
    McpEnvVar, McpScope, claude_mcp_add_argv, mcp_servers_json, validate_mcp_name,
};
use crate::onboarding::config::{absolute_path, resolve_config_path};
use crate::onboarding::output::format_command;

#[derive(Debug, Subcommand)]
pub enum McpCommand {
    /// Register MoeResearch as a Claude Code MCP server.
    Register(McpRegisterArgs),
}

#[derive(Debug, Args)]
pub struct McpRegisterArgs {
    /// Path to the MoeResearch TOML configuration file.
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Claude Code MCP server name.
    #[arg(long, default_value = "moeresearch")]
    pub name: String,
    /// Claude Code configuration scope for MCP registration.
    #[arg(long, value_enum, default_value_t = McpScope::Local)]
    pub scope: McpScope,
    /// Log the Claude command and JSON example without registering.
    #[arg(long)]
    pub dry_run: bool,
    /// Confirm writes to shared or user-level Claude Code registration.
    #[arg(long)]
    pub yes: bool,
    /// Path to the Claude Code executable.
    #[arg(long, default_value = "claude")]
    pub claude_bin: PathBuf,
    /// Path to the MoeResearch executable used in the MCP server command.
    #[arg(long = "moeresearch-bin")]
    pub moe_research_bin: Option<PathBuf>,
}

pub fn run(command: McpCommand) -> Result<()> {
    match command {
        McpCommand::Register(args) => run_register(args),
    }
}

fn resolve_moe_research_bin(moe_research_bin: Option<PathBuf>) -> Result<PathBuf> {
    match moe_research_bin {
        Some(path) => Ok(path),
        None => std::env::current_exe().map_err(|source| Error::Internal {
            message: format!("failed to locate current MoeResearch executable: {source}"),
        }),
    }
}

pub fn run_register(register_args: McpRegisterArgs) -> Result<()> {
    validate_mcp_name(&register_args.name)?;
    let config_path = absolute_path(&resolve_config_path(register_args.config))?;
    let moe_research_bin = resolve_moe_research_bin(register_args.moe_research_bin)?;

    if !register_args.dry_run && register_args.scope.needs_confirmation() && !register_args.yes {
        return Err(Error::InvalidInput {
            message: format!(
                "--scope {} changes shared or global Claude Code registration; pass --yes to confirm",
                register_args.scope.as_str()
            ),
        });
    }

    let config = load_config(Some(&config_path))?;
    let provider_envs = provider_env_vars(&config);
    let claude_args = claude_mcp_add_argv(
        &register_args.name,
        register_args.scope,
        &moe_research_bin,
        &config_path,
        &provider_envs,
    );

    if register_args.dry_run {
        let redacted_envs = redacted_env_vars(&provider_envs);
        let redacted_args = claude_mcp_add_argv(
            &register_args.name,
            register_args.scope,
            &moe_research_bin,
            &config_path,
            &redacted_envs,
        );
        tracing::info!(
            command = %format_command(&register_args.claude_bin, &redacted_args),
            config = %mcp_servers_json(&register_args.name, &moe_research_bin, &config_path, &provider_envs),
            "would register MoeResearch MCP server"
        );
        return Ok(());
    }

    let status = ProcessCommand::new(&register_args.claude_bin)
        .args(&claude_args)
        .status()
        .map_err(|source| Error::Internal {
            message: format!(
                "failed to execute {}: {source}",
                register_args.claude_bin.display()
            ),
        })?;

    if !status.success() {
        return Err(Error::Internal {
            message: format!("claude mcp add failed with status {status}"),
        });
    }

    Ok(())
}

fn provider_env_vars(config: &MoeResearchConfig) -> Vec<McpEnvVar> {
    let mut envs = BTreeMap::new();
    for provider in config.enabled_provider_envs() {
        if let Some(name) = provider.api_key_env
            && let Some(value) = std::env::var_os(name)
        {
            envs.entry(name.to_owned()).or_insert(value);
        }
    }

    envs.into_iter()
        .map(|(name, value)| McpEnvVar { name, value })
        .collect()
}

fn redacted_env_vars(envs: &[McpEnvVar]) -> Vec<McpEnvVar> {
    envs.iter()
        .map(|env| McpEnvVar {
            name: env.name.clone(),
            value: OsString::from("<redacted>"),
        })
        .collect()
}
