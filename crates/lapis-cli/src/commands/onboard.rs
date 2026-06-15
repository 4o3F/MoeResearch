use std::path::PathBuf;

use clap::Args;
use lapis_error::{Error, Result};

use crate::commands::check::CheckArgs;
use crate::commands::init::{self, InitArgs};
use crate::commands::mcp::{self, McpRegisterArgs};
use crate::onboarding::claude::McpScope;
use crate::onboarding::config::resolve_config_path;

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct OnboardArgs {
    /// Path to the Lapis TOML configuration file.
    #[arg(long)]
    pub config: Option<PathBuf>,
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
        init::run(InitArgs {
            config: Some(config_path.clone()),
            force: false,
            dry_run: true,
            non_interactive: false,
            enable_openai: args.enable_openai,
            enable_grok: args.enable_grok,
            enable_exa: args.enable_exa,
        })?;
        if args.register_mcp {
            mcp::run_register(McpRegisterArgs {
                config: Some(config_path),
                name: "lapis".to_owned(),
                scope: args.scope,
                dry_run: true,
                yes: args.yes,
                claude_bin: std::path::PathBuf::from("claude"),
                lapis_bin: None,
            })?;
        } else {
            log_register_next_step(&config_path, args.scope);
        }
        return Ok(());
    }

    if !config_path.exists() {
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
