# Layer 1 Prompt: Agent Allocation (Product-Requirements — PM DeepResearch)

> Mapping reference consumed by [`task-decomposition-product-requirements.md`](task-decomposition-product-requirements.md). It defines, for product-requirements deep research: 八段 PR-FAQ skeleton → aspect → persona prompt, the per-tier aspect subset, EA + Strategist **balanced** ownership rationale, the **five hard-gate segments** (段3 4-risks 全 / 段4 ≥3 候选 / 段5 非目标 显式 / 段6 三套指标 / 段8 TM-11 falsification), the "working backwards" PR-FAQ assembly note, and intent overlay.

## Two personas (each supplies one persona portion of `instructions`)

Same two persona prompts as competitive / product-capability / innovation-direction (MoeResearch has no persona concept; persona = prompt). For every search-enabled aspect, Layer 1 assembles the selected persona, then `prompts/layer1/common/model-search-tool-contract.md`, then a request-specific Run Binding. Cross-cutting quality gates TM-4 (epistemic tagging) + TM-11 (falsifiability) apply to both; **TM-11 is the open-questions aspect's hard gate** under product-requirements:

| key | file | angle | owns (in this profile) | TM weighting |
|---|---|---|---|---|
| `experience-analyst` | [`../../layer2/pm-deep-research/persona-experience-analyst.md`](../../layer2/pm-deep-research/persona-experience-analyst.md) | user / experience / evidence | **段2 / 段4 / 段5** (3 of 8) | **平衡** — TM-1 / TM-2 / TM-6 / TM-12 |
| `strategist` | [`../../layer2/pm-deep-research/persona-strategist.md`](../../layer2/pm-deep-research/persona-strategist.md) | strategy / trade-off / foresight | **段1 (lead) / 段3 / 段6 / 段7 / 段8** (5 of 8) | **平衡** — TM-3 / TM-5 / TM-9 / TM-11；段8 强制 TM-11 |

> **EA + Strategist 平衡是本 profile 的关键差异**：product-requirements 把 "用户价值" (EA 段2/4/5) 与 "可行性 / 权衡 / 验证" (Strategist 段3/6/8) 平衡承接，因 PRD 前置物的核心契约 = 用户价值 + 工程可行性 + 商业可行性，三者缺一不可。**不写 layer-2 EA-deep / Strategist-futurist 变体** — 复用通用 EA + 通用 Strategist persona。

## 八段 PR-FAQ skeleton → aspect → persona

| id | 段 | persona | question (template) | evidence standard → `success_criteria` |
|---|---|---|---|---|
| `pr-faq-frame` | 1 | strategist (lead) | 给定 {subject} + audience，写 ≤300 字 PR：headline / sub-headline / 客户引言 (虚构但符合 JTBD) / 内部 FAQ ≥5 / 外部 FAQ ≥3 | 价值先于功能；**禁止实现细节** (无技术架构 / 模块名 / DB schema 等)；FAQ 计数达标；客户引言带具体场景 (非套话) |
| `jtbd-odi-kano` | 2 | **experience-analyst** | {subject} 的核心 JTBD 是什么? 拆 ≥5 desired outcomes 跑 ODI (Imp + max(0, Imp − Sat))，每个分 Kano 类型 | ≥5 outcomes，每个含 Imp/Sat/Opp + Kano + 证据 ref；估算时强标 TM-4；Opp 公式正确；underserved (>10) ≥1 |
| `cagan-four-risks` | 3 | strategist (**hard gate: all classes**) | 价值 / 可用性 / 可行性 / 商业风险及应对 | 四类各有证据等级、≥1 来源 ref、应对策略 + TM-3 |
| `ost-solution-space` | 4 | **experience-analyst** (+ Strategist 借入做可行性快评) | 段2 每 underserved outcome × ≥3 解决方案候选 + 每候选 ≥1 最危险假设 + 既有 / 竞争方案对照 | **≥3 候选** (缺即 gap fail) + ≥2 最危险假设 + 对照 + 每候选 "可行性 + 用户价值 + 风险" 快评 |
| `requirements-and-metrics` | 5+6 | strategist | 功能 / 非功能 / 非目标 + 主/次/护栏指标 | 每功能 trace outcome；显式非目标；三套指标各有定义、计算、数据源、阈值、频率 |

