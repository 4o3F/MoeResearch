# Layer1 ↔ Lapis 编排接口（WS4 · 通用 + capability 路由）

> Status: Phase 2 WS4 产出（2026-05-29，已随 Phase 2 签收；2026-05-29 交叉审计修订；Phase 2′ G4 泛化，2026-05-30）。
> 目的：把通用规格 [`pm-deep-research-spec.md`](pm-deep-research-spec.md) + 各能力 profile（[`capabilities/`](capabilities/)，v2.0 = [`capabilities/competitive.md`](capabilities/competitive.md)）的工作流落到 **Lapis 真实 MCP 接口**（依据 [`../mcp-usage.md`](../mcp-usage.md)），明确每步谁做、传什么、schema 缺口怎么补。
> **Phase 2′ G4 变更点**：①§1 加 capability 路由步（Step 1.5）；②§2 从"五维→aspect_tasks"泛化为"**profile skeleton → aspect_tasks**"（competitive 为 v2.0 实例）；③§4 4-tier 后处理、§5 预算、§6 引擎边界、§7 降级**对所有 capability 通用**，不变。**Lapis MCP 边界绝不变动**（仍 `aspect_research` + `deep_research`）。

---

## 0. 关键现实约束（先读）

对照 Lapis MCP 实际表面（`docs/mcp-usage.md`），早期草稿的假设需要修正：

| 草稿假设 | Lapis 实际 | 影响 |
|---|---|---|
| 有 `research_plan` 工具做拆解 | **不存在**。只有 `aspect_research` + `deep_research` | **拆解（5 维→aspect 列表）必须在 Skill 层做**，作为 `deep_research.aspect_tasks` 传入 |
| 有 `compare_reports` 工具 | **不存在** | **跨竞品对比/综合/13 章报告拼装在 Skill 层做** |
| aspect report 带 `dimension/persona/decision_intent/visual_evidence/user_jobs/gap_status` | Lapis `AspectReport` 只有 `findings/assumptions/risks/counterarguments/open_questions/confidence/limitations` | 产品结构字段**当前无 schema 位**，v2.0 用 prompt+Skill 编码承载（见 §3）|
| evidence 有 `tier/credibility A-E/visual` | Lapis `Evidence` 有 `source_type`(7 枚举) + `confidence`(low/med/high) | 4-tier 与视觉证据由 **Skill 映射/装配**（见 §4）|

**结论**：v2.0 **不改 Lapis 源码**——PM DeepResearch 作为 Skill 消费上游 Lapis 原样接口；产品方法论（五维/人格/证据完整性）通过 **`aspect_agent_prompt` 注入 + Skill 层装配**实现。可选的 Rust schema 小幅扩展留 Phase 3（§6）。

---

## 1. 端到端步骤（谁做 / 传什么）

