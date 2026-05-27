use std::path::{Path, PathBuf};

use snafu::ResultExt;

use lapis_error::{ConfigIoSnafu, ConfigParseSnafu, Error, Result};

use crate::LapisConfig;

pub fn load_config(path: Option<&Path>) -> Result<LapisConfig> {
    let path = path.map_or_else(default_config_path, Path::to_path_buf);

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
    load_config_from_str(&content, path)
}

fn load_config_from_str(content: &str, path: PathBuf) -> Result<LapisConfig> {
    let config: LapisConfig = toml::from_str(content).context(ConfigParseSnafu { path })?;
    config.validate()?;
    Ok(config)
}

fn default_config_path() -> PathBuf {
    PathBuf::from("lapis.toml")
}
