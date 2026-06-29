use std::path::{Path, PathBuf};

use snafu::ResultExt;

use moe_research_error::{ConfigIoSnafu, ConfigParseSnafu, Error, Result};

use crate::MoeResearchConfig;

pub fn load_config(path: Option<&Path>) -> Result<MoeResearchConfig> {
    let path = path.map_or_else(|| PathBuf::from("moeresearch.toml"), Path::to_path_buf);

    if !path.exists() {
        return Err(Error::ConfigIo {
            path,
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "configuration file not found",
            ),
        });
    }

    let content = std::fs::read_to_string(&path).context(ConfigIoSnafu { path: path.clone() })?;
    let config: MoeResearchConfig = toml::from_str(&content).context(ConfigParseSnafu { path })?;
    config.validate()?;
    Ok(config)
}