| `open-questions-experiments` | 8 | strategist (**TM-11 hard gate**) | 未决问题清单 + 每个 "为何还未决" + **"靠什么会决"** (discovery sprint / prototype / A-B test 等可执行实验设计) + 下一步 owner / 时间窗 | **每未决问题 TM-11 hard gate**：必须含 "靠什么会决" (实验设计)；缺 → 强制 backfill；不可写 "需要更多研究" 此类空话 |

> **段3 Cagan 4-risks**：一个 `cagan-four-risks` aspect 覆盖四类风险；缺失证据成为 gap、assumption 或 follow-up。

### EA + Strategist balanced persona ownership note

One MoeResearch aspect = one persona，所以 profile §5 标 "EA + Strategist 双签 / 共同" 在 MoeResearch 接口下必须收敛到单一 persona 主笔：
- **段1 PR-FAQ** 主笔 = strategist (PR 综合 + 价值主张更适合 strategist trade-off 思维)，但段尾必须留 EA-跨签 check (final-report Phase B 校验：客户引言是否符合 JTBD)。
- **段4 OST** 主笔 = EA (用户视角穷举候选)，但候选可行性快评由 strategist 借入 (final-report Phase B 用段3 4-risks evidence ids 做交叉对照)。
- **段5 需求** 主笔 = EA (功能需求 trace 回 outcome 是用户视角)，但非功能 + 非目标由 strategist 借入 (final-report 校验非目标 + 非功能 完备性)。

### 五个 hard gate 段

This profile has **5 hard-gate segments** because product-requirements research is meant to become PRD/development input, not a strategy discussion:

| 段 | hard gate | 触发缺口 | 处理 |
|---|---|---|---|
| 3 | **Cagan 4-risks 全覆盖** | 缺任一 risk class | Phase A backfill；仍缺即整段 0 分 |
| 4 | **OST ≥3 候选 / outcome** | <3 候选 | 整段 0 分 + Phase A backfill |
| 5 | **非目标 显式列** | 缺 "不做什么" 段 | 整段 0 分 + Phase A backfill (PR-FAQ 文化核心) |
| 6 | **三套指标 (主/次/护栏)** | 缺一套 | 整段 0 分 + Phase A backfill |
| 8 | **TM-11 falsification 100%** | 任一未决问题缺 "靠什么会决" | 整段 0 分 + Phase A backfill |

> **Hard-gate density 最高**：四个 capabilities 中最严，因 PRD 前置物是 build-input (写完直接接开发/实验)，缺任一 hard gate 都会让下游开发卡住或返工。Strategist + EA 双侧人格在各自段内必须 enforce 本段 hard gate，不可外推到 final-report Phase C 兜底 (那是兜底机制，不是首选)。

### Working backwards PR-FAQ assembly note

PR-FAQ 段1 看似在最前，但 Amazon working backwards 实际流程是：先 JTBD + ODI (段2) → 综合 4-risks + OST (段3+4) → 写需求 + metrics (段5+6) → **回填** PR-FAQ (段1 价值主张语句)。

MoeResearch aspects 跑完后，**final-report Phase B 在 chapter assembly 阶段做 PR-FAQ 回填校验**：段1 strategist 在 aspect-research 中写出的 PR-FAQ structure (headline / FAQ 等) 在 final-report 中必须 verify "价值主张语句是否 trace 回段2 ODI outcomes"。这是 product-requirements 8-section template 的关键 narrative 流。

**实践建议**：段1 strategist 在 `success_criteria` 中允许写 "PR-FAQ structure 完整，价值主张语句待 final-report Phase B 用段2 ODI evidence ids 回填校验"，不要求段1 单独 self-contained。

### Intent overlay

- `build` (本期 default): 段1 PR-FAQ 强制 "新产品上线日" 风格 (Amazon 经典 PR-FAQ format); 段4 OST 强制 ≥1 "新建 vs 复用既有平台" 对比; 段6 metrics 主指标含激活/留存 leading metric.
- `improve`: 段2 必含 user-side baseline (current Imp/Sat 数据); 段3 4-risks 重在 value/usability; 段4 OST 强制 ≥1 既有方案 对照; 段6 metrics 含 guardrail "不能让已有指标恶化".

### Sports / fitness / health overlay

When `subject_domain` or the request involves sports, fitness, training, recovery, wearables, nutrition, weight, injury, return-to-play, REDs, wellness, or health claims, layer these requirements onto the same aspect allocation:

