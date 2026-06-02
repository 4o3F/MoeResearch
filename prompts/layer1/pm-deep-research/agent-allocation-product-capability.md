# Layer 1 Prompt: Agent Allocation (Product-Capability — PM DeepResearch)

> Canonical mapping reference consumed by [`task-decomposition-product-capability.md`](task-decomposition-product-capability.md). It defines, for product-capability deep research: 六段 skeleton → aspect → persona prompt, the per-tier aspect subset, segment-5 persona ownership note, build-intent overlay on 段6, and the EA-heavy / Strategist-light TM rationale. Authority: product-capability profile [§1 装配契约 / §2 六段骨架 / §5 人格 / TM 分配](../../../docs/pm-deep-research/capabilities/product-capability.md) + universal frame [spec §3 (personas / 13 TM)](../../../docs/pm-deep-research/pm-deep-research-spec.md) + [interface §2](../../../docs/pm-deep-research/orchestration-interface.md).

## Two personas (each = one inline `aspect_agent_prompt`)

Same two persona prompts as competitive (Lapis has no persona concept; persona = prompt). Cross-cutting quality gates TM-4 (epistemic tagging) + TM-11 (falsifiability) apply to both:

| key | file | angle | owns (in this profile) | TM weighting (profile §0 表 5) |
|---|---|---|---|---|
| `experience-analyst` | [`../layer2/persona-experience-analyst.md`](../layer2/persona-experience-analyst.md) | user / experience / evidence | **段1-4** (capability domain JTBD / single-domain teardown / experience paths / Kano in-domain) | **重** — TM-1 / TM-2 / TM-6 / TM-10 / TM-12 |
| `strategist` | [`../layer2/persona-strategist.md`](../layer2/persona-strategist.md) | strategy / trade-off / foresight | **段5-6** (ODI in-domain / benchmark + build-cost + upgrade) | **轻** — TM-9 (杠杆点) + TM-13 (前瞻) |


## 六段 skeleton → aspect → persona

| aspect_id | 段 | persona | research_question (template) | evidence standard → `success_criteria` (profile §2) |
|---|---|---|---|---|
| `capability-domain-jtbd` | 1 | **experience-analyst** | 在 {capability_domain} 内, {target} 服务的 user jobs 是什么? 能力域边界含什么 / 排除什么 / 为何? | 能力域 boundary + ≥1 排除理由; ≥3 job statement (situation→motivation→outcome) |
| `capability-teardown-deep` | 2 | experience-analyst | {target} + 2-3 个 best-in-class benchmark 在该能力域内的功能 / 步骤上各得几分（含步数 / 错误率 / 时延）? | 每行附 visual_evidence 或操作步数; 纯文字 = 标 assumption; teardown matrix ≥1 张 (markdown table 或 fenced JSON) |
| `experience-paths-breakpoints` | 3 | experience-analyst | 该能力域 user journey ≥1 张完整路径图; ≥3 断点 (drop / 卡顿 / 错误 / 困惑), 每断点 visual + ≥3 同模式 Tier-3 用户证据? | 每断点 ≥1 visual_evidence (screenshot/video/app-store URL) + ≥3 条同模式评论 (同模式才非孤例, 否则标"单证据不立") |
| `kano-in-domain` | 4 | experience-analyst | 段2 功能 + 段3 断点按 Must-be / Performance / Attractive 怎么分? | 分级有用户证据 or 显式标 TM-4 practitioner 诠释 |
| `odi-in-domain` | 5 | **strategist** (EA-sourced data folded in) | 段1 user jobs 拆 ≥3 desired outcomes 跑 ODI（Imp + max(0, Imp − Sat)）? underserved >10 的 outcome 喂入段6 | Imp/Sat 估算时标 TM-4; Opp 公式正确; underserved 列 ≥1 |
| `benchmark-buildcost-upgrade` | 6 | strategist | best-in-class 2-3 对手该能力域的 changelog / 版本时间线 + build-cost 估算 + pre-mortem 升级方向? | benchmark 选择理由 (why best-in-class); changelog datable (≥1 张时间线); build-cost 区间; pre-mortem 三死因 |

### 段5 persona ownership note

