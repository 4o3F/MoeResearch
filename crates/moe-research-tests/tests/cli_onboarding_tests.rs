use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_ID: AtomicUsize = AtomicUsize::new(0);

const BASE_CONFIG: &str = r#"
[logging]
format = "json"

[network]
inactivity_timeout_ms = 120000
max_retries = 2
retry_backoff_ms = 200
user_agent = "moeresearch/0.1.0"

[search.providers.exa]
enabled = false
base_url = "https://api.exa.ai"
api_key_env = "EXA_API_KEY"
inactivity_timeout_ms = 120000

[search.providers.tavily]
enabled = false
base_url = "https://api.tavily.com"
api_key_env = "TAVILY_API_KEY"
inactivity_timeout_ms = 120000

[search.providers.grok]
enabled = false
base_url = "https://api.x.ai/v1"
api_key_env = "XAI_API_KEY"
inactivity_timeout_ms = 120000
model = "grok-4.3"

[model.providers.openai]
enabled = false
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY"
inactivity_timeout_ms = 120000
model = "gpt-5.5"

[budget.research]
max_agents = -1
max_concurrent_agents = -1
max_total_model_calls = -1
max_total_search_calls = -1
total_timeout_ms = -1
max_tokens = -1

[budget.per_agent]
max_turns = -1
max_tool_calls = -1
max_search_calls = -1
timeout_ms = -1
"#;

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .parent()
        .expect("workspace")
        .to_path_buf()
}

fn temp_path(name: &str) -> PathBuf {
    let id = TEST_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "moe-research-cli-onboarding-{}-{id}-{name}",
        std::process::id()
    ))
}

fn moe_research_command() -> Command {
    let mut command = Command::new(env!("CARGO"));
    command.current_dir(workspace()).args([
        "run",
        "--quiet",
        "--locked",
        "-p",
        "moe-research-cli",
        "--",
    ]);
    command
}

fn write_config(path: &Path, content: &str) {
    std::fs::write(path, content).expect("write config");
}

fn assert_generated_search_provider_without_model(
    content: &str,
    provider: &str,
    base_url: &str,
    api_key_env: &str,
) {
    let config = toml::from_str::<toml::Value>(content).expect("parse generated config");
    let provider_config = config["search"]["providers"][provider]
        .as_table()
        .expect("search provider table");

    assert_eq!(provider_config["enabled"].as_bool(), Some(true));
    assert_eq!(provider_config["base_url"].as_str(), Some(base_url));
    assert_eq!(provider_config["api_key_env"].as_str(), Some(api_key_env));
    assert_eq!(
        provider_config["inactivity_timeout_ms"].as_integer(),
        Some(120_000)
    );
    assert!(provider_config.get("model").is_none());
    assert!(provider_config.get("api_key").is_none());
    assert!(!content.contains("api_key ="));
}

#[test]
fn help_exposes_onboarding_commands() {
    let output = moe_research_command()
        .arg("--help")
        .output()
        .expect("run help");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("assets"));
    assert!(stdout.contains("serve"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("check"));
    assert!(stdout.contains("onboard"));
    assert!(stdout.contains("mcp"));
}

