use std::path::{Path, PathBuf};

use snafu::ResultExt;

use crate::error::{ConfigIoSnafu, ConfigParseSnafu, Result};
use crate::schema::config::LapisConfig;

pub fn load_config(path: Option<&Path>) -> Result<LapisConfig> {
    let path = path.map_or_else(default_config_path, Path::to_path_buf);

    if !path.exists() {
        tracing::warn!(
            config_path = %path.display(),
            "user config not found; falling back to default configuration"
        );
        return Ok(LapisConfig::default());
    }

    let content = std::fs::read_to_string(&path).context(ConfigIoSnafu { path: path.clone() })?;
    load_config_from_str(&content, path)
}

pub fn load_config_from_str(content: &str, path: PathBuf) -> Result<LapisConfig> {
    let config: LapisConfig = toml::from_str(content).context(ConfigParseSnafu { path })?;
    config.validate_env_keys()?;
    Ok(config)
}

fn default_config_path() -> PathBuf {
    PathBuf::from("lapis.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_default_when_config_file_is_absent() {
        let config = load_config(Some(Path::new("missing-lapis.toml"))).expect("default config");

        assert_eq!(config.network.timeout_ms, 30_000);
        assert_eq!(config.search.enabled_count(), 0);
    }

    #[test]
    fn rejects_plain_api_key_field() {
        let input = r#"
            [search.providers.exa]
            enabled = true
            base_url = "https://api.exa.ai"
            api_key = "secret"
        "#;

        let err = load_config_from_str(input, PathBuf::from("lapis.toml")).unwrap_err();

        assert!(err.to_string().contains("unknown field `api_key`"));
    }
}
