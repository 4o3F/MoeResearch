use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use moe_research_config::MoeResearchConfig;
use moe_research_config::load_config;
use moe_research_error::Result;
use moe_research_net::reqwest_client::ReqwestNetworkClient;

static CONFIG_ID: AtomicUsize = AtomicUsize::new(0);

const VALID_CONFIG: &str = r#"
[logging]
format = "json"

[network]
timeout_ms = 30000
max_retries = 2
retry_backoff_ms = 200
user_agent = "moeresearch/0.1.0"

[search.providers.exa]
enabled = false
base_url = "https://api.exa.ai"
api_key_env = "EXA_API_KEY"
timeout_ms = 30000

[search.providers.tavily]
enabled = false
base_url = "https://api.tavily.com"
api_key_env = "TAVILY_API_KEY"
timeout_ms = 30000

[search.providers.grok]
enabled = false
base_url = "https://api.x.ai/v1"
api_key_env = "XAI_API_KEY"
timeout_ms = 30000
model = "grok-4.3"

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

fn load_config_from_test_str(content: &str) -> Result<MoeResearchConfig> {
    let id = CONFIG_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "moe-research-config-test-{}-{id}.toml",
        std::process::id()
    ));

    std::fs::write(&path, content).expect("write test config");
    let result = load_config(Some(&path));
    let _ = std::fs::remove_file(&path);
    result
}

#[test]
fn rejects_missing_config_file() {
    let err = load_config(Some(Path::new("missing-moeresearch.toml"))).unwrap_err();

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
    let input = VALID_CONFIG.replace("user_agent = \"moeresearch/0.1.0\"", "user_agent = \"   \"");

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(
        err.to_string()
            .contains("network.user_agent must not be empty")
    );
}

#[test]
fn rejects_invalid_network_user_agent_header_value() {
    let input = VALID_CONFIG.replace(
        "user_agent = \"moeresearch/0.1.0\"",
        "user_agent = \"moeresearch\\u0000\"",
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
        "user_agent = \"moeresearch/0.1.0\"",
        "user_agent = \"moeresearch/0.1.0\"\n\n[network.limits]\nmax_timeout_ms = 30000",
    );

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(err.to_string().contains("unknown field `limits`"));
}

#[test]
fn rejects_network_stream_knobs() {
    for field in [
        "stream_max_event_bytes",
        "stream_max_events",
        "stream_log_events",
    ] {
        let input = VALID_CONFIG.replace(
            "user_agent = \"moeresearch/0.1.0\"",
            &format!("user_agent = \"moeresearch/0.1.0\"\n{field} = 100"),
        );

        let err = load_config_from_test_str(&input).unwrap_err();

        assert!(
            err.to_string()
                .contains(&format!("unknown field `{field}`")),
            "unexpected error for {field}: {err}"
        );
    }
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
    let err = match ReqwestNetworkClient::new(0, 2, 200, "moe-research-test/0.0.0") {
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
        "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"XAI_API_KEY\"\ntimeout_ms = 30000\nmodel = \"grok-4.3\"",
        "[search.providers.grok]\nenabled = true\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"PATH\"\ntimeout_ms = 30000",
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
            "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"XAI_API_KEY\"",
            "[search.providers.grok]\nenabled = true\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"PATH\"",
        )
        .replace(
            "[model.providers.openai]\nenabled = false\nbase_url = \"https://api.openai.com/v1\"\napi_key_env = \"OPENAI_API_KEY\"",
            "[model.providers.openai]\nenabled = true\nbase_url = \"https://api.openai.com/v1\"\napi_key_env = \"PATH\"",
        );

    let config = load_config_from_test_str(&input).expect("config");

    assert_eq!(
        config.search.providers["grok"].model.as_deref(),
        Some("grok-4.3")
    );
    assert_eq!(
        config.model.providers["openai"].model.as_deref(),
        Some("gpt-5.5")
    );
}

/// Exa search provider does NOT take a `model` field; an enabled Exa stanza
/// without a model must validate cleanly. Regression guard for the
/// per-provider validation split (Commit 2 / M7).
#[test]
fn accepts_enabled_exa_search_provider_without_model() {
    let input = VALID_CONFIG.replace(
        "[search.providers.exa]\nenabled = false\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"EXA_API_KEY\"\ntimeout_ms = 30000",
        "[search.providers.exa]\nenabled = true\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"PATH\"\ntimeout_ms = 30000",
    );

    let config = load_config_from_test_str(&input).expect("Exa without model must validate");
    assert!(config.search.providers["exa"].enabled);
    assert!(config.search.providers["exa"].model.is_none());
}