| Segment | Added requirement |
|---|---|
| 段1 PR-FAQ | Ban over-claiming: no diagnosis / treatment / injury-prevention / guaranteed recovery / guaranteed weight-loss copy without direct regulatory and clinical evidence. |
| 段3 Cagan risks | Include safety and regulatory implications inside value / usability / feasibility / business risk where relevant. |
| 段5 Requirements | Include Safety Boundary, No-go Claims, and escalation / referral rules for high-risk states. |
| 段6 Metrics | Add health/safety guardrails: adverse feedback, pain reports, training interruption, overtraining signals, professional referral rate, explanation error rate, or comparable domain metrics. |
| 段8 Open questions | Include measurement validity, derived metric validity, and action validity tests when wearable data drives recommendations. |

This overlay does not add a new persona or aspect. It tightens success criteria for the existing segments.

## Per-tier aspect subsets

| tier | aspects | rationale |
|---|---|---|
| `quick` | `pr-faq-frame` + `jtbd-odi-kano` | PR-FAQ + 机会验证 = 最小可决策的 "这事值不值得做" |
| `standard` | + `cagan-four-risks`, `ost-solution-space` (**4 total**) | 加 4 风险 + 解空间 |
| `deep` | + `requirements-and-metrics`, `open-questions-experiments` (**6 total**) | 加需求 / metrics / 未决问题；`evidence_pack` 由 final-report 聚合 |

> Per-tier counts: quick = 2, standard = 4, deep = 6.
>
> Quick 段1+段2 (PR-FAQ + ODI) 是经典最小集 ("有没有用户痛点 + 写得出一段 PR 吗"); standard 加段3+段4 (能不能做 + 怎么做) 是评审会最小集; deep 加段5-段8 (需求 + 度量 + 证据 + 实验) 是开发 input deck.

## Limits

Apply explicit resource constraints in the user prompt in preference to the selected tier, then tighten against operator ceilings; do not silently replace the user's constraints.

## Provider selection per aspect

`model_provider` 和 `search_provider` 来自用户 allowlists (`available_*_providers`). 指引：
- **User-evidence-heavy** (`jtbd-odi-kano`, `ost-solution-space`, `requirements-and-metrics`) → synthesis provider that surfaces user reviews (e.g. `grok`).
- **Entity-discovery-heavy** (`cagan-four-risks`, `requirements-and-metrics`) → semantic-discovery provider (e.g. `exa`).
- **Synthesis** (`pr-faq-frame`, `open-questions-experiments`) → synthesis provider (e.g. `grok`).
- 只配一个 search provider 时全用之.

## Invariants

1. 每 search-enabled aspect → exactly one persona prompt, then `prompts/layer1/common/model-search-tool-contract.md`, then a request-specific Run Binding, inline (non-empty, < 64 KiB).
2. Aspects MECE across the 8 段 — 不重叠。
3. `success_criteria` 携带段的 evidence 标准→ 引擎据此 enforce 证据 bar.
4. `decision_intent` + `subject` + audience 写在 `context.summary` (aspect agents 读 it).
5. Evidence source type 与 evidence-level confidence 在 candidate selection 后由 host 拥有；4-tier credibility 是 Skill 后处理，绝非模型输出字段。
6. **EA + Strategist balanced invariant**: EA owns segments 2/4; strategist owns 1/3/5–8. `cagan-four-risks` stays one aspect; execution count follows the selected tier.
7. **段3 / 段4 / 段5 / 段6 / 段8 是 hard floor aspects** — 缺对应 hard gate (4-risks 全 / ≥3 候选 / 非目标 / 三套指标 / TM-11) → 整段 0 分, 拒绝软化. 4 profile 中 hard-gate density 最高 (5 hard gates), by design.
8. 段1 PR-FAQ 不可包含实现细节 (技术架构 / 模块名 / DB schema 等) — strategist 在 success_criteria 中显式禁止.
9. 段4 OST 候选可借段3 4-risks viability evidence ids 做快评，减少独立 search 消耗。

## Run Binding handoff

For every search-enabled aspect, persona selection is followed by the complete inline assembly order: selected persona Markdown, then `prompts/layer1/common/model-search-tool-contract.md`, then the request-specific Run Binding. The binding is derived from that aspect and `policy.search` according to `moe.run_binding.v1`; it carries only semantic `allowed_*` intent choices, safe defaults, literal aspect ID/name anchors, and evidence-closure hints. It must not expose provider routing, budgets, domains, raw policy tool fields, or credentials.

This three-part order is mandatory for every search-enabled aspect. The fixed-category rule is profile-neutral: fixed `academic` permits `general` or `academic`; an unset category permits the full source-focus vocabulary.
