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
| [`research-agent-product.md`](research-agent-product.md) | Product and architecture background. |

## 7. Safety rules

- Do not commit real API keys or local secret files.
- Prefer config-driven limits over hidden hard-coded caps.
- Keep user-facing MCP errors public-safe.
- Keep provider raw request and response bodies out of normal output.
- Run formatting, tests, and Clippy before shipping code changes.
