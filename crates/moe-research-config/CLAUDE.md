[根目录](../../CLAUDE.md) > [crates](../) > **moe-research-config**

# moe-research-config 模块文档

## 变更记录 (Changelog)

| 时间 | 变更 |
| --- | --- |
| 2026-06-29 13:22:02 | 初次扫描并生成模块级文档。 |

## 模块职责

`moe-research-config` 是运行时配置边界，负责将 `moeresearch.toml` 解析为强类型 DTO，并执行结构校验与启用 provider 的环境变量检查。

职责范围：

- 读取 TOML 配置文件。
- 验证网络配置、limits 配置、provider 配置结构。
- 拒绝未知 provider、未知字段、非法超时、非法环境变量名。
- 保证启用 provider 必须设置且能读取 `api_key_env` 对应环境变量。
- 用 `-1` 或 `null` 表达 unlimited limit。

## 入口与启动

- crate 入口：`src/lib.rs`
- 主要 API：`load_config(path: Option<&Path>) -> Result<MoeResearchConfig>`
- 默认文件：未传 path 时读取当前目录 `moeresearch.toml`。

## 对外接口

导出内容：

- `load_config`
- `MoeResearchConfig`
- `LoggingConfig`, `NetworkConfig`
- `ModelProviderRegistry`, `ModelProviderEndpoint`
- `SearchProviderRegistry`, `SearchProviderEndpoint`
- `LimitsConfig`, `ResearchLimitsConfig`, `AgentLimitsConfig`
- `ConfigLimit`, `CountLimit`, `DurationLimitMs`, `TokenLimit`
- `GrokReasoningEffort`
- `EnabledProviderEnv`

## 关键依赖与配置

- 依赖 `moe-research-error` 输出统一错误。
- 使用 `serde`、`schemars`、`toml`、`snafu`、`reqwest::header::HeaderValue`。
- Provider 支持规则：
  - model provider 当前只允许 `openai`，启用时必须有 `model`。
  - search provider 当前允许 `exa`、`grok`、`tavily`。
  - `grok` 启用时必须有 `model`，可配置 `reasoning_effort` 与 `max_output_tokens`。
  - `exa` 与 `tavily` 不接受 `model`、`reasoning_effort`、`max_output_tokens`。

## 数据模型

核心数据模型是配置 DTO，不包含数据库或持久化 schema。

关键规则：

- `NetworkConfig.timeout_ms` 必须大于 0。
- `NetworkConfig.user_agent` 必须非空且能作为 HTTP header value。
- `LimitsConfig.research.max_concurrent_agents` 不能超过 `max_agents`。
- `ConfigLimit<T>` 序列化为整数：`-1` 表示 unlimited，非负整数表示有限值。
- `api_key_env` 必须是合法环境变量名，启用 provider 时该变量必须存在。

## 测试与质量

主要测试：

- `crates/moe-research-tests/tests/config_tests.rs`

覆盖点：

- 缺失配置文件与缺失 section。
- 网络超时、user-agent、retry 值。
- 禁止明文 `api_key`。
- provider-specific 字段规则。
- unlimited limits wire format。
- enabled provider 缺少环境变量或 model。

建议验证：

```bash
cargo test -p moe-research-tests config
```

## 常见问题 (FAQ)

- 为什么 disabled provider 也要拒绝未知字段？配置使用 `deny_unknown_fields`，避免配置漂移与误以为生效的字段。
- 为什么 Exa/Tavily 不能配置 `model`？搜索 provider 配置只保存基础设施参数；模型类参数只属于 Grok search 或 model provider。
- 为什么 enabled provider 校验环境变量？启动时需要保证 provider 可构造，真实 API reachability 仍由后续 provider 调用决定。

## 相关文件清单

- `Cargo.toml`
- `src/lib.rs`
- `src/loader.rs`
- `src/types.rs`
- `src/limit.rs`
