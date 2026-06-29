# Configuration Guide

This guide describes the MoeResearch configuration file used by `moeresearch serve`.

## 1. Configuration file location

For new users, generate a user-level starter config and follow the provider prompts:

```bash
moeresearch init --config ~/.config/moeresearch/moeresearch.toml
```

For automation, use `--non-interactive` plus explicit provider flags such as `--enable-openai`.

Then validate it:

```bash
moeresearch check --config ~/.config/moeresearch/moeresearch.toml
```

You can also copy the example configuration before running the server:

```bash
cp moeresearch.example.toml moeresearch.toml
```

By default, `moeresearch serve` reads `moeresearch.toml` from the current working directory. You can pass an explicit path:

```bash
moeresearch serve --config /absolute/path/to/moeresearch.toml
```

## 2. Secret handling

MoeResearch onboarding preserves the `api_key_env` secret model. The CLI does not accept raw provider keys and does not write keys to config.

Do not put real API keys in `moeresearch.toml`.

Provider entries store environment variable names in `api_key_env`; the server reads the corresponding environment variable at startup. `moeresearch mcp register` copies current values for enabled provider environment variables into Claude Code registration, while dry-run output redacts those values.

```toml
[model.providers.openai]
enabled = true
api_key_env = "OPENAI_API_KEY"
```

Then export the key outside the config file:

```bash
export OPENAI_API_KEY="..."
```

## 3. Basic shape

`moeresearch init` generates this same shape. Without provider flags it asks which providers to enable and which environment variable names to reference. With `--non-interactive`, all providers stay disabled unless enable flags are passed.

`moeresearch check` uses the same config validation as `moeresearch serve`: the TOML shape must be valid, and enabled providers must reference environment variables that are set. It then performs local MCP checks when requested. `moeresearch check --live` does not call provider APIs in v1; provider key correctness and endpoint reachability probes are explicitly deferred.

```toml
[logging]
format = "json"

[network]
timeout_ms = 120000
max_retries = 2
retry_backoff_ms = 200
user_agent = "moeresearch/0.1.0"

[search.providers.exa]
enabled = false
base_url = "https://api.exa.ai"
api_key_env = "EXA_API_KEY"
timeout_ms = 120000

[search.providers.tavily]
enabled = false
base_url = "https://api.tavily.com"
api_key_env = "TAVILY_API_KEY"
timeout_ms = 120000

[search.providers.grok]
enabled = false
base_url = "https://api.x.ai/v1"
api_key_env = "XAI_API_KEY"
timeout_ms = 120000
model = "grok-4.3"

[model.providers.openai]
enabled = false
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY"
timeout_ms = 120000
model = "gpt-5.5"
```

## 4. Enable providers

To enable a provider:

1. Set `enabled = true`.
2. Set `base_url` for the provider endpoint.
3. Set `api_key_env` to the environment variable name that contains the secret.
4. Set `model` only for providers that support it. Among search providers, only Grok accepts `model`.
5. Export the referenced environment variable before starting the server.

Example:

```toml
[model.providers.openai]
enabled = true
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY"
timeout_ms = 120000
model = "gpt-5.5"

[search.providers.tavily]
enabled = true
base_url = "https://api.tavily.com"
api_key_env = "TAVILY_API_KEY"
timeout_ms = 120000

[search.providers.grok]
enabled = true
base_url = "https://api.x.ai/v1"
api_key_env = "XAI_API_KEY"
timeout_ms = 120000
model = "grok-4.3"
reasoning_effort = "high"
max_output_tokens = 1024
```

```bash
export OPENAI_API_KEY="..."
export XAI_API_KEY="..."
export TAVILY_API_KEY="..."
```

Only enabled providers require their environment variables to be set.

Search provider configuration is infrastructure-only: endpoint URL, credentials, timeout, model where supported, and provider-specific response knobs. Among search providers, only `[search.providers.grok]` accepts `model`; Exa and Tavily reject it even when disabled. Grok supports optional `max_output_tokens` and `reasoning_effort`; `reasoning_effort` must be one of `none`, `low`, `medium`, or `high`. Set `none` to disable Grok reasoning, or omit the field to leave the provider default in effect. Per-query search tuning belongs in MCP request policy or the model-facing search tool call, not in `moeresearch.toml`.

Do not configure search `depth`, `content_level`, `recency`, `category`, Exa-native request fields such as `type`, `contents`, `highlights`, `text`, or `maxAgeHours`, or Tavily-native request fields such as `search_depth`, `topic`, `time_range`, `include_answer`, or `include_raw_content` under `[search.providers.*]`; unknown fields fail configuration validation.

## 5. Network settings

```toml
[network]
timeout_ms = 120000
max_retries = 2
retry_backoff_ms = 200
user_agent = "moeresearch/0.1.0"
```

Fields:

| Field | Meaning |
| --- | --- |
| `timeout_ms` | Default request timeout in milliseconds. Generated configs use `120000` for research-safe provider calls. Must be greater than zero. |
| `max_retries` | Number of retry attempts for retryable network failures. |
| `retry_backoff_ms` | Backoff base in milliseconds. |
| `user_agent` | HTTP user-agent value. Must be non-empty and valid as a header value. |

## 6. Budget settings

`moeresearch.example.toml` defines budget limits at two levels: global research and per-agent execution.

```toml
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
```

Rules:

- `-1` means unlimited.
- Values other than `-1` must be positive where a runnable budget is required.
- `max_concurrent_agents` must not exceed `max_agents` when both are finite.
- Request budgets passed through MCP must not exceed these configured limits.
- Production deployments should use explicit limits instead of unlimited values.

## 7. Logging

The default CLI log format is JSON:

```bash
moeresearch serve --config moeresearch.toml --log-format json
```

Other supported formats:

```bash
moeresearch serve --config moeresearch.toml --log-format compact
moeresearch serve --config moeresearch.toml --log-format pretty
```

Use `RUST_LOG` to adjust log levels:

```bash
RUST_LOG=moe_research=debug moeresearch serve --config moeresearch.toml --log-format pretty
```

Logs are written to stderr so stdout remains available for MCP protocol messages.

## 8. Troubleshooting

### `configuration file not found`

Fix it by copying the example file or passing an explicit path:

```bash
cp moeresearch.example.toml moeresearch.toml
moeresearch serve --config /absolute/path/to/moeresearch.toml
```

### `environment variable ... is not set`

An enabled provider references an environment variable that is not exported.

```bash
export OPENAI_API_KEY="..."
export EXA_API_KEY="..."
export XAI_API_KEY="..."
export TAVILY_API_KEY="..."
```

### `model must not be empty`

Enabled providers that require a model must set a non-empty `model` value.

### `provider is not configured`

The MCP request selected a provider that is disabled, unavailable, or not named exactly as configured.

Check:

- the provider has `enabled = true`;
- the provider name in the MCP request matches the config key;
- the required API key environment variable is exported;
- the MCP request policy allows that provider name.