#[test]
fn accepts_enabled_tavily_search_provider_without_model() {
    let input = VALID_CONFIG.replace(
        "[search.providers.tavily]\nenabled = false\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"TAVILY_API_KEY\"\ntimeout_ms = 30000",
        "[search.providers.tavily]\nenabled = true\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"PATH\"\ntimeout_ms = 30000",
    );

    let config = load_config_from_test_str(&input).expect("Tavily without model must validate");
    assert!(config.search.providers["tavily"].enabled);
    assert!(config.search.providers["tavily"].model.is_none());
}

/// Unknown model provider names must be rejected at config validation time
/// so a typo cannot reach the runtime service registry. Regression guard
/// for Commit 2 / M14.
#[test]
fn rejects_unknown_model_provider() {
    let input = VALID_CONFIG.replace("[model.providers.openai]", "[model.providers.totally-fake]");

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(
        err.to_string()
            .contains("unknown model.providers.totally-fake provider"),
        "unexpected error: {err}"
    );
}

/// Same fail-fast guarantee for unknown search provider names.
#[test]
fn rejects_unknown_search_provider() {
    let input = VALID_CONFIG.replace("[search.providers.exa]", "[search.providers.totally-fake]");

    let err = load_config_from_test_str(&input).unwrap_err();

    assert!(
        err.to_string()
            .contains("unknown search.providers.totally-fake provider"),
        "unexpected error: {err}"
    );
}

/// `ReqwestNetworkClient::new` now takes a `user_agent` string and must
/// reject any value that cannot be parsed as an HTTP header value (Commit
/// 2 / M9).
#[test]
fn network_client_rejects_invalid_user_agent() {
    let err = match ReqwestNetworkClient::new(30_000, 2, 200, "moeresearch\u{0000}") {
        Ok(_) => panic!("network client should reject invalid user_agent"),
        Err(error) => error,
    };

    assert!(
        err.to_string()
            .contains("invalid network.user_agent header"),
        "unexpected error: {err}"
    );
}

/// The removed Grok `search_context_size` knob must fail loudly instead of
/// being silently ignored, so operators notice stale xAI-incompatible config.
#[test]
fn rejects_grok_legacy_search_context_size() {
    let input = VALID_CONFIG.replace(
        "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"XAI_API_KEY\"\ntimeout_ms = 30000\nmodel = \"grok-4.3\"",
        "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"XAI_API_KEY\"\ntimeout_ms = 30000\nmodel = \"grok-4.3\"\nsearch_context_size = \"low\"",
    );

    let err = load_config_from_test_str(&input).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("unknown field"), "unexpected error: {err}");
    assert!(
        message.contains("search_context_size"),
        "unexpected error: {err}"
    );
}

/// `search.providers.grok.max_output_tokens` must be strictly greater than
/// zero when set; zero is not a meaningful response budget.
#[test]
fn rejects_grok_zero_max_output_tokens() {
    let input = VALID_CONFIG.replace(
        "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"XAI_API_KEY\"\ntimeout_ms = 30000\nmodel = \"grok-4.3\"",
        "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"XAI_API_KEY\"\ntimeout_ms = 30000\nmodel = \"grok-4.3\"\nmax_output_tokens = 0",
    );

    let err = load_config_from_test_str(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("max_output_tokens must be greater than zero"),
        "unexpected error: {err}"
    );
}

#[test]
fn accepts_grok_reasoning_effort() {
    let input = VALID_CONFIG.replace(
        "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"XAI_API_KEY\"\ntimeout_ms = 30000\nmodel = \"grok-4.3\"",
        "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"XAI_API_KEY\"\ntimeout_ms = 30000\nmodel = \"grok-4.3\"\nreasoning_effort = \"high\"",
    );

    let config = load_config_from_test_str(&input).expect("config");
    assert_eq!(
        config.search.providers["grok"]
            .reasoning_effort
            .map(|effort| effort.as_str()),
        Some("high")
    );
}

