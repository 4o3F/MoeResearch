#![warn(clippy::pedantic)]
#![allow(dead_code, clippy::struct_field_names, clippy::missing_errors_doc)]

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use lapis_core::logging::LogFormat;
use lapis_core::net::NetworkClient;
use lapis_core::net::reqwest_client::ReqwestNetworkClient;

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

#[tokio::main]
async fn main() -> lapis_core::error::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Serve { config, log_format } => {
            lapis_core::logging::init(log_format)?;
            let config = lapis_core::config::load_config(config.as_deref())?;
            config.validate()?;
            tracing::info!(
                search_providers = config.search.enabled_count(),
                model_providers = config.model.enabled_count(),
                "lapis core initialized"
            );

            let network: Arc<dyn NetworkClient> =
                Arc::new(ReqwestNetworkClient::from_config(&config.network)?);
            let model_service = lapis_core::model::service::build_model_service(&config, &network)?;
            let search_service =
                lapis_core::search::service::build_search_service(&config, &network)?;

            lapis_core::mcp::serve_stdio(model_service, search_service, config.budget).await
        }
    }
}
