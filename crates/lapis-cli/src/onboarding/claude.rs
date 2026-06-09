use std::ffi::OsString;
use std::path::Path;

use clap::ValueEnum;
use lapis_error::{Error, Result};
use serde_json::{Map, json};

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum McpScope {
    /// Store registration in the current Claude Code project.
    Local,
    /// Store registration in shared project configuration.
    Project,
    /// Store registration in the user-level Claude Code configuration.
    User,
}

impl McpScope {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Project => "project",
            Self::User => "user",
        }
    }

    #[must_use]
    pub const fn needs_confirmation(self) -> bool {
        matches!(self, Self::Project | Self::User)
    }
}

pub fn validate_mcp_name(name: &str) -> Result<()> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(Error::InvalidInput {
            message: "MCP server name must contain only letters, numbers, `_`, `-`, or `.`"
                .to_owned(),
        });
    }
    Ok(())
}

#[must_use]
pub fn claude_mcp_add_argv(
    name: &str,
    scope: McpScope,
    lapis_bin: &Path,
    config_path: &Path,
) -> Vec<OsString> {
    vec![
        OsString::from("mcp"),
        OsString::from("add"),
        OsString::from("--transport"),
        OsString::from("stdio"),
        OsString::from("--scope"),
        OsString::from(scope.as_str()),
        OsString::from(name),
        OsString::from("--"),
        lapis_bin.as_os_str().to_owned(),
        OsString::from("serve"),
        OsString::from("--config"),
        config_path.as_os_str().to_owned(),
    ]
}

#[must_use]
pub fn mcp_servers_json(name: &str, lapis_bin: &Path, config_path: &Path) -> String {
    let mut servers = Map::new();
    servers.insert(
        name.to_owned(),
        json!({
            "type": "stdio",
            "command": lapis_bin.display().to_string(),
            "args": ["serve", "--config", config_path.display().to_string()],
        }),
    );

    serde_json::to_string_pretty(&json!({ "mcpServers": servers }))
        .expect("MCP server config must serialize")
}