#[test]
fn rejects_grok_xhigh_reasoning_effort() {
    let input = VALID_CONFIG.replace(
        "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"XAI_API_KEY\"\ntimeout_ms = 30000\nmodel = \"grok-4.3\"",
        "[search.providers.grok]\nenabled = false\nbase_url = \"https://api.x.ai/v1\"\napi_key_env = \"XAI_API_KEY\"\ntimeout_ms = 30000\nmodel = \"grok-4.3\"\nreasoning_effort = \"xhigh\"",
    );

    let err = load_config_from_test_str(&input).unwrap_err();
    let message = err.to_string();
    assert!(
        message.contains("unknown variant"),
        "unexpected error: {err}"
    );
    assert!(message.contains("xhigh"), "unexpected error: {err}");
}

#[test]
fn rejects_exa_model() {
    let input = VALID_CONFIG.replace(
        "[search.providers.exa]\nenabled = false\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"EXA_API_KEY\"\ntimeout_ms = 30000",
        "[search.providers.exa]\nenabled = false\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"EXA_API_KEY\"\ntimeout_ms = 30000\nmodel = \"unused\"",
    );

    let err = load_config_from_test_str(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("search.providers.exa.model is not supported"),
        "unexpected error: {err}"
    );
}

#[test]
fn rejects_exa_reasoning_effort() {
    let input = VALID_CONFIG.replace(
        "[search.providers.exa]\nenabled = false\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"EXA_API_KEY\"\ntimeout_ms = 30000",
        "[search.providers.exa]\nenabled = false\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"EXA_API_KEY\"\ntimeout_ms = 30000\nreasoning_effort = \"high\"",
    );

    let err = load_config_from_test_str(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("search.providers.exa.reasoning_effort is only supported by grok"),
        "unexpected error: {err}"
    );
}

#[test]
fn rejects_exa_max_output_tokens() {
    let input = VALID_CONFIG.replace(
        "[search.providers.exa]\nenabled = false\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"EXA_API_KEY\"\ntimeout_ms = 30000",
        "[search.providers.exa]\nenabled = false\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"EXA_API_KEY\"\ntimeout_ms = 30000\nmax_output_tokens = 1024",
    );

    let err = load_config_from_test_str(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("search.providers.exa.max_output_tokens is only supported by grok"),
        "unexpected error: {err}"
    );
}

#[test]
fn rejects_tavily_model() {
    let input = VALID_CONFIG.replace(
        "[search.providers.tavily]\nenabled = false\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"TAVILY_API_KEY\"\ntimeout_ms = 30000",
        "[search.providers.tavily]\nenabled = false\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"TAVILY_API_KEY\"\ntimeout_ms = 30000\nmodel = \"unused\"",
    );

    let err = load_config_from_test_str(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("search.providers.tavily.model is not supported"),
        "unexpected error: {err}"
    );
}

#[test]
fn rejects_tavily_reasoning_effort() {
    let input = VALID_CONFIG.replace(
        "[search.providers.tavily]\nenabled = false\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"TAVILY_API_KEY\"\ntimeout_ms = 30000",
        "[search.providers.tavily]\nenabled = false\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"TAVILY_API_KEY\"\ntimeout_ms = 30000\nreasoning_effort = \"high\"",
    );

    let err = load_config_from_test_str(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("search.providers.tavily.reasoning_effort is only supported by grok"),
        "unexpected error: {err}"
    );
}

#[test]
fn rejects_tavily_max_output_tokens() {
    let input = VALID_CONFIG.replace(
        "[search.providers.tavily]\nenabled = false\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"TAVILY_API_KEY\"\ntimeout_ms = 30000",
        "[search.providers.tavily]\nenabled = false\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"TAVILY_API_KEY\"\ntimeout_ms = 30000\nmax_output_tokens = 1024",
    );

    let err = load_config_from_test_str(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("search.providers.tavily.max_output_tokens is only supported by grok"),
        "unexpected error: {err}"
    );
}

/// Omitting Grok's optional response-size cap must still validate cleanly.
#[test]
fn accepts_grok_with_no_max_output_tokens() {
    let config = load_config_from_test_str(VALID_CONFIG).expect("default config must validate");
    assert!(config.search.providers["grok"].max_output_tokens.is_none());
    assert!(config.search.providers["grok"].reasoning_effort.is_none());
}