| 步 | 动作 | 执行方 | 输入 → 输出 |
|---|---|---|---|
| 1 | 触发识别 + decision_intent 推断 + 复杂度路由 | **Skill** | 用户问题 → `decision_intent`（Enter/Differentiate/…）+ tier（Quick/Standard/Deep）|
| **1.5** | **capability 路由 → profile 装配** | **Skill** | decision_intent + 触发线索 → `capability`（competitive / product-capability / innovation-direction / product-requirements）→ 加载对应 [profile](capabilities/) 的装配契约（skeleton / report_template / persona_tm_weighting / capability_specific gap & floor，[通用 spec §1 / §11](pm-deep-research-spec.md#1-能力路由模型trigger--intent--capability--complexity--profile)）|
| 2 | profile skeleton → aspect 拆解 | **Skill** | 装配契约 + 目标 → `aspect_tasks[]`（competitive 实例见 §2；其它 profile 的 skeleton-段→aspect 映射在各 profile §2 + v2.x 实施时另起）|
| 3 | 人格 prompt 装配 | **Skill** | 每个 aspect 填 `role` + `aspect_agent_prompt`（按 profile 的 `persona_tm_weighting` 注入 2 人格 + TM 权重 + 输出契约，§3）|
| 4 | 预算/策略装配 | **Skill** | tier → `budget` + `model_policy`/`search_policy`/`evidence_policy`（§5）|
| 5 | 调 `deep_research` | **Skill→Lapis** | 上述组装为一次 `deep_research` 调用；Lapis 并行跑各 aspect 的 agent loop（带 search）并聚合 |
| 6 | 跨 aspect Gap 检测 | **Skill** | `DeepResearchResult` → 按通用规格 §9.1 + profile §3.1 清单查缺口 → 必要时对缺口 aspect 再调 `aspect_research`（≤Deep 2 轮）|
| 7 | 证据分级 + 视觉证据装配 | **Skill** | `evidence_index` → 4-tier + 展示标签 + visual_evidence 表（§4）|
| 8 | 综合 + 报告装配 | **Skill** | aspect_reports + 证据 → 按 profile 的 `report_template`（family A 13 章 / family B 8 段 PR-FAQ）+ 机会矩阵 / 解空间 / Roadmap |
| 9 | 自评 quality floor | **Skill** | 报告 → 通用 §9.2 + profile §3.2 + [rubric §6 实例化](../evaluation/rubric.md#6-capability-profile-deltab1--b3-实例化--capability-specific-floor-追加项)；不达标标警告或弃权 |

> Lapis 负责的是 **步骤 5 内部**：每个 aspect 的多轮 agent loop、并行调度、搜索、逐条 evidence 绑定 finding、预算执行、partial 聚合。其余编排智能在 Skill。

---

## 2. Profile skeleton → `aspect_tasks` 映射

每个 aspect = 一个独立 agent（自带搜索预算）。每 capability profile 的 skeleton 段（通用 §11 装配契约第 3 字段）映射到 aspect_tasks；下表 = **v2.0 competitive 实例**（profile §1 的 5 段 + Build 意图额外 build-cost aspect）。其它 capability 的 skeleton→aspect 映射在各 [profile §2](capabilities/) 给出 / v2.x 实施时另起；本步骤通用机制不变（profile skeleton 段 = aspect_tasks 项）。

**v2.0 · competitive 实例**（5–6 aspect）：

| aspect_id | 五维来源 | role | research_question（示意）|
|---|---|---|---|
| `job-and-competitive-set` | 维度 1 | product strategist | 用户雇这个产品完成什么 job？按 job 谁是真实竞争集（含非显性替代者）？|
| `capability-and-importance` | 维度 2+3 | product experience analyst | 目标产品与竞品在买家关注维度上的能力对位如何？哪些功能按 Kano 是 Must-be/Performance/Attractive？|
| `opportunity-gaps` | 维度 4 | product strategist | 各 desired outcome 的 Importance/Satisfaction？ODI 机会分排序？|
| `positioning-whitespace` | 维度 5 | product strategist | buyer-validated 轴上各家 value curve 如何？白地在哪？维持性 vs 颠覆性威胁？|
| `experience-paths`（Deep）| 维度 2 深化 | product experience analyst | 核心路径（进入/操作/反馈/留存/转化）的体验断点 + 视觉证据 |

- **Quick**：只跑 `job-and-competitive-set` + `capability-and-importance`。
- **Standard**：前 4 个。**Deep**：全部 5 个（+ 按需每竞品 profile）。
- `scope`/`boundaries`/`success_criteria` 由 Skill 按各维度的"证据标准"（competitive profile §1）填，使 Lapis 的 success_criteria 即我们的证据门槛。
- **Build/Not Build 意图补充**：`capability-and-importance`（或单列一个 aspect）的 `aspect_agent_prompt` 须要求拉取竞品 **release notes / App Store 版本历史**，用迭代节奏估算 build-cost（[通用 §4.4 M-Changelog](pm-deep-research-spec.md#m-changelogchangelog--版本时间线--实际动作证据) + [competitive profile §1 支撑段「迭代节奏与建设成本」](capabilities/competitive.md#1-b1-五维骨架报告主干)），把复杂度估算 + 版本时间线写进 `Finding.claim`，证据 url 指向版本历史页。

> **其它 capability 的 skeleton→aspect 映射（v2.x 实施时落地）**：
> - product-capability：6 段（能力域 + JTBD / 单域 teardown 深度 / 体验路径 + 断点 / Kano 域内 / ODI 域内 / benchmark + build-cost）→ 6 aspect（[profile §2](capabilities/product-capability.md#2-6-段骨架报告主干)）。
> - innovation-direction：8 段（趋势 / unmet outcomes / 白地 canvas / 未来能力映射 / 颠覆 + 可防御性 / pre-mortem / build-cost / 推荐下注）→ 6-8 aspect（可合并相关段）（[profile §2](capabilities/innovation-direction.md#2-8-段骨架报告主干)）。
> - product-requirements：8 段 PR-FAQ → 5-8 aspect（机会验证 / 四风险 / OST 解空间 / 需求 / 成功度量 可合可拆）（[profile §2](capabilities/product-requirements.md#2-8-段骨架b2-模板报告主干)）。

---

## 3. 产品结构字段如何承载（无 Rust 改动）

Lapis `Finding.claim` 是自由文本，`finding_type ∈ {fact, interpretation, recommendation, risk, assumption}`，`Evidence` 带 url/source_type/confidence。v2.0 用如下约定承载产品结构：

| 规格要的字段 | v2.0 承载方式（prompt + Skill）|
|---|---|
| `decision_intent` | 写入 `shared_context.summary`，对所有 aspect 可见 |
| 能力对位矩阵 / Kano / ODI 打分 / 定位 | `aspect_agent_prompt` 要求 agent 把结构化结果作为 **Markdown 表/JSON 块写进 `Finding.claim`**；Skill 解析装配 |
| `visual_evidence`（截图/视频 URL）| **选一条 url 指向媒体的搜索结果证据**，provenance 逐字复制（**不可改写 `summary`/`snippet`**，否则破坏 byte-equal 校验）；media_type/observed_feature/related_claim 写进**引用它的 `Finding.claim`** 结构块；Skill 据此抽成视觉证据表（§4）|
| TM-4 认识论标注 | 复用 `finding_type`（fact/interpretation/assumption）+ `confidence`；推测类入 `assumptions`，反论入 `counterarguments`（Lapis 原生有此字段，正好承 TM-11）|
| 4-tier 可信度 | Skill 后处理：`Evidence.source_type` + URL 域名 → 4-tier + 展示标签（§4）|

> **`aspect_agent_prompt` = 人格落点**：Skill 把 2 人格（Experience Analyst / Strategist）的 TM-laden system prompt + "输出契约（哪些结构进 claim、证据要带 url、缺视觉证据要进 open_questions）" 作为 inline prompt 传入。Lapis 无 persona 概念——**persona 即 prompt**。

---

## 4. 证据分级与视觉证据装配（Skill 后处理）

Lapis `Evidence.source_type` ∈ `{official, documentation, news, blog, forum, repository, unknown}`。Skill 映射到规格 §6.1 的 4-tier + 展示标签：

| Lapis source_type | + 域名启发式 | 4-tier | 展示标签 |
|---|---|---|---|
| official / documentation | 官网/财报/应用商店/**release notes·版本历史**/.gov/.edu | Tier 1–2 | High |
| news / blog | 主流媒体/具名评测/开发者博客 | Tier 3 | Medium |
| forum | 应用商店评论/社媒/论坛 | Tier 3（社区子类）| Low（仅情绪/线索/假设）|
| unknown | 无日期/无法追溯 | Tier 4 | Unknown（不进核心结论）|

- **视觉证据**：Skill 扫 `evidence_index` + 各 `Finding.claim` 里的视觉标注块，凡 url 指向图片/视频/应用商店页 → 进规格 §6.2 的 visual_evidence 表（Ch 7）。媒体元数据（media_type/observed_feature）来自 claim 标注，**不来自被改写的 Evidence 字段**（保 byte-equal）。Deep 模式若 <5 条 → **Skill 层外部步骤**触发 Layer 2 浏览器（agent-browser/browser-use 走系统 Chrome）补抓——这是 Skill 在 Claude Code 侧的能力，**非 Lapis aspect agent 能力（aspect agent 只暴露 `search`）**，抓后回填。
- **原子核验/语句级审计**（FActScore/DeepTRACE）：Skill 对关键 finding 抽样核验 `claim` 能否从 `evidence_refs` 指向的源推出（CiteEval）；不达标降置信或弃权。

---

## 5. 预算映射（规格复杂度 tier → Lapis budget）

依据 Track A A5（先估难度→分级预算→信息增益门控）。示意值（Phase 3 调参）：

| tier | `budget.max_agents` | 每 aspect `max_search_calls` | `max_concurrent_agents` | `total_timeout_ms` |
|---|---|---|---|---|
| Quick | 2 | 3 | 2 | 120000 |
| Standard | 4 | 6 | 2 | 300000 |
| Deep | 5–6 | **4**（R4-c canonical；原 8 过搜 hard-kill，详 §5.1）| 2–3 | 600000 |

> **完整 budget 形状**（WS-B 生成 `DeepResearchRequest` 必须给齐，照 `prompts/layer1/task-decomposition.md` 输出 schema）：顶层 `budget{ max_agents, max_concurrent_agents, max_total_model_calls, max_total_search_calls, total_timeout_ms, max_tokens }` + 每 aspect `budget{ max_turns, max_tool_calls, max_search_calls, timeout_ms }`。上表只列了 tier 间差异最大的几项，其余字段也必须填。
> **per-aspect `timeout_ms` 用 600000（10min）**：D3 实测 CPA(gpt-5.5)+grok-4.3 较慢，300000 会 `budget_exceeded`。服务端 toml budget 全 `-1` 不限，瓶颈只在请求里的 per-aspect timeout。

- `evidence_policy.require_evidence_for_findings = true` 恒开（强制"宁少但真"——finding 必须带 evidence）。`min_evidence_per_finding`：Standard=1，Deep=2。
- `model_policy.allowed_providers` / `search_policy.allowed_providers`：由用户 key 配置决定（无 Exa key 则只 grok）。**注意**：policy 的 allowed_providers 是授权白名单**不是 fallback 顺序**——降级顺序在 Skill 控制。
- Gap 第 2 轮补搜用单独 `aspect_research` 调用（带 `shared_context.prior_sources` = 已有证据，避免重复）。

### 5.1 上游 search tuning 字段（`e04398d5` / [#7](https://github.com/4o3F/Lapis/issues/7)，R4-d 回补）

上游 2026-05-30 给 `SearchRequest` / `SearchPolicy` 加了 4 个 **provider-neutral Option 字段**（`schema_version="0.1"` 不变；全 Option，**不传即默认行为**，现有 prompts 不传仍合法）。

> **⚠️ 机制更正（R4-c 实测引擎 `9db7464` 源码，2026-06-01）**：这 4 个字段**只能挂全局 `search_policy`**——`crates/lapis-workflow/src/workflow.rs::aspect_requests` 把同一个 `search_policy` clone 进**每个** aspect；`AspectSpec`（`research.rs`）只有 `search_provider`，**没有 per-aspect search 字段**。早前本节说的「挂到单 aspect 的 `aspect.search_*`」**在此引擎不存在**。要 per-aspect 差异化只能靠 `aspect_agent_prompt` 指示 agent 在调 `search` 工具时自带 `SearchToolArgs`（运行时、非确定性），属未来工作。policy 语义（`policy.rs`）：`depth`/`content_level`/`recency` 是 **ceiling + default**（agent 可往低 rank 调，不能超）；`category` 是 **exact-match**（与 policy 不同即报错）→ **`category` 不能全局**（会逼所有 aspect 同一类）。

**✅ R4-c 已验证启用（v2.0 competitive，2026-06-01，纯引擎 golden 23/24，A4 1→2）**：canonical 配置 = `recency="fresh"` + `max_results_per_query=5`（已写进 [task-decomposition.md](../../prompts/layer1/pm-deep-research/task-decomposition.md) search_policy）。**不用 `depth=high_recall`**（其 prompt-hint 怂恿过搜，撞引擎 search-budget **hard-kill**——`agent_loop.rs` 对超预算的 search 是秒杀整个 aspect、无优雅合成回退；rerun1 cap=8+high_recall **仅 1/6 收敛**——其中 2 个 aspect（capability/build-cost 各搜 9 次超 cap=8）被 search-budget 秒死，另 2 个 `mutated_evidence_provenance`、1 个 network）；**不全局用 `content_level=detailed`**（与 `mutated_evidence_provenance` 失败相关）。**铁律**：在 hard-kill 引擎上 search-tuning **只挑「不增搜索次数」的字段**（`recency`/`max_results` 安全；`depth=high_recall` 危险）。

| 字段 | enum 值 | 默认 | 含义 / 用法 |
|---|---|---|---|
| `depth` | `low_latency` / `balanced` / `high_recall` | `balanced` | 检索召回 vs 延迟权衡。`high_recall` = 广度优先扫盲（趋势 / 替代品 / 非显性竞争集）；`low_latency` = 快查单一事实 |
| `content_level` | `compact` / `standard` / `detailed` | `standard` | 每条 evidence 返回的正文厚度。`detailed` = 高价值少量 evidence 需厚 context（ODI Imp/Sat 估算依据、机会验证段）；`compact` = 只要标题/摘要 |
| `recency` | `default` / `live` / `fresh` / `recent` / `cached` | `default` | 时间新鲜度偏好。`fresh` / `recent` = changelog / 版本时间线 / 趋势扫 / 新闻类 aspect（直接解 M4 build-cost A4=1）；`cached` = 容忍旧快照换速度 |
| `category` | `organizations` / `people` / `academic` / `news` / `personal_sites` / `financial_filings` / `code` | 无（不限） | entity-discovery 类别提示。`organizations` = 找竞品 / 真实竞争集 / benchmark best-in-class；`academic` = 找方法论原始出处；`news` = 失败/成功案例时效扫 |

**字段示例**（只能挂全局 `search_policy`，所有 aspect 继承；**无 per-aspect 覆盖**）：

```jsonc
"search_policy": {
  "allowed_providers": ["grok"],
  "max_results_per_query": 5,
  "freshness": null,
  // ↓ R4-c canonical（v2.0 已启用）：只挑「不增搜索次数」的安全字段
  "recency": "fresh",
  // depth=high_recall ❌ 过搜 hard-kill；content_level=detailed ❌ provenance 改写；category ❌ exact-match 不能全局
  "language": null, "region": null, "include_domains": [], "exclude_domains": []
}
```

- **provider 翻译**：Exa / Grok adapter 各自把这 4 个 neutral 字段翻成自家参数；Exa 同时真正请求 `contents` 拿回 `text` / `summary`（厚 evidence summary → 受益 A2 引用准确性 + B3 核心证据可读性）。
- **不是 vendor DTO**：这 4 个是 Lapis 中立字段，**不要**emit 原始 Exa/Grok 参数名（同 [§5 / decomposition rules 第 8 条](#5-预算映射规格复杂度-tier--lapis-budget) provider 名是逻辑 config 名的纪律）。
- **现状（2026-06-02，4 能力全部走完）**：
  - **v2.0 competitive ✅ canonical**（2026-06-01）`recency=fresh` + `max_results=5` + per-aspect cap=4；锚点 22→23，A4 1→2。
  - **v2.1 product-capability ✅ canonical**（2026-06-02）`recency=fresh` + `max_results=5` + per-aspect cap=3；锚点 23 持平、无回归、enrichment +5 ev / +2 domain。
  - **v2.2 innovation-direction ✅ canonical（cap=6 必须）**（2026-06-02）`recency=fresh` + `max_results=5` + **per-aspect cap=6**（不是 v2.0=4 / v2.1=3）；recommended-bets 综合下注 aspect 在 recency=fresh prompt-hint 下 search appetite ~6，cap=5 持续 hard-kill。锚点 24 持平、+11 ev。
  - **v2.3 product-requirements ⛔ NOT canonical**（2026-06-02，honest 不挂）；requirements-fn-nfn-nongoals 综合 FR/NFR aspect 在 recency=fresh 下结构性 synthesis-time fragile（4 次 retry 模式齐：cap=8 search hard-kill / cap=9 runtime / cap=9+aspect_timeout=900s 撞 execution_policy=600s 复校 / cap=9+双侧 900s 撞 CPA SSE flake）；family B PR-FAQ 合成强度比 v2.2 recommended-bets 高一档。rerun9 anchor 24/24 沿用。等引擎支持 per-aspect `search_policy` override 或 per-aspect `execution_policy.timeout_ms` 后回头。
  - **通则**：每能力 per-aspect cap 因综合 aspect 强度而异；`category` exact-match 不能全局——4 能力的 per-aspect category 启用都延后引擎层。

---

## 6. 引擎边界：第一版不动引擎；schema 扩展作为「需求」提给上游（Heye 2026-05-29 确认）

**引擎不是我们做的**——Lapis 由上游 **4o3F** 维护，PM DeepResearch 是消费方（PM DR 不修引擎，需求走 issue）。因此：

- **v2.0 第一版不碰引擎**：纯 prompt+Skill 承载产品字段（§3），用 [rubric](../evaluation/rubric.md) + 黄金样例实测是否够稳。
- **后续若实测承载不稳**（agent 不照约定填 claim / 漏视觉证据 / 需机器强校验），把下列 schema 扩展整理成**需求清单提给 4o3F 上游**（我们提需求，不自己改引擎源码）：
  - `Evidence` 加 `media_type` + `visual` 标记；`source_type` 扩 `app_store|social|video|research`。
  - `AspectReport` 加 `extensions: object`（自由结构）承载五维结构化输出。
  - `EvidencePolicy` 加 `require_visual_evidence_for_aspects: string[]`。
- 规格已将这些标为**可选**，不构成 v2.0 阻塞。

### 6.1 上游已实装变更（2026-05-30，本地 vendored 待 Phase 4 同步）

4o3F 在 2026-05-30 合入两项与 PM DeepResearch 直接相关的修复 / 能力（本地 vendored 副本同步延后到一键安装窗口）：

| 上游 commit / issue | 内容 | 对 Skill 层的影响 | 处理 |
|---|---|---|---|
| `e0979f8c` / [#8 closed](https://github.com/4o3F/Lapis/issues/8) | `fix(net): raise SSE event cap for reasoning streams` — 放宽 SSE event-count guard（保留 byte caps） | 长推理流不再被 4096 事件硬上限砍断 | **本地 SSE 补丁可在 Phase 4 vendored 同步时撤除**；当前补丁与上游修复目标一致、不冲突 |
| `e04398d5` / [#7 closed](https://github.com/4o3F/Lapis/issues/7) | `feat(search): add provider-neutral search tuning` — `SearchRequest`/`SearchPolicy` 新增 4 个 provider-neutral Option 字段：`depth`（`low_latency`/`balanced`/`high_recall`）、`content_level`（`compact`/`standard`/`detailed`）、`recency`（`default`/`live`/`fresh`/`recent`/`cached`）、`category`（`organizations`/`people`/`academic`/`news`/`personal_sites`/`financial_filings`/`code`）；Exa/Grok adapter 各自翻译；Exa 真正请求 `contents` 拿到 `text`/`summary` | 启用后在**全局 `search_policy`** 设这些字段（引擎无 per-aspect search 字段，详 §5.1），提高 entity-discovery 精度（如 `recency=fresh` 抓 changelog；`category` 因 exact-match 不能全局） | `schema_version="0.1"` 不变、新字段全 Option → 现有 prompts 不传也合法；正式启用见 §5.1 4 能力 R4-c 现状 |

> **Why 不立刻同步 vendored**：vendored 副本是工作参考，删/同步统一放到一键安装器拉上游构建那一步；中途同步会牵动一批 Rust 文件、增加 v2.x 实施期的耦合面。Golden 跑用现有本地副本 + SSE 补丁即可。

**启用红利触发场景**（字段挂**全局 `search_policy`**、非 per-aspect——"涉及 aspect"列只表意图，见表后 R4-c 修正注）：

| 上游新字段 | 触发场景 | 涉及 capability / aspect | 期望解决 / 受益 rubric 维 |
|---|---|---|---|
| `category=organizations` | entity-discovery：找竞品 / 真实竞争集 / benchmark best-in-class | competitive `job-and-competitive-set` + `positioning-whitespace`；product-capability `benchmark-buildcost-upgrade`；innovation-direction 段 4 未来能力映射；product-requirements 4 风险 / Cagan business viability aspect | A1 引用充分性 + A4 来源质量（命中率↑→可用证据↑）|
| `recency=fresh` / `recent` | changelog / 版本时间线 / 趋势扫 / 新闻类 aspect | competitive `build-cost-version-history`；product-capability `benchmark-buildcost-upgrade`；innovation-direction 段 1 趋势扫描；任何看 release notes 的 aspect | **直接解 M4 A4=1 扣分**（build-cost 当时仅 2 证据，命中率↑可在同 `max_search_calls` 下回补来源数）|
| Exa `contents`（`text` / `summary` 真正返回） | 所有 finding 的 quoted evidence summary 厚度 | 全部 layer1/layer2 prompts | A2 引用准确性（CiteEval）+ B3 核心证据机制（per-cell 证据可读性 / 评论原文可摘）|
| `content_level=detailed` | 高价值少量 evidence 需厚 context | competitive `opportunity-gaps`；product-capability `odi-in-domain`；product-requirements 机会验证段 + 三套指标段 | B4 机会量化严谨（ODI Imp/Sat 估算依据可追溯）|
| `depth=high_recall` ⚠️ | ~~广度优先扫盲~~ **R4-c 实测危险：勿用** | — | ~~B1 skeleton 覆盖~~ |

> **⚠️ R4-c 实测修正（2026-06-01，纯引擎 golden）**：
> - **本表的"涉及 aspect"列是误导的**——引擎无 per-aspect search 字段（§5.1），这些字段只能挂**全局 `search_policy`**、对所有 aspect 一视同仁。表里的 per-aspect 意图只能靠 `aspect_agent_prompt` 指示 agent 运行时自带 `SearchToolArgs`（非确定性，未来工作）。
> - **`recency=fresh` ✅ 已验证**：v2.0 build-cost 经引擎 fresh 路径产 4 条 dated official 版本史证据（锚点 2）→ **A4 1→2 坐实**（正是本表预测）。
> - **`depth=high_recall` ❌ 危险勿用**：其 `prompt_hint` 怂恿 agent 多搜，撞引擎 search-budget **hard-kill**（`agent_loop.rs` 超预算秒杀整个 aspect）；rerun1 cap=8+high_recall **仅 1/6 收敛**——2 个 aspect 被 search-budget 秒死（capability/build-cost），另 2 个 provenance 改写、1 个 network。
> - **`content_level=detailed` ⚠️ 不全局用**：与 `mutated_evidence_provenance` 失败相关。
> - **`category` 不能全局**（exact-match）→ per-aspect prompt-guided，延后 WS-U。
> - **铁律**：hard-kill 引擎上只挑「不增搜索次数」的字段（`recency`/`max_results` 安全）。
> - **4 能力 R4-c 最终状态（2026-06-02）**：v2.0 ✅ canonical（cap=4）· v2.1 ✅ canonical（cap=3）· v2.2 ✅ canonical（**cap=6 必须**——recommended-bets 在 recency-fresh prompt-hint 下 search appetite ~6）· v2.3 ⛔ NOT canonical（requirements-fn-nfn-nongoals synthesis-time fragile，4 retry 模式齐：cap=8 search hard-kill / cap=9 runtime / 双 timeout 900s 仍 600s execution_policy 复校 / 撞 CPA SSE flake）。**关键**：recency=fresh 的 prompt-hint 把综合 aspect 的 search appetite **推高 1-2 次**——每能力的 cap 因综合 aspect 强度而异，**不能一刀切**。

---

## 7. 降级（MCP 不可用）

`deep_research` 调用失败（`provider_unavailable`/`network_failed`/进程不可用）→ Skill 退化为 **Claude-only**：直接用搜索 MCP（grok/exa）按五维方法论自己跑 + 装配报告，证据纪律不变（规格 §10）。`status=partial` 时用已完成 aspect + 把 `failed_aspects` 标为 gap。

## 待办
- [ ] WS3 黄金样例实跑时，验证 §3 的 prompt 承载方案是否够稳（决定 §6 是否触发）。
- [ ] Phase 3：把 §2 映射、§3 prompt 契约、§5 预算落成 `prompts/layer1/*` + `prompts/layer2/persona-*.md` 实体文件。
