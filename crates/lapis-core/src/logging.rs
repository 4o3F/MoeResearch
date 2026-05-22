use clap::ValueEnum;
use tracing_subscriber::EnvFilter;

use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum LogFormat {
    Compact,
    Pretty,
    Json,
}

pub fn init(format: LogFormat) -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("lapis=info"));

    match format {
        LogFormat::Compact => tracing_subscriber::fmt().with_env_filter(filter).try_init(),
        LogFormat::Pretty => tracing_subscriber::fmt()
            .pretty()
            .with_env_filter(filter)
            .try_init(),
        LogFormat::Json => tracing_subscriber::fmt()
            .json()
            .flatten_event(true)
            .with_env_filter(filter)
            .try_init(),
    }
    .map_err(|source| Error::LoggingInit {
        message: source.to_string(),
    })
}
