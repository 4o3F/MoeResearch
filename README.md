# Lapis

Lapis is a Rust-based deep research MCP core service. It wraps model calls, search providers, budget enforcement, structured schema validation, and an MCP stdio server in a local process for use by Claude Code Skills or other MCP clients.

This project is not a general-purpose chatbot and does not provide a Web UI. Its primary role is to act as a configurable research orchestration backend with replaceable providers.

## Features

- **MCP stdio server**: starts with `lapis serve` and exposes research tools to MCP clients.
- **Deep research workflows**: supports both single-aspect research and multi-aspect deep research.
- **Config-driven providers**: model and search providers are enabled through `lapis.toml`.
- **No secrets in config**: API keys are read from environment variables; config files only store environment variable names.
- **Budget enforcement**: supports global research and per-agent limits for calls, search usage, timeout, and tokens.
- **Structured results**: research outputs, errors, evidence, and budget usage follow stable schemas.

## Project Layout

```text
.
├── Cargo.toml                 # Cargo workspace configuration
├── lapis.example.toml         # Example configuration
├── crates/
│   ├── lapis-cli/             # lapis CLI binary entrypoint and composition root
│   ├── lapis-config/          # TOML configuration DTOs and loader
│   ├── lapis-error/           # Transport-neutral error API
│   ├── lapis-mcp/             # MCP envelope, server, and tool adapter
│   ├── lapis-model/           # Model provider boundary and OpenAI adapter
│   ├── lapis-net/             # Network client, redaction, retry, and wire tracing
│   ├── lapis-search/          # Search provider boundary and Exa/Grok adapters
│   ├── lapis-workflow/        # Research workflow, policies, budgets, and reports
│   └── lapis-tests/           # Integration tests
├── docs/                      # Product and design documentation
├── prompts/                   # Layer 1 / Layer 2 research prompt assets
└── skills/                    # Claude Code Skill examples
```

The workspace default member is `crates/lapis-cli`, and the binary name is `lapis`.

## Requirements

- A Rust toolchain that supports Rust 2024 edition.
- An MCP client that can run stdio MCP servers.
- At least one enabled model provider. Research tasks that use search also need at least one enabled search provider.

## Installation

### 1. Clone the repository

```bash
git clone <repo-url>
cd Lapis
```

If you are already inside this repository, start from the next step.

### 2. Build the release binary

```bash
cargo build --release
```

The compiled binary is written to:

```text
target/release/lapis
```

### 3. Optional: install into Cargo's bin directory

```bash
cargo install --path crates/lapis-cli --locked
```

After installation, run:

```bash
lapis --help
```

## Configuration

Copy the example configuration:

```bash
cp lapis.example.toml lapis.toml
```

By default, `lapis` reads `lapis.toml` from the current working directory. You can also pass an explicit path with `--config`.

### Basic configuration shape

```toml
[logging]
format = "json"

[network]
timeout_ms = 30000
max_retries = 2
retry_backoff_ms = 200
user_agent = "lapis/0.1.0"

[search.providers.exa]
enabled = false
api_key_env = "EXA_API_KEY"

[search.providers.grok]
enabled = false
api_key_env = "XAI_API_KEY"
model = "grok-4.3"

[model.providers.openai]
enabled = false
api_key_env = "OPENAI_API_KEY"
model = "gpt-5.5"
```

To enable a provider:

1. Set the provider's `enabled` field to `true`.
2. Export the environment variable named by `api_key_env`.
3. For providers that require a model name, set a non-empty `model` value.

Example: enable an OpenAI-compatible model provider and the Grok search provider.

```toml
[model.providers.openai]
enabled = true
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY"
timeout_ms = 30000
model = "gpt-5.5"

[search.providers.grok]
enabled = true
base_url = "https://api.x.ai/v1"
api_key_env = "XAI_API_KEY"
timeout_ms = 30000
model = "grok-4.3"
max_output_tokens = 1024
```

Then export the required keys:

```bash
export OPENAI_API_KEY="..."
export XAI_API_KEY="..."
```

Do not write real API keys into `lapis.toml`.

## Usage

### Show help

```bash
cargo run -- --help
cargo run -- serve --help
```

