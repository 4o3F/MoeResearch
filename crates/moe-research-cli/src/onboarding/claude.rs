use std::ffi::OsString;
use std::path::Path;

use clap::ValueEnum;
use moe_research_error::{Error, Result};
use serde_json::{Map, Value, json};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpEnvVar {
    pub name: String,
    pub value: OsString,
}

impl McpEnvVar {
    fn assignment(&self) -> OsString {
        let mut assignment = OsString::from(&self.name);
        assignment.push("=");
        assignment.push(&self.value);
        assignment
    }
}

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
    moe_research_bin: &Path,
    config_path: &Path,
    env_vars: &[McpEnvVar],
) -> Vec<OsString> {
    let mut argv = vec![
        OsString::from("mcp"),
        OsString::from("add"),
        OsString::from("--transport"),
        OsString::from("stdio"),
        OsString::from("--scope"),
        OsString::from(scope.as_str()),
    ];
    for env in env_vars {
        argv.push(OsString::from("--env"));
        argv.push(env.assignment());
    }
    argv.extend([
        OsString::from(name),
        OsString::from("--"),
        moe_research_bin.as_os_str().to_owned(),
        OsString::from("serve"),
        OsString::from("--config"),
        config_path.as_os_str().to_owned(),
    ]);
    argv
}

#[must_use]
pub fn mcp_servers_json(
    name: &str,
    moe_research_bin: &Path,
    config_path: &Path,
    env_vars: &[McpEnvVar],
) -> String {
    let mut server = Map::new();
    server.insert("type".to_owned(), json!("stdio"));
    server.insert(
        "command".to_owned(),
        json!(moe_research_bin.display().to_string()),
    );
    server.insert(
        "args".to_owned(),
        json!(["serve", "--config", config_path.display().to_string()]),
    );
    if !env_vars.is_empty() {
        server.insert("env".to_owned(), Value::Object(redacted_env_json(env_vars)));
    }

    let mut servers = Map::new();
    servers.insert(name.to_owned(), Value::Object(server));

    serde_json::to_string_pretty(&json!({ "mcpServers": servers }))
        .expect("MCP server config must serialize")
}

fn redacted_env_json(env_vars: &[McpEnvVar]) -> Map<String, Value> {
    env_vars
        .iter()
        .map(|env| (env.name.clone(), json!("<redacted>")))
        .collect()
}
