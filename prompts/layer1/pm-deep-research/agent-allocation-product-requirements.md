# Layer 1 Prompt: Agent Allocation (Product-Requirements — PM DeepResearch)

> Validated mapping reference consumed by [`task-decomposition-product-requirements.md`](task-decomposition-product-requirements.md). It defines, for product-requirements deep research: 八段 PR-FAQ skeleton → aspect → persona prompt, the per-tier aspect subset, EA + Strategist **balanced** ownership rationale, the **five hard-gate segments** (段3 4-risks 全 / 段4 ≥3 候选 / 段5 非目标 显式 / 段6 三套指标 / 段8 TM-11 falsification), the "working backwards" PR-FAQ assembly note, and intent overlay. Authority: product-requirements profile [§1 装配契约 / §2 八段骨架 / §5 人格 / TM 分配](../../../docs/pm-deep-research/capabilities/product-requirements.md) + universal frame [spec §3 (personas / 13 TM)](../../../docs/pm-deep-research/pm-deep-research-spec.md) + [interface §2](../../../docs/pm-deep-research/orchestration-interface.md).

## Two personas (each = one inline `aspect_agent_prompt`)

Same two persona prompts as competitive / product-capability / innovation-direction (Lapis has no persona concept; persona = prompt). Cross-cutting quality gates TM-4 (epistemic tagging) + TM-11 (falsifiability) apply to both; **TM-11 is the open-questions aspect's hard gate** under product-requirements:

| key | file | angle | owns (in this profile) | TM weighting (profile §1 表 5) |
|---|---|---|---|---|
| `experience-analyst` | [`../layer2/persona-experience-analyst.md`](../layer2/persona-experience-analyst.md) | user / experience / evidence | **段2 / 段4 / 段5** (3 of 8) | **平衡** — TM-1 / TM-2 / TM-6 / TM-12 |
| `strategist` | [`../layer2/persona-strategist.md`](../layer2/persona-strategist.md) | strategy / trade-off / foresight | **段1 (lead) / 段3 / 段6 / 段7 / 段8** (5 of 8) | **平衡** — TM-3 / TM-5 / TM-9 / TM-11；段8 强制 TM-11 |

> **EA + Strategist 平衡是本 profile 的关键差异**：product-requirements 把 "用户价值" (EA 段2/4/5) 与 "可行性 / 权衡 / 验证" (Strategist 段3/6/8) 平衡承接，因 PRD 前置物的核心契约 = 用户价值 + 工程可行性 + 商业可行性，三者缺一不可。**不写 layer-2 EA-deep / Strategist-futurist 变体** — 通用 EA 与通用 Strategist 已经在各自侧重的 profile 下验证足够，平衡 profile 下二者均不需新变体。

## 八段 PR-FAQ skeleton → aspect → persona

