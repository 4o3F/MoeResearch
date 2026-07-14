[根目录](../../CLAUDE.md) > [crates](../) > **moe-research-workflow**

# moe-research-workflow 模块文档

## 变更记录 (Changelog)

| 时间 | 变更 |
| --- | --- |
| 2026-06-29 13:22:02 | 初次扫描并生成模块级文档。 |
| 2026-07-09 | 更新 v0.2 控制面请求 schema：`task` / `limits` / `policy` / `context`。 |
| 2026-07-09 | 增加 Phase 2 normalizer：请求/config limits 合并为 `EffectiveResearchPlan` / `EffectiveAspectPlan` 后再进入 runtime。 |
| 2026-07-09 | 增加 Phase 3 `AspectPromptInput`：Layer 2 user prompt 只接收 LLM-visible 投影。 |
| 2026-07-10 | Phase 3 责任边界重组：`research/`、`report/`、`workflow/`、`runtime/` 分目录；公开 API 路径与 schema 0.2 语义不变。 |

## 模块职责

`moe-research-workflow` 是 MoeResearch 的核心编排模块。它定义 MCP 工具请求/响应 schema，执行单 aspect agent loop 和多 aspect deep research orchestration，并负责 limits、policy、工具授权、输出校验、证据命名空间和 partial result 语义。

## 入口与启动

- crate 入口：`src/lib.rs`
- 对外主函数：
  - `aspect_research(request, model_service, search_service, web_fetch_service, budget_config)`
  - `deep_research(request, model_service, search_service, web_fetch_service, budget_config)`
- 核心运行时：`src/runtime/agent.rs`（agent loop 与 model turn）和 `src/runtime/tools/`（tool policy、Search/WebFetch 执行、evidence assembly），配套 `budget`、`deadline`、`model_turn`
- 多 aspect 编排：`src/workflow/{aspect,deep,aggregation}.rs`
- 请求/计划/prompt：`src/research/{request,plan,prompt}.rs`
- 报告 + 校验：`src/report/{mod,validator}.rs`

## 对外接口

主要导出：

- 请求：`AspectResearchRequest`, `DeepResearchRequest`, `AspectRequest`, `ResearchTask`, `ResearchContext`, `ResearchPolicy`
- 报告：`AspectResearchOutput`, `AspectResearchFailure`, `AspectResearchResult`, `DeepResearchResult`, `AspectReport`, `Finding`, `Evidence`, `OpenQuestion`, `AspectFailure`, `DeepResearchFailure`
- 策略：`ModelPolicy`, `SearchPolicy`, `EvidencePolicy`, `OutputPolicy`, `ExecutionPolicy`, `ToolName`
- 限额：`BudgetConfig`, `ResearchLimits`, `AgentLimits`, `Limit`

运行时 guard、tool policy、validator 不从 crate root 暴露；外部调用方应通过 `aspect_research()` / `deep_research()` 进入 normalizer 后的执行路径。

## 关键依赖与配置

- 依赖 `moe-research-model::ModelService`、`moe-research-search::SearchService` 与 `moe-research-web-fetch::WebFetchService`；三个 service 均作为必需运行时依赖注入。
- `SUPPORTED_SCHEMA_VERSIONS = ["0.2"]`（定义于 `src/research/request.rs`）。
- 模型可见工具为按 aspect allowlist 暴露的 `search` 与 `web_fetch`；WebFetch 不属于顶层 MCP tool。
- Deep research 并发由 normalized `limits.max_concurrent_agents` 控制。
- 请求 limits 和 operator config limits 在 normalizer 中取更严格值；`Unlimited` 表示该层不加限制，不表示覆盖另一层有限值。

## 数据模型

v0.2 request 形态：

- `AspectResearchRequest`: `schema_version`, `request_id`, `task: AspectRequest`, `policy: ResearchPolicy`, `context: ResearchContext`
- `DeepResearchRequest`: `schema_version`, `request_id`, `task: ResearchTask`, `limits: ResearchLimits`, `policy: ResearchPolicy`, `context: ResearchContext`
- `AspectRequest`: aspect id/name/role/question/scope/boundaries/success_criteria, inline `instructions`, tool list, explicit providers, and per-agent `limits`
- `ResearchPolicy`: `model`, `search`, `evidence`, `output`, `execution`; execution policy has no timeout field

关键流程：

