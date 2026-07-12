# Layer 1 Prompt: Agent Allocation (Innovation-Direction — PM DeepResearch)

> Canonical mapping reference consumed by [`task-decomposition-innovation-direction.md`](task-decomposition-innovation-direction.md). It defines, for innovation-direction deep research: 八段 skeleton → aspect → persona prompt, the per-tier aspect subset, segment-2 sole-EA ownership note, segment-6 pre-mortem-three-死因 hard rule, segment-8 TM-11 falsifiability hard gate, intent overlay, and the Strategist-heavy / EA-light TM rationale.

## Two personas (each supplies one persona portion of `instructions`)

Same two persona prompts as competitive / product-capability (MoeResearch has no persona concept; persona = prompt). For every search-enabled aspect, Layer 1 assembles the selected persona, then `prompts/layer1/common/model-search-tool-contract.md`, then a request-specific Run Binding. Cross-cutting quality gates TM-4 (epistemic tagging) + TM-11 (falsifiability) apply to both; **TM-11 is the recommended-bets aspect's hard gate** under innovation-direction:

| key | file | angle | owns (in this profile) | TM weighting |
|---|---|---|---|---|
| `experience-analyst` | [`../../layer2/pm-deep-research/persona-experience-analyst.md`](../../layer2/pm-deep-research/persona-experience-analyst.md) | user / experience / evidence | **段2 only** (unmet outcomes via ODI underserved) | **轻** — TM-1 / TM-6 only |
| `strategist` | [`../../layer2/pm-deep-research/persona-strategist.md`](../../layer2/pm-deep-research/persona-strategist.md) | strategy / trade-off / foresight | **段1, 3, 4, 5, 6, 7, 8** (7 of 8) | **重** — TM-3 / TM-5 / TM-7 / TM-8 / TM-9 / TM-13；段8 强制 TM-11 |

> Strategist-heavy 是本 profile 的关键差异：innovation-direction 关心"跨现状看未来 12-36 月" — 7 段全部需要 strategist 的 trade-off / foresight / falsifiability 思维. EA 仅在段2 unmet outcomes 出场 (用户视角找 underserved). **不写 Strategist-futurist variant** — 复用通用 strategist persona。

## 八段 skeleton → aspect → persona

| id | 段 | persona | question (template) | evidence standard → `success_criteria` |
|---|---|---|---|---|
| `trend-scan` | 1 | strategist | {subject_domain} 在 {time_window_months} 月内市场/技术/竞争 3 类 ≥3 条核心趋势是什么? 每条 Tier 1/2 来源 + 时间窗? | ≥3 趋势 × {market/tech/competition}; 每条 Tier 1/2 + 时间窗 |
| `unmet-outcomes` | 2 | **experience-analyst** (sole EA) | 该赛道 user jobs 拆 ≥3 desired outcomes 跑 ODI, underserved >10 的 outcome 有哪些? 为何 underserved? | ODI Imp/Sat 标 TM-4 practitioner; underserved ≥3; Opp 公式正确 |
| `whitespace-canvas` | 3 | strategist | 用 buyer-validated 或未来 emerging 轴画 value curve, 标白地; 为何无人占据 + 未来 12-36 月谁可能占据? | canvas ≥1 张 (markdown table 或 fenced JSON); 白地附 "为何无人占据" + "谁可能占据" |
| `future-capability-map` | 4 | strategist (+ EA 借段2 unmet 对位) | 跨候选能力类型 (AI / 硬件 / 内容 / 社区 / 数据) 逐项映射: 能干什么 + 我方现状承载力 + 与段2 unmet 对位? | ≥2 候选能力类型; 每候选 "能干什么" 必须 Tier 1/2 技术依据; 与段2 unmet 对位标注 |
| `disruption-defensibility` | 5 | strategist | 用 M-Disruption 判 sustaining vs disruptive; 用 M-Cagan-4Risks 商业可行性维评护城河/锁定/规模效应; 每威胁附依据? | 每威胁 Christensen 判定 + 依据; 防御性维度 (护城河/锁定/规模效应) 各附依据 |
| `pre-mortem-top3` | 6 | strategist | 假设 12-18 月后已失败, 列三大死因 (Tigers / Paper Tigers / Elephants), 每死因附机制 + 触发条件? | **≥3 死因强制**; 每个 = (机制 + 触发条件); 拒绝 hand-wave; TM-8 强制 |
| `build-cost-feasibility` | 7 | strategist (+ EA changelog借入) | 同能力域对手历史迭代节奏 = build-cost 下界代理; build-cost 区间 + 4 风险中可行性/商业可行性维评? | build-cost 显式区间 (如 "6-12 月达成基础能力") + TM-4 标证据等级; ≥1 changelog 时间线 |
| `recommended-bets` | 8 | strategist (**TM-11 hard gate**) | 综合段2-7, 给 1-3 个可证伪下注方向, 每个 4 风险评级 + 显性权衡 + 验证条件? | **TM-11 强制门** — 每下注 ≥1 falsifiability 条件 (leading indicator + 阈值); TM-5 显性权衡 ("选 X = 放弃 Y"); 4 风险 (TM-3) 评级 |

