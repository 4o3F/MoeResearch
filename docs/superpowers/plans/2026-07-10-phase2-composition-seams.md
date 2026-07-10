# Phase 2 — Composition Seams Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fence CLI composition mapping and dual-primitive seams so `serve` is a thin host path, limit/Grok mappings live in one explicit module, and MCP error codes share a single `as_str` source with `ErrorCode`—without new crates or graph changes.

**Architecture:** Keep dual domain types (`ConfigLimit` vs `Limit`, config `GrokReasoningEffort` vs search `GrokReasoningEffort`, `ErrorCode` vs `ToolErrorCode`). Extract all config→runtime wiring into `crates/moe-research-cli/src/compose.rs` as pure/`pub(crate)` functions. Make `ToolErrorCode` a thin mirror of `ErrorCode` via `From` and delegated `as_str`. Optionally rename workflow `log_safe` only if import churn is tiny; otherwise defer to Phase 4 with an explicit note.

**Tech Stack:** Rust 2024 workspace (`moe-research-*`), binary-only CLI (`moeresearch`), `cargo test -p moe-research-cli` unit tests via `#[cfg(test)]` in `compose.rs`, integration contract tests in `moe-research-tests`.

## Global Constraints

- Source of truth for findings: `docs/superpowers/specs/2026-07-10-architecture-audit-report.md` (B3, B5, B6, B7; B11 if cheap)
- Roadmap: `docs/superpowers/plans/2026-07-10-architecture-remediation-roadmap.md` Phase 2
- **Keep dual types:** `ConfigLimit` / `Limit` stay separate; **no** shared leaf crate; **no** `config → workflow` dependency
- CLI remains the **only** composition root; mapping stays CLI-local in `compose`
- Provider registration: small static `match` or table in `compose` — **not** a plugin framework
- Dual `GrokReasoningEffort`: keep both enums; mapping **only** in `compose`; do **not** move search enum into config
- Dual error codes: public MCP snake_case strings must stay **byte-identical**; extend schema/mcp tests
- Single search provider per call; no silent multi-provider fallback
- Workflow regressions stay in `crates/moe-research-tests` (CLI mapping unit tests may live in the bin crate)
- Prefer smallest fix (YAGNI / simple design); no schema 0.2 field changes
- Large structural work should use a **new git branch** (recommended for Phase 2)
- Plan task bodies are English; no placeholders

## Out of scope

- `agent_loop` / `research.rs` / `workflow.rs` splits (Phase 3)
- Assets packaging / Layer 1 skill contracts (Phase 1)
- A8 enabled-provider discovery MCP tool (Phase 4)
- `deny_unknown_fields` on report DTOs (Phase 4 / A7)
- Unifying deep vs aspect partial envelope shapes
- New model/search providers beyond existing OpenAI / Exa / Grok / Tavily wiring

## Findings closed by this plan

| ID | Summary | Closure mechanism |
| --- | --- | --- |
| B3 | Dual limit primitives | Explicit `map_limit` / `build_workflow_budget` in `compose`; dual types kept |
| B5 | CLI composition root vs product surface | `compose` module owns DI; `serve` stays thin host (logging + load + call) |
| B6 | Dual Grok effort + stringly providers | `map_grok_reasoning_effort` + static match builders only in `compose` |
| B7 | Dual `ErrorCode` enums | `From<ErrorCode> for ToolErrorCode` + single `as_str` source on `ErrorCode` |
| B11 | `log_safe` name collision | Rename workflow module **only if** ≤4 call sites and no public API break; else defer note |

---

## File map

| Path | Action | Responsibility |
| --- | --- | --- |
| `crates/moe-research-cli/src/compose.rs` | **Create** | Composition root: limit map, budget build, Grok map, network/model/search builders, enabled-provider name helpers, unit tests |
| `crates/moe-research-cli/src/main.rs` | **Modify** | `mod compose;` |
| `crates/moe-research-cli/src/commands/serve.rs` | **Modify** | Thin host: logging, load config, call `compose`, `serve_stdio` |
| `crates/moe-research-mcp/src/envelope.rs` | **Modify** | `ToolErrorCode::as_str` delegates to `ErrorCode`; add `From<ErrorCode>` |
| `crates/moe-research-mcp/src/tools.rs` | **Modify** | Replace manual `tool_error_code` match with `ErrorCode` → `ToolErrorCode` via `From` |
| `crates/moe-research-tests/tests/mcp_tests.rs` | **Modify** | Add 1:1 `ErrorCode` ↔ `ToolErrorCode` mapping / `as_str` parity tests |
| `crates/moe-research-workflow/src/log_safe.rs` | **Optional rename** | Only if Task 5 decides rename is cheap → `error_log_safe.rs` (or keep + defer note) |
| `crates/moe-research-workflow/src/lib.rs` | **Optional** | Module declaration rename if Task 5 renames |
| `crates/moe-research-workflow/src/{agent_loop,validator,tool_policy,workflow}.rs` | **Optional** | Update imports if Task 5 renames |
| `docs/development.md` | **Modify** | Document `compose` as CLI composition module |
| `crates/moe-research-cli/CLAUDE.md` | **Modify** | Mention `src/compose.rs` under entry / related files |
| `CLAUDE.md` (root) | **Modify if needed** | One-line note that CLI composition mapping lives in `compose` |

