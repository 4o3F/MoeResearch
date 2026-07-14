[根目录](../../CLAUDE.md) > [crates](../) > **moe-research-mcp**

# moe-research-mcp 模块文档

## 变更记录 (Changelog)

| 时间 | 变更 |
| --- | --- |
| 2026-06-29 13:22:02 | 初次扫描并生成模块级文档。 |

## 模块职责

`moe-research-mcp` 是 MoeResearch 的 MCP 适配层。它不实现研究逻辑，而是将 MCP tool call 转换为 `moe-research-workflow` 调用，并把结果或错误包装成稳定的 `ToolEnvelope<T>`。

## 入口与启动

- crate 入口：`src/lib.rs`
- server：`src/server.rs`
- tools：`src/tools.rs`
- envelope：`src/envelope.rs`
- 启动函数：`serve_stdio(model_service, search_service, web_fetch_service, budget_config)`

CLI `serve` 会构造依赖并调用：

```rust
moe_research_mcp::serve_stdio(
    model_service,
    search_service,
    web_fetch_service,
    budget_config,
)
.await
```

## 对外接口

MCP tools：

| Tool | 请求 | 响应 |
| --- | --- | --- |
| `get_runtime_capabilities` | `RuntimeCapabilitiesRequest` | `ToolEnvelope<RuntimeCapabilities>` |
| `aspect_research` | `AspectResearchRequest` | `ToolEnvelope<AspectResearchResult>` |
| `deep_research` | `DeepResearchRequest` | `ToolEnvelope<DeepResearchResult>` |

Envelope 字段：

- `schema_version`
- `request_id`
- `run_id`
- `status`: `ok`, `partial`, `failed`
- `data`
- `error`

## 关键依赖与配置

- 依赖 `rmcp` 的 server、stdio transport 与 schema 支持。
- 依赖 `moe-research-workflow` 执行核心逻辑。
- 依赖 `moe-research-model`、`moe-research-search` 与可选 `moe-research-web-fetch` service 实例。
- `MoeResearchMcpServer` 持有 `Arc<ModelService>`、`Arc<SearchService>`、`Arc<WebFetchService>` 与 `BudgetConfig`；WebFetch service 始终存在，内部启用状态决定 capability 是否广告该工具。
- `web_fetch` 仅通过 `get_runtime_capabilities.aspect_tools` 广告，不进入顶层 MCP tool catalog。

## 数据模型

本模块的主要公共模型：

- `ToolEnvelope<T>`：所有工具响应统一外壳。
- `ToolError`：公共错误 payload；包含 canonical `aspect_id` 与 host-owned `diagnostic`（stage、可选 model/search 一基 turn 序号）。
- `ToolStatus`：`Ok`、`Partial`、`Failed`。
- `ToolErrorCode`：与 `moe-research-error::ErrorCode` 一一映射。

注意：envelope schema 明确不包含 trace payload、warnings、SSE stream 等内部细节。

## 测试与质量

主要测试：

- `crates/moe-research-tests/tests/mcp_tests.rs`
- `crates/moe-research-tests/tests/schema_tests.rs`

覆盖点：

- 公开工具包含只读 `get_runtime_capabilities`、`aspect_research`、`deep_research`；内部 `search` 不公开。
- 能力快照返回 live provider 名称与 operator limits、空列表成功且永不 partial。
- 成功、失败、partial envelope 行为。
- error code 映射和 retryable 逻辑。
- schema 不泄漏 trace/runtime metadata。

建议验证：

```bash
cargo test -p moe-research-tests mcp
cargo test -p moe-research-tests schema
```

## 常见问题 (FAQ)

- 为什么 `aspect_research` 的 `run_id` 是 `None`？单 aspect 工具没有 deep run 聚合 ID。
- 为什么 partial deep research envelope 没有 `error`？成功返回的数据中包含 `failed_aspects`，整体状态为 `partial`。
- 可以暴露 search tool 吗？不可以。搜索是 workflow 内部模型工具，不是 MCP 顶层工具。

## 相关文件清单

- `Cargo.toml`
- `src/lib.rs`
- `src/server.rs`
- `src/tools.rs`
- `src/envelope.rs`
