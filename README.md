# MoeResearch

MoeResearch is a Rust-based deep research MCP core service. It exposes structured research tools over MCP stdio for Claude Code Skills and other MCP clients.

MoeResearch is not a general-purpose chatbot and does not provide a Web UI. Its role is to act as a local, configurable research backend with structured request and response schemas.

## Features

- **MCP stdio server**: starts with `moeresearch serve` and exposes research tools to MCP clients.
- **Structured research tools**: supports single-aspect and multi-aspect research workflows.
- **Config-driven providers**: model and search providers are enabled through `moeresearch.toml`.
- **No secrets in config**: API keys are read from environment variables referenced by config.
- **Budget enforcement**: supports research-level and per-aspect limits.
- **Stable envelopes**: tool outputs use public-safe `ToolEnvelope<T>` responses.

## Quick Start

Install MoeResearch from a tagged release first. `cargo-dist` is configured to generate GitHub Release installers, Homebrew and npm publishing metadata, MSI packages, and updater support.

### Install

- **GitHub Releases, macOS/Linux**: download the generated shell installer from the tagged release assets, then run it locally:

  ```bash
  sh ./<downloaded-installer>.sh
  ```

- **GitHub Releases, Windows**: run the generated PowerShell installer from the release assets, or download and run the generated `.msi` package.
- **Homebrew**: download the generated `moeresearch.rb` formula from the tagged release assets, then install it locally:

  ```bash
  brew install ./moeresearch.rb
  ```

  Automatic Homebrew tap publishing is not configured, so no separate tap repository is required.

- **npm**: once the package is published, install the configured CLI package globally:

  ```bash
  npm install -g @4o3f/moeresearch
  ```

  `cargo-dist` also prints a version-pinned npm install hint in release notes for project-local installs.

- **Source fallback**: build locally when release artifacts are not available or when developing:

  ```bash
  cargo build --release
  ./target/release/moeresearch --help
  ```

If you built from source, replace `moeresearch` in the commands below with `./target/release/moeresearch`.

Create a starter configuration and follow the provider prompts:

```bash
moeresearch init --config ~/.config/moeresearch/moeresearch.toml
```

For scripted setup, pass provider flags such as `--enable-openai` with `--non-interactive`. MoeResearch keeps secrets out of config; provider entries use `api_key_env`.

```bash
export OPENAI_API_KEY="..."
export XAI_API_KEY="..."
export EXA_API_KEY="..."
export TAVILY_API_KEY="..."
```

Check local readiness:

```bash
moeresearch check --config ~/.config/moeresearch/moeresearch.toml
```

`moeresearch check --live` reports provider reachability probes as deferred in v1; it does not validate real API key correctness.

Register MoeResearch with Claude Code:

```bash
moeresearch mcp register --scope local --config ~/.config/moeresearch/moeresearch.toml
```

By default, registration records the current `moeresearch` executable path. Pass `--moeresearch-bin` only when Claude Code should launch a different binary. Registration validates enabled-provider environment variables before invoking `claude` and forwards their current values into Claude Code registration with redacted dry-run output.

Start the MCP server manually when needed:

```bash
moeresearch serve --config ~/.config/moeresearch/moeresearch.toml
```

Or run through Cargo during development:

```bash
cargo run -- onboard --config ~/.config/moeresearch/moeresearch.toml --dry-run
cargo run -- serve --config ~/.config/moeresearch/moeresearch.toml
```

Logs are written to stderr. MCP protocol messages are exchanged over stdin and stdout.

### Maintainer release prerequisites

Before creating a release tag:

- Ensure the npm scope/package for `@4o3f/moeresearch` is available to publish.
- Configure `NPM_TOKEN` as a GitHub secret with publish access to `@4o3f/moeresearch`.
- Homebrew formula generation is enabled, but automatic tap publishing is intentionally disabled to avoid requiring a separate tap repository.
- `cargo-dist` generates the MSI package with a license sidecar. Windows code signing is a later hardening step unless configured separately.

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
| [`docs/pm-deep-research.md`](docs/pm-deep-research.md) | End-to-end PM DeepResearch setup and usage guide. |
| [`docs/configuration.md`](docs/configuration.md) | Runtime configuration, providers, budgets, logging, and troubleshooting. |
| [`docs/development.md`](docs/development.md) | Repository layout and contributor commands. |
| [`docs/research-agent-product.md`](docs/research-agent-product.md) | Product and architecture background. |

## Requirements

- An MCP client that can run stdio MCP servers.
- At least one enabled model provider.
- A search provider for aspects that allow search.
- Rust toolchain with Rust 2024 edition support when building from source or developing.

## Common Development Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## License

This project is licensed under the GNU Affero General Public License v3.0. See `LICENSE` for details.