### 段2 sole-EA persona ownership note

One MoeResearch aspect = one persona, 所以 profile §5 标 "EA 看 unmet" 在本 profile 收敛到段2 一个 aspect. 段4 中 "对位" 维度 (能力候选 vs unmet) 由 `future-capability-map` (strategist) 通过 `context.prior_sources` 引用段2 EA aspect 输出 fold-in. **不另起 dedicated EA aspect for 段4 对位** (避免 8→9 aspect 增预算 + 增 wave).

### 段6 pre-mortem 强制三死因 hard rule

`pre-mortem-top3` 的 `success_criteria` 必须显式列：
- 死因数量 ≥3 (Christensen Tigers / Paper Tigers / Elephants 三型可对应, 但非强制对应);
- 每死因 = (具体机制 + 触发条件);
- 拒绝 hand-wave 风险 (如 "市场不接受" / "团队执行不力" / "竞争激烈" 这类无机制无触发的泛泛风险);
- TM-8 强制 (Cagan 4 风险评级).

未达标 → Phase A (final-report) 触发段6 backfill 一轮; 仍不达标 → 标缺口 + 降置信, 不为分数注水.

### 段8 TM-11 hard gate

`recommended-bets` 的 `success_criteria` 必须显式列：
- 每推荐下注 ≥1 "什么条件下错" (leading indicator + 阈值, 如 "AI 教练赛道 12 月内 Apple/OpenAI 未发 health agent 通用 SDK → 押 vertical Coach 不投空");
- 每推荐下注 ≥1 显性权衡 (TM-5 "选 X = 放弃 Y");
- 每推荐下注 4 风险 (value / usability / feasibility / business viability) 评级 (high/medium/low + 一句依据).

缺 falsifiability 条件 → aspect 整段 0 分 (TM-11 是 floor 不是 soft preference). **未来下注的核心质量在"如何知道押错了"**.

### Intent overlay

- `ai-upgrade` (本期 default): 段1 / 段4 / 段8 每 aspect `max_search_calls` +1; 段4 强制 ≥1 AI capability candidate; 段1 强制 ≥1 trend 来自技术成熟度类来源.
- `enter`: 段4 / 段5 / 段7 加重; `context.summary` 强调 "新赛道, 现状承载力可能为零"; 段4 our_carry_capacity 字段允许显式标 "none / minimal".
- `differentiate`: 段3 / 段5 / 段8 加重; 段8 强制显性权衡 (TM-5); 段3 canvas 必须含 buyer-validated 轴而非纯 emerging 轴.

## Per-tier aspect subsets

| tier | aspects | rationale |
|---|---|---|
| `quick` | `trend-scan`, `recommended-bets` | 趋势 + 收敛下注 = 最小可决策 ("现在大方向是什么, 押哪 1-2 个" 快速读) |
| `standard` | + `unmet-outcomes`, `whitespace-canvas`, `future-capability-map` (5 total) | 加 unmet + 白地 + 未来能力 (决策依据 + 押注根据) |
| `deep` / `deep_evidence_pack` | + `disruption-defensibility`, `pre-mortem-top3`, `build-cost-feasibility` (**8 total**) | 加颠覆/可防御性 + pre-mortem (三死因强制) + build-cost; 段8 TM-11 hard gate 全 tier 启用 |

