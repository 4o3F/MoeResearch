use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::Args;
use lapis_error::{Error, Result};

use crate::commands::check::CheckArgs;
use crate::commands::init::{self, InitArgs};
use crate::commands::mcp::{self, McpRegisterArgs};
use crate::onboarding::claude::{McpEnvVar, McpScope, claude_mcp_add_argv, mcp_servers_json};
use crate::onboarding::config::{ConfigPlan, absolute_path, resolve_config_path};
use crate::onboarding::output::format_command;

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct OnboardArgs {
    /// Path to the Lapis TOML configuration file.
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Overwrite an existing configuration file during setup.
    #[arg(long)]
    pub force: bool,
    /// Claude Code configuration scope for MCP registration.
    #[arg(long, value_enum, default_value_t = McpScope::Local)]
    pub scope: McpScope,
    /// Log generated config and optional MCP registration without writing changes.
    #[arg(long)]
    pub dry_run: bool,
    /// Confirm writes to shared or user-level Claude Code registration.
    #[arg(long)]
    pub yes: bool,
    /// Register Lapis with Claude Code after config setup.
    #[arg(long)]
    pub register_mcp: bool,
    /// Enable the `OpenAI` model provider in the generated config.
    #[arg(long)]
    pub enable_openai: bool,
    /// Enable the Grok search provider in the generated config.
    #[arg(long)]
    pub enable_grok: bool,
    /// Enable the Exa search provider in the generated config.
    #[arg(long)]
    pub enable_exa: bool,
}

pub fn run(args: OnboardArgs) -> Result<()> {
    let has_provider_flags = provider_flags_set(&args);
    let config_path = resolve_config_path(args.config);

    if args.register_mcp && args.scope.needs_confirmation() && !args.yes && !args.dry_run {
        return Err(Error::InvalidInput {
            message: format!(
                "--scope {} changes shared or global Claude Code registration; pass --yes to confirm",
                args.scope.as_str()
            ),
        });
    }

    if args.dry_run {
        let generated_env_names = if config_path.exists() && !args.force {
            ensure_existing_config_allows_flags(&config_path, has_provider_flags)?;
            tracing::info!(config = %config_path.display(), "would use existing Lapis config");
            None
        } else {
            init::run(InitArgs {
                config: Some(config_path.clone()),
                force: args.force,
                dry_run: true,
                non_interactive: false,
                enable_openai: args.enable_openai,
                enable_grok: args.enable_grok,
                enable_exa: args.enable_exa,
            })?;
            Some(generated_provider_env_names(
                args.enable_openai,
                args.enable_grok,
                args.enable_exa,
            ))
        };
        if args.register_mcp {
            let register_args = McpRegisterArgs {
                config: Some(config_path.clone()),
                name: "lapis".to_owned(),
                scope: args.scope,
                dry_run: true,
                yes: args.yes,
                claude_bin: std::path::PathBuf::from("claude"),
                lapis_bin: None,
            };
            if let Some(env_names) = generated_env_names {
                log_generated_mcp_dry_run(&register_args, env_names)?;
            } else {
                mcp::run_register(register_args)?;
            }
        } else {
            log_register_next_step(&config_path, args.scope);
        }
        return Ok(());
    }

    if config_path.exists() {
        if args.force {
            init::run(InitArgs {
                config: Some(config_path.clone()),
                force: true,
                dry_run: false,
                non_interactive: false,
                enable_openai: args.enable_openai,
                enable_grok: args.enable_grok,
                enable_exa: args.enable_exa,
            })?;
        } else {
            ensure_existing_config_allows_flags(&config_path, has_provider_flags)?;
            tracing::info!(config = %config_path.display(), "using existing Lapis config");
        }
    } else {
        init::run(InitArgs {
            config: Some(config_path.clone()),
            force: false,
            dry_run: false,
            non_interactive: false,
            enable_openai: args.enable_openai,
            enable_grok: args.enable_grok,
            enable_exa: args.enable_exa,
        })?;
    }

    crate::commands::check::run(CheckArgs {
        config: Some(config_path.clone()),
        verbose: false,
        json: false,
        live: false,
        no_mcp: false,
    })?;

    if args.register_mcp {
        mcp::run_register(McpRegisterArgs {
            config: Some(config_path),
            name: "lapis".to_owned(),
            scope: args.scope,
            dry_run: false,
            yes: args.yes,
            claude_bin: std::path::PathBuf::from("claude"),
            lapis_bin: None,
        })?;
    } else {
        log_register_next_step(&config_path, args.scope);
    }

    Ok(())
}

fn log_register_next_step(config_path: &std::path::Path, scope: McpScope) {
    tracing::info!(
        scope = scope.as_str(),
        config = %config_path.display(),
        "next: lapis mcp register --scope <scope> --config <path>"
    );
}

fn provider_flags_set(args: &OnboardArgs) -> bool {
    args.enable_openai || args.enable_grok || args.enable_exa
}

fn generated_provider_env_names(
    enable_openai: bool,
    enable_grok: bool,
    enable_exa: bool,
) -> Vec<String> {
    let plan = ConfigPlan::new(enable_openai, enable_grok, enable_exa);
    let mut names = BTreeSet::new();
    if plan.openai.enabled {
        names.insert(plan.openai.api_key_env);
    }
    if plan.grok.enabled {
        names.insert(plan.grok.api_key_env);
    }
    if plan.exa.enabled {
        names.insert(plan.exa.api_key_env);
    }
    names.into_iter().collect()
}

fn log_generated_mcp_dry_run(args: &McpRegisterArgs, env_names: Vec<String>) -> Result<()> {
    let config_arg = args.config.as_deref().ok_or_else(|| Error::Internal {
        message: "onboard generated dry-run missing config path".to_owned(),
    })?;
    let config_path = absolute_path(config_arg)?;
    let lapis_bin = std::env::current_exe().map_err(|source| Error::Internal {
        message: format!("failed to locate current Lapis executable: {source}"),
    })?;
    let env_vars = env_names
        .into_iter()
        .map(|name| McpEnvVar {
            name,
            value: OsString::from("<redacted>"),
        })
        .collect::<Vec<_>>();
    let claude_args =
        claude_mcp_add_argv(&args.name, args.scope, &lapis_bin, &config_path, &env_vars);

    tracing::info!(
        command = %format_command(&args.claude_bin, &claude_args),
        config = %mcp_servers_json(&args.name, &lapis_bin, &config_path, &env_vars),
        "would register Lapis MCP server"
    );
    Ok(())
}

fn ensure_existing_config_allows_flags(config_path: &Path, has_provider_flags: bool) -> Result<()> {
    if has_provider_flags {
        return Err(Error::InvalidInput {
            message: format!(
                "{} already exists; provider flags only apply when generating a config. Edit {} or rerun with `lapis onboard --force --config {}`",
                config_path.display(),
                config_path.display(),
                config_path.display()
            ),
        });
    }

    Ok(())
}