1. `AspectResearchRequest` / `DeepResearchRequest` 先经过 normalizer，完成 schema version、ID、provider、policy、limits、prompt 大小等校验。
2. normalizer 将 request limits 与 operator config limits 取更严格值，生成 `EffectiveResearchPlan` / `EffectiveAspectPlan`。
3. `AgentRuntime` 只消费 effective aspect plan，并负责 agent loop、model turn 与统一 tool dispatch；tool policy、Search/WebFetch 执行和 evidence assembly 位于 `runtime/tools/`。`instructions` 作为 system prompt，`AspectPromptInput` 作为 user prompt。
4. `AspectPromptInput` 只包含 aspect intent、context、可用工具和 evidence/output 要求，不包含 limits、provider allowlist、selected provider 或 execution policy。
5. 模型如返回 tool call，`ToolPolicyGuard` 对 `search` 只接受 `query`、optional `max_results` 和 required semantic `intent`，对 `web_fetch` 只接受 required `url` 与 `prompt`。
6. 已选 `SearchProvider` 将 intent 与 `SearchPolicy::intent_constraints()` 解析成一个实际 `SearchRequest`；hard policy 冲突在 dispatch 前拒绝，绝不 fallback 或聚合。WebFetch 经独立 service 抓取并由独立无工具模型处理。
7. `AgentBudgetGuard` 与 `ResearchBudgetGuard` 在实际 provider dispatch 前消费运行时预算；WebFetch 消耗 generic tool 与研究模型预算，但不消耗 search budget。
8. Search/WebFetch 成功结果被转成 host-owned candidate evidence；搜索还将 `intent_resolution` 返回给模型。
9. 模型最终只输出同一 `AspectResearchResult` 的 projection：`aspect_report` + ID-only `selected_evidence`。
10. `OutputValidator` rehydrate host evidence，检查报告字段、finding/evidence 引用、selected IDs，并推导 `supports_findings`。
11. 失败会生成 host-owned `FailureDiagnostic`：稳定 `stage`，以及可选的一基 `model_turn` / `search_turn`；aspect identity 继续由外层 `aspect_id` 承载。
12. Deep research 汇总 aspect reports、evidence index、open questions、coverage/confidence/budget usage。

重要约束：

- `instructions` 非空，最大 64 KiB。
- search-enabled aspect 必须显式指定 `search_provider`。
- aspect 必须显式指定 `model_provider`。
- Deep research `task.aspects[].id` 必须唯一。
- duplicate tool call id 会在 dispatch 前拒绝。
- evidence 在 deep result 中按 `aspect_id:original_id` 命名空间化。
- public request/policy DTO 使用 `serde(deny_unknown_fields)`；已删除的 v0.1 字段必须被拒绝。

## 测试与质量

主要测试：

- `deep_research_tests.rs`
- `orchestrator_tests.rs`
- `policy_validator_tests.rs`
- `schema_tests.rs`
- `mcp_tests.rs` 中也覆盖 envelope 侧行为。

覆盖点：

- 多 aspect 并发、partial result、fail_fast。
- agent/global budget、timeout、token usage。
- provider allowlist 与显式 provider。
- semantic search tool args、intent resolution 与 policy 限制。
- output validator、ID-only evidence selection、host rehydration 与 provenance 防篡改。
- schema roundtrip 与 public contract。

建议验证：

```bash
cargo test -p moe-research-tests deep_research
cargo test -p moe-research-tests orchestrator
cargo test -p moe-research-tests policy_validator
cargo test -p moe-research-tests schema
```

## 常见问题 (FAQ)

- 为什么 request limits 和 config limits 都存在？配置是 operator 硬上限，请求只能进一步收紧。
- 为什么 runtime 仍叫 `AgentBudgetGuard` / `ResearchBudgetGuard`？这是运行时消耗记账语义；LLM-visible request schema 使用 `limits`。
- 为什么 partial result 有时返回 Ok？当 `allow_partial_results = true` 且已有有价值结果时，deep research 返回 `DeepResearchResult` 并记录 failed aspects。
- 为什么模型只选择 evidence ID？候选 provenance 由 host rehydrate，模型无法改写来源、URL、snippet、source type 或 evidence-level confidence；`supports_findings` 由 finding references 自动推导。
- 为什么不把 prompt 文件 IO 放在 Rust core？Layer 1 Skill 负责 prompt 选择、替换和版本固定，Rust core 只接收 inline `instructions`。

## 相关文件清单

- `Cargo.toml`
- `src/lib.rs`
- `src/budget.rs`
- `src/limit.rs`
- `src/policy.rs`
- `src/error_log_safe.rs`
- `src/research/{mod,request,plan,prompt}.rs`
- `src/report/{mod,validator}.rs`
- `src/workflow/{mod,aspect,deep,aggregation}.rs`
- `src/runtime/{mod,agent,budget,deadline,model_turn}.rs`
- `src/runtime/tools/{mod,policy,search,web_fetch}.rs`