> Per-tier counts: quick = 2, standard = 5, deep = 8 — all 8 segments are expanded in Deep. Deep `max_agents=8` / `max_concurrent_agents=3` / `total_timeout_ms=1800000` (3 waves), per-aspect `timeout_ms=600000`. 详 [`task-decomposition-innovation-direction.md`](task-decomposition-innovation-direction.md) Step 4.
>
> Quick 段1+段8 跳过中间段 6 个看似激进, 但 quick 短路径本就是 "headline + bet收敛" 的最小可决策；中间 6 段属 standard+ 的"为什么这么押"展开.

## Limits per aspect (hand off to `task-decomposition-innovation-direction.md` Step 4)

每 aspect 自带 `limits { max_turns, max_tool_calls, max_search_calls, timeout_ms }`. Per-tier 关键值: per-aspect `max_search_calls` = 3 (quick) / 5 (standard) / 6 (deep); per-aspect `timeout_ms` = **600000 恒**. Top-level `limits`: deep `max_total_model_calls=60` / `max_total_search_calls=50`. If an aspect reaches limits, retry sequentially once with `context.prior_sources`; do not raise the search cap.

## Provider selection per aspect

`model_provider` 和 `search_provider` 来自用户 allowlists (`available_*_providers`). 指引：
- **Entity-discovery-heavy** (`trend-scan` 找 emerging 玩家; `future-capability-map` 找新能力 / 跨界玩家如 OpenAI / Apple; `disruption-defensibility` 找潜在颠覆者) → semantic-discovery provider (e.g. `exa`).
- **User-evidence-heavy** (`unmet-outcomes` 找 underserved outcome 用户证据) → synthesis provider that 能 surface 大量 user reviews (e.g. `grok`).
- **Synthesis** (`whitespace-canvas`, `pre-mortem-top3`, `build-cost-feasibility`, `recommended-bets`) → synthesis provider (e.g. `grok`).
- 只配一个 search provider 时全用之.

## Invariants

1. 每 search-enabled aspect → exactly one persona prompt, then `prompts/layer1/common/model-search-tool-contract.md`, then a request-specific Run Binding, inline (non-empty, < 64 KiB).
2. Aspects MECE across the 8 段 — 不重叠.
3. `success_criteria` 携带段的 evidence 标准→ 引擎据此 enforce 证据 bar.
4. `decision_intent` + `subject_domain` + `time_window_months` 写在 `context.summary` (aspect agents 读 it).
5. Evidence source type 与 evidence-level confidence 在 candidate selection 后由 host 拥有；4-tier credibility 是 Skill 后处理，绝非模型输出字段。
6. **Strategist-heavy invariant**: 8 aspects 中 7 个 (段1/3/4/5/6/7/8) 由 strategist 拥有; 1 个 (段2) 由 EA 拥有. 若某课题 strategist-load 不平衡 (如 subject_domain 已知不需 trend scan), 先合段 (如段1 折叠进段4), 不要切给 EA.
7. **段6 + 段8 是 hard floor aspect** — 缺 (3 死因 / falsifiability) → 整段 0 分, 拒绝软化.
8. 段7 build-cost-feasibility 的 changelog 证据可从段1 trend-scan / 段4 future-capability-map 的 prior evidence_index 借用, 减少独立 search 消耗。

## Run Binding handoff

For every search-enabled aspect, persona selection is followed by the complete inline assembly order: selected persona Markdown, then `prompts/layer1/common/model-search-tool-contract.md`, then the request-specific Run Binding. The binding is derived from that aspect and `policy.search` according to `moe.run_binding.v1`; it carries only semantic `allowed_*` intent choices, safe defaults, literal aspect ID/name anchors, and evidence-closure hints. It must not expose provider routing, budgets, domains, raw policy tool fields, or credentials.

This three-part order is mandatory for every search-enabled aspect. The fixed-category rule is profile-neutral: fixed `academic` permits `general` or `academic`; an unset category permits the full source-focus vocabulary.