#[test]
fn init_dry_run_does_not_write_config_or_plain_secret_field() {
    let config_path = temp_path("dry-run.toml");

    let output = moe_research_command()
        .args(["init", "--dry-run", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init dry-run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(!config_path.exists());
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.contains("api_key_env"), "stderr: {stderr}");
    assert!(!stderr.contains("api_key ="), "stderr: {stderr}");
}

#[test]
fn init_guided_setup_requires_tty() {
    let config_path = temp_path("guided.toml");

    let output = moe_research_command()
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run guided init");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(!config_path.exists());
    assert!(stderr.contains("not a TTY"), "stderr: {stderr}");
}

#[test]
fn init_writes_valid_config_without_raw_api_key() {
    let config_path = temp_path("moeresearch.toml");

    let output = moe_research_command()
        .args(["init", "--non-interactive", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    let content = std::fs::read_to_string(&config_path).expect("read generated config");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(content.contains("[model.providers.openai]"));
    assert!(content.contains("[search.providers.tavily]"));
    assert!(content.contains("api_key_env"));
    assert!(content.contains("inactivity_timeout_ms = 120000"));
    assert!(!content.contains("inactivity_timeout_ms = 30000"));
    assert!(!content.contains("api_key ="));
    moe_research_config::load_config(Some(&config_path))
        .unwrap_or_else(|error| panic!("generated config should be valid: {error}"));
    let _ = std::fs::remove_file(&config_path);
}

#[test]
fn init_enable_exa_writes_config_without_model_or_raw_api_key() {
    let config_path = temp_path("exa.toml");

    let output = moe_research_command()
        .args(["init", "--non-interactive", "--enable-exa", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read_to_string(&config_path).expect("read generated config");
    let _ = std::fs::remove_file(&config_path);
    assert_generated_search_provider_without_model(
        &content,
        "exa",
        "https://api.exa.ai",
        "EXA_API_KEY",
    );
}

#[test]
fn init_enable_tavily_writes_config_without_model_or_raw_api_key() {
    let config_path = temp_path("tavily.toml");

    let output = moe_research_command()
        .args(["init", "--non-interactive", "--enable-tavily", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read_to_string(&config_path).expect("read generated config");
    let _ = std::fs::remove_file(&config_path);
    assert_generated_search_provider_without_model(
        &content,
        "tavily",
        "https://api.tavily.com",
        "TAVILY_API_KEY",
    );
}

#[test]
fn init_refuses_to_overwrite_without_force() {
    let config_path = temp_path("existing.toml");
    write_config(&config_path, BASE_CONFIG);

    let output = moe_research_command()
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = std::fs::remove_file(&config_path);

    assert!(!output.status.success());
    assert!(stderr.contains("--force"), "stderr: {stderr}");
}

#[test]
fn check_missing_config_returns_actionable_failure() {
    let config_path = temp_path("missing.toml");

    let output = moe_research_command()
        .args(["check", "--config"])
        .arg(&config_path)
        .arg("--no-mcp")
        .output()
        .expect("run check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.contains("config"), "stderr: {stderr}");
    assert!(
        stderr.contains("moeresearch onboard --force --config"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_distinguishes_missing_enabled_provider_env() {
    let config_path = temp_path("missing-env.toml");
    let missing_env = format!("MOERESEARCH_TEST_MISSING_OPENAI_KEY_{}", std::process::id());
    let config = BASE_CONFIG
        .replace(
            "[model.providers.openai]\nenabled = false",
            "[model.providers.openai]\nenabled = true",
        )
        .replace(
            "api_key_env = \"OPENAI_API_KEY\"",
            &format!("api_key_env = \"{missing_env}\""),
        );
    write_config(&config_path, &config);

    let output = moe_research_command()
        .args(["check", "--config"])
        .arg(&config_path)
        .arg("--no-mcp")
        .output()
        .expect("run check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = std::fs::remove_file(&config_path);

    assert!(!output.status.success());
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.contains("config"), "stderr: {stderr}");
    assert!(stderr.contains(&missing_env), "stderr: {stderr}");
    assert!(
        stderr.contains("export the referenced api_key_env"),
        "stderr: {stderr}"
    );
    assert!(!stderr.contains("onboard --force"), "stderr: {stderr}");
}

#[test]
fn check_without_enabled_model_provider_fails_readiness() {
    let config_path = temp_path("no-model.toml");
    write_config(&config_path, BASE_CONFIG);

    let output = moe_research_command()
        .args(["check", "--config"])
        .arg(&config_path)
        .arg("--no-mcp")
        .output()
        .expect("run check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = std::fs::remove_file(&config_path);

    assert!(!output.status.success());
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.contains("model"), "stderr: {stderr}");
    assert!(
        stderr.contains("no model provider is enabled"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_live_is_explicitly_deferred() {
    let config_path = temp_path("live-deferred.toml");
    let config = BASE_CONFIG
        .replace(
            "[model.providers.openai]\nenabled = false",
            "[model.providers.openai]\nenabled = true",
        )
        .replace("api_key_env = \"OPENAI_API_KEY\"", "api_key_env = \"PATH\"");
    write_config(&config_path, &config);

    let output = moe_research_command()
        .args(["check", "--config"])
        .arg(&config_path)
        .args(["--no-mcp", "--live"])
        .output()
        .expect("run live check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = std::fs::remove_file(&config_path);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.contains("model:openai"), "stderr: {stderr}");
    assert!(
        stderr.contains("provider reachability probe is deferred in v1"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_disabled_search_provider_does_not_require_env_var() {
    let config_path = temp_path("disabled-search.toml");
    let config = BASE_CONFIG
        .replace(
            "[model.providers.openai]\nenabled = false",
            "[model.providers.openai]\nenabled = true",
        )
        .replace("api_key_env = \"OPENAI_API_KEY\"", "api_key_env = \"PATH\"");
    write_config(&config_path, &config);

    let output = moe_research_command()
        .args(["check", "--config"])
        .arg(&config_path)
        .arg("--no-mcp")
        .output()
        .expect("run check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = std::fs::remove_file(&config_path);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.contains("config"), "stderr: {stderr}");
    assert!(stderr.contains("search"), "stderr: {stderr}");
    assert!(
        !stderr.contains("EXA_API_KEY is not set"),
        "stderr: {stderr}"
    );
    assert!(
        !stderr.contains("TAVILY_API_KEY is not set"),
        "stderr: {stderr}"
    );
}

#[test]
fn onboard_enable_openai_writes_config_and_logs_registration_step() {
    let config_path = temp_path("onboard-openai.toml");

    let output = moe_research_command()
        .args(["onboard", "--config"])
        .arg(&config_path)
        .arg("--enable-openai")
        .env("OPENAI_API_KEY", "test-key")
        .output()
        .expect("run onboard");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let content = std::fs::read_to_string(&config_path).expect("read generated config");
    let _ = std::fs::remove_file(&config_path);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(content.contains("[model.providers.openai]"));
    assert!(content.contains("enabled = true"));
    assert!(
        stderr.contains("moeresearch mcp register"),
        "stderr: {stderr}"
    );
}

#[test]
fn onboard_dry_run_without_register_mcp_only_prints_next_step() {
    let config_path = temp_path("onboard-dry-run.toml");

    let output = moe_research_command()
        .args(["onboard", "--dry-run", "--config"])
        .arg(&config_path)
        .output()
        .expect("run onboard dry-run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(!config_path.exists());
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(
        stderr.contains("would write MoeResearch config"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains("moeresearch mcp register"),
        "stderr: {stderr}"
    );
    assert!(!stderr.contains("claude mcp add"), "stderr: {stderr}");
}

#[test]
fn onboard_dry_run_register_mcp_for_new_config_does_not_require_written_config() {
    let config_path = temp_path("onboard-dry-run-register.toml");

    let output = moe_research_command()
        .args([
            "onboard",
            "--dry-run",
            "--register-mcp",
            "--enable-openai",
            "--config",
        ])
        .arg(&config_path)
        .env("OPENAI_API_KEY", "raw-test-secret")
        .output()
        .expect("run onboard dry-run register");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(!config_path.exists());
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(
        stderr.contains("would write MoeResearch config"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("claude mcp add"), "stderr: {stderr}");
    assert!(stderr.contains("OPENAI_API_KEY"), "stderr: {stderr}");
    assert!(stderr.contains("<redacted>"), "stderr: {stderr}");
    assert!(!stderr.contains("raw-test-secret"));
}

#[test]
fn onboard_dry_run_register_mcp_for_tavily_redacts_env_value() {
    let config_path = temp_path("onboard-dry-run-register-tavily.toml");

    let output = moe_research_command()
        .args([
            "onboard",
            "--dry-run",
            "--register-mcp",
            "--enable-tavily",
            "--config",
        ])
        .arg(&config_path)
        .env("TAVILY_API_KEY", "raw-test-secret")
        .output()
        .expect("run onboard dry-run register");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(!config_path.exists());
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(
        stderr.contains("would write MoeResearch config"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("claude mcp add"), "stderr: {stderr}");
    assert!(stderr.contains("TAVILY_API_KEY"), "stderr: {stderr}");
    assert!(stderr.contains("<redacted>"), "stderr: {stderr}");
    assert!(!stderr.contains("raw-test-secret"));
}

#[test]
fn onboard_existing_config_rejects_provider_flags_without_force() {
    let config_path = temp_path("onboard-existing-flags.toml");
    write_config(&config_path, BASE_CONFIG);

    let output = moe_research_command()
        .args(["onboard", "--config"])
        .arg(&config_path)
        .arg("--enable-openai")
        .output()
        .expect("run onboard existing");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let content = std::fs::read_to_string(&config_path).expect("read config");
    let _ = std::fs::remove_file(&config_path);

    assert!(!output.status.success());
    assert!(stderr.contains("--force"), "stderr: {stderr}");
    assert!(content.contains("enabled = false"));
}

#[test]
fn onboard_existing_config_without_provider_flags_continues_check() {
    let config_path = temp_path("onboard-existing-check.toml");
    write_config(&config_path, BASE_CONFIG);

    let output = moe_research_command()
        .args(["onboard", "--config"])
        .arg(&config_path)
        .output()
        .expect("run onboard existing");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = std::fs::remove_file(&config_path);

    assert!(!output.status.success());
    assert!(
        stderr.contains("using existing MoeResearch config"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains("no model provider is enabled"),
        "stderr: {stderr}"
    );
}

#[test]
fn onboard_force_allows_regenerating_existing_config() {
    let config_path = temp_path("onboard-force.toml");
    write_config(&config_path, BASE_CONFIG);

    let output = moe_research_command()
        .args(["onboard", "--force", "--enable-openai", "--config"])
        .arg(&config_path)
        .env("OPENAI_API_KEY", "test-key")
        .output()
        .expect("run onboard force");
    let content = std::fs::read_to_string(&config_path).expect("read config");
    let _ = std::fs::remove_file(&config_path);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(content.contains("enabled = true"));
    assert!(content.contains("inactivity_timeout_ms = 120000"));
}

#[test]
fn mcp_register_dry_run_logs_claude_command_and_json_example() {
    let config_path = temp_path("register.toml");
    let env_name = format!("MOERESEARCH_TEST_OPENAI_KEY_{}", std::process::id());
    let config = BASE_CONFIG
        .replace(
            "[model.providers.openai]\nenabled = false",
            "[model.providers.openai]\nenabled = true",
        )
        .replace(
            "api_key_env = \"OPENAI_API_KEY\"",
            &format!("api_key_env = \"{env_name}\""),
        );
    write_config(&config_path, &config);

    let output = moe_research_command()
        .args(["mcp", "register", "--dry-run", "--config"])
        .arg(&config_path)
        .env(&env_name, "raw-test-secret")
        .output()
        .expect("run mcp dry-run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = std::fs::remove_file(&config_path);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.contains("claude mcp add"), "stderr: {stderr}");
    assert!(stderr.contains(&env_name), "stderr: {stderr}");
    assert!(stderr.contains("<redacted>"), "stderr: {stderr}");
    assert!(stderr.contains(" serve --config"), "stderr: {stderr}");
    assert!(stderr.contains("mcpServers"));
    assert!(stderr.contains("stdio"));
    assert!(!stderr.contains("raw-test-secret"));

    let name_index = stderr
        .find("--scope local moeresearch")
        .expect("server name before env flag");
    let env_flag_index = stderr[name_index..]
        .find(" -e ")
        .map(|index| name_index + index)
        .expect("env flag after server name");
    let env_name_index = stderr[env_flag_index..]
        .find(&env_name)
        .map(|index| env_flag_index + index)
        .expect("env name after env flag");
    let redacted_index = stderr[env_name_index..]
        .find("<redacted>")
        .map(|index| env_name_index + index)
        .expect("redacted env value");
    let separator_index = stderr[redacted_index..]
        .find(" -- ")
        .map(|index| redacted_index + index)
        .expect("command separator after env assignment");
    let serve_index = stderr[separator_index..]
        .find(" serve --config")
        .map(|index| separator_index + index)
        .expect("server command after separator");
    assert!(name_index < env_flag_index, "stderr: {stderr}");
    assert!(env_flag_index < separator_index, "stderr: {stderr}");
    assert!(separator_index < serve_index, "stderr: {stderr}");
}

#[test]
fn mcp_register_accepts_documented_moeresearch_bin_flag() {
    let config_path = temp_path("register-custom-bin.toml");
    let custom_bin = temp_path("custom-moeresearch");
    write_config(&config_path, BASE_CONFIG);

    let output = moe_research_command()
        .args(["mcp", "register", "--dry-run", "--config"])
        .arg(&config_path)
        .arg("--moeresearch-bin")
        .arg(&custom_bin)
        .output()
        .expect("run mcp dry-run with custom bin");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = std::fs::remove_file(&config_path);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(
        stderr.contains(&custom_bin.display().to_string()),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("moeresearch"), "stderr: {stderr}");
}

#[test]
fn mcp_register_invokes_fake_claude_with_local_scope_by_default() {
    let fake_claude = temp_path("fake-claude");
    let argv_path = temp_path("fake-claude-argv.txt");
    write_fake_claude(&fake_claude, &argv_path);

    let config_path = temp_path("register.toml");
    let env_name = format!("MOERESEARCH_TEST_REGISTER_KEY_{}", std::process::id());
    let config = BASE_CONFIG
        .replace(
            "[model.providers.openai]\nenabled = false",
            "[model.providers.openai]\nenabled = true",
        )
        .replace(
            "api_key_env = \"OPENAI_API_KEY\"",
            &format!("api_key_env = \"{env_name}\""),
        );
    write_config(&config_path, &config);
    let output = moe_research_command()
        .args(["mcp", "register", "--claude-bin"])
        .arg(&fake_claude)
        .arg("--config")
        .arg(&config_path)
        .env(&env_name, "raw-test-secret")
        .output()
        .expect("run mcp register");
    let argv = std::fs::read_to_string(&argv_path).expect("read fake argv");
    let _ = std::fs::remove_file(&fake_claude);
    let _ = std::fs::remove_file(&argv_path);
    let _ = std::fs::remove_file(&config_path);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let args = argv.lines().collect::<Vec<_>>();
    assert!(args.starts_with(&[
        "mcp",
        "add",
        "--transport",
        "stdio",
        "--scope",
        "local",
        "moeresearch",
    ]));
    let name_index = args
        .iter()
        .position(|arg| *arg == "moeresearch")
        .expect("server name");
    let env_flag_index = args.iter().position(|arg| *arg == "-e").expect("env flag");
    let separator_index = args.iter().position(|arg| *arg == "--").expect("separator");
    let env_assignment = format!("{env_name}=raw-test-secret");
    assert!(name_index < env_flag_index);
    assert!(env_flag_index < separator_index);
    assert_eq!(args[env_flag_index + 1], env_assignment.as_str());
    assert!(args[separator_index + 1..].contains(&"serve"));
    assert!(args[separator_index + 1..].contains(&"--config"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("raw-test-secret"));
}

#[test]
fn mcp_register_missing_config_does_not_invoke_claude() {
    let fake_claude = temp_path("fake-claude-missing-config");
    let argv_path = temp_path("fake-claude-missing-argv.txt");
    write_fake_claude(&fake_claude, &argv_path);
    let config_path = temp_path("missing-register.toml");

    let output = moe_research_command()
        .args(["mcp", "register", "--claude-bin"])
        .arg(&fake_claude)
        .arg("--config")
        .arg(&config_path)
        .output()
        .expect("run mcp register");
    let _ = std::fs::remove_file(&fake_claude);

    assert!(!output.status.success());
    assert!(!argv_path.exists(), "fake claude should not be invoked");
}

#[test]
fn mcp_register_enabled_missing_env_does_not_invoke_claude() {
    let fake_claude = temp_path("fake-claude-missing-env");
    let argv_path = temp_path("fake-claude-missing-env-argv.txt");
    write_fake_claude(&fake_claude, &argv_path);
    let config_path = temp_path("missing-env-register.toml");
    let missing_env = format!(
        "MOERESEARCH_TEST_REGISTER_MISSING_KEY_{}",
        std::process::id()
    );
    let config = BASE_CONFIG
        .replace(
            "[model.providers.openai]\nenabled = false",
            "[model.providers.openai]\nenabled = true",
        )
        .replace(
            "api_key_env = \"OPENAI_API_KEY\"",
            &format!("api_key_env = \"{missing_env}\""),
        );
    write_config(&config_path, &config);

    let output = moe_research_command()
        .args(["mcp", "register", "--claude-bin"])
        .arg(&fake_claude)
        .arg("--config")
        .arg(&config_path)
        .output()
        .expect("run mcp register");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = std::fs::remove_file(&fake_claude);
    let _ = std::fs::remove_file(&config_path);

    assert!(!output.status.success());
    assert!(stderr.contains(&missing_env), "stderr: {stderr}");
    assert!(!argv_path.exists(), "fake claude should not be invoked");
}

#[test]
fn mcp_register_rejects_invalid_server_name_before_invoking_claude() {
    let fake_claude = temp_path("fake-claude-invalid-name");
    let argv_path = temp_path("fake-claude-invalid-argv.txt");
    write_fake_claude(&fake_claude, &argv_path);
    let config_path = temp_path("invalid-name-register.toml");
    write_config(&config_path, BASE_CONFIG);

    let output = moe_research_command()
        .args(["mcp", "register", "--name", "bad name;rm", "--claude-bin"])
        .arg(&fake_claude)
        .arg("--config")
        .arg(&config_path)
        .output()
        .expect("run mcp register");
    let _ = std::fs::remove_file(&fake_claude);
    let _ = std::fs::remove_file(&config_path);

    assert!(!output.status.success());
    assert!(!argv_path.exists(), "fake claude should not be invoked");
}

#[test]
fn mcp_register_user_scope_requires_confirmation() {
    let config_path = temp_path("register-user.toml");

    let output = moe_research_command()
        .args(["mcp", "register", "--scope", "user", "--config"])
        .arg(&config_path)
        .output()
        .expect("run mcp register");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("--yes"), "stderr: {stderr}");
}

#[cfg(unix)]
fn write_fake_claude(path: &Path, argv_path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(
        path,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n",
            argv_path.display()
        ),
    )
    .expect("write fake claude");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .expect("chmod fake claude");
}

#[cfg(not(unix))]
fn write_fake_claude(path: &Path, argv_path: &Path) {
    std::fs::write(
        path,
        format!(
            "@echo off\r\n:loop\r\nif \"%1\"==\"\" exit /b 0\r\necho %1>>\"{}\"\r\nshift\r\ngoto loop\r\n",
            argv_path.display()
        ),
    )
    .expect("write fake claude");
}