**Do not create:** new crates, `lib.rs` for CLI (binary stays binary-only), plugin registries, shared limit leaf types.

---

### Task 1: Compose module scaffold + pure limit mapping (TDD)

**Files:**
- Create: `crates/moe-research-cli/src/compose.rs`
- Modify: `crates/moe-research-cli/src/main.rs`
- Test: unit tests inside `compose.rs` (`#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `moe_research_config::{ConfigLimit, LimitsConfig}`, `moe_research_workflow::{AgentLimits, BudgetConfig, Limit, ResearchLimits}`
- Produces:
  - `pub(crate) fn map_limit<T>(limit: ConfigLimit<T>) -> Limit<T>`
  - `pub(crate) fn build_workflow_budget(config: &LimitsConfig) -> BudgetConfig`

- [ ] **Step 1: Create branch (recommended)**

```bash
cd /home/f4o3/EngineerProjects/Lapis
git checkout -b phase2/composition-seams
```

- [ ] **Step 2: Wire empty `compose` module and declare it from `main.rs`**

In `crates/moe-research-cli/src/main.rs`, add `mod compose;` next to existing module declarations:

```rust
mod commands;
mod compose;
mod onboarding;
```

Create `crates/moe-research-cli/src/compose.rs` with a minimal placeholder and failing tests first:

```rust
//! CLI composition root: pure config → runtime mapping and service wiring.
//!
//! Domain crates stay free of host wiring. All ConfigLimit / Grok / provider
//! registration mapping lives here so `commands/serve` stays a thin host.

use moe_research_config::{ConfigLimit, LimitsConfig};
use moe_research_workflow::{AgentLimits, BudgetConfig, Limit, ResearchLimits};

/// Map operator config limit into workflow budget limit (dual types intentionally kept).
#[must_use]
pub(crate) fn map_limit<T>(limit: ConfigLimit<T>) -> Limit<T> {
    match limit {
        ConfigLimit::Limited(value) => Limit::limited(value),
        ConfigLimit::Unlimited => Limit::unlimited(),
    }
}

/// Build workflow `BudgetConfig` from operator `LimitsConfig`.
#[must_use]
pub(crate) fn build_workflow_budget(config: &LimitsConfig) -> BudgetConfig {
    BudgetConfig {
        research: ResearchLimits {
            max_agents: map_limit(config.research.max_agents),
            max_concurrent_agents: map_limit(config.research.max_concurrent_agents),
            max_total_model_calls: map_limit(config.research.max_total_model_calls),
            max_total_search_calls: map_limit(config.research.max_total_search_calls),
            total_timeout_ms: map_limit(config.research.total_timeout_ms),
            max_tokens: map_limit(config.research.max_tokens),
        },
        per_agent: AgentLimits {
            max_turns: map_limit(config.per_agent.max_turns),
            max_tool_calls: map_limit(config.per_agent.max_tool_calls),
            max_search_calls: map_limit(config.per_agent.max_search_calls),
            timeout_ms: map_limit(config.per_agent.timeout_ms),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moe_research_config::{AgentLimitsConfig, ResearchLimitsConfig};

    #[test]
    fn map_limit_maps_limited_and_unlimited() {
        assert_eq!(map_limit(ConfigLimit::limited(7_usize)), Limit::limited(7));
        assert_eq!(
            map_limit::<usize>(ConfigLimit::unlimited()),
            Limit::unlimited()
        );
        assert_eq!(map_limit(ConfigLimit::limited(0_u64)), Limit::limited(0));
    }

    #[test]
    fn build_workflow_budget_maps_every_field() {
        let config = LimitsConfig {
            research: ResearchLimitsConfig {
                max_agents: ConfigLimit::limited(3),
                max_concurrent_agents: ConfigLimit::limited(2),
                max_total_model_calls: ConfigLimit::limited(10),
                max_total_search_calls: ConfigLimit::unlimited(),
                total_timeout_ms: ConfigLimit::limited(60_000),
                max_tokens: ConfigLimit::limited(100_000),
            },
            per_agent: AgentLimitsConfig {
                max_turns: ConfigLimit::limited(5),
                max_tool_calls: ConfigLimit::limited(8),
                max_search_calls: ConfigLimit::limited(4),
                timeout_ms: ConfigLimit::unlimited(),
            },
        };

        let budget = build_workflow_budget(&config);

        assert_eq!(budget.research.max_agents, Limit::limited(3));
        assert_eq!(budget.research.max_concurrent_agents, Limit::limited(2));
        assert_eq!(budget.research.max_total_model_calls, Limit::limited(10));
        assert_eq!(budget.research.max_total_search_calls, Limit::unlimited());
        assert_eq!(budget.research.total_timeout_ms, Limit::limited(60_000));
        assert_eq!(budget.research.max_tokens, Limit::limited(100_000));
        assert_eq!(budget.per_agent.max_turns, Limit::limited(5));
        assert_eq!(budget.per_agent.max_tool_calls, Limit::limited(8));
        assert_eq!(budget.per_agent.max_search_calls, Limit::limited(4));
        assert_eq!(budget.per_agent.timeout_ms, Limit::unlimited());
    }
}
```

Note: If `ConfigLimit` / `Limit` constructors differ slightly from the snippets above, match the constructors already used in `serve.rs` / workflow tests (`Limit::limited` / `Limit::unlimited` / `ConfigLimit::Limited` / `ConfigLimit::Unlimited` or their inherent helpers). Prefer the same style as current `serve.rs`.

- [ ] **Step 3: Run unit tests for the CLI binary crate**

```bash
cargo test -p moe-research-cli --lib 2>&1 | tail -20
# Binary-only crates run unit tests via:
cargo test -p moe-research-cli map_limit -- --nocapture
cargo test -p moe-research-cli build_workflow_budget -- --nocapture
```

Expected: PASS for the two tests (module compiles with `main.rs` declaring `mod compose`).

If `cargo test -p moe-research-cli` fails because of missing `[[test]]` or binary layout, use:

```bash
cargo test -p moe-research-cli --bin moeresearch map_limit
cargo test -p moe-research-cli --bin moeresearch build_workflow_budget
```

Rust allows `#[cfg(test)]` modules inside bin crates; tests are discovered when testing the bin target.

- [ ] **Step 4: Commit**

```bash
git add crates/moe-research-cli/src/compose.rs crates/moe-research-cli/src/main.rs
git commit -m "$(cat <<'EOF'
refactor(cli): extract pure limit mapping into compose module

Fence ConfigLimit→Limit and BudgetConfig mapping for Phase 2 composition seams.
EOF
)"
```

---

### Task 2: Move Grok effort mapping + provider credential helpers into compose

**Files:**
- Modify: `crates/moe-research-cli/src/compose.rs`
- Modify: `crates/moe-research-cli/src/commands/serve.rs` (only after Task 3 if preferred; Task 2 can keep serve still owning builders temporarily **or** implement helpers here ready for Task 3)
- Test: `compose.rs` `#[cfg(test)]`

**Interfaces:**
- Consumes: `moe_research_config::GrokReasoningEffort`, `moe_research_search::GrokReasoningEffort`, `std::env`, `moe_research_error::{Error, Result}`
- Produces:
  - `pub(crate) fn map_grok_reasoning_effort(effort: config::GrokReasoningEffort) -> search::GrokReasoningEffort`
  - `pub(crate) fn provider_api_key(kind: &str, name: &str, api_key_env: Option<&String>) -> Result<String>`
  - `pub(crate) fn provider_model(kind: &str, name: &str, model: Option<&String>) -> Result<String>`

- [ ] **Step 1: Write failing unit tests for Grok mapping**

Append to `compose.rs` tests:

```rust
    use moe_research_config::GrokReasoningEffort as ConfigGrokEffort;
    use moe_research_search::GrokReasoningEffort as SearchGrokEffort;

    #[test]
    fn map_grok_reasoning_effort_is_exhaustive_and_1to1() {
        let cases = [
            (ConfigGrokEffort::None, SearchGrokEffort::None),
            (ConfigGrokEffort::Low, SearchGrokEffort::Low),
            (ConfigGrokEffort::Medium, SearchGrokEffort::Medium),
            (ConfigGrokEffort::High, SearchGrokEffort::High),
        ];
        for (from, expected) in cases {
            assert_eq!(map_grok_reasoning_effort(from), expected);
            assert_eq!(from.as_str(), expected.as_str());
        }
    }
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p moe-research-cli --bin moeresearch map_grok_reasoning_effort -- --nocapture
```

Expected: FAIL with unresolved name `map_grok_reasoning_effort` (or similar).

- [ ] **Step 3: Implement Grok map + credential helpers (moved verbatim from serve)**

Add to `compose.rs` (imports as needed):

```rust
use std::env;

use moe_research_config::GrokReasoningEffort as ConfigGrokReasoningEffort;
use moe_research_error::{Error, Result};
use moe_research_search::GrokReasoningEffort as SearchGrokReasoningEffort;

#[must_use]
pub(crate) fn map_grok_reasoning_effort(
    effort: ConfigGrokReasoningEffort,
) -> SearchGrokReasoningEffort {
    match effort {
        ConfigGrokReasoningEffort::None => SearchGrokReasoningEffort::None,
        ConfigGrokReasoningEffort::Low => SearchGrokReasoningEffort::Low,
        ConfigGrokReasoningEffort::Medium => SearchGrokReasoningEffort::Medium,
        ConfigGrokReasoningEffort::High => SearchGrokReasoningEffort::High,
    }
}

pub(crate) fn provider_api_key(
    kind: &str,
    name: &str,
    api_key_env: Option<&String>,
) -> Result<String> {
    let api_key_env = api_key_env.ok_or_else(|| Error::ProviderUnavailable {
        provider: format!("{kind}:{name}"),
        message: "enabled provider must set api_key_env".to_owned(),
        retryable: false,
    })?;

    env::var(api_key_env).map_err(|_| Error::ProviderUnavailable {
        provider: format!("{kind}:{name}"),
        message: format!("environment variable {api_key_env} is not set"),
        retryable: false,
    })
}

pub(crate) fn provider_model(kind: &str, name: &str, model: Option<&String>) -> Result<String> {
    let Some(model) = model
        .as_ref()
        .map(|value| value.trim())
        .filter(|model| !model.is_empty())
    else {
        return Err(Error::ConfigInvalid {
            message: format!("{kind}.providers.{name}.model must be set"),
        });
    };
    Ok(model.to_owned())
}
```

Optional env-sensitive tests (keep light):

```rust
    #[test]
    fn provider_model_rejects_missing_and_blank() {
        assert!(provider_model("model", "openai", None).is_err());
        assert!(provider_model("model", "openai", Some(&"  ".to_owned())).is_err());
        assert_eq!(
            provider_model("model", "openai", Some(&" gpt-test ".to_owned())).unwrap(),
            "gpt-test"
        );
    }

    #[test]
    fn provider_api_key_requires_env_name() {
        let err = provider_api_key("search", "exa", None).unwrap_err();
        assert_eq!(err.code().as_str(), "provider_unavailable");
    }
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p moe-research-cli --bin moeresearch map_grok -- --nocapture
cargo test -p moe-research-cli --bin moeresearch provider_ -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/moe-research-cli/src/compose.rs
git commit -m "$(cat <<'EOF'
refactor(cli): centralize Grok effort and provider credential mapping in compose
EOF
)"
```

---

### Task 3: Move service builders + enabled-provider helpers; thin `serve.rs`

**Files:**
- Modify: `crates/moe-research-cli/src/compose.rs`
- Modify: `crates/moe-research-cli/src/commands/serve.rs`

**Interfaces:**
- Consumes: `MoeResearchConfig`, `Arc<dyn NetworkClient>`, provider types from model/search/net
- Produces:
  - `pub(crate) fn enabled_model_provider_names(config: &MoeResearchConfig) -> Vec<&str>`
  - `pub(crate) fn enabled_search_provider_names(config: &MoeResearchConfig) -> Vec<&str>`
  - `pub(crate) fn build_network_client(config: &MoeResearchConfig) -> Result<Arc<dyn NetworkClient>>`
  - `pub(crate) fn build_model_service(config: &MoeResearchConfig, network: &Arc<dyn NetworkClient>) -> Result<ModelService>`
  - `pub(crate) fn build_search_service(config: &MoeResearchConfig, network: &Arc<dyn NetworkClient>) -> Result<SearchService>`

Provider registration policy (locked): keep a **small static match** in `compose` (not plugins):

```text
model:  "openai" → OpenAiProvider
search: "exa" | "grok" | "tavily" → respective providers
unknown → Error::ConfigInvalid
```

- [ ] **Step 1: Move builders into `compose.rs` (verbatim behavior from current `serve.rs`)**

Add imports and functions. Target shape:

```rust
use std::sync::Arc;

use moe_research_config::MoeResearchConfig;
use moe_research_model::{ModelService, OpenAiProvider};
use moe_research_net::NetworkClient;
use moe_research_net::reqwest_client::ReqwestNetworkClient;
use moe_research_search::{
    ExaSearchProvider, GrokSearchProvider, SearchService, TavilySearchProvider,
};

pub(crate) fn enabled_model_provider_names(config: &MoeResearchConfig) -> Vec<&str> {
    config
        .model
        .providers
        .iter()
        .filter_map(|(name, provider)| provider.enabled.then_some(name.as_str()))
        .collect()
}

pub(crate) fn enabled_search_provider_names(config: &MoeResearchConfig) -> Vec<&str> {
    config
        .search
        .providers
        .iter()
        .filter_map(|(name, provider)| provider.enabled.then_some(name.as_str()))
        .collect()
}

pub(crate) fn build_network_client(
    config: &MoeResearchConfig,
) -> Result<Arc<dyn NetworkClient>> {
    Ok(Arc::new(ReqwestNetworkClient::new(
        config.network.inactivity_timeout_ms,
        config.network.max_retries,
        config.network.retry_backoff_ms,
        &config.network.user_agent,
    )?))
}

pub(crate) fn build_model_service(
    config: &MoeResearchConfig,
    network: &Arc<dyn NetworkClient>,
) -> Result<ModelService> {
    let mut service = ModelService::new();

    for (name, provider) in &config.model.providers {
        if !provider.enabled {
            continue;
        }

        match name.as_str() {
            "openai" => {
                let api_key = provider_api_key("model", name, provider.api_key_env.as_ref())?;
                let model = provider_model("model", name, provider.model.as_ref())?;
                service.register(OpenAiProvider::new(
                    network.clone(),
                    provider.base_url.clone(),
                    api_key,
                    provider
                        .inactivity_timeout_ms
                        .or(Some(config.network.inactivity_timeout_ms)),
                    model,
                ));
            }
            other => {
                return Err(Error::ConfigInvalid {
                    message: format!("unknown model provider `{other}`"),
                });
            }
        }
    }

    Ok(service)
}

pub(crate) fn build_search_service(
    config: &MoeResearchConfig,
    network: &Arc<dyn NetworkClient>,
) -> Result<SearchService> {
    let mut service = SearchService::new();

    for (name, provider) in &config.search.providers {
        if !provider.enabled {
            continue;
        }

        let api_key = provider_api_key("search", name, provider.api_key_env.as_ref())?;
        match name.as_str() {
            "exa" => service.register(ExaSearchProvider::new(
                network.clone(),
                provider.base_url.clone(),
                api_key,
                provider
                    .inactivity_timeout_ms
                    .or(Some(config.network.inactivity_timeout_ms)),
            )),
            "grok" => service.register(GrokSearchProvider::with_request_options(
                network.clone(),
                provider.base_url.clone(),
                api_key,
                provider
                    .inactivity_timeout_ms
                    .or(Some(config.network.inactivity_timeout_ms)),
                provider_model("search", name, provider.model.as_ref())?,
                provider.max_output_tokens,
                provider.reasoning_effort.map(map_grok_reasoning_effort),
            )),
            "tavily" => service.register(TavilySearchProvider::new(
                network.clone(),
                provider.base_url.clone(),
                api_key,
                provider
                    .inactivity_timeout_ms
                    .or(Some(config.network.inactivity_timeout_ms)),
            )),
            other => {
                return Err(Error::ConfigInvalid {
                    message: format!("unknown search provider `{other}`"),
                });
            }
        }
    }

    Ok(service)
}
```

Optional pure test for enabled-name helpers (no network):

```rust
    // Construct a minimal MoeResearchConfig only if existing config test helpers
    // or Default exist. If building a full config in unit tests is heavy, skip
    // and rely on integration onboarding/serve paths already covered by
    // moe-research-tests. Prefer not inventing a second config fixture layer.
```

Do **not** invent a heavyweight config fixture if none exists; YAGNI.

- [ ] **Step 2: Rewrite `serve.rs` as thin host**

Target `commands/serve.rs` body for composition (keep `ServeArgs`, `LogFormat`, `init_logging`, parent-pid helpers as host concerns):

```rust
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Args, ValueEnum};
use moe_research_config::load_config;
use moe_research_error::{Error, Result};
use tracing_subscriber::EnvFilter;

use crate::compose::{
    build_model_service, build_network_client, build_search_service, build_workflow_budget,
    enabled_model_provider_names, enabled_search_provider_names,
};

// ... ServeArgs + LogFormat unchanged ...

pub async fn run(args: ServeArgs) -> Result<()> {
    let log_format = args.log_format;
    let config_source = if args.config.is_some() {
        "explicit"
    } else {
        "default"
    };
    let config_filename = args
        .config
        .as_ref()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned);
    let (effective_filter, filter_source) = init_logging(log_format)?;
    let ppid = parent_pid_best_effort();
    let parent_process_name = ppid.as_deref().and_then(parent_process_name_best_effort);
    let launcher_hint = parent_process_name.as_deref().unwrap_or("unknown");

    tracing::info!(
        event = "serve_starting",
        // ... same fields as today ...
        "moeresearch serve starting"
    );

    let config = load_config(args.config.as_deref())?;
    let workflow_budget = build_workflow_budget(&config.limits);
    let enabled_model_providers = enabled_model_provider_names(&config);
    let enabled_search_providers = enabled_search_provider_names(&config);
    tracing::info!(
        event = "serve_initialized",
        // ... same fields as today, using workflow_budget + enabled_* ...
        "moeresearch initialized"
    );

    let network = build_network_client(&config)?;
    let model_service = build_model_service(&config, &network)?;
    let search_service = build_search_service(&config, &network)?;

    moe_research_mcp::serve_stdio(model_service, search_service, workflow_budget).await
}

// keep init_logging + parent_pid helpers; DELETE local map_limit / builders / Grok map
```

Remove from `serve.rs` all of: `map_limit`, `build_workflow_budget`, `map_grok_reasoning_effort`, `build_model_service`, `build_search_service`, `provider_api_key`, `provider_model`, `enabled_*`, and unused imports (`ConfigLimit`, `GrokReasoningEffort`, provider types, `BudgetConfig`, etc.).

- [ ] **Step 3: Compile + run CLI + MCP-related tests**

```bash
cargo test -p moe-research-cli --bin moeresearch
cargo test -p moe-research-tests cli_onboarding -- --nocapture
cargo test -p moe-research-tests mcp -- --nocapture
```

Expected: all PASS; no behavior change.

- [ ] **Step 4: Commit**

```bash
git add crates/moe-research-cli/src/compose.rs crates/moe-research-cli/src/commands/serve.rs
git commit -m "$(cat <<'EOF'
refactor(cli): thin serve host and own DI in compose module

Closes composition density for B5/B6; limit mapping remains dual-typed (B3).
EOF
)"
```

---

### Task 4: Single source for error-code strings (B7)

**Files:**
- Modify: `crates/moe-research-mcp/src/envelope.rs`
- Modify: `crates/moe-research-mcp/src/tools.rs`
- Modify: `crates/moe-research-tests/tests/mcp_tests.rs`
- Touch only if needed: `crates/moe-research-error/src/lib.rs` (already has `ErrorCode::as_str`; do **not** remove `ToolErrorCode` enum — keep MCP serde surface)

**Interfaces:**
- Consumes: `moe_research_error::ErrorCode`
- Produces:
  - `impl From<ErrorCode> for ToolErrorCode`
  - `ToolErrorCode::as_str` → `ErrorCode::from(self).as_str()` **or** match once then call `ErrorCode::*.as_str()` so string literals live only on `ErrorCode`
  - `tools::tool_error_code` becomes `ToolErrorCode::from(code)` (or inline `.into()`)

**Public contract freeze:** snake_case strings must remain:

```text
invalid_input
unsupported_schema_version
config_invalid
provider_unavailable
network_failed
budget_exceeded
tool_policy_denied
schema_validation_failed
timeout
partial_result
internal
```

- [ ] **Step 1: Write failing/extended contract tests first**

In `crates/moe-research-tests/tests/mcp_tests.rs`, add (near existing `tool_error_code_as_str_matches_serde`):

```rust
use moe_research_error::ErrorCode;

/// Every transport-neutral ErrorCode must map 1:1 onto ToolErrorCode with
/// identical as_str() values. Adding a new ErrorCode without ToolErrorCode
/// (or vice versa) must fail this test.
#[test]
fn tool_error_code_mirrors_error_code_1to1() {
    let pairs = [
        (ErrorCode::InvalidInput, ToolErrorCode::InvalidInput),
        (
            ErrorCode::UnsupportedSchemaVersion,
            ToolErrorCode::UnsupportedSchemaVersion,
        ),
        (ErrorCode::ConfigInvalid, ToolErrorCode::ConfigInvalid),
        (
            ErrorCode::ProviderUnavailable,
            ToolErrorCode::ProviderUnavailable,
        ),
        (ErrorCode::NetworkFailed, ToolErrorCode::NetworkFailed),
        (ErrorCode::BudgetExceeded, ToolErrorCode::BudgetExceeded),
        (ErrorCode::ToolPolicyDenied, ToolErrorCode::ToolPolicyDenied),
        (
            ErrorCode::SchemaValidationFailed,
            ToolErrorCode::SchemaValidationFailed,
        ),
        (ErrorCode::Timeout, ToolErrorCode::Timeout),
        (ErrorCode::PartialResult, ToolErrorCode::PartialResult),
        (ErrorCode::Internal, ToolErrorCode::Internal),
    ];

    assert_eq!(
        pairs.len(),
        11,
        "update this table when ErrorCode variants change"
    );

    for (domain, tool) in pairs {
        let mapped: ToolErrorCode = domain.into();
        assert_eq!(mapped, tool, "From mapping mismatch for {domain:?}");
        assert_eq!(
            domain.as_str(),
            tool.as_str(),
            "as_str mismatch for {domain:?}"
        );
        assert_eq!(
            domain.as_str(),
            serde_json::to_value(tool)
                .expect("serialize")
                .as_str()
                .expect("string"),
            "serde rename drift for {tool:?}"
        );
    }
}
```

- [ ] **Step 2: Run test — expect FAIL until `From` exists**

```bash
cargo test -p moe-research-tests tool_error_code_mirrors_error_code_1to1 -- --nocapture
```

Expected: FAIL (`From` not implemented) or compile error.

- [ ] **Step 3: Implement thin mirror in `envelope.rs`**

Replace duplicate string table with single source:

```rust
use moe_research_error::ErrorCode;

// ToolErrorCode enum variants stay as-is for JsonSchema + serde.

impl From<ErrorCode> for ToolErrorCode {
    fn from(code: ErrorCode) -> Self {
        match code {
            ErrorCode::InvalidInput => Self::InvalidInput,
            ErrorCode::UnsupportedSchemaVersion => Self::UnsupportedSchemaVersion,
            ErrorCode::ConfigInvalid => Self::ConfigInvalid,
            ErrorCode::ProviderUnavailable => Self::ProviderUnavailable,
            ErrorCode::NetworkFailed => Self::NetworkFailed,
            ErrorCode::BudgetExceeded => Self::BudgetExceeded,
            ErrorCode::ToolPolicyDenied => Self::ToolPolicyDenied,
            ErrorCode::SchemaValidationFailed => Self::SchemaValidationFailed,
            ErrorCode::Timeout => Self::Timeout,
            ErrorCode::PartialResult => Self::PartialResult,
            ErrorCode::Internal => Self::Internal,
        }
    }
}

impl ToolErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        // Single source of truth for public snake_case identifiers:
        // ErrorCode::as_str. Keep match exhaustive so a new ToolErrorCode
        // variant forces an update; strings are not re-typed here.
        match self {
            Self::InvalidInput => ErrorCode::InvalidInput.as_str(),
            Self::UnsupportedSchemaVersion => ErrorCode::UnsupportedSchemaVersion.as_str(),
            Self::ConfigInvalid => ErrorCode::ConfigInvalid.as_str(),
            Self::ProviderUnavailable => ErrorCode::ProviderUnavailable.as_str(),
            Self::NetworkFailed => ErrorCode::NetworkFailed.as_str(),
            Self::BudgetExceeded => ErrorCode::BudgetExceeded.as_str(),
            Self::ToolPolicyDenied => ErrorCode::ToolPolicyDenied.as_str(),
            Self::SchemaValidationFailed => ErrorCode::SchemaValidationFailed.as_str(),
            Self::Timeout => ErrorCode::Timeout.as_str(),
            Self::PartialResult => ErrorCode::PartialResult.as_str(),
            Self::Internal => ErrorCode::Internal.as_str(),
        }
    }
}
```

If `const fn` calling `ErrorCode::as_str` is fine (it is already `const fn` on `ErrorCode`), keep `const`. If clippy/compiler complains about non-const path, drop `const` on `ToolErrorCode::as_str` only if necessary — prefer keeping `const` because `ErrorCode::as_str` is already `const fn`.

- [ ] **Step 4: Simplify `tools.rs` mapping**

Replace:

```rust
fn tool_error_code(code: ErrorCode) -> ToolErrorCode {
    match code { /* 11 arms */ }
}
```

with:

```rust
fn tool_error_code(code: ErrorCode) -> ToolErrorCode {
    ToolErrorCode::from(code)
}
```

or inline at call site:

```rust
code: error.code().into(),
```

and delete `tool_error_code` if unused.

- [ ] **Step 5: Run MCP + schema tests**

```bash
cargo test -p moe-research-tests tool_error_code -- --nocapture
cargo test -p moe-research-tests mcp -- --nocapture
cargo test -p moe-research-tests schema -- --nocapture
cargo clippy -p moe-research-mcp --all-targets -- -D warnings
```

Expected: PASS; public strings unchanged.

- [ ] **Step 6: Commit**

```bash
git add crates/moe-research-mcp/src/envelope.rs crates/moe-research-mcp/src/tools.rs crates/moe-research-tests/tests/mcp_tests.rs
git commit -m "$(cat <<'EOF'
refactor(mcp): derive ToolErrorCode strings from ErrorCode

Keep dual enums for transport isolation; share as_str source and From mapping (B7).
EOF
)"
```

---

### Task 5: B11 `log_safe` collision — cheap rename or explicit defer

**Files (if rename):**
- Rename: `crates/moe-research-workflow/src/log_safe.rs` → `crates/moe-research-workflow/src/error_log_safe.rs`
- Modify: `crates/moe-research-workflow/src/lib.rs` (`mod error_log_safe;`)
- Modify imports in:
  - `crates/moe-research-workflow/src/agent_loop.rs`
  - `crates/moe-research-workflow/src/validator.rs`
  - `crates/moe-research-workflow/src/tool_policy.rs`
  - `crates/moe-research-workflow/src/workflow.rs`

**Do not touch** `crates/moe-research-net/src/log_safe.rs` (wire redaction; different purpose; `pub(crate)` only).

**Decision gate (run before coding):**

```bash
rg -n "log_safe" crates/moe-research-workflow --type rust
```

Expected current call sites: module decl + 4 files. That is **cheap** (≤5 edits). Proceed with rename.

If during implementation more public re-exports or test references appear, **stop** and defer:

1. Leave names as-is.
2. Append under this plan’s “Deferred” section: `B11 deferred to Phase 4 — import surface larger than expected`.

- [ ] **Step 1: Rename module file and declaration**

```bash
git mv crates/moe-research-workflow/src/log_safe.rs \
       crates/moe-research-workflow/src/error_log_safe.rs
```

In `lib.rs`:

```rust
mod error_log_safe;
```

Update imports:

```rust
// before
use crate::log_safe::error_message_for_log;
// after
use crate::error_log_safe::error_message_for_log;
```

Same for `json_error_message_for_log`, `safe_evidence_id_for_log`, `safe_model_identifier_for_log`.

- [ ] **Step 2: Verify**

```bash
cargo test -p moe-research-tests deep_research -- --nocapture
cargo test -p moe-research-tests orchestrator -- --nocapture
cargo test -p moe-research-tests policy_validator -- --nocapture
cargo clippy -p moe-research-workflow --all-targets -- -D warnings
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/moe-research-workflow
git commit -m "$(cat <<'EOF'
refactor(workflow): rename log_safe to error_log_safe

Disambiguate workflow error scrubbing from net wire redaction (B11).
EOF
)"
```

If deferred instead of renamed:

```bash
# no code commit; document in plan retrospective only
```

---

### Task 6: Documentation sync

**Files:**
- Modify: `docs/development.md`
- Modify: `crates/moe-research-cli/CLAUDE.md`
- Modify (light): root `CLAUDE.md` CLI row if it still implies all wiring lives only in `serve.rs`

- [ ] **Step 1: Update `docs/development.md` workspace layout note**

Under CLI description / layout, add an explicit bullet (English docs stay English):

```markdown
- `crates/moe-research-cli` is the binary entrypoint and composition root.
  - Host commands live under `src/commands/` (`serve`, `init`, `check`, `onboard`, `mcp`, `assets`).
  - Config → runtime wiring lives in `src/compose.rs` (`map_limit`, budget build, provider registration, Grok effort mapping).
  - `commands/serve.rs` loads config, initializes logging, calls `compose`, then `moe_research_mcp::serve_stdio`.
```

- [ ] **Step 2: Update `crates/moe-research-cli/CLAUDE.md`**

- Related files list: add `src/compose.rs`
- `serve` row: note that DI is delegated to `compose`
- Changelog row for Phase 2 date

Example related-files addition:

```markdown
- `src/compose.rs` — composition root (limit/Grok maps, network/model/search builders)
```

- [ ] **Step 3: Root `CLAUDE.md` (only if wording is wrong)**

If the CLI index line still says composition happens only in `serve`, adjust to:

```markdown
| `crates/moe-research-cli` | CLI binary, config init/check, MCP register, **composition in `src/compose.rs`**, serve host | `src/main.rs`, `src/compose.rs`, `src/commands/*` | ...
```

Do **not** rewrite the whole architecture doc.

- [ ] **Step 4: Commit**

```bash
git add docs/development.md crates/moe-research-cli/CLAUDE.md CLAUDE.md
git commit -m "$(cat <<'EOF'
docs: document CLI compose module as composition root
EOF
)"
```

---

### Task 7: Full workspace verification + self-review

**Files:** none new (verification only)

- [ ] **Step 1: Format + lint + test**

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p moe-research-tests
```

Expected: all green.

- [ ] **Step 2: Behavior smoke (no live keys required for unit paths)**

```bash
cargo run -p moe-research-cli -- --help
cargo run -p moe-research-cli -- serve --help
```

- [ ] **Step 3: Grep guardrails (no regressions of dual-map locations)**

```bash
# map_limit / build_workflow_budget should only live in compose (not serve)
rg -n "fn map_limit|fn build_workflow_budget|fn map_grok_reasoning_effort|fn build_model_service|fn build_search_service" \
  crates/moe-research-cli/src

# ToolErrorCode as_str must not re-hardcode string literals
rg -n '"invalid_input"|"budget_exceeded"' crates/moe-research-mcp/src/envelope.rs

# config still must not depend on workflow
rg -n "moe-research-workflow|moe_research_workflow" crates/moe-research-config
```

Expected:
- composition functions only under `compose.rs`
- no duplicate string literals table in MCP envelope (only via `ErrorCode::as_str`)
- zero workflow deps from config

- [ ] **Step 4: Phase retrospective note (append to this plan file after implementation)**

Append a short section:

```markdown
## Phase 2 retrospective

- Date:
- Branch:
- B3/B5/B6/B7: closed | residual notes
- B11: renamed | deferred (reason)
- Verification commands + result
```

- [ ] **Step 5: Final commit only if docs/retrospective edited**

```bash
git add docs/superpowers/plans/2026-07-10-phase2-composition-seams.md
git commit -m "$(cat <<'EOF'
docs(plan): record Phase 2 composition seams retrospective
EOF
)"
```

---

## Self-review (plan author)

### 1. Spec / finding coverage

| Finding | Task(s) | Residual risk |
| --- | --- | --- |
| B3 dual limits | Task 1, Task 3 | Dual types remain by design; footgun reduced by single map site |
| B5 CLI identity | Task 3, Task 6 | Product commands (`assets`, onboard) stay in CLI; only composition fenced |
| B6 dual Grok + stringly providers | Task 2, Task 3 | Static match remains; no plugin framework |
| B7 dual ErrorCode | Task 4 | Enums stay dual; strings single-sourced |
| B11 log_safe | Task 5 | Rename if cheap; else explicit Phase 4 defer |

Out-of-scope items (agent_loop, assets, A8, report deny_unknown_fields) intentionally have **no** tasks.

### 2. Placeholder scan

No TBD/TODO steps. Each code-bearing step includes concrete signatures or full functions. Test commands and expected outcomes specified.

### 3. Type consistency

- `map_limit: ConfigLimit<T> → Limit<T>`
- `build_workflow_budget: &LimitsConfig → BudgetConfig`
- `map_grok_reasoning_effort: config::GrokReasoningEffort → search::GrokReasoningEffort`
- `build_network_client: &MoeResearchConfig → Result<Arc<dyn NetworkClient>>`
- `build_model_service` / `build_search_service` take `&MoeResearchConfig` + `&Arc<dyn NetworkClient>`
- `From<ErrorCode> for ToolErrorCode`; `ToolErrorCode::as_str` → `ErrorCode::as_str`
- CLI tests: `#[cfg(test)]` in `compose.rs` under bin crate (no new lib target)

### 4. Hard constraints respected

- No new leaf crate / no contracts crate
- No `config → workflow`
- Mapping CLI-local in `compose`
- Public MCP error strings stable
- Dual Grok enums retained
- Schema 0.2 untouched

---

## Definition of done

Phase 2 is done when:

1. `serve.rs` no longer contains limit/Grok/provider registration mapping.
2. `compose.rs` owns those pure/DI functions with unit tests for pure maps.
3. `ToolErrorCode` maps from `ErrorCode` and shares string identifiers; mcp tests cover 1:1 parity.
4. B11 is either renamed cheaply or explicitly deferred in the plan retrospective.
5. Docs mention `compose` as composition root.
6. `cargo fmt --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` all pass.
)
