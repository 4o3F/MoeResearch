# Capability Profile · 产品需求深度调研 (product-requirements)

> Status: ✅ **端到端验证完成**（v2.3 M7 21/24 → R4-g C1 补丁 22 → **R4-e 段3 cagan 拆 4 micro-aspect + #9 引擎全量复跑 24/24**，2026-05-31）；family B 8 段 PR-FAQ 模板首落地。本 profile 装配 + 方法组合骨架经 M7 / M7.5 / #9 三轮坐实，详 §6。
> 通用 frame：[`../pm-deep-research-spec.md`](../pm-deep-research-spec.md)（人格 / 13 TM / 4-tier 证据 / 视觉证据 / 反幻觉 / 行文 floor / 优雅降级 / Lapis 接口边界等**所有跨能力机制**以通用规格为准）。
> 方法论：B2 PR-FAQ 八段模板（Amazon working backwards + Cagan 4 风险 + Torres OST + Kano 1984）—— 来源均一手核实，详 PM DeepResearch 源仓 track B 调研。

---

## 0. 核心问题

对已选定的机会 / 问题，产出**可验证、可落地的 requirement / PRD 前置物**。区别于其它 3 个 profile（都产 13 章报告）——本 profile 产 **8 段 PR-FAQ 模板**，验证通用规格能承载**不同报告族**（这是 Phase 2′ 通用化的关键证据之一）。

> 与 [competitive](competitive.md) / [product-capability](product-capability.md) / [innovation-direction](innovation-direction.md) 的区别：前 3 个回答"研究"问题（市场 / 能力 / 未来格局如何），本 profile 回答"决策已定，把需求写好"问题——下游接 PRD / 开发 / 实验，不是接战略讨论。

---

## 1. 装配契约（§通用规格 §11 六字段）

| 字段 | 值 |
|---|---|
| **1. decision_intent_affinity** | `build` / `improve`（已选定问题，要产 PRD 前置物）|
| **2. mece6_emphasis** | primary = `M2` User & JTBD（首要）+ `M4` Product & Experience Capabilities（解空间）；supporting = `M6` Future Capability（升级方向纳入解空间）+ `M3` Competitive Landscape（solution benchmarking，**轻**）；contextual = `M1` Market Context + `M5` Business & Growth Model |
| **3. skeleton** | **B2 八段模板** = ①PR-FAQ → ②机会验证（JTBD + ODI 前 5 + Kano + Opportunity Landscape）→ ③Cagan 四风险 → ④Torres OST 解空间（≥3 候选 + 最危险假设）→ ⑤需求（功能 + 非功能 + 非目标）→ ⑥成功度量（主 / 次 / 护栏）→ ⑦证据与来源 → ⑧未决问题 & 下一步（详 §2）|
| **4. report_template** | family = **B（8 段 PR-FAQ 模板，非 13 章）**——这是验证通用规格能承载**不同报告族**的关键 |
| **5. persona_tm_weighting** | EA + Strategist **平衡**——价值由 EA 取证，可行性 / 商业可行性 / 权衡由 Strategist 取证：EA = `[TM-1, TM-2, TM-6, TM-12]`（Job→Feature→Gap + metrics-informed + 弦外之音 + 言行分离）；Strategist = `[TM-3, TM-5, TM-9, TM-11]`（四风险 + 显性权衡 + 杠杆 + 可证伪）；跨人格门 = `[TM-4, TM-11]` 全员 |
| **6. capability_specific** | aspect_fields: `pr_faq` / `jtbd_statement` / `top5_outcomes_odi` / `four_risks` / `solution_space` / `requirements` / `non_goals` / `metrics_tree` / `evidence_table` / `open_questions`；gap_checks: §3.1；floor_items: §3.2 |

---

## 2. 8 段骨架（B2 模板，报告主干）

每段 = 主用方法（引通用 §4）+ 证据标准 + 必填项。**本 profile 报告族 = B 8 段，非 13 章**。