One Lapis aspect = one persona, 所以 profile §5 标段5 "Strategist + EA"（Strategist 结论 + EA 数据）不能字面切. **`odi-in-domain` 由 `strategist` 拥有**, 其 `research_question` + `success_criteria` 强制要求 Imp/Sat 估算依据从段3 体验路径用户证据 + 段4 Kano 分级 (EA prior aspect 已产出) 回引 → 通过 `shared_context.prior_sources` 喂入. **不另起 dedicated EA-ODI aspect**（避免拆 6→7 aspect 增预算 + 增 wave）.

### Build-intent overlay (decision_intent = build)

段6 `benchmark-buildcost-upgrade` 的 `success_criteria` 必须包括：从 best-in-class 2-3 对手 release notes / App Store version history 拉 datable 版本时间线、估算 build-cost 区间（spec §3「迭代节奏与建设成本」, TM-12 say-vs-do）. supporting evidence `url` 必须指向 version-history / release-notes 页. 与 competitive `build-cost-version-history` aspect 一致; product-capability fold 进段6 (不另起独立 aspect).

## Per-tier aspect subsets

| tier | aspects | rationale |
|---|---|---|
| `quick` | `capability-domain-jtbd`, `capability-teardown-deep` | 能力域 + 单域 teardown = 最小可决策（能力做得多好的快速读）|
| `standard` | + `experience-paths-breakpoints`, `kano-in-domain` (4 total) | 加体验路径 + Kano（断点 visual 强制启用）|
| `deep` / `deep_evidence_pack` | + `odi-in-domain`, `benchmark-buildcost-upgrade` (**6 total**) | 加 ODI 域内 + benchmark + build-cost; 体验路径图 ≥1 + 断点 visual ≥每断点 1 张 (profile §3.2 floor) |

> Per-tier 计数 vs. competitive：quick 2 (vs 2), standard 4 (vs 4), deep **6 (vs 5)** — 多 1 段（段5 ODI 域内单独成 aspect; competitive 的 ODI 与 capability matrix 同 aspect 合并）. Deep `max_agents=6` / `max_concurrent_agents=3` / `total_timeout_ms=1200000` (2 waves), per-aspect `timeout_ms=600000` 不变. 详 [`task-decomposition-product-capability.md`](task-decomposition-product-capability.md) Step 4.

## Budget per aspect (hand off to `task-decomposition-product-capability.md` Step 4)

每 aspect 自带 `budget { max_turns, max_tool_calls, max_search_calls, timeout_ms }`. Per-tier 关键值: per-aspect `max_search_calls` = 3 (quick) / 6 (standard) / 8 (deep); per-aspect `timeout_ms` = **600000 恒**. Top-level `budget` 同 v2.1 plan §5: deep `max_total_model_calls=80` / `max_total_search_calls=60` (适度上调 from competitive deep 70/56, 因 6 段比 5 段多 1 aspect).

## Provider selection per aspect

`model_provider` 和 `search_provider` 来自用户 allowlists (`available_*_providers`). 指引：
- **Entity-discovery-heavy** (`capability-domain-jtbd` 边界论证 / find non-obvious user jobs; `benchmark-buildcost-upgrade` for best-in-class 选择) → semantic-discovery provider (e.g. `exa`).
- **User-evidence-heavy** (`experience-paths-breakpoints` 找断点同模式评论; `kano-in-domain` 找用户证据) → synthesis provider that 能 surface 大量 user reviews (e.g. `grok`).
- **Synthesis** (`capability-teardown-deep`, `odi-in-domain`) → synthesis provider (e.g. `grok`).
- 只配一个 search provider 时全用之.


## Invariants

1. 每 aspect → exactly one persona prompt, inline (verbatim, non-empty, < 64 KiB).
2. Aspects MECE across the 6 段 — 不重叠.
3. `success_criteria` 携带段的 evidence 标准（profile §2 / §3.1 gap）→ 引擎据此 enforce 证据 bar.
4. `decision_intent` + `capability_domain` 写在 `shared_context.summary` (aspect agents 读 it).
5. Downstream `Evidence.source_type` 用 Lapis 7-value 集; 4-tier credibility 是 Skill 后处理 (interface §4), never an engine enum.
6. EA-heavy invariant: 6 aspects 中 4 个（段1-4）由 EA 拥有; 2 个（段5-6）由 Strategist 拥有. 若某课题 EA-load 不平衡（如能力域已知不需找 jobs）, 先合段（如段1 折叠进段2）, 不要切给 Strategist.
