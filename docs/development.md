# Development Guide

This guide contains repository layout and local development commands for contributors.

## 1. Workspace layout

```text
.
├── Cargo.toml                 # Cargo workspace configuration
├── moeresearch.example.toml         # Example runtime configuration
├── crates/
│   ├── moe-research-cli/             # CLI binary entrypoint and composition root
│   ├── moe-research-config/          # TOML configuration DTOs and loader
│   ├── moe-research-error/           # Transport-neutral error API
│   ├── moe-research-mcp/             # MCP envelope, server, and tool adapter
│   ├── moe-research-model/           # Model provider boundary and OpenAI adapter
│   ├── moe-research-net/             # Network client, redaction, retry, and wire tracing
│   ├── moe-research-search/          # Search provider boundary and Exa/Grok/Tavily adapters
│   ├── moe-research-workflow/        # Research workflow, policies, budgets, and reports
│   └── moe-research-tests/           # Integration tests
├── docs/                      # Product and user documentation
├── prompts/                   # Prompt assets
└── skills/                    # Claude Code Skill examples
```

The workspace default member is `crates/moe-research-cli`, and the binary name is `moeresearch`.

- `crates/moe-research-cli` is the binary entrypoint and composition root.
  - Host commands live under `src/commands/` (`serve`, `init`, `check`, `onboard`, `mcp`, `assets`).
  - Config → runtime wiring lives in `src/compose.rs` (`map_limit`, budget build, provider registration, Grok effort mapping).
  - `commands/serve.rs` loads config, initializes logging, calls `compose`, then `moe_research_mcp::serve_stdio`.
  - Pure composition mapping tests live in `crates/moe-research-tests` (`cli_compose_tests.rs`), not in CLI sources.

## 2. Requirements

- Rust toolchain with Rust 2024 edition support.
- Provider API credentials for live integration runs.
- An MCP client for end-to-end MCP testing.

## 3. Build

Development build:

```bash
cargo build
```

Release build:

```bash
cargo build --release
```

Install locally through Cargo:

```bash
cargo install --path crates/moe-research-cli --locked
```

### Version display and build provenance

`moeresearch -V` prints only the package SemVer. `moeresearch --version` adds
compile-time build provenance:

```text
moeresearch 0.2.9
local version: v0.2.9-<distance>-g<short-sha>
git commit: <40-character-sha>
dirty: false
profile: debug
target: x86_64-unknown-linux-gnu
```

- `local version` comes from `git describe --tags --always` at build time.
- `git commit` is the full 40-character SHA, and `dirty` is the build-time
  working-tree state.
- Git is optional. Builds without a Git checkout or `git` on `PATH` still
  succeed and show `unknown` for Git-related fields.
- Validated one-line overrides are available through
  `MOERESEARCH_LOCAL_VERSION`, `MOERESEARCH_GIT_COMMIT`, and
  `MOERESEARCH_GIT_DIRTY`.
- Package SemVer (`CARGO_PKG_VERSION`) is unchanged and remains the default
  version for `assets install` release downloads.
- The release workflow intentionally uses `fetch-depth: 0` for binary builds;
  retain that setting after regenerating cargo-dist CI.

## 4. Run locally

Generate a local config with one model provider enabled:

```bash
cargo run -- init --config moeresearch.toml --non-interactive --enable-openai --force
export OPENAI_API_KEY="..."
```

Check config and MCP readiness:

```bash
cargo run -- check --config moeresearch.toml
```

Preview Claude Code MCP registration:

```bash
cargo run -- mcp register --config moeresearch.toml --dry-run
```

Start the MCP server manually:

```bash
cargo run -- serve --config moeresearch.toml
```

With explicit log format:

```bash
cargo run -- serve --config moeresearch.toml --log-format compact
```

## 5. Checks

Run formatting check:

```bash
cargo fmt --all -- --check
```

Run tests:

```bash
cargo test --workspace
```

Run Clippy with warnings denied:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Run only the integration test crate:

```bash
cargo test -p moe-research-tests
```

## 6. Documentation map

| Document | Purpose |
| --- | --- |
| [`configuration.md`](configuration.md) | Runtime config, providers, budgets, logging, and troubleshooting. |
| [`mcp-usage.md`](mcp-usage.md) | MCP-only client interface: JSON-RPC lifecycle, tools, requests, responses, and errors. |
| [`research-skills.md`](research-skills.md) | Research Skill asset installation, profile routing, and prompt layout. |
| [`academic-deep-research.md`](academic-deep-research.md) | Academic research profile guide. |
| [`technical-evaluation.md`](technical-evaluation.md) | Technical evaluation profile guide. |
| [`research-agent-product.md`](research-agent-product.md) | Product and architecture background. |

## 7. Safety rules

- Do not commit real API keys or local secret files.
- Prefer config-driven limits over hidden hard-coded caps.
- Keep user-facing MCP errors public-safe.
- Keep provider raw request and response bodies out of normal output.
- Run formatting, tests, and Clippy before shipping code changes.
