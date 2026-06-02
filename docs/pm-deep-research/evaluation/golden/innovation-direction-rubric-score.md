# R4-c v2.2 评分 · AI Coach innovation-direction — search-tuning canonical（纯引擎 8/8 golden）

> 对象：**纯引擎 8/8 golden** `r4c-v22-merged-8of8.result.json`（5 deep + 3 aspect_research 补跑；recommended-bets 在 cap=5 持续 hard-kill 后改 cap=6 一次成）。
> 引擎=上游 `9db7464` verbatim；模型 gpt-5.5（CPA）/ 搜索 grok。
> 配置（R4-c canonical for v2.2，与 v2.0/v2.1 不同）：per-aspect `max_search_calls=6`（**不是** v2.0 cap=4 / v2.1 cap=3——v2.2 综合 bets aspect 自然 appetite 高，需 cap=6 才不被 hard-kill）+ 全局 `search_policy.recency=fresh` + `max_results_per_query=5`（**无** depth/content_level/category）。
> 锚点：rerun9 = **24/24 满分**（C1=2 战略图 ≥7、TM-11 hard gate 100%）。
> 闸门：R1 引擎漂移 — 任一维跌破锚点即暂停。

## 总分：24 / 24 = 锚点持平、**无回归** ✅

| # | 维度 | 锚点(rerun9) | R4-c v22 | 依据 |
|---|---|:--:|:--:|---|
| A1 | 引用充分性 | 2 | **2** | 38/38 findings 全挂 evidence_refs；**dangling refs = 0** |
| A2 | 引用准确性 | 2 | **2** | 74 evidence 含 official 45/doc 6/news 11/forum 11/unknown 1；evidence_refs 真实页面（Strava press / Garmin / Apple HealthKit / Runna / TrainingPeaks docs / industry blog）；recommended-bets cap=6 retry 干净（无 mutation 复发）|
| A3 | 无支撑率 | 2 | **2** | finding_type + epistemic_status (TM-4) 全标；ODI/Kano 全 estimated；assumption inline |
| A4 | 来源质量与多样性 | 2 | **2** | 74 evidence (+11 vs rerun9 63) / **23 domains** (+1 vs 22) / 5 source_types；每 aspect ≥5 evidence（min pre-mortem=5，max unmet-outcomes=15；其余 7-11）；**结构性 A4 已锚点 = 2** 持平 |
| A5 | 置信度校准+TM-4 | 2 | **2** | 全 medium + 逐 finding epistemic / 逐 evidence confidence |
| B1 | 骨架覆盖 (8 段) | 2 | **2** | 8 aspect 全 dedicated：trend-scan / unmet-outcomes / whitespace-canvas / future-capability-map / disruption-defensibility / pre-mortem-top3 / recommended-bets / build-cost-feasibility |
| B2 | 真实研究单元 | 2 | **2** | 各 aspect 独立研究单元 + JTBD 锚定（继承 rerun9）|
| B3 | 核心证据机制 | 2 | **2** | ODI matrix + capability map + Christensen 分级 + pre-mortem 3 死因（recommended-bets 3 bets 全带 why+falsification trigger）|
| B4 | 机会量化严谨 | 2 | **2** | ODI 公式 + 阈值 + estimated 标注（unmet-outcomes 5 findings 含 underserved 列表）|
| C1 | 视觉证据 (战略图类) | 2 | **2** | 数据支持渲染 ≥7 战略图（strategy canvas / capability map / ODI matrix / Christensen / pre-mortem 死因 / P0/P1/P2 bet 表 / trend scan）由 Tier-1 official(45) 支撑——**结构同 rerun9 类型判据**（rubric §6.4 family A） |
| C2 | 专家思维动作 TM | 2 | **2** | TM-1/2/4/6/7/11(bets falsifiable) 系统体现 |
| C3 | 可落地 | 2 | **2** | recommended-bets P0/P1/P2 分层下注 + build-cost-feasibility roadmap |
| | **合计** | **24** | **24** | 持平 ✅ |

## R1 闸门判定：**PASS**（无任一维跌破锚点；满分持平）

- A4=2 持平（结构性已达标）；R4-c canonical 增 +11 evidence / +1 domain 让 A4 更稳，分数不动（rubric 上限 2）。
- C1=2 与 search-tuning 解耦——v2.2 用 family A 战略图（chart 类）类型判据 ≥5+Tier-1 支撑，数据继续支持。
- floor 全过 + 5 hard gate（含 TM-11 推荐 bets 全 falsifiable）；dangling refs = 0。

## R4-c v2.2 canonical 结论（已坐实）

### 关键工程发现：v2.2 cap=6（与 v2.0/v2.1 不同）

| 能力 | 锚点 per-aspect cap | R4-c canonical cap | recency=fresh 加后病理 |
|---|:---:|:---:|---|
| v2.0 (5 段 deep) | 2（pre-#9 SSE workaround） | **4** | rerun1 cap=8+depth=high_recall 被 hard-kill；cap=4 稳 |
| v2.1 (6 段 deep) | 3 | **3** | cap=3 即可（M5/rerun9/R4-c 三轮收敛） |
| v2.2 (8 段 deep) | 5（rerun9） | **6 ✅ 必须** | **cap=5 持续 hard-kill recommended-bets（2 次 retry 均失败，第 6 次 search 被拒）**；cap=6 头一次 retry ✅ 干净通过 |

**Why v2.2 需 cap=6**：`recommended-bets` aspect 是综合下注 aspect，吃前 7 个 aspect 的 prior_sources 输出后做最终下注合成。recency=fresh 的 prompt-hint 把 model "再多搜一次"的倾向（M6 验证过）推到 ~6 search appetite。

### 工程层面验证

1. **R4-c canonical 触发 v2.2 over-search 边际**：v2.2 历史 cap=5 + 无 recency 是稳态；加 recency=fresh prompt-hint 后 recommended-bets 触底 hard-kill。诚实结论：**v2.2 R4-c canonical 必须配 cap=6**，否则别启用 search-tuning。
2. **search-tuning 非破坏**：cap=6 解决后，A2/A3 无 mutated_evidence_provenance 复发；A4 enrichment +11 ev / +1 domain；其它维全持平。
3. **C1/TM-11 与 search-tuning 解耦**：v2.2 family A 战略图类型 + bets falsifiable 是 Phase B 合成属性，data ≥ rerun9 即可支撑同样的渲染——非回归。

### 已知瑕疵 / 训练

- **recommended-bets 是 v2.2 的 search-tuning 边际 aspect**：未来如延伸更多 final-bet/synthesis-heavy aspect，需重新校 cap（每加 1 个综合 aspect，可能需 cap +1）。
- **trend-scan / unmet-outcomes 第一次 retry 因 transient grok latency 失败**（前者运行时超时，后者第一次 cap=5 触底），第二次 retry 即正常——不是 R4-c 引发，是 grok 抖。

### 写回上游 prompt

将 R4-c v2.2 canonical 配置写进 v2.2 的 `task-decomposition-innovation-direction.md`：
- search_policy: `recency=fresh` + `max_results_per_query=5`
- per-aspect deep: `max_search_calls=6`（**比 rerun9 anchor +1**，因为 recency=fresh prompt-hint 推 recommended-bets appetite 4→6）
- top max_total_search_calls: 推荐 50（8 aspect × 6 max = 48 + 小 headroom）

锚点更新：v2.2 search-tuning canonical 配置经纯引擎 8/8 验证；分数 24/24 锚点持平。**关键约束**：v2.2 deep tier 加 search-tuning 时**必须同时**抬 per-aspect cap 5→6，否则 recommended-bets aspect 必死。
