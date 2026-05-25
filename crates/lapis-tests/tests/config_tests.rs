use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use lapis_core::config::loader::load_config;
use lapis_core::error::Result;
use lapis_core::net::reqwest_client::ReqwestNetworkClient;
use lapis_core::schema::config::LapisConfig;

static CONFIG_ID: AtomicUsize = AtomicUsize::new(0);

const VALID_CONFIG: &str = r#"
[logging]
format = "json"

[network]
timeout_ms = 30000
max_retries = 2
retry_backoff_ms = 200
user_agent = "lapis/0.1.0"

[search.providers.exa]
enabled = false
base_url = "https://api.exa.ai"
api_key_env = "EXA_API_KEY"
timeout_ms = 30000

[search.providers.grok]
enabled = false
base_url = "https://api.x.ai"
api_key_env = "XAI_API_KEY"
timeout_ms = 30000
model = "grok-4.20-fast"

[model.providers.openai]
enabled = false
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY"
timeout_ms = 30000
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

fn load_config_from_test_str(content: &str) -> Result<LapisConfig> {
    let id = CONFIG_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "lapis-config-test-{}-{id}.toml",
        std::process::id()
    ));

    std::fs::write(&path, content).expect("write test config");
    let result = load_config(Some(&path));
    let _ = std::fs::remove_file(&path);
    result
}

#[test]
fn rejects_missing_config_file() {
    let err = load_config(Some(Path::new("missing-lapis.toml"))).unwrap_err();

    assert!(err.to_string().contains("configuration I/O failed"));
    assert!(err.to_string().contains("configuration file not found"));
}

#[test]
fn rejects_missing_required_config_section() {
    let input = r#"
        [logging]
        format = "json"
    "#;

    let err = load_config_from_test_str(input).unwrap_err();

    assert!(err.to_string().contains("missing field `network`"));
}

#[test]
fn rejects_zero_network_timeout() {
    let input = VALID_CONFIG.replace("timeout_ms = 30000", "timeout_ms = 0");

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(
        err.to_string()
            .contains("network.timeout_ms must not be zero")
    );
}

#[test]
fn rejects_empty_network_user_agent() {
    let input = VALID_CONFIG.replace("user_agent = \"lapis/0.1.0\"", "user_agent = \"   \"");

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(
        err.to_string()
            .contains("network.user_agent must not be empty")
    );
}

#[test]
fn rejects_invalid_network_user_agent_header_value() {
    let input = VALID_CONFIG.replace(
        "user_agent = \"lapis/0.1.0\"",
        "user_agent = \"lapis\\u0000\"",
    );

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(
        err.to_string()
            .contains("network.user_agent must be a valid HTTP header value")
    );
}

#[test]
fn accepts_zero_retry_values() {
    let input = VALID_CONFIG
        .replace("max_retries = 2", "max_retries = 0")
        .replace("retry_backoff_ms = 200", "retry_backoff_ms = 0");

    load_config_from_test_str(&input).expect("zero retry values are valid");
}

#[test]
fn rejects_network_limits_section() {
    let input = VALID_CONFIG.replace(
        "user_agent = \"lapis/0.1.0\"",
        "user_agent = \"lapis/0.1.0\"\n\n[network.limits]\nmax_timeout_ms = 30000",
    );

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(err.to_string().contains("unknown field `limits`"));
}

#[test]
fn rejects_zero_provider_timeout() {
    let input = VALID_CONFIG.replace(
        "[search.providers.exa]\nenabled = false\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"EXA_API_KEY\"\ntimeout_ms = 30000",
        "[search.providers.exa]\nenabled = false\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"EXA_API_KEY\"\ntimeout_ms = 0",
    );

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(
        err.to_string()
            .contains("search.providers.exa.timeout_ms must not be zero")
    );
}

#[test]
fn rejects_budget_config_with_invalid_relative_limits() {
    let input = VALID_CONFIG
        .replace("max_agents = -1", "max_agents = 2")
        .replace("max_concurrent_agents = -1", "max_concurrent_agents = 3");

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(
        err.to_string()
            .contains("budget.research.max_concurrent_agents must not exceed")
    );
}

