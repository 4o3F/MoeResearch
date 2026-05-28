# Lapis

Lapis is a Rust-based deep research MCP core service. It exposes structured research tools over MCP stdio for Claude Code Skills and other MCP clients.

Lapis is not a general-purpose chatbot and does not provide a Web UI. Its role is to act as a local, configurable research backend with structured request and response schemas.

## Features

- **MCP stdio server**: starts with `lapis serve` and exposes research tools to MCP clients.
- **Structured research tools**: supports single-aspect and multi-aspect research workflows.
- **Config-driven providers**: model and search providers are enabled through `lapis.toml`.
- **No secrets in config**: API keys are read from environment variables referenced by config.
- **Budget enforcement**: supports research-level and per-aspect limits.
- **Stable envelopes**: tool outputs use public-safe `ToolEnvelope<T>` responses.

## Quick Start

Build the release binary:

```bash
cargo build --release
```

Copy the example configuration:

```bash
cp lapis.example.toml lapis.toml
```

Edit `lapis.toml`, enable the providers you need, and export the referenced API key environment variables.

Start the MCP server:

```bash
./target/release/lapis serve --config lapis.toml
```

Or run through Cargo during development:

```bash
cargo run -- serve --config lapis.toml
```

Logs are written to stderr. MCP protocol messages are exchanged over stdin and stdout.

## MCP Tools

The server exposes two MCP tools:

| Tool | Purpose |
| --- | --- |
| `aspect_research` | Runs one research aspect and returns an `AspectResearchResult`. |
| `deep_research` | Runs multiple research aspects and returns a `DeepResearchResult`. |

Supported MCP request `schema_version`:

```text
0.1
```

See [`docs/mcp-usage.md`](docs/mcp-usage.md) for the full MCP client interface, including JSON-RPC lifecycle messages, request payloads, response envelopes, and error formats.

## Documentation

| Document | Purpose |
| --- | --- |
| [`docs/mcp-usage.md`](docs/mcp-usage.md) | MCP-only client interface and tool schemas. |
| [`docs/configuration.md`](docs/configuration.md) | Runtime configuration, providers, budgets, logging, and troubleshooting. |
| [`docs/development.md`](docs/development.md) | Repository layout and contributor commands. |
| [`docs/research-agent-product.md`](docs/research-agent-product.md) | Product and architecture background. |

## Requirements

- Rust toolchain with Rust 2024 edition support.
- An MCP client that can run stdio MCP servers.
- At least one enabled model provider.
- A search provider for aspects that allow search.

## Common Development Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## License

This project is licensed under the GNU Affero General Public License v3.0. See `LICENSE` for details.
