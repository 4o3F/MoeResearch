use std::fmt::Write;
use std::path::{Path, PathBuf};

use moe_research_error::{Error, Result};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct ProviderSelections {
    pub openai: bool,
    pub grok: bool,
    pub exa: bool,
    pub tavily: bool,
}

impl ProviderSelections {
    #[must_use]
    pub const fn any(self) -> bool {
        self.openai || self.grok || self.exa || self.tavily
    }
}

#[derive(Clone, Debug)]
pub struct ConfigPlan {
    pub network_inactivity_timeout_ms: u64,
    pub openai: ProviderPlan,
    pub grok: ProviderPlan,
    pub exa: ProviderPlan,
    pub tavily: ProviderPlan,
}

impl ConfigPlan {
    #[must_use]
    pub fn new(selections: ProviderSelections) -> Self {
        let mut plan = Self::default();
        plan.openai.enabled = selections.openai;
        plan.grok.enabled = selections.grok;
        plan.exa.enabled = selections.exa;
        plan.tavily.enabled = selections.tavily;
        plan
    }

    #[must_use]
    pub const fn model_enabled(&self) -> bool {
        self.openai.enabled
    }

    #[must_use]
    pub const fn search_enabled(&self) -> bool {
        self.grok.enabled || self.exa.enabled || self.tavily.enabled
    }
}

impl Default for ConfigPlan {
    fn default() -> Self {
        Self {
            network_inactivity_timeout_ms: 120_000,
            openai: ProviderPlan::new(
                false,
                "https://api.openai.com/v1",
                "OPENAI_API_KEY",
                120_000,
                Some("gpt-5.5"),
            ),
            grok: ProviderPlan::new(
                false,
                "https://api.x.ai/v1",
                "XAI_API_KEY",
                120_000,
                Some("grok-4.3"),
            ),
            exa: ProviderPlan::new(false, "https://api.exa.ai", "EXA_API_KEY", 120_000, None),
            tavily: ProviderPlan::new(
                false,
                "https://api.tavily.com",
                "TAVILY_API_KEY",
                120_000,
                None,
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProviderPlan {
    pub enabled: bool,
    pub base_url: String,
    pub api_key_env: String,
    pub inactivity_timeout_ms: u64,
    pub model: Option<String>,
}

impl ProviderPlan {
    #[must_use]
    pub fn new(
        enabled: bool,
        base_url: &str,
        api_key_env: &str,
        inactivity_timeout_ms: u64,
        model: Option<&str>,
    ) -> Self {
        Self {
            enabled,
            base_url: base_url.to_owned(),
            api_key_env: api_key_env.to_owned(),
            inactivity_timeout_ms,
            model: model.map(str::to_owned),
        }
    }
}

#[must_use]
pub fn resolve_config_path(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(default_user_config_path)
}

#[must_use]
pub fn default_user_config_path() -> PathBuf {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home)
            .join("moeresearch")
            .join("moeresearch.toml");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("moeresearch")
            .join("moeresearch.toml");
    }
    PathBuf::from("moeresearch.toml")
}

pub fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    std::env::current_dir()
        .map(|current_dir| current_dir.join(path))
        .map_err(|source| Error::Internal {
            message: format!("failed to resolve current directory: {source}"),
        })
}

#[must_use]
pub fn render_config(plan: &ConfigPlan) -> String {
    format!(
        r#"[logging]
format = "json"

[network]
inactivity_timeout_ms = {network_inactivity_timeout_ms}
max_retries = 2
retry_backoff_ms = 200
user_agent = "moeresearch/0.1.0"

[search.providers.exa]
enabled = {}
base_url = {}
api_key_env = {}
inactivity_timeout_ms = {exa_timeout_ms}

[search.providers.tavily]
enabled = {}
base_url = {}
api_key_env = {}
inactivity_timeout_ms = {tavily_timeout_ms}

[search.providers.grok]
enabled = {}
base_url = {}
api_key_env = {}
inactivity_timeout_ms = {grok_timeout_ms}
model = {}

[model.providers.openai]
enabled = {}
base_url = {}
api_key_env = {}
inactivity_timeout_ms = {openai_timeout_ms}
model = {}

[limits.research]
max_agents = -1
max_concurrent_agents = -1
max_total_model_calls = -1
max_total_search_calls = -1
total_timeout_ms = -1
max_tokens = -1

[limits.per_agent]
max_turns = -1
max_tool_calls = -1
max_search_calls = -1
timeout_ms = -1
"#,
        plan.exa.enabled,
        toml_string(&plan.exa.base_url),
        toml_string(&plan.exa.api_key_env),
        plan.tavily.enabled,
        toml_string(&plan.tavily.base_url),
        toml_string(&plan.tavily.api_key_env),
        plan.grok.enabled,
        toml_string(&plan.grok.base_url),
        toml_string(&plan.grok.api_key_env),
        toml_string(plan.grok.model.as_deref().unwrap_or("grok-4.3")),
        plan.openai.enabled,
        toml_string(&plan.openai.base_url),
        toml_string(&plan.openai.api_key_env),
        toml_string(plan.openai.model.as_deref().unwrap_or("gpt-5.5")),
        network_inactivity_timeout_ms = plan.network_inactivity_timeout_ms,
        exa_timeout_ms = plan.exa.inactivity_timeout_ms,
        tavily_timeout_ms = plan.tavily.inactivity_timeout_ms,
        grok_timeout_ms = plan.grok.inactivity_timeout_ms,
        openai_timeout_ms = plan.openai.inactivity_timeout_ms,
    )
}

pub fn write_config_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        let parent_existed = parent.exists();
        std::fs::create_dir_all(parent).map_err(|source| Error::ConfigIo {
            path: parent.to_path_buf(),
            source,
        })?;
        if !parent_existed {
            set_private_dir_permissions(parent)?;
        }
    }

    let tmp_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let tmp_path = path.with_file_name(format!(
        ".{}.tmp-{}-{tmp_suffix}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("moeresearch.toml"),
        std::process::id()
    ));
    std::fs::write(&tmp_path, content).map_err(|source| Error::ConfigIo {
        path: tmp_path.clone(),
        source,
    })?;
    set_private_file_permissions(&tmp_path)?;
    std::fs::rename(&tmp_path, path).map_err(|source| Error::ConfigIo {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(())
}

fn toml_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            ch if ch.is_control() => {
                write!(output, "\\u{:04X}", ch as u32).expect("writing to String must succeed");
            }
            ch => output.push(ch),
        }
    }
    output.push('"');
    output
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(|source| {
        Error::ConfigIo {
            path: path.to_path_buf(),
            source,
        }
    })
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_private_dir_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).map_err(|source| {
        Error::ConfigIo {
            path: path.to_path_buf(),
            source,
        }
    })
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_path: &Path) -> Result<()> {
    Ok(())
}
