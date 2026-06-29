[根目录](../../CLAUDE.md) > [crates](../) > **moe-research-model**

# moe-research-model 模块文档

## 变更记录 (Changelog)

| 时间 | 变更 |
| --- | --- |
| 2026-06-29 13:22:02 | 初次扫描并生成模块级文档。 |

## 模块职责

`moe-research-model` 是模型 provider 边界。它定义统一 `ModelProvider` trait、模型请求/响应 DTO、模型 service 路由，并实现 OpenAI Responses API SSE 适配器。

## 入口与启动

- crate 入口：`src/lib.rs`
- trait：`src/provider.rs`
- service：`src/service.rs`
- OpenAI provider：`src/openai.rs`
- DTO：`src/types.rs`

`moe-research-cli` 根据配置启用 provider：

- 当前 model provider 名称：`openai`
- 默认配置示例模型：`gpt-5.5`

## 对外接口

导出内容：

- `ModelProvider`
- `ModelService`
- `OpenAiProvider`
- `ModelRequest`, `ModelResponse`
- `ModelInputItem`, `ModelMessage`, `ModelMessageRole`
- `ModelTool`, `ModelToolCall`, `ModelToolOutput`
- `ModelResponseFormat`, `JsonSchemaFormat`
- `TokenUsage`

调用模式：

1. workflow 构造 `ModelRequest`。
2. `ModelPolicy::apply_to` 注入 temperature/max_tokens 并检查 allowlist。
3. `ModelService::complete` 校验 provider 与 request schema。
4. 具体 provider 执行请求并返回标准化 `ModelResponse`。

## 关键依赖与配置

- 依赖 `moe-research-net` 发送 SSE 请求。
- OpenAI adapter 使用 bearer SSE POST 到 `{base_url}/responses`。
- `parallel_tool_calls` 固定为 `false`，由 agent loop 串行处理工具调用。
- OpenAI strict JSON schema 会递归补充 object 的 `additionalProperties = false` 与 `required`。
- provider 原始失败通过 `Error` 映射，不向公共 envelope 泄漏 body。

## 数据模型

关键 DTO：

- `ModelRequest`: provider、model、previous_response_id、input、tools、response_format、temperature、max_tokens。
- `ModelResponse`: provider、model、response_id、content、tool_calls、output_items、usage。
- `TokenUsage`: input/output/total tokens，支持 merge 和 total fallback。

校验规则：

- provider 不能为空且不能带首尾空白。
- input 不能为空。
- message content、tool call id/name、tool output call_id 不能为空。
- temperature 必须在 `[0.0, 2.0]`。
- `max_tokens = 0` 非法。

## 测试与质量

主要测试：

- `crates/moe-research-tests/tests/model_tests.rs`

覆盖点：

- provider 显式路由，不 fallback。
- policy 注入 temperature/max_tokens。
- request 校验在 provider dispatch 前执行。
- OpenAI SSE completed/failed/incomplete/error 事件处理。
- OpenAI response/tool call/usage 映射。

建议验证：

```bash
cargo test -p moe-research-tests model
```

## 常见问题 (FAQ)

- 为什么没有默认模型 fallback？provider 与 model 必须由配置或请求显式选择，避免隐式行为。
- 为什么 Responses API 使用 SSE？当前 OpenAI provider 按流式 responses 事件组装最终 response。
- 为什么 `output_items` 需要保留？agent loop 使用它构造 replayable conversation state。

## 相关文件清单

- `Cargo.toml`
- `src/lib.rs`
- `src/provider.rs`
- `src/service.rs`
- `src/types.rs`
- `src/openai.rs`
