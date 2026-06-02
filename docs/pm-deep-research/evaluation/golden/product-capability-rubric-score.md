# R4-c v2.1 评分 · Runna product-capability — search-tuning canonical（纯引擎 6/6 golden）

> 对象：**纯引擎 6/6 golden** `r4c-v21-merged-6of6.result.json`（3 deep + 3 aspect_research 补跑，全部 R4-c 配置一致；experience-paths 第二次 retry 成功，第一次因单 grok 6.4 min 抖死）。
> 引擎=上游 `9db7464` verbatim；模型 gpt-5.5（CPA）/ 搜索 grok。
> 配置（R4-c canonical）：per-aspect `max_search_calls=3`（**保持** rerun9 锚点 budget、不上调避 hard-kill 病理）+ 全局 `search_policy.recency=fresh` + `max_results_per_query=5`（**无** depth=high_recall、**无** content_level=detailed、**无** category）。
> 锚点：rerun9 = **23/24**（A4=2 每 aspect ≥6 ev；唯一非满 = C1=1 in-app 录屏类型 Layer-2 backfill 待办）。
> 闸门：R1 引擎漂移 — 任一维跌破锚点即暂停。

## 总分：23 / 24 = 锚点持平、**无回归** ✅

| # | 维度 | 锚点(rerun9) | R4-c v21 | 依据 |
|---|---|:--:|:--:|---|
| A1 | 引用充分性 | 2 | **2** | 26/26 findings 全挂 evidence_refs；**dangling refs = 0** |
| A2 | 引用准确性 | 2 | **2** | 53 evidence 含 official 32/doc 2/blog 2/forum 7/news 3/unknown 7；evidence_refs 真实页面（runna.app / training peaks docs / strava api docs / reddit running posts / runners world / fellrnr blog）；finding type 标注（fact/interpretation/recommendation/assumption）|
| A3 | 无支撑率 | 2 | **2** | finding_type + epistemic_status (TM-4) 全标；ODI/Kano 全 estimated；assumption inline 显式 |
| A4 | 来源质量与多样性 | 2 | **2** | **持平且更稳**：53 evidence (+5 vs rerun9 48) / **15 domains** (+2 vs 13) / **6 source_types**（rerun9 4 种 → +documentation +news）；**每 aspect ≥6 evidence**（min benchmark-buildcost=6，max capability-domain-jtbd=11；其余 7-11）；R4-c canonical 加 5 evidence + 2 domain 但**结构性 A4 已锚点 = 2**，不上 3（rubric 无 >2 档）|
| A5 | 置信度校准+TM-4 | 2 | **2** | 全 medium + 逐 finding epistemic_status / 逐 evidence confidence；assumption 显式 |
| B1 | 骨架覆盖 (6 段) | 2 | **2** | 6 aspect 全 dedicated：capability-domain-jtbd / capability-teardown-deep / kano-in-domain / odi-in-domain / experience-paths-breakpoints / benchmark-buildcost-upgrade |
| B2 | 真实研究单元 | 2 | **2** | 能力域 boundary + user jobs（situation→motivation→outcome）+ practitioner interpretation 标注 |
| B3 | 核心证据机制 | 2 | **2** | experience-paths 完整路径图 + 断点 BP-X + kano（5 findings） + odi 多源齐 |
| B4 | 机会量化严谨 | 2 | **2** | odi 完整公式 + kano 4 类分级 + estimated 标 |
| C1 | 视觉证据 (录屏类) | **1** | **1 =** | visual_evidence 指向 App Store/产品页（非断点瞬间原生 UI 录屏）；< 5 → C1=1；同 rerun9 待办（升 2 需 Layer-2 视觉 backfill，与 search-tuning 无关），**非回归** |
| C2 | 专家思维动作 TM | 2 | **2** | TM-1/4 + 断点诊断 + Opinionated Coaching 护城河 |
| C3 | 可落地 | 2 | **2** | 断点 → 修复优先级 + benchmark-buildcost 升级路径 |
| | **合计** | **23** | **23** | 持平 ✅ |

## R1 闸门判定：**PASS**（无任一维跌破锚点）

- A4=2 持平（结构性已达标）；R4-c canonical **增 5 evidence + 2 domain + 2 source_type 多样性**，让 A4 更稳，但 rubric 上限就是 2，分数不动。
- C1=1=1（in-app 录屏结构限制，同 M5/rerun9 历史；升 2 需 Layer-2 backfill，与 search-tuning 无关）；**非回归**。
- floor 全过；dangling refs = 0。

## R4-c v2.1 canonical 结论（已坐实）

### 工程层面验证

1. **保守配置可工作**：保留 rerun9 per-aspect `max_search_calls=3` + 加 `recency=fresh` + `max_results_per_query=5`（3→5）—— 6/6 顺利 R4-c 跑出（1 个 deep_research 3/6 + 3 个 aspect_research 补跑成功；其中 experience-paths 第一次因 grok 单 search 6.4 min 异常超时被杀，第二次正常）。**未触发 R4-c v2.0 验证过的 hard-kill 病理**（per-aspect 3 < cap=4 危险线）。
2. **search-tuning 非破坏**：A2/A3 无 mutated_evidence_provenance 复发；A4 enrichment +5 ev / +2 domain / +2 source_type（多样性扩，不引入 noise）；其它维全持平。
3. **C1=1 与 search-tuning 解耦**：in-app 录屏类型证据需 Layer-2 视觉 backfill（aspect agent 通过浏览器抓 UI 帧），任何 search-tuning（包括 recency=fresh）都无法搜出这类原生帧——这是工程结构性边界，**不在 search-tuning 范围**。

### 已知瑕疵

- **一次 grok latency 异常**：experience-paths 第一次 retry 中一次 single search 耗 384s（6.4 min），与正常 grok 12-24s 显著偏离；非配置问题、是 grok 上游抖一下；第二次 retry 23s 即正常。**结论**：若复现，加 wall-clock retry，不动 budget。

### 写回上游 prompt

将 R4-c canonical 配置写进 v2.1 的 `task-decomposition-product-capability.md`：
- search_policy: `recency=fresh` + `max_results_per_query=5`
- per-aspect deep: `max_search_calls=3`（保留锚点 budget，**勿上调**）

锚点更新：v2.1 search-tuning canonical 配置经纯引擎 6/6 验证；分数 23/24 锚点不变（A4 already 2）；增量价值 = 工程一致性 + 多样性更稳 + Layer-2 backfill 抽象边界明晰。