| aspect_id | 段 | persona | research_question (template) | evidence standard → `success_criteria` (profile §2) |
|---|---|---|---|---|
| `pr-faq-frame` | 1 | strategist (lead) | 给定 {subject} + audience，写 ≤300 字 PR：headline / sub-headline / 客户引言 (虚构但符合 JTBD) / 内部 FAQ ≥5 / 外部 FAQ ≥3 | 价值先于功能；**禁止实现细节** (无技术架构 / 模块名 / DB schema 等)；FAQ 计数达标；客户引言带具体场景 (非套话) |
| `jtbd-odi-kano` | 2 | **experience-analyst** | {subject} 的核心 JTBD 是什么? 拆 ≥5 desired outcomes 跑 ODI (Imp + max(0, Imp − Sat))，每个分 Kano 类型 | ≥5 outcomes，每个含 Imp/Sat/Opp + Kano + 证据 ref；估算时强标 TM-4；Opp 公式正确；underserved (>10) ≥1 |
| `cagan-risk-value` | 3 | strategist (**hard gate: 本类全**) | 价值风险：用户付费意愿如何? 评证据等级 high/medium/low + 来源 refs + 应对策略 | 价值风险描述 + 证据等级 + ≥1 来源 ref + 应对策略 + TM-3 |
| `cagan-risk-usability` | 3 | strategist (**hard gate: 本类全**) | 可用性风险：解释性 UI 用户能懂吗? 评证据等级 + 来源 + 应对 | 可用性风险描述 + 证据等级 + ≥1 来源 ref + 应对策略 + TM-3 |
| `cagan-risk-feasibility` | 3 | strategist (**hard gate: 本类全**) | 可行性风险：工程能做出吗 (数据融合 + AI coaching + human-in-loop)? 评证据 + 来源 + 应对 | 可行性风险描述 + 证据等级 + ≥1 来源 ref + 应对策略 + TM-3 |
| `cagan-risk-business` | 3 | strategist (**hard gate: 本类全**) | 商业可行性风险：订阅 + 抽成 能赚钱吗? 评证据 + 来源 + 应对 | 商业可行性风险描述 + 证据等级 + ≥1 来源 ref + 应对策略 + TM-3 |
| `ost-solution-space` | 4 | **experience-analyst** (+ Strategist 借入做可行性快评) | 段2 每 underserved outcome × ≥3 解决方案候选 + 每候选 ≥1 最危险假设 + 既有 / 竞争方案对照 | **≥3 候选** (缺即 gap fail) + ≥2 最危险假设 + 对照 + 每候选 "可行性 + 用户价值 + 风险" 快评 |
| `requirements-fn-nfn-nongoals` | 5 | **experience-analyst** (+ Strategist 借入做非功能/非目标) | 功能需求列表 (每条 outcome 语句 + Kano 标) + 非功能需求 (性能 / 安全 / 合规) + **非目标** (明确 "不做什么" + 为何不做) | 每功能 trace 回段2 outcome (gap fail if not)；**非目标显式列** (gap fail if missing) + 每个 "为何不做" 理由；非功能至少含性能 + 安全 |
| `metrics-tree` | 6 | strategist (+ EA TM-2 metrics-informed) | 主指标 leading (北极星 / 激活 / 完成率) + 次指标 secondary (细分 / 漏斗) + 护栏 guardrails (不能让什么变差) 三套全有 | **主 / 次 / 护栏 全有** (缺一即 gap fail)；每指标 5 字段全 (定义 / 计算方式 / 数据来源 / 成功标准 / 采集频率)；TM-9 杠杆点筛 leading |
| `evidence-table` | 7 **(OPTIONAL)** | 跨人格 TM-4 (strategist 主笔) | 一手 / 二手来源表 + 每条声明置信度 + 4-tier 全套 | **默认不 spin**：4-tier 表由 final-report Phase B 跨段聚合产出 (等价)。仅 standalone evidence pack 时 spin，此时 4-tier 全套 (≥1 each tier 或 显式声明 absence reason) + 每声明 confidence label + TM-4 全员 + **"聚合 prior aspects，不重复 search"**（Lapis `evidence_refs` 不许 cite prior_sources by id，meta-aggregation 自搜会 schema_validation_failed）|
| `open-questions-experiments` | 8 | strategist (**TM-11 hard gate**) | 未决问题清单 + 每个 "为何还未决" + **"靠什么会决"** (discovery sprint / prototype / A-B test 等可执行实验设计) + 下一步 owner / 时间窗 | **每未决问题 TM-11 hard gate**：必须含 "靠什么会决" (实验设计)；缺 → 强制 backfill；不可写 "需要更多研究" 此类空话 |

> **段3 Cagan 4-risks = 4 micro-aspect**：段3 以 4 个 single-class micro-aspect 落地（`cagan-risk-value` / `-usability` / `-feasibility` / `-business`），每个只评 1 类风险、`max_search_calls=4`、bounded 预算强制收敛。**Why**：单个 4-class `cagan-4risks` aspect 在 strategist persona 下 search-saturation（持续搜证不收敛，多次 retries 全失败）；4 个 bounded single-class 任务各自收敛，恢复 dedicated 段3 输出深度。段3 hard gate "4 类全覆盖" = "4 个 micro-aspect 全 present 且各自该类完备"；final-report Phase B 段3 从 4 micro-aspect 装配。段3 仍是 **Strategist 拥有的一个段**（段所有权不变）。

### EA + Strategist balanced persona ownership note

One Lapis aspect = one persona，所以 profile §5 标 "EA + Strategist 双签 / 共同" 在 Lapis 接口下必须收敛到单一 persona 主笔：
- **段1 PR-FAQ** 主笔 = strategist (PR 综合 + 价值主张更适合 strategist trade-off 思维)，但段尾必须留 EA-跨签 check (final-report Phase B 校验：客户引言是否符合 JTBD)。
- **段4 OST** 主笔 = EA (用户视角穷举候选)，但候选可行性快评由 strategist 借入 (final-report Phase B 用段3 4-risks evidence ids 做交叉对照)。
- **段5 需求** 主笔 = EA (功能需求 trace 回 outcome 是用户视角)，但非功能 + 非目标由 strategist 借入 (final-report 校验非目标 + 非功能 完备性)。

