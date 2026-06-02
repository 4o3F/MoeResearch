# 评分 · product-capability · Runna 训练计划自适应能力 — 最终权威黄金

> 对象：**纯引擎 6/6 golden** `r4c-v21-merged-6of6.result.json`（3 deep + 3 aspect_research 补跑；experience-paths 第二次 retry 成功，第一次因单 grok 抖死）。
> 引擎=上游 `9db7464` verbatim；模型 gpt-5.5（CPA）/ 搜索 grok。
> 配置：per-aspect `max_search_calls=3`（不上调避 budget-overflow 病理）+ 全局 `search_policy.recency=fresh` + `max_results_per_query=5`（**无** depth=high_recall、**无** content_level=detailed、**无** category）。
> 闸门：任一维跌破先前 23/24 锚点即暂停。

## 总分：23 / 24 ✅

| # | 维度 | 分数 | 依据 |
|---|---|:--:|---|
| A1 | 引用充分性 | **2** | 26/26 findings 全挂 evidence_refs；**dangling refs = 0** |
| A2 | 引用准确性 | **2** | 53 evidence 含 official 32/doc 2/blog 2/forum 7/news 3/unknown 7；evidence_refs 真实页面（runna.app / training peaks docs / strava api docs / reddit running posts / runners world / fellrnr blog）；finding type 标注（fact/interpretation/recommendation/assumption）|
| A3 | 无支撑率 | **2** | finding_type + epistemic_status (TM-4) 全标；ODI/Kano 全 estimated；assumption inline 显式 |
| A4 | 来源质量与多样性 | **2** | 53 evidence / **15 domains** / **6 source_types**；**每 aspect ≥6 evidence**（min benchmark-buildcost=6，max capability-domain-jtbd=11）；search-tuning 增 5 evidence + 2 domain + 2 source_type 多样性，不上 3（rubric 无 >2 档）|
| A5 | 置信度校准+TM-4 | **2** | 全 medium + 逐 finding epistemic_status / 逐 evidence confidence；assumption 显式 |
| B1 | 骨架覆盖 (6 段) | **2** | 6 aspect 全 dedicated：capability-domain-jtbd / capability-teardown-deep / kano-in-domain / odi-in-domain / experience-paths-breakpoints / benchmark-buildcost-upgrade |
| B2 | 真实研究单元 | **2** | 能力域 boundary + user jobs（situation→motivation→outcome）+ practitioner interpretation 标注 |
| B3 | 核心证据机制 | **2** | experience-paths 完整路径图 + 断点 BP-X + kano（5 findings） + odi 多源齐 |
| B4 | 机会量化严谨 | **2** | odi 完整公式 + kano 4 类分级 + estimated 标 |
| C1 | 视觉证据 (录屏类) | **1** | visual_evidence 指向 App Store/产品页（非断点瞬间原生 UI 录屏）；< 5 → C1=1；升 2 需 Layer-2 视觉 backfill（与 search-tuning 无关）|
| C2 | 专家思维动作 TM | **2** | TM-1/4 + 断点诊断 + Opinionated Coaching 护城河 |
| C3 | 可落地 | **2** | 断点 → 修复优先级 + benchmark-buildcost 升级路径 |
| | **合计** | **23** | |

## 闸门判定：**PASS** ✅

- A4=2（结构性已达标）；search-tuning 增 5 evidence + 2 domain + 2 source_type 多样性，rubric 上限就是 2，分数不动。
- C1=1（in-app 录屏结构限制；升 2 需 Layer-2 backfill，与 search-tuning 无关）；**非回归**。
- floor 全过；dangling refs = 0。

## 工程验证结论

1. **保守配置可工作**：per-aspect `max_search_calls=3` + `recency=fresh` + `max_results_per_query=5` —— 6/6 顺利跑出（3 deep_research + 3 aspect_research 补跑成功；其中 experience-paths 第一次因 grok 单 search 6.4 min 异常超时被杀，第二次正常）。**未触发 budget-overflow 病理**（per-aspect 3 < 4 危险线）。
2. **search-tuning 非破坏**：A2/A3 无 mutated_evidence_provenance 复发；A4 多样性扩，不引入 noise；其它维全持平。
3. **C1=1 与 search-tuning 解耦**：in-app 录屏类型证据需 Layer-2 视觉 backfill（aspect agent 通过浏览器抓 UI 帧），任何 search-tuning（包括 recency=fresh）都无法搜出这类原生帧——这是工程结构性边界，**不在 search-tuning 范围**。

## 已知瑕疵

- **一次 grok latency 异常**：experience-paths 第一次 retry 中一次 single search 耗 384s（6.4 min），与正常 grok 12-24s 显著偏离；非配置问题、是 grok 上游抖动；第二次 retry 23s 即正常。**结论**：若复现，加 wall-clock retry，不动 budget。

## 配置写回（已在 task-decomposition-product-capability.md 体现）

- search_policy: `recency=fresh` + `max_results_per_query=5`
- per-aspect deep: `max_search_calls=3`（**勿上调**）

分数 23/24；增量价值 = 工程一致性 + 多样性更稳 + Layer-2 backfill 抽象边界明晰。
