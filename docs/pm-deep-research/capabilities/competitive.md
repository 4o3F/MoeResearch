# Capability Profile · 竞品深度研究 (competitive)

> 通用 frame：[`../pm-deep-research-spec.md`](../pm-deep-research-spec.md)（人格 / 13 TM / 4-tier 证据 / 视觉证据 / 反幻觉 / 行文 floor / 优雅降级 / Lapis 接口边界等**所有跨能力机制**以通用规格为准；本 profile 仅落 competitive-specific 装配）。
> 上游： plan §3.1、Phase 2 review、Phase 3 M4。

---

## 0. 装配契约（§通用规格 §11 六字段）

| 字段 | 值 |
|---|---|
| **1. decision_intent_affinity** | `enter` / `differentiate` / `build` / `ai-upgrade`（带竞品对照）|
| **2. mece6_emphasis** | primary = `M3` Competitive Landscape + `M4` Product & Experience Capabilities；supporting = `M2` User & JTBD + `M6` Future Capability；contextual = `M1` Market Context + `M5` Business & Growth Model |
| **3. skeleton** | **B1 五维**：①Job 与真实竞争集 → ②能力对位矩阵 → ③Kano 重要性分级 → ④ODI 缺口打分 → ⑤定位与白地（详 §1）+ 支撑段（威胁分级 / Porter 仅行业 / SWOT 仅沟通 / Cagan 速写 / changelog-build-cost）|
| **4. report_template** | family = **A（13 章）**；weighting = Ch 5/6/9 重；do_not_drop = Ch 4/5/6/7/9/12/13（Deep 模式）|
| **5. persona_tm_weighting** | EA = `[TM-1, TM-2, TM-12]`（teardown + Kano + 言行分离）；Strategist = `[TM-3, TM-5, TM-8, TM-9, TM-13]`（去险 + 权衡 + pre-mortem + 杠杆 + 前瞻）；跨人格门 = `[TM-4, TM-11]`（全员）|
| **6. capability_specific** | aspect_fields: `capability_matrix` / `kano_grades` / `opportunity_scores` / `positioning` / `target_product` / `user_jobs` / `feature_architecture`；gap_checks: §3.1；floor_items: §3.2 |

---

## 1. B1 五维骨架（报告主干）

每维 = **主用方法**（引通用规格 §4 方法库）+ **competitive-specific 证据标准** + **报告落点** + **关联人格 / TM**。