### 五个 hard gate 段（profile §3.1 强制项）

本 profile 有 **5 个 hard gate 段**（4 profiles 中最多）：

| 段 | hard gate | 触发缺口 | 处理 |
|---|---|---|---|
| 3 | **Cagan 4-risks 全覆盖**（4 micro-aspect 全 present） | 缺任一 risk-class micro-aspect 或其类未完备 | 缺失 micro-aspect 触发 Phase A backfill；仍缺即整段 0 分 |
| 4 | **OST ≥3 候选 / outcome** | <3 候选 | 整段 0 分 + Phase A backfill |
| 5 | **非目标 显式列** | 缺 "不做什么" 段 | 整段 0 分 + Phase A backfill (PR-FAQ 文化核心) |
| 6 | **三套指标 (主/次/护栏)** | 缺一套 | 整段 0 分 + Phase A backfill |
| 8 | **TM-11 falsification 100%** | 任一未决问题缺 "靠什么会决" | 整段 0 分 + Phase A backfill |

> **Hard-gate density 最高**：4 profile 中最严，因 PRD 前置物是 build-input (写完直接接开发/实验)，缺任一 hard gate 都会让下游开发卡住或返工。Strategist + EA 双侧人格在各自段内必须 enforce 本段 hard gate，不可外推到 final-report Phase C 兜底 (那是兜底机制，不是首选)。

### "working backwards" PR-FAQ assembly note

PR-FAQ 段1 看似在最前，但 Amazon working backwards 实际流程是：先 JTBD + ODI (段2) → 综合 4-risks + OST (段3+4) → 写需求 + metrics (段5+6) → **回填** PR-FAQ (段1 价值主张语句)。

Lapis 8 段并行 (≤3 concurrent) 跑完后，**final-report Phase B 在 chapter assembly 阶段做 PR-FAQ 回填校验**：段1 strategist 在 aspect-research 中写出的 PR-FAQ structure (headline / FAQ 等) 在 final-report 中必须 verify "价值主张语句是否 trace 回段2 ODI outcomes"。这是 **8 段 PR-FAQ 模板**不同于 13 章模板的关键 narrative 流（product-requirements 独有）。

**实践建议**：段1 strategist 在 `success_criteria` 中允许写 "PR-FAQ structure 完整，价值主张语句待 final-report Phase B 用段2 ODI evidence ids 回填校验"，不要求段1 单独 self-contained。

### Intent overlay

- `build` (本期 default): 段1 PR-FAQ 强制 "新产品上线日" 风格 (Amazon 经典 PR-FAQ format); 段4 OST 强制 ≥1 "新建 vs 复用既有平台" 对比; 段6 metrics 主指标含激活/留存 leading metric; 段1 `max_search_calls` +1 (PR-FAQ 借鉴 Amazon-style 范例).
- `improve`: 段2 必含 user-side baseline (current Imp/Sat 数据); 段3 4-risks 重在 value/usability; 段4 OST 强制 ≥1 既有方案 对照; 段6 metrics 含 guardrail "不能让已有指标恶化".

## Per-tier aspect subsets

| tier | aspects | rationale |
|---|---|---|
| `quick` | `pr-faq-frame` + `jtbd-odi-kano` | PR-FAQ + 机会验证 = 最小可决策的 "这事值不值得做" |
| `standard` | + `cagan-risk-{value,usability,feasibility,business}` (段3×4 micro) + `ost-solution-space` (**7 total**) | 加 4 风险 (4 micro-aspect) + 解空间 (可进入 PRD 评审会的最小集) |
| `deep` / `deep_evidence_pack` | + `requirements-fn-nfn-nongoals` + `metrics-tree` + `open-questions-experiments` (**10 mandatory**) + `evidence-table` **(OPTIONAL，仅 `deep_evidence_pack` 或显式 evidence pack 意图)** | 加需求 / metrics / 未决问题 (PRD 前置物全套)；4-tier 证据表默认由 final-report Phase B 聚合，不单独占 aspect |

> quick 2, standard **7** (段3 拆 4 micro), deep **10 mandatory + 段7 OPTIONAL**. Deep `max_agents=11`（10 mandatory + 1 预留可选段7）/ `max_concurrent_agents=3` / `total_timeout_ms=2400000` (11/3=4 waves), per-aspect `timeout_ms=600000` 不变；段3 cagan micro per-aspect `max_search_calls=4`. Standard `max_agents=7` / `total_timeout_ms=1800000` (7/3=3 waves). 详 [`task-decomposition-product-requirements.md`](task-decomposition-product-requirements.md) Step 4.
>
> Quick 段1+段2 (PR-FAQ + ODI) 是 8 段 PR-FAQ 最小集; standard 加段3+段4 (能不能做 + 怎么做) 是评审会最小集; deep 加段5-段8 (需求 + 度量 + 证据 + 实验) 是开发 input deck.

