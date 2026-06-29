[根目录](../../CLAUDE.md) > [crates](../) > **moe-research-error**

# moe-research-error 模块文档

## 变更记录 (Changelog)

| 时间 | 变更 |
| --- | --- |
| 2026-06-29 13:22:02 | 初次扫描并生成模块级文档。 |

## 模块职责

`moe-research-error` 提供 transport-neutral 的统一错误 API。所有生产 crate 使用同一个 `Error` 与 `Result<T>`，再由 MCP 层映射为公共 `ToolErrorCode` 和 public-safe message。

核心目标：

- 错误分类稳定。
- provider 原始响应、host 路径、header 值、secret 不进入公共 envelope。
- schema validator 的诊断可以携带经过整理的 code/path/message。
- 支持 `retryable()` 判断，便于调用方理解重试语义。

## 入口与启动

- crate 入口：`src/lib.rs`
- 导出：`Error`, `ErrorCode`, `Result<T, E = Error>`

## 对外接口

`ErrorCode` 稳定值：

- `invalid_input`
- `unsupported_schema_version`
- `config_invalid`
- `provider_unavailable`
- `network_failed`
- `budget_exceeded`
- `tool_policy_denied`
- `schema_validation_failed`
- `timeout`
- `partial_result`
- `internal`

`Error` 提供：

- `code() -> ErrorCode`
- `retryable() -> bool`
- `public_message() -> String`

## 关键依赖与配置

- 使用 `snafu` 定义错误枚举。
- 使用 `serde_json` 与 `toml` 错误类型包装 JSON/TOML 失败。
- 不直接读取配置或执行 I/O；只表达错误。

## 数据模型

本模块无业务数据模型或持久化数据。重要约束是公共消息边界：

- 大多数错误的 `public_message()` 是稳定、脱敏的泛化文本。
- `BudgetExceeded` 允许返回预算耗尽细节。
- `SchemaValidationFailed` 允许返回 validator curated diagnostic。
- `HttpStatus` 只公开 HTTP status，不公开 body。

## 测试与质量

没有独立测试文件，主要通过以下集成测试间接覆盖：

- `mcp_tests.rs`: Error 到 ToolError 映射。
- `config_tests.rs`: ConfigInvalid / ProviderUnavailable。
- `deep_research_tests.rs`: BudgetExceeded / PartialResult。
- `network_policy_tests.rs`: 脱敏边界。

建议验证：

```bash
cargo test --workspace
```

## 常见问题 (FAQ)

- 能否把原始 provider body 放进错误 message？不能。原始 body 只能进入受控 tracing/wire trace，且必须脱敏。
- 为什么 `SchemaValidationFailed` 比其他错误公开更多信息？它只允许公开 validator 生成的安全诊断，便于调用方修正 JSON path。
- `retryable()` 是否等同于一定重试？不是，只表示同一请求在不同时间可能有不同结果。

## 相关文件清单

- `Cargo.toml`
- `src/lib.rs`
