[根目录](../../CLAUDE.md) > [crates](../) > **moe-research-tests**

# moe-research-tests 模块文档

## 变更记录 (Changelog)

| 时间 | 变更 |
| --- | --- |
| 2026-06-29 13:22:02 | 初次扫描并生成模块级文档。 |

## 模块职责

`moe-research-tests` 是 workspace 的集成测试与回归测试 crate。项目偏好将 workflow regression tests 放在这里，而不是生产 source modules。

## 入口与启动

- crate manifest：`Cargo.toml`
- 测试目录：`tests/*.rs`
- support mock：`tests/support/*`

运行：

```bash
cargo test -p moe-research-tests
cargo test --workspace
```

## 对外接口

这是测试 crate，不对生产代码提供运行时 API。它依赖所有生产 crate 并验证跨 crate contract。

## 关键依赖与配置

依赖：

- `moe-research-config`
- `moe-research-error`
- `moe-research-mcp`
- `moe-research-model`
- `moe-research-net`
- `moe-research-search`
- `moe-research-workflow`
- `rmcp`, `tokio`, `serde_json`, `toml`, `tracing`

测试中大量使用 mock provider、mock network client、sequence model/search provider 与 support builders。

## 数据模型

本模块复用生产 DTO，并提供测试构造器：

- `tests/support/research.rs`: 构造 aspect/deep request、fake model/search service、result JSON、token usage、预算配置。
- `tests/support/network.rs`: mock network client、SSE event/response helpers。

## 测试与质量

测试文件职责：

| 文件 | 覆盖范围 |
| --- | --- |
| `cli_onboarding_tests.rs` | CLI help、init、check、onboard、MCP register dry-run。 |
| `config_tests.rs` | TOML schema、provider/env、budget、secret handling。 |
| `mcp_tests.rs` | MCP tool registry、envelope、partial/failed semantics。 |
| `deep_research_tests.rs` | deep research 并发、partial、fail_fast、budget。 |
| `model_tests.rs` | ModelService、ModelPolicy、OpenAI provider adapter。 |
| `search_tests.rs` | SearchService、SearchPolicy、Exa/Grok/Tavily provider adapter。 |
| `network_policy_tests.rs` | Debug 脱敏策略。 |
| `wire_trace_tests.rs` | 网络 wire trace 与截断/脱敏。 |
| `orchestrator_tests.rs` | agent loop、工具调用、预算、provider dispatch。 |
| `policy_validator_tests.rs` | 经由 public workflow 入口验证 policy 与 output validator 行为。 |
| `schema_tests.rs` | JSON schema、wire format、runtime metadata 不泄漏。 |

## 常见问题 (FAQ)

- 新功能测试放哪里？跨 crate、workflow、CLI、MCP contract 都优先放在本 crate。
- 可以依赖真实 API key 做测试吗？默认不应依赖；使用 mock network/provider。
- 测试是否应该验证 stdout/stderr 边界？CLI/MCP 相关改动应验证 stdout 不污染 MCP/CLI 输出，日志走 stderr。

## 相关文件清单

- `Cargo.toml`
- `tests/cli_onboarding_tests.rs`
- `tests/config_tests.rs`
- `tests/mcp_tests.rs`
- `tests/deep_research_tests.rs`
- `tests/model_tests.rs`
- `tests/search_tests.rs`
- `tests/network_policy_tests.rs`
- `tests/wire_trace_tests.rs`
- `tests/orchestrator_tests.rs`
- `tests/policy_validator_tests.rs`
- `tests/schema_tests.rs`
- `tests/support/mod.rs`
- `tests/support/research.rs`
- `tests/support/network.rs`