## Budget per aspect (hand off to `task-decomposition-product-requirements.md` Step 4)

每 aspect 自带 `budget { max_turns, max_tool_calls, max_search_calls, timeout_ms }`. Per-tier 关键值: per-aspect `max_search_calls` = 3 (quick) / 5 (standard) / 6 (deep); **段3 cagan micro-aspect (任一 tier) 用更小预算 `max_turns=5` / `max_tool_calls=6` / `max_search_calls=4`**（单 1 类风险，bounded 预算强制收敛，破 search-saturation pathology）; per-aspect `timeout_ms` = **600000 恒**. Top-level `budget`: deep `max_agents=11` / `max_total_model_calls=80` / `max_total_search_calls=64` / `total_timeout_ms=2400000`; standard `max_agents=7` / `max_total_model_calls=42` / `max_total_search_calls=32` / `total_timeout_ms=1800000`. 实测撞 budget → 顺序重试 + prior_sources baseline.

## Provider selection per aspect

`model_provider` 和 `search_provider` 来自用户 allowlists (`available_*_providers`). 指引：
- **User-evidence-heavy** (`jtbd-odi-kano` 找 desired outcomes 用户证据; `ost-solution-space` 找既有方案的用户反馈; `requirements-fn-nfn-nongoals` 找类似产品的 outcome 实现案例) → synthesis provider that surfaces user reviews (e.g. `grok`).
- **Entity-discovery-heavy** (`cagan-risk-*` 4 micro-aspect 找类似产品的 value/usability/feasibility/business 案例; `metrics-tree` 找 best-practice metric trees 来自 SaaS / consumer 公司 product docs) → semantic-discovery provider (e.g. `exa`).
- **Synthesis** (`pr-faq-frame`, `evidence-table`, `open-questions-experiments`) → synthesis provider (e.g. `grok`).
- 只配一个 search provider 时全用之.

> **Search-tuning note**: product-requirements profile does not currently apply `recency=fresh` or `category` fields — search-tuning for `requirements-fn-nfn-nongoals` is structurally fragile (see [`task-decomposition-product-requirements.md`](task-decomposition-product-requirements.md) output-schema note). Revisit when the engine supports per-aspect `search_policy` override.

## Invariants

1. 每 aspect → exactly one persona prompt, inline (verbatim, non-empty, < 64 KiB).
2. Aspects MECE across the 8 段 — 不重叠. **例外：段3 = 4 个 cagan single-class micro-aspect（value/usability/feasibility/business），共属段3、在段3 内部按风险类别 MECE 分区；跨段仍不重叠。**
3. `success_criteria` 携带段的 evidence 标准（profile §2 / §3.1 gap）→ 引擎据此 enforce 证据 bar.
4. `decision_intent` + `subject` + audience 写在 `shared_context.summary` (aspect agents 读 it).
5. Downstream `Evidence.source_type` 用 Lapis 7-value 集; 4-tier credibility 是 Skill 后处理 (interface §4), never an engine enum.
6. **EA + Strategist balanced invariant（按段所有权，非 aspect 计数）**: **段所有权** EA 3 段 (段2/4/5) / Strategist 5 段 (段1/3/6/7/8) — 此平衡是 profile 关键契约，**不变**。段3 以 4 micro-aspect 落地，故 deep **aspect 计数** = EA 3 / Strategist 8 (段1 + 段3×4 + 段6/7/8)，但这只是段3 单段的执行展开，不改段所有权平衡。若某课题 EA-load 不平衡 (如 subject 已有完整 JTBD 不需 EA 段2 deep)，先合段 (如段2+段4 合并)，不要切给 strategist (会破坏用户视角)。
7. **段3 / 段4 / 段5 / 段6 / 段8 是 hard floor aspects** — 缺对应 hard gate (4-risks 全 / ≥3 候选 / 非目标 / 三套指标 / TM-11) → 整段 0 分, 拒绝软化. 4 profile 中 hard-gate density 最高 (5 hard gates), by design.
8. 段1 PR-FAQ 不可包含实现细节 (技术架构 / 模块名 / DB schema 等) — strategist 在 success_criteria 中显式禁止.
9. 段4 OST 候选可借段3 4-risks viability evidence ids 做快评, 减少独立 search 消耗.
