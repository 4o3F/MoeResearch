#![warn(clippy::pedantic)]
#![allow(
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

use clap::{Parser, Subcommand};
use moe_research_cli::commands;
use moe_research_error::{Error, Result};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "moeresearch")]
#[command(version, about = "MoeResearch Rust Core")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Download and install MoeResearch asset bundles.
    Assets(commands::assets::AssetsArgs),
    /// Run the MoeResearch MCP server over stdio.
    Serve(commands::serve::ServeArgs),
    /// Create a MoeResearch configuration file.
    Init(commands::init::InitArgs),
    /// Validate local MoeResearch configuration and readiness.
    Check(commands::check::CheckArgs),
    /// Create config and guide Claude Code MCP registration.
    Onboard(commands::onboard::OnboardArgs),
    /// Manage MoeResearch MCP registration for Claude Code.
    Mcp {
        #[command(subcommand)]
        command: commands::mcp::McpCommand,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Assets(args) => {
            init_cli_logging()?;
            commands::assets::run(args).await
        }
        Command::Serve(args) => commands::serve::run(args).await,
        Command::Init(args) => {
            init_cli_logging()?;
            commands::init::run(args)
        }
        Command::Check(args) => {
            init_cli_logging()?;
            commands::check::run(args)
        }
        Command::Onboard(args) => {
            init_cli_logging()?;
            commands::onboard::run(&args)
        }
        Command::Mcp { command } => {
            init_cli_logging()?;
            commands::mcp::run(command)
        }
    }
}

fn init_cli_logging() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("moeresearch=info,moe_research=info")),
        )
        .without_time()
        .with_target(false)
        .try_init()
        .map_err(|source| Error::LoggingInit {
            message: source.to_string(),
        })
}
