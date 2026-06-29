[根目录](../../CLAUDE.md) > [crates](../) > **moe-research-search**

# moe-research-search 模块文档

## 变更记录 (Changelog)

| 时间 | 变更 |
| --- | --- |
| 2026-06-29 13:22:02 | 初次扫描并生成模块级文档。 |

## 模块职责

`moe-research-search` 是搜索 provider 边界。它定义统一 `SearchProvider` trait、标准化搜索请求/响应 DTO、搜索 service 路由，并实现 Exa、Grok、Tavily 三个 provider。

## 入口与启动

- crate 入口：`src/lib.rs`
- trait 与 provider re-export：`src/provider.rs`
- service：`src/service.rs`
- DTO：`src/types.rs`
- concrete provider：`src/provider/exa.rs`, `src/provider/grok.rs`, `src/provider/tavily.rs`

`moe-research-cli` 根据配置注册启用的 search provider。

## 对外接口

导出内容：

- `SearchProvider`
- `SearchService`
- `ExaSearchProvider`
- `GrokSearchProvider`, `GrokReasoningEffort`
- `TavilySearchProvider`
- `SearchRequest`, `SearchResponse`, `SearchResult`
- `Freshness`, `SearchDepth`, `SearchContentLevel`, `SearchRecency`, `SearchCategory`

路由原则：

- 每次 search request 明确指定一个 provider。
- allowlist 只做授权，不表达 fallback 顺序。
- 选定 provider 失败时不自动 fallback 到其他 provider。

## 关键依赖与配置

- 依赖 `moe-research-net` 发送 JSON 或 SSE。
- Exa：JSON POST 到 `search`，支持 category、domain、freshness、content level 映射。
- Grok：SSE POST 到 `responses`，用 web search tool 和 prompt 生成搜索结果，支持 `reasoning_effort` 与 `max_output_tokens`。
- Tavily：JSON POST 到 `search`，支持 depth/topic/time_range/raw content 映射。

## 数据模型

`SearchRequest` 字段：provider、query、max_results、freshness、depth、content_level、recency、category、language、region、include_domains、exclude_domains。

校验规则：

- provider 必须非空且无首尾空白。
- query 必须非空。
- max_results 必须大于 0。
- policy 层会进一步限制 provider allowlist、max results、depth/content/recency/category 等。

## 测试与质量

主要测试：

- `crates/moe-research-tests/tests/search_tests.rs`

覆盖点：

- 显式 provider dispatch。
- 失败不 fallback。
- allowlist 与 invalid request 拒绝。
- provider-specific request/response 映射。
- Exa category 冲突规则。
- Grok SSE 事件组装与 citation/source 映射。
- Tavily result clamp 与 raw content summary。

建议验证：

```bash
cargo test -p moe-research-tests search
```

## 常见问题 (FAQ)

- 为什么 search provider 不聚合多个服务？项目约束要求每次搜索调用只使用一个 provider。
- SearchPolicy 的 `allowed_providers` 为空是什么意思？当前语义是没有 provider 被允许，因此会拒绝请求。
- Grok Search 为什么使用模型式 responses？Grok provider 当前通过 xAI responses + web search 能力返回可引用结果。

## 相关文件清单

- `Cargo.toml`
- `src/lib.rs`
- `src/provider.rs`
- `src/service.rs`
- `src/types.rs`
- `src/provider/exa.rs`
- `src/provider/grok.rs`
- `src/provider/tavily.rs`