#[test]
fn accepts_unlimited_budget_config_values() {
    let config = load_config_from_test_str(VALID_CONFIG).expect("config");

    assert!(config.budget.research.max_agents.is_unlimited());
    assert!(config.budget.research.max_concurrent_agents.is_unlimited());
    assert!(config.budget.research.max_total_model_calls.is_unlimited());
    assert!(config.budget.research.max_total_search_calls.is_unlimited());
    assert!(config.budget.research.total_timeout_ms.is_unlimited());
    assert!(config.budget.research.max_tokens.is_unlimited());
    assert!(config.budget.per_agent.max_turns.is_unlimited());
    assert!(config.budget.per_agent.max_tool_calls.is_unlimited());
    assert!(config.budget.per_agent.max_search_calls.is_unlimited());
    assert!(config.budget.per_agent.timeout_ms.is_unlimited());
}

#[test]
fn rejects_budget_config_values_below_minus_one() {
    let input = VALID_CONFIG.replace("max_agents = -1", "max_agents = -2");

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(err.to_string().contains("budget limit must be -1"));
}

#[test]
fn network_client_rejects_zero_timeout() {
    let err = match ReqwestNetworkClient::new(0, 2, 200) {
        Ok(_) => panic!("network client should reject zero timeout"),
        Err(error) => error,
    };

    assert!(
        err.to_string()
            .contains("network.timeout_ms must not be zero")
    );
}

#[test]
fn rejects_plain_api_key_field() {
    let input = VALID_CONFIG.replace(
        "api_key_env = \"EXA_API_KEY\"",
        "api_key_env = \"EXA_API_KEY\"\napi_key = \"secret\"",
    );

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(err.to_string().contains("unknown field `api_key`"));
}

#[test]
fn rejects_enabled_model_provider_without_model() {
    let input = VALID_CONFIG.replace(
        "[model.providers.openai]\nenabled = false\nbase_url = \"https://api.openai.com/v1\"\napi_key_env = \"OPENAI_API_KEY\"\ntimeout_ms = 30000\nmodel = \"gpt-5.5\"",
        "[model.providers.openai]\nenabled = true\nbase_url = \"https://api.openai.com/v1\"\napi_key_env = \"PATH\"\ntimeout_ms = 30000",
    );

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(
        err.to_string()
            .contains("model.providers.openai.model must be set")
    );
}

#[test]
fn rejects_enabled_grok_search_provider_without_model() {
    let input = VALID_CONFIG.replace(
        "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai\"\napi_key_env = \"XAI_API_KEY\"\ntimeout_ms = 30000\nmodel = \"grok-4.20-fast\"",
        "[search.providers.grok]\nenabled = true\nbase_url = \"https://api.x.ai\"\napi_key_env = \"PATH\"\ntimeout_ms = 30000",
    );

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(
        err.to_string()
            .contains("search.providers.grok.model must be set")
    );
}

#[test]
fn accepts_provider_model_config() {
    let input = VALID_CONFIG
        .replace(
            "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai\"\napi_key_env = \"XAI_API_KEY\"",
            "[search.providers.grok]\nenabled = true\nbase_url = \"https://api.x.ai\"\napi_key_env = \"PATH\"",
        )
        .replace(
            "[model.providers.openai]\nenabled = false\nbase_url = \"https://api.openai.com/v1\"\napi_key_env = \"OPENAI_API_KEY\"",
            "[model.providers.openai]\nenabled = true\nbase_url = \"https://api.openai.com/v1\"\napi_key_env = \"PATH\"",
        );

    let config = load_config_from_test_str(&input).expect("config");

    assert_eq!(
        config.search.providers["grok"].model.as_deref(),
        Some("grok-4.20-fast")
    );
    assert_eq!(
        config.model.providers["openai"].model.as_deref(),
        Some("gpt-5.5")
    );
}