### 维度 1 · Job 与真实竞争集
- **方法**：[M-JTBD](../pm-deep-research-spec.md#m-jtbdjobs-to-be-done) — JTBD（Christensen / Moesta switch interview）。先写 job statement（situation→motivation→outcome），再据"完成同一 job"找出**非显性替代者**（不止同品类）。
- **证据标准**：明确 job statement + 找出至少 1 个非显性竞争者并给纳入理由。
- **报告落点**：开篇"谁是真正的对手"；重构分析单元（**Ch 4** JTBD + **Ch 5** 竞品图谱）。
- **人格 / TM**：Strategist 框定竞争集 + Experience Analyst 做 JTBD / TM-1。
- **真实案例（已核）**：Christensen 奶昔（对手是香蕉 / 百吉饼 / 无聊）；Facebook vs MySpace（两个不同 job）。

### 维度 2 · 能力对位矩阵 (feature teardown)
- **方法**：[M-Teardown](../pm-deep-research-spec.md#m-teardown功能对位矩阵--feature-teardown) — 跨竞品功能对位矩阵，按买家关注维度评分（12 维 rubric，每格 1–5 分）。
- **证据标准（强制）**：**每格必须附证据**——截图 / 应用商店评论数 / 操作步数 / 视频时间点。无证据的格标为假设。
- **报告落点**："功能版图"，竞品报告的事实底座（**Ch 6**）。
- **人格 / TM**：Experience Analyst / TM-1 + TM-2 metrics-informed + TM-12 言行分离。

### 维度 3 · 功能重要性分级 (Kano)
- **方法**：[M-Kano](../pm-deep-research-spec.md#m-kanokano-模型) — Kano 模型。每竞品每功能标 Must-be / Performance / Attractive。
- **证据标准**：分级须有用户证据（评论 / 调研）或明确标 practitioner 诠释（TM-4）。
- **报告落点**："什么才真正影响用户"，叠在能力矩阵上（**Ch 6**）。
- **人格 / TM**：Experience Analyst。
- **真实案例（已核）**：Kano 1984 原始论文（J-STAGE 核实）；GitLab 对 12 功能跑过正式 Kano 调研。

### 维度 4 · 竞争缺口打分 (ODI)
- **方法**：[M-ODI](../pm-deep-research-spec.md#m-odiopportunity-algorithm) — Opportunity Algorithm（完整公式见通用规格）。
- **证据标准**：Importance / Satisfaction 来自一手问卷优先；无一手时用研究证据 / 市场代理**估算 + TM-4 标注证据等级**（ADR-0006 决策 3）。
- **报告落点**：**机会矩阵的直接打分逻辑**（**Ch 9**）。
- **人格 / TM**：Strategist。
- **真实案例（已核）**：Cordis"最小化再闭塞概率"未满足缺口 → 市占 1%→20%，J&J $109/股收购。

### 维度 5 · 定位与白地
- **方法**：[M-Positioning](../pm-deep-research-spec.md#m-positioningstrategy-canvas--感知图) — Strategy Canvas / 感知图。用 **buyer-validated 轴**画 value curve，标白地。
- **证据标准**：轴必须是买家验证过的购买维度（非臆造）；白地须给"为何无人占据"的解释或假设。
- **报告落点**："竞争定位图"（**Ch 5** / 独立定位小节）。
- **人格 / TM**：Strategist / TM-9 杠杆点 + TM-13 面向市场的未来。

### 支撑段（非 MECE 核心，但标准报告段落）

| 段落 | 方法（通用规格 §4）| 用法约束 |
|---|---|---|
| 威胁分级 | [M-Disruption](../pm-deep-research-spec.md#m-disruptionchristensen-颠覆理论) Christensen | 每竞品标"维持性 vs 颠覆性"威胁 |
| 市场结构语境 | [M-Porter](../pm-deep-research-spec.md#m-porterporter-五力)（**仅行业层、可选**）| **禁止在产品层误用**（Porter 是行业结构工具）|
| 战略小结 | [M-SWOT](../pm-deep-research-spec.md#m-swot仅沟通层)（**仅沟通层**）| 证据收齐后用，每格转成带证据的具体含义；**不作发现工具** |
| 竞品速写 | [M-Cagan-4Risks](../pm-deep-research-spec.md#m-cagan-4riskscagan-四大风险) Cagan 3 强项 / 3 弱项变形 | 每个竞品 profile 的快速开头 |
| build-cost 实际动作 | [M-Changelog](../pm-deep-research-spec.md#m-changelogchangelog--版本时间线--实际动作证据)（**Build 意图强制**）| changelog / release notes / 版本时间线 = TM-12 言行分离的直接落点 + build-cost 外部代理 |

> **风险优先竞品分析**（Reforge / Sachin Rekhi "Connected" 案例）：先问"什么会杀死这个策略"——作为 Strategist 的 pre-mortem（TM-8）切入。

> **迭代节奏与建设成本（实际动作证据）**：判断"该不该建某功能"不能只看用户价值，必须研究**建设成本**——竞品 **changelog / 版本更新历史 / release notes 时间线**是最硬的外部证据。
> - **它是对手的"实际动作"**：营销说什么是"言"，版本里真正发了什么、隔多久发是"行"。迭代节奏暴露对手**真实投入优先级**（revealed strategy），是 **TM-12 言行分离**的直接落点——以 changelog 为准，不以官网宣传为准。
> - **它是 build-cost 的外部代理**：某能力对手迭代了多少个版本才稳定（如自适应计划从首发到成熟经历几轮）≈ 我方自建的复杂度 / 成本下界，喂入机会矩阵的「复杂度」列与 Build/Not Build 决策。
> - **证据标准**：给出可追溯的版本时间线（App Store/Google Play 版本历史、官方 release notes、第三方发布追踪如 AppFigures）。陷阱：营销化 release notes 会隐藏真实工作量、捆绑发布、灰度未写入——拿不到可靠时间线时**标假设，不臆断节奏**。

---

## 2. 五维 → 13 章映射（报告模板族 A 装配）

竞品报告用 13 章承载，五维是喂入各章的**分析主干**：

| 章 | 标题 | 主要来源（五维 / 支撑）|
|---|---|---|
| 1 | 研究结论摘要 | 全部收敛：核心判断 + 推荐 + 置信度 + 最大不确定性 |
| 2 | 研究输入与边界 | decision_intent、目标产品、人群、排除范围 |
| 3 | 目标产品定位与现状 | 竞品速写（Cagan 3 强 3 弱）|
| 4 | 用户人群与 JTBD | **维度 1**（job statement）|
| 5 | 竞品与替代方案图谱 | **维度 1**（真实竞争集）+ **维度 5**（定位图）+ 威胁分级 |
| 6 | 功能架构与体验路径 | **维度 2**（能力对位矩阵）+ **维度 3**（Kano）|
| 7 | 视觉证据资产表 | 通用规格 §6.2 visual_evidence |
| 8 | AI / 新能力映射 | （AI Upgrade 意图时展开；否则可裁剪）|
| 9 | 产品机会矩阵 | **维度 4**（ODI 打分）+ 通用规格 §7.4 机会矩阵模板 |
| 10 | Roadmap 建议 | Strategist：P0/P1/P2 + 依赖 + 验证条件 |
| 11 | 验证实验与指标 | 指标定义模板（通用规格 §7.4）|
| 12 | 风险、冲突与开放问题 | TM-8 pre-mortem + 低置信 / 冲突证据 |
| 13 | 附录：来源与搜索记录 | Evidence Table + Search Queries + Source List（含 tier/标签）|

**裁剪规则**：
- **Quick**：Ch 1 + 核心判断 + 来源（含标签）。
- **Standard**：Ch 1/2/4/5/6/9/13 + 简化机会矩阵。
- **Deep**：全 13 章，**不得删** Ch 4/5/6/7/9/12/13。

---

## 3. Competitive-specific Gap 检测 + Quality Floor

通用 gap / floor 见通用规格 §9；以下为 **competitive-specific** 追加项。

### 3.1 Competitive Gap 检测

| 检测项 | 不合格条件 | 处理 |
|---|---|---|
| 竞品覆盖 | < 3 竞品且未说明原因 | 补直接 / 间接 / 替代竞品 |
| 真实竞争集 | 只有同品类竞品 | 按维度 1 补非显性替代者 |
| 能力矩阵证据 | 矩阵格无证据 | 补截图 / 评论数 / 步数，否则标假设 |
| ODI 打分 | 无 Imp/Sat 依据 | 补一手或标估算 + TM-4 |
| 机会优先级 | 只有建议，无价值 / 复杂度 / 风险 | 补机会矩阵 |
| 建设成本（Build 意图）| 判"该不该建"却无 build-cost 估算 / 忽视竞品迭代节奏 | 补 changelog / 版本历史分析，估算复杂度 |
| 指标与验证 | 无实验 / 指标 | 补验证计划 |

### 3.2 Competitive Quality Floor（Deep 模式追加项）

| 质量项 | 最低要求 |
|---|---|
| 目标产品基础资料 | ≥3 来源，优先 Tier 1/2 |
| 竞品数量 | ≥3，覆盖直接 / 间接 / 替代 |
| 能力对位矩阵 | 每格有证据或标假设 |
| 机会矩阵 | ≥5 机会点，每个评估价值 / 复杂度 / 证据 / 优先级 |

---

## 4. Aspect Schema · Competitive 扩展字段

通用扩展字段见通用规格 §8；本 profile 追加：

```json
{
  "capability": "competitive",
  "target_product": { "name": "", "positioning": "", "core_scenarios": [], "target_users": [] },
  "user_jobs": [],
  "feature_architecture": [],
  "capability_matrix": [],       // 维度 2：每格含证据 ref
  "kano_grades": [],             // 维度 3：{feature, kano_type, evidence_refs}
  "opportunity_scores": [],      // 维度 4：{outcome, importance, satisfaction, score, estimated:bool}
  "positioning": {}              // 维度 5：{axes:[buyer-validated], value_curves:[], whitespace:[]}
}
```


---

## 5. 人格 / TM 详尽分配（五维 × 人格）

跨人格质量门（TM-4 / TM-11）注入所有 aspect；以下为五维主分配（同通用规格 §3.3 的 EA + Strategist 平衡）：

| 五维 | 主人格 | TM |
|---|---|---|
| 1 Job 与真实竞争集 | Strategist 框定 + Experience 做 JTBD | TM-1 |
| 2 能力对位矩阵 | Experience Analyst | TM-1/2/12 |
| 3 功能重要性 (Kano) | Experience Analyst | TM-1/6 |
| 4 竞争缺口 (ODI) | Strategist | TM-9 |
| 5 定位与白地 | Strategist | TM-9/13 |
| 支撑：威胁 / 定位 / 速写 / build-cost | Strategist | TM-8/12/13 |

> **CI/Market 吸收说明**：早期母版的 Competitive Intelligence Analyst 职能（竞品图谱、功能矩阵）落到 Experience Analyst 的 teardown + Strategist 的定位 / 威胁；Market & Context Analyst 职能在竞品研究里降为 Strategist 的市场语境输入（Porter 仅行业层）。

---

## 7. De-AI Voice Pass 哨兵（competitive-specific）

通用 voice pass 见 [`skills/pm-deep-research/prompts/layer1/phase-d-voice-pass.md`](../../../skills/pm-deep-research/prompts/layer1/phase-d-voice-pass.md)。本 profile 追加 competitive-specific 加权项：

| 哨兵 | 触发条件 | 不达标动作 |
|---|---|---|
| TM-4 全员 | Ch 6 能力矩阵 + Ch 9 ODI 每格未标 TM-4 | 补 fact/interpretation/assumption/speculation 标注 |
| Imp/Sat estimated flag | ODI 打分无 `estimated:true/false` | 标 estimated + TM-4 |
| ≥3 user-evidence per Kano 分级 | Ch 6 Kano 分级单源 | 补证据或标 practitioner 诠释（TM-4）|

voice pass **不准**洗去：Cagan 速写 "3 弱项" 段 / changelog + build-cost 段 / Porter 仅行业层免责语 / SWOT 仅沟通层免责语 / Ch 12 风险与开放问题段。