#[test]
fn rejects_secret_like_api_key_env_value() {
    let input = VALID_CONFIG.replace(
        "api_key_env = \"EXA_API_KEY\"",
        "api_key_env = \"sk-secret-like-value\"",
    );

    let err = load_config_from_test_str(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("api_key_env must be a valid environment variable name"),
        "unexpected error: {err}"
    );
    assert!(!err.to_string().contains("sk-secret-like-value"));
}

#[test]
fn enabled_provider_requires_environment_variable() {
    let missing_env = format!("MOERESEARCH_TEST_MISSING_OPENAI_KEY_{}", std::process::id());
    let input = VALID_CONFIG.replace(
        "[model.providers.openai]\nenabled = false\nbase_url = \"https://api.openai.com/v1\"\napi_key_env = \"OPENAI_API_KEY\"",
        &format!(
            "[model.providers.openai]\nenabled = true\nbase_url = \"https://api.openai.com/v1\"\napi_key_env = \"{missing_env}\""
        ),
    );
    let id = CONFIG_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "moe-research-config-test-{}-{id}.toml",
        std::process::id()
    ));

    std::fs::write(&path, input).expect("write test config");
    let err = load_config(Some(&path)).expect_err("enabled provider should require env var");
    let _ = std::fs::remove_file(&path);

    assert!(err.to_string().contains(&missing_env));
}

#[test]
fn enabled_tavily_requires_environment_variable() {
    let missing_env = format!("MOERESEARCH_TEST_MISSING_TAVILY_KEY_{}", std::process::id());
    let input = VALID_CONFIG.replace(
        "[search.providers.tavily]\nenabled = false\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"TAVILY_API_KEY\"\ntimeout_ms = 30000",
        &format!(
            "[search.providers.tavily]\nenabled = true\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"{missing_env}\"\ntimeout_ms = 30000"
        ),
    );
    let id = CONFIG_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "moe-research-config-test-{}-{id}.toml",
        std::process::id()
    ));

    std::fs::write(&path, input).expect("write test config");
    let err = load_config(Some(&path)).expect_err("enabled Tavily should require env var");
    let _ = std::fs::remove_file(&path);

    assert!(err.to_string().contains(&missing_env));
}

#[test]
fn rejects_exa_search_tuning_in_config() {
    for field in [
        "depth = \"balanced\"",
        "content_level = \"standard\"",
        "recency = \"fresh\"",
        "category = \"news\"",
        "type = \"auto\"",
        "contents = {}",
        "maxAgeHours = 24",
    ] {
        let input = VALID_CONFIG.replace(
            "[search.providers.exa]\nenabled = false\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"EXA_API_KEY\"\ntimeout_ms = 30000",
            &format!(
                "[search.providers.exa]\nenabled = false\nbase_url = \"https://api.exa.ai\"\napi_key_env = \"EXA_API_KEY\"\ntimeout_ms = 30000\n{field}"
            ),
        );
        let err = load_config_from_test_str(&input).unwrap_err();
        assert!(
            err.to_string().contains("unknown field"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn rejects_tavily_search_tuning_in_config() {
    for field in [
        "depth = \"balanced\"",
        "content_level = \"standard\"",
        "recency = \"fresh\"",
        "category = \"news\"",
        "search_depth = \"advanced\"",
        "topic = \"news\"",
        "time_range = \"day\"",
        "include_answer = true",
        "include_raw_content = true",
    ] {
        let input = VALID_CONFIG.replace(
            "[search.providers.tavily]\nenabled = false\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"TAVILY_API_KEY\"\ntimeout_ms = 30000",
            &format!(
                "[search.providers.tavily]\nenabled = false\nbase_url = \"https://api.tavily.com\"\napi_key_env = \"TAVILY_API_KEY\"\ntimeout_ms = 30000\n{field}"
            ),
        );
        let err = load_config_from_test_str(&input).unwrap_err();
        assert!(
            err.to_string().contains("unknown field"),
            "unexpected error for {field}: {err}"
        );
    }
}

#[test]
fn rejects_grok_search_tuning_in_config() {
    let input = VALID_CONFIG.replace(
        "model = \"grok-4.3\"",
        "model = \"grok-4.3\"\ndepth = \"balanced\"",
    );

    let err = load_config_from_test_str(&input).unwrap_err();
    assert!(
        err.to_string().contains("unknown field"),
        "unexpected error: {err}"
    );
}
