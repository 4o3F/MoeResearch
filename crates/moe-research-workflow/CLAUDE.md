[根目录](../../CLAUDE.md) > [crates](../) > **moe-research-workflow**

# moe-research-workflow 模块文档

## 变更记录 (Changelog)

| 时间 | 变更 |
| --- | --- |
| 2026-06-29 13:22:02 | 初次扫描并生成模块级文档。 |

## 模块职责

`moe-research-workflow` 是 MoeResearch 的核心编排模块。它定义 MCP 工具请求/响应 schema，执行单 aspect agent loop 和多 aspect deep research orchestration，并负责预算、策略、工具授权、输出校验、证据命名空间和 partial result 语义。

## 入口与启动

- crate 入口：`src/lib.rs`
- 对外主函数：
  - `aspect_research(request, model_service, search_service, budget_config)`
  - `deep_research(request, model_service, search_service, budget_config)`
- 核心运行时：`src/agent_loop.rs`
- 多 aspect 编排：`src/workflow.rs`

## 对外接口

主要导出：

- 请求：`AspectResearchRequest`, `DeepResearchRequest`, `AspectSpec`, `AspectResearchTask`, `ResearchContext`
- 报告：`AspectResearchResult`, `DeepResearchResult`, `AspectReport`, `Finding`, `Evidence`, `OpenQuestion`, `AspectFailure`
- 策略：`ModelPolicy`, `SearchPolicy`, `EvidencePolicy`, `OutputPolicy`, `ExecutionPolicy`, `ToolName`
- 预算：`BudgetConfig`, `ResearchBudget`, `AgentBudget`, `Limit`
- 运行时：`AgentRuntime`, `AgentBudgetGuard`, `ResearchBudgetGuard`
- 工具：`SEARCH_TOOL_NAME`, `SearchToolArgs`, `search_model_tool`

## 关键依赖与配置

- 依赖 `moe-research-model::ModelService` 与 `moe-research-search::SearchService`。
- `SUPPORTED_SCHEMA_VERSIONS = ["0.1"]`。
- 当前模型可见工具只有 `search`。
- Deep research 并发由 `budget.max_concurrent_agents` 控制。
- 请求预算和 operator 配置预算取更严格值；`Unlimited` 表示该层不加限制，不表示覆盖另一层有限值。

## 数据模型

关键流程：

1. `AspectResearchRequest` / `DeepResearchRequest` 做 schema version、ID、provider、policy、budget、prompt 大小等校验。
2. `AgentRuntime` 构造初始 prompt/input，循环调用模型。
3. 模型如返回 tool call，`ToolPolicyGuard` 校验 tool 名称、允许列表和 args。
4. `AgentBudgetGuard` 与 `ResearchBudgetGuard` 在 provider dispatch 前消费预算。
5. 搜索结果被转成 candidate evidence。
6. 模型最终必须输出 `AspectResearchResult` JSON。
7. `OutputValidator` 检查报告字段、finding/evidence 引用、证据来源未被篡改。
8. Deep research 汇总 aspect reports、evidence index、open questions、coverage/confidence/budget usage。

重要约束：

- `aspect_agent_prompt` 非空，最大 64 KiB。
- search-enabled aspect 必须显式指定 `search_provider`。
- aspect 必须显式指定 `model_provider`。
- Deep research aspect_id 必须唯一。
- duplicate tool call id 会在 dispatch 前拒绝。
- evidence 在 deep result 中按 `aspect_id:original_id` 命名空间化。

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
- search tool args 与 policy 限制。
- output validator、evidence provenance、防篡改。
- schema roundtrip 与 public contract。

建议验证：

```bash
cargo test -p moe-research-tests deep_research
cargo test -p moe-research-tests orchestrator
cargo test -p moe-research-tests policy_validator
cargo test -p moe-research-tests schema
```

## 常见问题 (FAQ)

- 为什么 request 和 config 都有 budget？配置是 operator 硬上限，请求只能进一步收紧。
- 为什么 partial result 有时返回 Ok？当 `allow_partial_results = true` 且已有有价值结果时，deep research 返回 `DeepResearchResult` 并记录 failed aspects。
- 为什么要 byte-equal evidence provenance？防止模型把搜索工具返回的来源、URL、snippet 等 provenance 字段改写后仍被当作可信证据。
- 为什么不把 prompt 文件 IO 放在 Rust core？Layer 1 Skill 负责 prompt 选择、替换和版本固定，Rust core 只接收 inline prompt。

## 相关文件清单

- `Cargo.toml`
- `src/lib.rs`
- `src/research.rs`
- `src/report.rs`
- `src/workflow.rs`
- `src/agent_loop.rs`
- `src/policy.rs`
- `src/tool_policy.rs`
- `src/budget.rs`
- `src/runtime_budget.rs`
- `src/limit.rs`
- `src/validator.rs`
