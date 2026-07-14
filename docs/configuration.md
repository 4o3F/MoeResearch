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
inactivity_timeout_ms = 120000
max_retries = 2
retry_backoff_ms = 200
user_agent = "moeresearch/0.1.0"
# Optional proxy for all outbound HTTP requests.
# HTTP/HTTPS: proxy_url = "http://proxy.example.com:8080"
# SOCKS5 (local DNS): proxy_url = "socks5://127.0.0.1:1080"
# SOCKS5h (proxy DNS): proxy_url = "socks5h://127.0.0.1:1080"

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
inactivity_timeout_ms = 120000
model = "gpt-5.5"

[search.providers.tavily]
enabled = true
base_url = "https://api.tavily.com"
api_key_env = "TAVILY_API_KEY"
inactivity_timeout_ms = 120000

[search.providers.grok]
enabled = true
base_url = "https://api.x.ai/v1"
api_key_env = "XAI_API_KEY"
inactivity_timeout_ms = 120000
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

Search provider configuration is infrastructure-only: endpoint URL, credentials, timeout, model where supported, and provider-specific response knobs. Among search providers, only `[search.providers.grok]` accepts `model`; Exa and Tavily reject it even when disabled. Grok supports optional `max_output_tokens` and `reasoning_effort`; `reasoning_effort` must be one of `none`, `low`, `medium`, or `high`. Set `none` to disable Grok reasoning, or omit the field to leave the provider default in effect. Raw provider-neutral controls belong in MCP `policy.search`; model-facing retrieval choices use the separate semantic `search.intent` protocol (`source_focus`, `timeliness`, `coverage`, `detail`). Neither belongs in `moeresearch.toml`.

Do not configure search `depth`, `content_level`, `recency`, `category`, Exa-native request fields such as `type`, `contents`, `highlights`, `text`, or `maxAgeHours`, or Tavily-native request fields such as `search_depth`, `topic`, `time_range`, `include_answer`, or `include_raw_content` under `[search.providers.*]`; unknown fields fail configuration validation.

## 5. Internal WebFetch

`web_fetch` is an optional Layer 2 internal tool, not a top-level MCP tool. Its model endpoint is independent of `[model.providers.*]` and is never selected by an aspect request.

```toml
[web_fetch]
enabled = true
cache_ttl_ms = 900000
max_cache_entries = 128
max_redirects = 5

[web_fetch.model]
provider = "openai"
base_url = "https://api.openai.com/v1"
api_key_env = "WEB_FETCH_OPENAI_API_KEY"
inactivity_timeout_ms = 120000
model = "gpt-5.5-mini"
```

The endpoint must be absolute HTTPS without URL credentials. `WebFetchService` is always constructed alongside the model and search services; when WebFetch is disabled, its prompt processor remains disabled, its key environment variable is not required, and `aspect_tools` does not advertise `web_fetch`. Page and model traffic both use the shared `moe-research-net` client, including its timeout, retry, user-agent, and proxy settings. Public-document fetches reject private/reserved targets, credentials, sensitive query parameters, and unsafe DNS answers. Only same-host redirects are followed automatically. HTML, plain text, and Markdown are supported; JavaScript rendering, login state, PDF/OCR, images, audio, and video are not.

`cache_ttl_ms = 0` disables caching and `max_redirects = 0` disables automatic same-host redirects. `max_cache_entries` must be greater than zero. Cache hits use a process-local `RwLock`, so multiple aspect agents can read cached documents concurrently. Concurrent misses for the same normalized URL are coalesced into one page fetch; different URLs are never serialized behind one network lock.

## 6. Network settings

```toml
[network]
inactivity_timeout_ms = 120000
max_retries = 2
retry_backoff_ms = 200
user_agent = "moeresearch/0.1.0"
# Optional proxy for all outbound HTTP requests.
# HTTP/HTTPS: proxy_url = "http://proxy.example.com:8080"
# SOCKS5 (local DNS): proxy_url = "socks5://127.0.0.1:1080"
# SOCKS5h (proxy DNS): proxy_url = "socks5h://127.0.0.1:1080"
```

Fields:

| Field | Meaning |
| --- | --- |
| `inactivity_timeout_ms` | Outbound network inactivity timeout in milliseconds. For SSE responses, this is the maximum gap between events, not the total stream duration. For non-SSE responses, it bounds request dispatch/response header wait and full body read wait. Generated configs use `120000` for research-safe provider calls. Must be greater than zero. |
| `max_retries` | Number of retry attempts for retryable network failures. |
| `retry_backoff_ms` | Backoff base in milliseconds. |
| `user_agent` | HTTP user-agent value. Must be non-empty and valid as a header value. |
| `proxy_url` | Optional explicit proxy URL for all outbound HTTP requests. Supported schemes are `http`, `https`, `socks5`, and `socks5h`. |

