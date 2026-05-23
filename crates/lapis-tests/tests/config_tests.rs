use std::path::{Path, PathBuf};

use lapis_core::config::loader::{load_config, load_config_from_str};
use lapis_core::net::reqwest_client::ReqwestNetworkClient;
use lapis_core::schema::config::NetworkLimits;

#[test]
fn loads_default_when_config_file_is_absent() {
    let config = load_config(Some(Path::new("missing-lapis.toml"))).expect("default config");

    assert_eq!(config.network.timeout_ms, 30_000);
    assert_eq!(config.search.enabled_count(), 0);
    assert!(config.budget.max_agents.is_unlimited());
    assert!(config.budget.max_concurrent_agents.is_unlimited());
    assert!(config.budget.max_search_calls_per_agent.is_unlimited());
    assert!(config.budget.max_tool_calls_per_agent.is_unlimited());
    assert!(config.budget.max_turns_per_agent.is_unlimited());
    assert!(config.budget.max_total_model_calls.is_unlimited());
    assert!(config.budget.max_total_search_calls.is_unlimited());
    assert!(config.budget.max_agent_timeout_ms.is_unlimited());
    assert!(config.budget.max_total_timeout_ms.is_unlimited());
}

#[test]
fn rejects_network_values_above_configured_limits() {
    let input = r#"
        [network]
        timeout_ms = 30000
        max_retries = 6
        retry_backoff_ms = 200
        user_agent = "lapis/0.1.0"

        [network.limits]
        max_timeout_ms = 30000
        max_retries = 5
        max_retry_backoff_ms = 5000
    "#;

    let err = load_config_from_str(input, PathBuf::from("lapis.toml")).unwrap_err();

    assert!(err.to_string().contains("network.max_retries exceeds"));
}

#[test]
fn rejects_provider_timeout_above_configured_limit() {
    let input = r#"
        [network]
        timeout_ms = 30000

        [network.limits]
        max_timeout_ms = 30000

        [search.providers.exa]
        timeout_ms = 30001
    "#;

    let err = load_config_from_str(input, PathBuf::from("lapis.toml")).unwrap_err();

    assert!(
        err.to_string()
            .contains("exceeds network.limits.max_timeout_ms")
    );
}

#[test]
fn rejects_budget_config_with_invalid_relative_limits() {
    let input = r#"
        [budget]
        max_agents = 2
        max_concurrent_agents = 3
    "#;

    let err = load_config_from_str(input, PathBuf::from("lapis.toml")).unwrap_err();

    assert!(
        err.to_string()
            .contains("budget.max_concurrent_agents must not exceed")
    );
}

#[test]
fn accepts_unlimited_budget_config_values() {
    let input = r#"
        [budget]
        max_agents = -1
        max_concurrent_agents = -1
        max_search_calls_per_agent = -1
        max_tool_calls_per_agent = -1
        max_turns_per_agent = -1
        max_total_model_calls = -1
        max_total_search_calls = -1
        max_agent_timeout_ms = -1
        max_total_timeout_ms = -1
    "#;

    let config = load_config_from_str(input, PathBuf::from("lapis.toml")).expect("config");

    assert!(config.budget.max_agents.is_unlimited());
    assert!(config.budget.max_concurrent_agents.is_unlimited());
    assert!(config.budget.max_search_calls_per_agent.is_unlimited());
    assert!(config.budget.max_tool_calls_per_agent.is_unlimited());
    assert!(config.budget.max_turns_per_agent.is_unlimited());
    assert!(config.budget.max_total_model_calls.is_unlimited());
    assert!(config.budget.max_total_search_calls.is_unlimited());
    assert!(config.budget.max_agent_timeout_ms.is_unlimited());
    assert!(config.budget.max_total_timeout_ms.is_unlimited());
}

#[test]
fn rejects_budget_config_values_below_minus_one() {
    let input = r#"
        [budget]
        max_agents = -2
    "#;

    let err = load_config_from_str(input, PathBuf::from("lapis.toml")).unwrap_err();

    assert!(err.to_string().contains("budget limit must be -1"));
}

#[test]
fn network_client_rejects_values_above_configured_limits() {
    let limits = NetworkLimits {
        max_timeout_ms: 10,
        max_retries: 1,
        max_retry_backoff_ms: 5,
    };

    let err = match ReqwestNetworkClient::new(10, 2, 5, limits) {
        Ok(_) => panic!("network client should reject excessive retries"),
        Err(error) => error,
    };

    assert!(err.to_string().contains("network.max_retries exceeds"));
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

#[test]
fn rejects_enabled_model_provider_without_model() {
    let input = r#"
        [model.providers.openai-compatible]
        enabled = true
        base_url = "https://api.openai.com/v1"
        api_key_env = "PATH"
    "#;

    let err = load_config_from_str(input, PathBuf::from("lapis.toml")).unwrap_err();

    assert!(
        err.to_string()
            .contains("model.providers.openai-compatible.model must be set")
    );
}

#[test]
fn rejects_enabled_grok_search_provider_without_model() {
    let input = r#"
        [search.providers.grok]
        enabled = true
        base_url = "https://api.x.ai"
        api_key_env = "PATH"
    "#;

    let err = load_config_from_str(input, PathBuf::from("lapis.toml")).unwrap_err();

    assert!(
        err.to_string()
            .contains("search.providers.grok.model must be set")
    );
}

#[test]
fn accepts_provider_model_config() {
    let input = r#"
        [search.providers.grok]
        enabled = true
        base_url = "https://api.x.ai"
        api_key_env = "PATH"
        model = "grok-4.20-fast"

        [model.providers.openai-compatible]
        enabled = true
        base_url = "https://api.openai.com/v1"
        api_key_env = "PATH"
        model = "gpt-5.5"
    "#;

    let config = load_config_from_str(input, PathBuf::from("lapis.toml")).expect("config");

    assert_eq!(
        config.search.providers["grok"].model.as_deref(),
        Some("grok-4.20-fast")
    );
    assert_eq!(
        config.model.providers["openai-compatible"].model.as_deref(),
        Some("gpt-5.5")
    );
}
