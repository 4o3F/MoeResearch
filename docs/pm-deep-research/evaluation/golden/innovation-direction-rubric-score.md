# 评分 · innovation-direction AI Coach golden（纯引擎 8/8）

> 对象：**纯引擎 8/8 golden** `r4c-v22-merged-8of8.result.json`（5 deep_research + 3 aspect_research 补跑）。
> 引擎=上游 `9db7464` verbatim；模型 gpt-5.5 / 搜索 grok。
> 配置：per-aspect `max_search_calls=6` + 全局 `search_policy.recency=fresh` + `max_results_per_query=5`（无 depth/content_level/category）。
> 注：deep tier `max_search_calls=6` is required for this capability — the `recommended-bets` synthesis aspect reaches ~6 search calls when `recency=fresh` is active; lower values cause execution abort (see task-decomposition-innovation-direction.md budget notes).

## 总分：24 / 24 = 锚点持平、**无回归** ✅

| # | 维度 | 得分 | 依据 |
|---|---|:--:|---|
| A1 | 引用充分性 | **2** | 38/38 findings 全挂 evidence_refs；dangling refs = 0 |
| A2 | 引用准确性 | **2** | 74 evidence 含 official 45/doc 6/news 11/forum 11/unknown 1；evidence_refs 真实页面（Strava press / Garmin / Apple HealthKit / Runna / TrainingPeaks docs / industry blog）；recommended-bets retry 干净（无 mutation 复发）|
| A3 | 无支撑率 | **2** | finding_type + epistemic_status (TM-4) 全标；ODI/Kano 全 estimated；assumption inline |
| A4 | 来源质量与多样性 | **2** | 74 evidence / 23 domains / 5 source_types；每 aspect ≥5 evidence（min pre-mortem=5，max unmet-outcomes=15；其余 7-11）；`recency=fresh` 提升 dated changelog 命中率，enrichment +11 evidence / +1 domain vs baseline |
| A5 | 置信度校准+TM-4 | **2** | 全 medium + 逐 finding epistemic / 逐 evidence confidence |
| B1 | 骨架覆盖 (8 段) | **2** | 8 aspect 全 dedicated：trend-scan / unmet-outcomes / whitespace-canvas / future-capability-map / disruption-defensibility / pre-mortem-top3 / recommended-bets / build-cost-feasibility |
| B2 | 真实研究单元 | **2** | 各 aspect 独立研究单元 + JTBD 锚定 |
| B3 | 核心证据机制 | **2** | ODI matrix + capability map + Christensen 分级 + pre-mortem 3 死因（recommended-bets 3 bets 全带 why + falsification trigger）|
| B4 | 机会量化严谨 | **2** | ODI 公式 + 阈值 + estimated 标注（unmet-outcomes 5 findings 含 underserved 列表）|
| C1 | 视觉证据 (战略图类) | **2** | ≥7 战略图（strategy canvas / capability map / ODI matrix / Christensen / pre-mortem 死因 / P0/P1/P2 bet 表 / trend scan）由 Tier-1 official(45) 支撑；innovation-direction 使用 13-section narrative report 战略图类型判据（≥5 + Tier-1 支撑）|
| C2 | 专家思维动作 TM | **2** | TM-1/2/4/6/7/11(bets falsifiable) 系统体现 |
| C3 | 可落地 | **2** | recommended-bets P0/P1/P2 分层下注 + build-cost-feasibility roadmap |
| | **合计** | **24 / 24** | 满分 ✅ |

## 结论：无回归，满分

- A4=2：`recency=fresh` 提升 changelog 命中率，+11 evidence / +1 domain 增强来源多样性，score cap already at 2。
- C1=2 与 search-tuning 解耦——innovation-direction 使用战略图类型判据（≥5 + Tier-1 支撑），与 `recency=fresh` 的 evidence enrichment 无关。
- floor 全过 + TM-11 hard gate（推荐 bets 全 falsifiable）；dangling refs = 0。

## 关键工程发现：deep tier `max_search_calls=6`

| Capability | deep per-aspect cap | Notes |
|---|:---:|---|
| competitive (5-aspect deep) | **4** | `depth=high_recall` + higher cap causes execution abort; cap=4 converges cleanly |
| product-capability (6-aspect deep) | **3** | cap=3 sufficient across multiple runs |
| innovation-direction (8-aspect deep) | **6 ✅ required** | `recommended-bets` is a synthesis aspect consuming 7 prior outputs; with `recency=fresh` active, search appetite reaches ~6. cap=5 → execution abort on `recommended-bets` (budget_exceeded). cap=6 resolves cleanly. |

**`recommended-bets` search appetite**：this is the synthesis-heavy aspect of innovation-direction — it folds in prior_sources from all 7 earlier aspects and performs final bet synthesis. `recency=fresh` prompt-hint pushes model search appetite to ~6. When adding more synthesis-heavy aspects in future, re-validate the cap.

## Validated configuration

- `search_policy.recency=fresh` + `max_results_per_query=5`
- per-aspect deep: `max_search_calls=6`
- top-level: `max_total_search_calls=50` (8 aspects × 6 max = 48 + small headroom)
- **Do NOT** set `depth`, `content_level`, or `category` globally.
- `trend-scan` / `unmet-outcomes` first retry may hit transient search latency; second retry succeeds — not a config issue.

Reflected in `task-decomposition-innovation-direction.md` Step 4 budget notes.