`network.inactivity_timeout_ms` is an outbound provider inactivity timeout, not a total MCP tool duration. For `aspect_research`, set the outer MCP/client timeout higher than the effective aspect deadline (`task.limits.timeout_ms` when finite) plus provider retry/backoff slack. For `deep_research`, size the outer timeout against the full research deadline (`limits.total_timeout_ms` when finite, otherwise your chosen operational cap), not just the per-aspect timeout. If the outer client aborts earlier, it may miss a later `ok` or `partial` envelope.

When `network.proxy_url` is omitted, MoeResearch preserves its existing Reqwest environment-proxy behavior. Platform system-proxy discovery depends on the Reqwest features compiled into the binary. When it is set, MoeResearch disables automatic proxy discovery and routes every outbound HTTP request through the configured proxy; `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, and `NO_PROXY` do not bypass it. `socks5://` resolves target host names locally before connecting through the proxy. `socks5h://` sends target host names to the SOCKS proxy for proxy-side DNS resolution. SOCKS support is compiled through Reqwest's `socks` feature.

A proxy URL may include authentication, but MoeResearch redacts it from `Debug` output, ordinary errors, and runtime logs. Do not place unrelated secrets in the URL.

`moeresearch assets install` retains local `file://` asset reads. Remote asset downloads load the full MoeResearch configuration and use the same timeout, retry, user-agent, proxy, and safe-error path as provider traffic. Pass `--config <PATH>` when the configuration is not the default `moeresearch.toml` in the current working directory.

## 6. Limits settings

`moeresearch.example.toml` defines operator limits at two levels: global research and per-agent execution.

```toml
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
```

Rules:

- `-1` means unlimited.
- Values other than `-1` must be positive where a runnable limit is required.
- `max_concurrent_agents` must not exceed `max_agents` when both are finite.
- Request limits passed through MCP must not exceed these configured limits.
- Production deployments should use explicit limits instead of unlimited values.

### Effective limits

MoeResearch merges operator config limits with each request using a stricter-wins rule.
Skills should treat TOML ceilings as the real maximum. Use `moeresearch check` to validate
config; inspect serve stderr for `effective_limits_applied` when debugging unexpected
`budget_exceeded` responses. See `docs/mcp-usage.md` § `budget_exceeded`.

### Listing enabled providers

```bash
moeresearch check --config moeresearch.toml --show-providers --no-mcp
```

This prints enabled model and search provider **names** only (no API keys). For request assembly against a connected MCP server, prefer `get_runtime_capabilities` with schema `0.2`: it returns live registered provider keys and operator limit ceilings for that process. CLI output remains the offline/operator/old-server fallback. Skills may only tighten tiers against ceilings; runtime stricter-wins merging remains authoritative. Neither path exposes secrets, endpoints, proxy settings, or health probes.

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

Logs are written to stderr so stdout remains available for MCP protocol messages. Startup logs include safe operator diagnostics such as binary version, OS/arch, pid/ppid, parent process name when available, config source, enabled provider names, network retry settings, and operator limits summary in `operator_limits_research` and `operator_limits_per_agent`. They do not include provider keys, environment variable values, or full parent command lines.

For schema and workflow debugging, enable debug logs for the workflow and provider layers:

```bash
RUST_LOG=moe_research_workflow=debug,moe_research_mcp=debug,moe_research_model=debug,moe_research_search=debug \
  moeresearch serve --config moeresearch.toml --log-format json
```

For network retry and non-success HTTP diagnostics without normal body logging:

```bash
RUST_LOG=moe_research_net=debug \
  moeresearch serve --config moeresearch.toml --log-format json
```

Trace-level network wire logging emits bounded body metadata with sensitive fields and string content redacted through the existing safe wire formatter. Use it only in controlled debugging sessions:

```bash
RUST_LOG=moe_research_net=trace \
  moeresearch serve --config moeresearch.toml --log-format json
```

When an aspect fails with `schema_validation_failed`, inspect stderr for `output_validation_failed` and `output_validation_issues`. These events include complete validation issue metadata plus selected and candidate evidence id lists when IDs match the generated `ev-<search_turn>-<global_candidate_index>` format. The second component is global across prior successful results, not a per-turn result position. Non-generated or sensitive-looking IDs are redacted; logs do not include raw model output, search snippets, summaries, URLs, or search queries.

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
