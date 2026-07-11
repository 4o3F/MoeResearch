[根目录](../../CLAUDE.md) > [crates](../) > **moe-research-net**

# moe-research-net 模块文档

## 变更记录 (Changelog)

| 时间 | 变更 |
| --- | --- |
| 2026-06-29 13:22:02 | 初次扫描并生成模块级文档。 |

## 模块职责

`moe-research-net` 是统一网络出口，封装 HTTP JSON 与 SSE 请求、超时、重试、header/body 校验、wire trace 和敏感信息脱敏。

## 入口与启动

- crate 入口：`src/lib.rs`
- trait：`src/client.rs`
- reqwest 实现：`src/reqwest_client.rs`
- 请求/响应类型：`src/types.rs`
- provider HTTP helper：`src/provider_http.rs`
- 脱敏工具：`src/log_safe.rs`

`moe-research-cli` 根据 `network` 配置构造：

```rust
ReqwestNetworkClient::new(timeout_ms, max_retries, retry_backoff_ms, user_agent, proxy_url)
```

## 对外接口

`NetworkClient` trait：

- `send_json(NetworkRequest) -> JsonNetworkResponse`
- `send_sse(NetworkRequest) -> SseNetworkStream`
- `ReqwestNetworkClient::send_bytes(NetworkRequest) -> Vec<u8>` for binary downloads

辅助函数：

- `bearer_json_post`
- `bearer_sse_post`
- `provider_status_retryable`

## 关键依赖与配置

- 外部依赖：`reqwest`, `eventsource-stream`, `futures`, `tokio`, `uuid`, `tracing`。
- 网络配置来自 `moeresearch.toml`：`inactivity_timeout_ms`, `max_retries`, `retry_backoff_ms`, `user_agent`, 可选 `proxy_url`。显式代理支持 HTTP/HTTPS/SOCKS5/SOCKS5h，并覆盖环境代理发现。
- JSON 请求要求 Accept JSON；SSE 请求要求 Accept `text/event-stream`。
- 非 2xx 状态通过 `HttpStatus` 映射，429 与 5xx 可重试。
- Wire trace body 有上限，超过时输出截断 marker。

## 数据模型

- `Header`: name/value，Debug 时敏感 header value 会脱敏。
- `NetworkRequest`: method、url、headers、body、timeout_ms。
- `JsonNetworkResponse`: status、headers、body。
- `SseEvent`: event、data。
- `SseNetworkStream`: 异步事件流，drop 时 abort reader task。

脱敏覆盖：

- URL user/password/query/fragment。
- Authorization、API key、token、cookie、JWT 等 header/text/json 字段。
- 嵌套 JSON 字符串。

## 测试与质量

主要测试：

- `crates/moe-research-tests/tests/network_policy_tests.rs`
- `crates/moe-research-tests/tests/wire_trace_tests.rs`
- 也被 `model_tests.rs`、`search_tests.rs` 通过 mock network 间接覆盖。

建议验证：

```bash
cargo test -p moe-research-tests network
cargo test -p moe-research-tests wire_trace
```

## 常见问题 (FAQ)

- 可以在 Debug 中打印完整 request body 吗？不可以，必须通过 `SafeJson`/`SafeText`/`SafeWireBody` 脱敏。
- SSE caps 是否来自配置？当前实现包含内部防护上限；项目偏好是新增资源策略时优先配置驱动，修改前需确认设计。
- 为什么 response body 可能是 JSON 或字符串？JSON client 会尝试解析，否则作为字符串 body 返回。

## 相关文件清单

- `Cargo.toml`
- `src/lib.rs`
- `src/client.rs`
- `src/types.rs`
- `src/provider_http.rs`
- `src/reqwest_client.rs`
- `src/log_safe.rs`
