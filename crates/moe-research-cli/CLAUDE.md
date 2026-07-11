[根目录](../../CLAUDE.md) > [crates](../) > **moe-research-cli**

# moe-research-cli 模块文档

## 变更记录 (Changelog)

| 时间 | 变更 |
| --- | --- |
| 2026-07-10 | Phase 2：抽出 `src/compose.rs` 作为 composition root；`serve` 瘦身为 host；测试迁入 `moe-research-tests`。 |
| 2026-06-29 13:22:02 | 初次扫描并生成模块级文档。 |

## 模块职责

`moe-research-cli` 是 workspace 默认成员与唯一可发布二进制 `moeresearch` 的入口。它承担 composition root 职责：解析 CLI 命令、加载配置、初始化日志，经 `src/compose.rs` 构造网络/模型/搜索/workflow 服务，并启动 MCP stdio server。

crate 以 **bin + lib** 组织（`publish = false`）：`src/lib.rs` 暴露 `commands` / `compose` / `onboarding`，便于 `moe-research-tests` 覆盖 pure mapping；生产仍通过二进制 `moeresearch` 交付。

主要能力：

- `serve`: 启动 MCP stdio server（host：logging + load config + 调用 `compose` + `serve_stdio`）。
- `init`: 生成 `moeresearch.toml`，支持交互式或非交互式 provider 开关。
- `check`: 校验配置、启用 provider 的环境变量、本地 MCP smoke check。
- `onboard`: 组合 init、check、Claude Code MCP 注册引导。
- `mcp register`: 调用 `claude mcp add` 注册 stdio MCP server。
- `assets`: 安装 research-skills 等 Markdown 资产包；远程下载通过完整配置和统一网络 client。

## 入口与启动

- 库入口：`src/lib.rs`（composition / commands / onboarding）
- 二进制入口：`src/main.rs`
- Cargo bin：`[[bin]] name = "moeresearch"`, path = `src/main.rs`
- 默认 workspace member：根 `Cargo.toml` 的 `default-members = ["crates/moe-research-cli"]`

常用开发启动：

```bash
cargo run -- serve --config moeresearch.toml
cargo run -- init --config moeresearch.toml --non-interactive --enable-openai --force
cargo run -- check --config moeresearch.toml
cargo run -- mcp register --config moeresearch.toml --dry-run
cargo run -- onboard --config moeresearch.toml --dry-run
```

## 对外接口

CLI 命令由 `clap` 派生：

| 命令 | 文件 | 说明 |
| --- | --- | --- |
| `serve` | `src/commands/serve.rs` | Thin host：加载配置、初始化 tracing，委托 `src/compose.rs` 完成 budget/provider DI，再调用 `moe_research_mcp::serve_stdio`。 |
| `init` | `src/commands/init.rs` | 生成配置文件；不写明文 API key，只写 `api_key_env`。 |
| `check` | `src/commands/check.rs` | 配置校验、provider 环境变量检查、可选 live 提示、本地 MCP smoke check。 |
| `onboard` | `src/commands/onboard.rs` | 配置创建、检查、注册指引的一站式流程。 |
| `mcp register` | `src/commands/mcp.rs` | 生成或执行 `claude mcp add --transport stdio` 命令。 |
| `assets` | `src/commands/assets.rs` | 下载/安装 research-skills 等资产包。 |

## 关键依赖与配置

- 内部依赖：`moe-research-config`, `moe-research-error`, `moe-research-mcp`, `moe-research-model`, `moe-research-net`, `moe-research-search`, `moe-research-workflow`。
- 外部依赖：`clap`, `inquire`, `tokio`, `tracing`, `tracing-subscriber`, `serde_json`。
- 日志：普通 CLI 命令使用 stderr；`serve` 支持 `compact`、`pretty`、`json` 三种格式，默认 JSON。
- 配置默认路径：`$XDG_CONFIG_HOME/moeresearch/moeresearch.toml`，否则 `$HOME/.config/moeresearch/moeresearch.toml`，最后回退 `moeresearch.toml`。

## 数据模型

本模块不定义持久化数据模型；核心 DTO 来自 `moe-research-config` 和 `moe-research-workflow`。本模块内有 onboarding 辅助模型：

- `ConfigPlan` / `ProviderPlan`: 生成 starter TOML。
- `McpScope`: `local`、`project`、`user`。
- `McpEnvVar`: 注册 MCP server 时转发启用 provider 的环境变量值，dry-run 输出会脱敏。

## 测试与质量

相关测试（均在 `moe-research-tests`，不在本 crate 源码内嵌 `#[cfg(test)]`）：

- `crates/moe-research-tests/tests/cli_compose_tests.rs`: pure limit / Grok / credential mapping。
- `crates/moe-research-tests/tests/cli_onboarding_tests.rs`: CLI help、init、check、dry-run、配置生成、注册命令等。
- `crates/moe-research-tests/tests/cli_assets_tests.rs`: assets 打包/安装契约。
- `crates/moe-research-tests/tests/mcp_tests.rs`: 通过构造 server 验证 MCP 工具与 envelope 行为。

建议验证：

```bash
cargo test -p moe-research-tests --test cli_compose_tests
cargo test -p moe-research-tests cli_onboarding
cargo test -p moe-research-tests mcp
cargo clippy --workspace --all-targets -- -D warnings
```

## 常见问题 (FAQ)

- 为什么日志不能写 stdout？MCP stdio 协议消息占用 stdin/stdout，日志必须走 stderr。
- 为什么 config 不保存 key？项目约束要求 secrets 只从环境变量注入，配置只保存环境变量名。
- `check --live` 会验证真实 provider key 吗？当前版本仅提示 provider reachability probe deferred，不验证真实 key 正确性。
- `project` / `user` scope 为什么要 `--yes`？这些 scope 会写共享或用户级 Claude Code 配置，需要显式确认。

## 相关文件清单

- `Cargo.toml`
- `src/lib.rs`
- `src/main.rs`
- `src/compose.rs` — composition root（limit/Grok maps、network/model/search builders）
- `src/commands/mod.rs`
- `src/commands/serve.rs`
- `src/commands/init.rs`
- `src/commands/check.rs`
- `src/commands/onboard.rs`
- `src/commands/mcp.rs`
- `src/commands/assets.rs`
- `src/onboarding/config.rs`
- `src/onboarding/claude.rs`
- `src/onboarding/output.rs`
- `src/onboarding/prompt.rs`
- `src/onboarding/mod.rs`
