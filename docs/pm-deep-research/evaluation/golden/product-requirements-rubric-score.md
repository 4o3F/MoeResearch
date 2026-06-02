# R4-b 回归评分 · v2.3 explainable-coach PR-FAQ — #9 引擎全量复跑

> 对象：[`rerun9-golden-report-explainable-biometric-coach-prfaq.md`](rerun9-golden-report-explainable-biometric-coach-prfaq.md)（#9 引擎 `9db7464` 全量复跑；10/10 dedicated aspect，段3 由 4 cagan micro 装配）
> bundle：[`rerun9-bundle.md`](rerun9-bundle.md)（10 aspect / 48 findings / 79 evidence / dangling=0）
> 锚点：M7.5 [`m7_5-rubric-score.md`](m7_5-rubric-score.md) = **24/24**（B1/B3/C1 由 R4-e+R4-g 达 2）；亦对照 v2.2 M6 family A 最高锚点 24/24
> 闸门：R1 引擎漂移 — 任一维跌破锚点即暂停。

## 总分：24 / 24 = 锚点持平，**无回归** ✅

| # | 维度 | 锚点(M7.5) | 复跑(#9) | 依据 |
|---|---|:---:|:---:|---|
| A1 | 引用充分性 | 2 | **2** | 48/48 findings 全挂 evidence_refs |
| A2 | 引用准确性 | 2 | **2** | official 39/79；指向真实 Strava press / Runna·TP·WHOOP 官方定价 / HealthKit·WHOOP·Garmin API docs；解释类标 estimated/inferred |
| A3 | 无支撑率 | 2 | **2** | ODI/4-risks/OST/metrics 全 estimated_flag + TM-4 epistemic status；assumption inline 标注 |
| A4 | 来源质量与多样性 | 2 | **2** | 79 evidence / 36 域 / 6 source_type；每 aspect ≥5 evidence（feasibility=7/pr-faq=10/jtbd=15/ost=12/requirements=10）；≥1 Tier1 |
| A5 | 置信度校准+TM-4 | 2 | **2** | 全 aspect medium + 逐项 evidenced/estimated/assumption；段3 各类独立 confidence（value/business 含 low） |
| B1 | Skeleton 覆盖 (8 段 family B) | 2 | **2** | 7/7 mandatory 段全 dedicated（段1 pr-faq / 段2 jtbd-odi / **段3 4 cagan micro** / 段4 ost / 段5 requirements / 段6 metrics / 段8 open-questions）+ 段7 R4-f Phase B（非失败）|
| B2 | JTBD 真实研究单元 | 2 | **2** | 各 aspect 独立研究单元；JTBD 6-16 周备赛 + 非显性替代（Reddit 自解读/WHOOP/Strava）|
| B3 | 核心证据机制 (4-risks全 + OST≥3 + 三套指标) | 2 | **2** | 4 cagan micro 全 dedicated（4 类独立证据）+ OST 3 outcomes×3 候选=9 + metrics 三套（P1-3/S1-5/G1-4）|
| B4 | 机会量化严谨 (ODI/Kano) | 2 | **2** | ODI 完整公式 `Opp=Imp+max(0,Imp−Sat)` + Kano（O1=13 Must-be/O2=12 Attractive/O3=14 Must-be）+ 全 estimated 标注 |
| C1 | 视觉证据 (family B 表格) | 2 | **2** | **7 张语义表**（ODI / 4-risks / OST 9-cand / FR / NFR / metrics 3-set / OQ）≥5，Tier1 支撑（rubric §6.4 R4-g family B 判据）|
| C2 | 产品专家思维动作 (TM) | 2 | **2** | TM-1/3(四风险)/4/5/6/8(pre-mortem)/9(LNO)/11(falsification)/12(say-vs-do) 系统体现 |
| C3 | 可落地 | 2 | **2** | OST 首选方案 + FR-1~8 trace + metrics 三套 + 6 OQ 带 owner/date/pass-fail 排序（LNO）|
| | **合计** | **24** | **24** | |

## R1 闸门判定：**PASS**（无任一维跌破 M7.5 锚点，满分持平）

- 全 12 维 = 2，与 M7.5 24/24 完全持平；亦 = v2.2 M6 family A 最高锚点（family B PR-FAQ 模板在 cagan 拆分后达 family A 同等天花板，#9 引擎复现）。
- floor 全过 + 5 hard gate 通过（段3 4 dedicated micro 收敛 / OST≥3 / 非目标显式 / 三套指标 / TM-11 100%）；dangling refs = 0。

## R4-e 核心在 #9 引擎复现

| 项 | M7 (单 aspect) | M7.5 预演 | **#9 全量复跑** |
|---|---|---|---|
| cagan-4risks 收敛 | 5 retries 全败 (search-saturation) | 4/4 (deep 3 + retry 1) | **4/4**（deep 直接 value/usability/feasibility + business 1 retry）|
| 段3 输出性质 | 跨段代偿 | dedicated 4-risks | **dedicated 4-risks** |
| B1 / B3 | 1 / 1 | 2 / 2 | **2 / 2** |

**R4-e 假设在最终引擎坐实**：单 multi-class cagan-4risks aspect 拆为 4 single-class micro（max_search=4）后，每个 bounded 任务在预算内收敛 → search-saturation 病理消除，段3 由跨段代偿升为 dedicated 4-risks。

## 诚实标记（不为分数注水）

- **value**：explainability-specific WTP 无直接证据（相邻品类 medium / 解释本身付费 low）→ 段3 value 标 low、OQ2 待 Concierge MVP 校准。
- **business**：trainer marketplace unit economics = low（行业 blog take-rate，缺一手 CAC/attach/handle-time）→ OQ2/OQ4 待 ops 试点。
- **usability/feasibility**：证据为相邻领域迁移（mHealth/XAI/CDS + 官方 SDK 文档）+ 专家估计，无一手原型 → 标 medium、OQ1/OQ3/OQ6 待实验。
- 这些 low/medium 不影响 rubric 维度分（评 floor + 方法落点 + 证据机制完整性，不评"结论是否乐观"），恰证 A3/A5 = 2 的诚实性。

## 运行注记（#9 vs M7.5）

- #9 deep run 4/11（cagan value/usability/feasibility + open-questions）；6 aspect 经 aspect_research 补跑：pr-faq/jtbd-odi/cagan-business 首轮即过；ost/requirements/metrics 撞引擎严格 budget/provenance/SSE guard，按"prior_sources=[] + 低 search 强制早综合"配方收敛（ost search=6 最终过）。**全为环境/budget 调参，非 R4-e 或引擎回归**——cagan 4 micro 本身收敛行为与 M7.5 一致。
- 79 evidence（M7.5 为 85）；规模相当，A4 来源质量无回归。