### 段 1 · Press Release Frame（PR-FAQ）
- **方法**：[M-PR-FAQ](../pm-deep-research-spec.md#m-pr-faqamazon-press-release--faq) — Amazon PR-FAQ 风格，≤300 字
- **必填**：headline / sub-headline / 客户引言（虚构但符合 JTBD）/ 内部 FAQ 5+ / 外部 FAQ 3+
- **证据标准**：先讲价值再讲功能；禁止描述实现细节
- **人格 / TM**：Strategist + EA 双签

### 段 2 · 机会验证（JTBD + ODI + Kano + Opportunity Landscape）
- **方法**：[M-JTBD](../pm-deep-research-spec.md#m-jtbdjobs-to-be-done)（写 job statement）+ [M-ODI](../pm-deep-research-spec.md#m-odiopportunity-algorithm)（前 5 desired outcomes，Imp/Sat/Opp）+ [M-Kano](../pm-deep-research-spec.md#m-kanokano-模型)（每 outcome 分级）
- **必填**：job statement + 5 outcomes × {Imp, Sat, Opportunity Score, Kano 类型, 证据 ref}
- **证据标准**：Imp / Sat 来自一手优先；估算时强标 TM-4
- **人格 / TM**：EA / TM-1/2 + Strategist 收敛

### 段 3 · Cagan 四大风险
- **方法**：[M-Cagan-4Risks](../pm-deep-research-spec.md#m-cagan-4riskscagan-四大风险) — 价值 / 可用性 / 可行性 / 商业可行性，每类给证据等级 + 来源
- **必填**：4 风险 × {风险描述, 证据等级 high/medium/low, 来源 refs}
- **人格 / TM**：Strategist / TM-3（强制覆盖 4 类）

### 段 4 · Torres OST 解空间
- **方法**：[M-OST](../pm-deep-research-spec.md#m-osttorres-opportunity-solution-tree) — 每目标机会 ≥3 候选方案 + 最危险假设清单 + 既有 / 竞争方案
- **必填**：每目标 outcome × {≥3 候选, 最危险假设 ≥2, 既有 / 竞争方案对照}
- **证据标准**：候选方案附"可行性 + 用户价值 + 风险"快评
- **人格 / TM**：EA / TM-1 + Strategist / TM-3

### 段 5 · 需求（功能 + 非功能 + 非目标）
- **必填**：功能需求列表（每条 outcome 语句 + 标 Kano 类型）+ 非功能需求（性能 / 安全 / 合规等）+ 非目标（明确写"不做什么"）
- **证据标准**：每功能需求 trace 回段 2 的 outcome；非目标须给"为何不做"的理由
- **人格 / TM**：EA + Strategist

### 段 6 · 成功度量
- **必填**：3 套指标 — 主指标 leading（北极星 / 激活 / 完成率）+ 次指标 secondary（细分流量 / 流失漏斗）+ 护栏指标 guardrails（不能因新需求让其它指标恶化）
- **证据标准**：每指标给定义 / 计算方式 / 数据来源 / 成功标准
- **人格 / TM**：Strategist / TM-9（杠杆点筛指标）+ EA / TM-2（metrics-informed）

### 段 7 · 证据与来源
- **必填**：一手 / 二手来源表（只用真实 URL）+ 每条声明置信度
- **证据标准**：通用规格 §6 4-tier 全套；本段是 evidence_index 在 8 段模板里的归宿
- **人格 / TM**：跨人格门 TM-4 全员

### 段 8 · 未决问题 & 下一步
- **必填**：未决问题清单 + 验证实验设计（discovery sprint / prototype / A-B test 任一）+ 下一步 owner / 时间窗
- **证据标准**：每未决问题给"为何还未决" + "靠什么会决"
- **人格 / TM**：Strategist / TM-11（**强制：每个未决问题须可证伪 = "靠什么会决"**）+ EA

---

## 3. Product-requirements-specific Gap / Floor

通用 gap / floor 见 [通用规格 §9](../pm-deep-research-spec.md#9-gap-检测清单--quality-floor通用部分)；以下为本 profile 追加项。

### 3.1 Gap 检测

| 检测项 | 不合格条件 | 处理 |
|---|---|---|
| PR-FAQ 价值 vs 功能 | 把实现细节写进 PR 部分 | 重写为价值导向 |
| ODI outcomes | <5 outcomes / 无 Imp+Sat 数值 / 无 TM-4 标注 | 补 |
| Cagan 四风险覆盖 | 缺一类 | **强制补**（TM-3）|
| 解空间候选 | <3 候选 / 无"既有方案"对照 | **强制补**（OST 核心） |
| 最危险假设 | 段 4 无最危险假设清单 | 补 |
| 非目标 | 段 5 无"非目标"段 | **强制补**（PR-FAQ 文化核心）|
| 三套指标 | 缺主 / 次 / 护栏任一 | **强制补**（不能只给主指标）|
| 未决问题可证伪 | 未决问题无"靠什么会决" | **强制补**（TM-11） |

### 3.2 Quality Floor（Deep 模式追加项）

| 质量项 | 最低要求 |
|---|---|
| ODI outcomes | ≥5，每个含 Imp/Sat/Opp + Kano + 证据 ref |
| 四风险 | 4 类全覆盖，每类附证据等级 |
| 解空间候选 | 每目标 ≥3 候选 + 既有 / 竞争方案对照 |
| 最危险假设 | 每候选 ≥1 个最危险假设 |
| 非目标 | 显式列出（"不做什么"）|
| 三套指标 | 主 / 次 / 护栏全有，每指标 5 字段全 |
| 未决问题可证伪 | 每未决问题附"靠什么会决"（TM-11 强制）|

---

## 4. Aspect Schema · product-requirements 扩展字段

通用扩展见 [通用规格 §8](../pm-deep-research-spec.md#8-aspect-report-schema通用扩展字段)；本 profile 追加：

```json
{
  "capability": "product-requirements",
  "pr_faq": {
    "headline": "", "sub_headline": "", "customer_quote": "",
    "internal_faq": [], "external_faq": []
  },
  "jtbd_statement": "",
  "top5_outcomes_odi": [],          // {outcome, importance, satisfaction, opportunity, kano, evidence_refs, estimated:bool}
  "four_risks": {
    "value": {}, "usability": {}, "feasibility": {}, "business_viability": {}
  },
  "solution_space": [],             // {outcome_ref, candidates:[{name, risk_assessment, riskiest_assumptions}], existing_competitor_solutions}
  "requirements": {
    "functional": [],               // {req, outcome_ref, kano}
    "non_functional": [],
    "non_goals": []                 // 强制列
  },
  "metrics_tree": {
    "leading": [], "secondary": [], "guardrails": []
  },
  "open_questions": []              // {question, why_open, how_to_resolve}  ← how_to_resolve 强制（TM-11）
}
```

---

## 5. 人格 / TM 详尽分配（8 段 × 人格）

跨人格质量门（TM-4 / TM-11）注入所有 aspect；本 profile **EA + Strategist 平衡**：

| 段 | 主人格 | TM |
|---|---|---|
| 1 PR-FAQ | Strategist + EA 双签 | — |
| 2 机会验证 (JTBD/ODI/Kano) | EA | TM-1/2 + Strategist 收敛 |
| 3 Cagan 四风险 | Strategist | TM-3（强制 4 类）|
| 4 OST 解空间 | EA + Strategist | TM-1 + TM-3 |
| 5 需求 (功能 / 非功能 / 非目标) | EA + Strategist | — |
| 6 成功度量 | Strategist | TM-9 + EA / TM-2 |
| 7 证据与来源 | 跨人格门 | TM-4 全员 |
| 8 未决问题 + 下一步 | Strategist | TM-11（强制可证伪）|

---

## 6. 验证状态

- ✅ **v2.3 M7 端到端验证完成（2026-05-30，21/24 诚实自评 / family B 8 段 PR-FAQ 模板首落地证明）**
  - 黄金课题：Endurance-athlete Explainable Biometric Coach PR-FAQ（新创 build intent，narrative 接 v2.2 W1 P1）。
  - M7 baseline 21/24（family B 8 段, 6/8 dedicated aspect + 2 跨段代偿）已由 R4-e + #9 rerun9 24/24 替代（见下条）。
  - **核心 v2.3 增益**：
    - **family B 模板族首落地** — 与 v2.0/v2.1/v2.2 family A 13 章并列；通用规格 §7.1 模板族 B 实测可承载证明 ✅。
    - **family B 段间 narrative (working backwards / PR-FAQ ≤300 字 cap / 客户引言"When X I want Y so I can Z"句式 / 非目标显式段 / 5 hard gates 分布)** 全部 enforce 成功。
    - **5 hard gates 中 4 pass + 1 partial**：段4 OST ≥3 候选 ✅ / 段5 非目标显式 ✅ / 段6 三套指标全 ✅ / 段8 TM-11 falsification 7/7=100% ✅；段3 4-risks aspect 5 retries 全失败 → cross-aspect 跨段代偿 (confidence medium)。
    - **TM-11 hard gate 7/7=100%** ✅（含工程过程发现的 OQ-NEW process risk 自报）— family B 段8 falsification 强度与 v2.2 段8 hard gate 等价。
  - **关键 M7 发现 (诚实失分根因)**：
    - **段3 cagan-4risks aspect search-saturation pathology**：strategist persona 在 multi-class 4-risks 任务上无法在 18-call budget 内收敛（5 次 backfill 全失败：3 budget_exceeded + 2 schema_validation_failed）。Phase 4 R4-e 修复候选: 拆为 4 micro-aspect (1 risk class / aspect)。
    - **段7 evidence-table aspect schema_validation_failed**：Lapis evidence_refs 限定本 aspect search 输出，meta-aggregation aspect 设计性不合身。Phase 4 R4-f: 段7 in task-decomposition 标 OPTIONAL，默认 fallback to final-report Phase B 跨段聚合。
    - **rubric §6.4 C1 视觉 vs family B 模板语义错位**：family B 天然出 table 不出 chart，rubric C1 chart-centric 描述不适配。Phase 4 R4-g: rubric §6.4 + §3 (C1 描述) 加 family B 适配段（≥5 张语义 table 等价 chart 类型）。
  - **承载性证明**：本 profile 装配契约（EA + Strategist 平衡 + family B 8 段 + 5 hard gates + 段7 fallback path）可承载；**段3 失败是 prompt 设计 finding 而非装配契约缺陷**，profile §1 / §3 不需修改；M7.5 拆 cagan 为 4 micro-aspect 后 B1 1→2 + B3 1→2 已坐实（见下 R4-e 条）。
- ✅ **R4-e + #9 引擎全量复跑验证完成（2026-05-31，24/24，[最终报告](../evaluation/golden/product-requirements-prfaq.md) / [打分](../evaluation/golden/product-requirements-rubric-score.md)）**：段3 cagan-4risks 拆 4 micro-aspect（value/usability/feasibility/business，每 max_search=4 + 1 class evidence）消除 search-saturation 病理 → 段3 升为 **dedicated 4-risks**（M7.5 预演 4/4 收敛 + #9 全量复跑 cagan 4/4 dedicated 双坐实）。B1 1→2 + B3 1→2（叠加 R4-g C1 1→2）= **21→24**；改 task-decomposition / agent-allocation / final-report 三编排文件 + deep tier 10 mandatory aspect（段3=4 micro，段7 R4-f OPTIONAL）。**不注水**：value explainability-WTP / business trainer-econ 仍标 low（恰证 A3/A5 诚实性）。
- 先验合理性来源：B2 八段模板已校验真实可追溯（Amazon PR-FAQ、Cagan 4 风险、Torres OST、Kano 1984 原始论文均一手核实）；通用规格 §7.2 模板族 B 已为该模板预留 — M7 端到端验证。本 profile 的所有方法均已在 v2.0 [competitive](competitive.md) 或方法库（M-JTBD/Kano/ODI/4Risks/OST）中验证可调用，方法论无新引入。