Current CLI commands:

```text
lapis <COMMAND>

Commands:
  serve    Start the MCP stdio server
  help     Print help
```

`serve` options:

```text
--config <CONFIG>          Path to the configuration file
--log-format <LOG_FORMAT>  Log format: compact, pretty, json; defaults to json
```

### Start the MCP server locally

Development mode:

```bash
cargo run -- serve --config lapis.toml
```

Release binary:

```bash
./target/release/lapis serve --config lapis.toml
```

If installed with `cargo install`:

```bash
lapis serve --config lapis.toml
```

Logs are written to stderr by default, and the MCP protocol communicates over stdio.

### Configure an MCP client

Point the MCP client's server command to `lapis` and pass `serve --config <config-path>`. A generic configuration shape is:

```json
{
  "mcpServers": {
    "lapis": {
      "command": "/absolute/path/to/lapis",
      "args": ["serve", "--config", "/absolute/path/to/lapis.toml"]
    }
  }
}
```

During development, you can run through Cargo directly:

```json
{
  "mcpServers": {
    "lapis": {
      "command": "cargo",
      "args": ["run", "--", "serve", "--config", "/absolute/path/to/lapis.toml"]
    }
  }
}
```

For production or regular use, prefer a release binary so the MCP client does not trigger Cargo builds on startup.

## MCP Tools

The server currently exposes two research tools:

| Tool | Purpose |
| --- | --- |
| `aspect_research` | Runs one research aspect and returns an `AspectResearchResult`. |
| `deep_research` | Runs a multi-aspect deep research plan and returns a `DeepResearchResult`. |

Tool inputs and outputs use schemas owned by the domain crates (`lapis-workflow`, `lapis-mcp`, `lapis-model`, and `lapis-search`). The upper orchestration layer should:

- Split the user request into explicit research aspects.
- Select one model provider for each aspect.
- Select one search provider for each search-enabled aspect.
- Set a research budget that does not exceed the global limits in `lapis.toml`.
- Validate returned evidence and synthesize the final report.

The `skills/deep-research.md` file shows the recommended pattern for using a Claude Code Skill as the Layer 1 orchestration layer.

## Budget Configuration

`lapis.example.toml` defines budget limits at two levels: global research and per-agent execution.

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

- `-1` means unlimited for that dimension.
- Values other than `-1` must satisfy non-zero and upper-bound validation.
- `max_concurrent_agents` must not exceed `max_agents`.
- Production deployments should use explicit limits to avoid unbounded resource usage.

## Logging

The default log format is JSON:

```bash
lapis serve --config lapis.toml --log-format json
```

Other supported formats:

```bash
lapis serve --config lapis.toml --log-format compact
lapis serve --config lapis.toml --log-format pretty
```

Use `RUST_LOG` to adjust log levels:

```bash
RUST_LOG=lapis=debug lapis serve --config lapis.toml --log-format pretty
```

## Development

Common checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run only the CLI:

```bash
cargo run -- serve --config lapis.toml
```

Run only the integration test crate:

```bash
cargo test -p lapis-tests
```

## Troubleshooting

### `configuration file not found`

`lapis` looks for `lapis.toml` in the current working directory by default. Fix it by copying the example file or passing an explicit path:

```bash
cp lapis.example.toml lapis.toml
# Or pass an explicit path
lapis serve --config /absolute/path/to/lapis.toml
```

### `environment variable ... is not set`

Enabled providers check the environment variable named by `api_key_env`. Export the variable for each enabled provider:

```bash
export OPENAI_API_KEY="..."
export EXA_API_KEY="..."
export XAI_API_KEY="..."
```

Only variables for enabled providers are required.

### `model must not be empty`

When `model.providers.openai` or `search.providers.grok` is enabled, `model` must be a non-empty string.

### `search provider is not configured` / `model provider is not configured`

The request selected a provider that is not enabled or not configured. Check that:

- The corresponding provider has `enabled = true` in `lapis.toml`.
- The provider name in the MCP request matches the config key, such as `openai`, `exa`, or `grok`.
- The required API key environment variable has been exported.

## License

This project is licensed under the GNU Affero General Public License v3.0. See `LICENSE` for details.
