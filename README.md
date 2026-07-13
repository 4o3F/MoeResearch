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

Install MoeResearch from a tagged release first. `cargo-dist` is configured to generate one GitHub Release installer per supported platform family, plus updater support.

### Install

- **GitHub Releases, macOS/Linux**: download the generated shell installer from the tagged release assets, then run it locally:

  ```bash
  sh ./<downloaded-installer>.sh
  ```

- **GitHub Releases, Windows**: run the generated PowerShell installer from the release assets.

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

Install MoeResearch research skill assets:

```bash
moeresearch assets install research-skills \
  --config ~/.config/moeresearch/moeresearch.toml
```

Remote asset downloads load the full MoeResearch configuration. Pass the same `--config` path used by `serve` when it is not the default `moeresearch.toml` in the current directory; the configured network proxy, timeout, retry, and user-agent settings then apply to the download. The installed Markdown assets include PM DeepResearch, Academic DeepResearch, Technical Evaluation, common evidence prompts, and the Typst report contract. Claude Code receives one discoverable `deep-research` skill directory for the unified research entry; academic and technical profiles are used through that entry with explicit profile instructions, not as separate sibling Skill directories. Academic and Technical final reports are reviewable `typst-project-v1` source projects, not Rust-generated PDFs; compilation is an explicit caller-side Typst action. The installer downloads the release asset for the current `moeresearch` version, not GitHub `latest`. The default install target is the Claude Code user skill discovery path:

```text
~/.claude/skills/deep-research/SKILL.md
```

For a project-level Claude Code skill, use:

```bash
moeresearch assets install research-skills \
  --config ~/.config/moeresearch/moeresearch.toml \
  --client claude-code --scope project
```

For non-Claude clients or manual loading, install the same release assets with repository-style sibling directories:

```bash
moeresearch assets install research-skills \
  --config ~/.config/moeresearch/moeresearch.toml \
  --target ~/.config/moeresearch/assets \
  --layout repo
```

Generic target installs are not Claude Code discovery installs. `moeresearch mcp register` registers the backend server only; `moeresearch assets install` installs client-side research Skill and prompt assets.

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

- Release workflow packaging uploads `research-skills-assets-v{version}.tar.gz` and `research-skills-assets-v{version}.manifest.json`; the research skills asset now contains all MoeResearch research Skill Markdown roots, and the CLI verifies manifest and SHA-256 checksums before install.
- `cargo-dist` generates shell installers for macOS/Linux and a PowerShell installer for Windows.

## MCP Tools

The server exposes three MCP tools:

| Tool | Purpose |
| --- | --- |
| `get_runtime_capabilities` | Read-only live provider names and operator limit ceilings for Layer 1 request assembly. |
| `aspect_research` | Runs one research aspect and returns an `AspectResearchResult`. |
| `deep_research` | Runs multiple research aspects and returns a `DeepResearchResult`. |

Supported MCP request `schema_version`:

```text
0.2
```

See [`docs/mcp-usage.md`](docs/mcp-usage.md) for the full MCP client interface, including JSON-RPC lifecycle messages, request payloads, response envelopes, and error formats.

## Documentation

| Document | Purpose |
| --- | --- |
| [`docs/mcp-usage.md`](docs/mcp-usage.md) | MCP-only client interface and tool schemas. |
| [`docs/research-skills.md`](docs/research-skills.md) | Research Skill asset installation, layout, and profile routing. |
| [`docs/pm-deep-research.md`](docs/pm-deep-research.md) | End-to-end PM DeepResearch setup and usage guide. |
| [`docs/academic-deep-research.md`](docs/academic-deep-research.md) | Academic DeepResearch usage guide. |
| [`docs/technical-evaluation.md`](docs/technical-evaluation.md) | Technical Evaluation usage guide. |
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
