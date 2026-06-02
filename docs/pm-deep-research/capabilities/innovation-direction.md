# Capability Profile · 创新方向研究 (innovation-direction)

> 通用 frame：[`../pm-deep-research-spec.md`](../pm-deep-research-spec.md)（人格 / 13 TM / 4-tier 证据 / 视觉证据 / 反幻觉 / 行文 floor / 优雅降级 / Lapis 接口边界等**所有跨能力机制**以通用规格为准）。

---

## 0. 核心问题

未来 / 白地机会在哪？押哪个新能力（AI / 硬件 / 内容 / 社区 / 数据）？这一注会不会死？比 [competitive](competitive.md) profile 更"前瞻 + 战略下注"：从现状外推到未来轨迹，识别 underserved + whitespace + 颠覆威胁，给一个可证伪的下注方向。

> 与 competitive 的区别：competitive 描述当下竞争集与差异化，innovation 跨过当下看未来 12-36 个月 + 评估"下哪一注"的可防御性。

---

## 1. 装配契约（§通用规格 §11 六字段）

| 字段 | 值 |
|---|---|
| **1. decision_intent_affinity** | `ai-upgrade` / `enter`（新方向）/ `differentiate`（未来下注）|
| **2. mece6_emphasis** | primary = `M6` Future Capability & Strategic Opportunity；supporting = `M1` Market Context（趋势）+ `M2` User & JTBD（未满足）+ `M3` Competitive Landscape（白地）；contextual = `M4` Product & Experience Capabilities（现状对新能力的承载力）+ `M5` Business & Growth Model |
| **3. skeleton** | 8 段：①趋势扫（市场 / 技术 / 竞争）→ ②未满足 job + outcome（ODI underserved）→ ③白地 + 定位 canvas → ④未来能力映射（AI / 硬件 / 数据 / 社区）→ ⑤颠覆威胁与可防御性 → ⑥pre-mortem → ⑦build-cost & 可行性 → ⑧推荐下注 + 验证条件（详 §2）|
| **4. report_template** | family = **A（13 章变体）**；weighting = **Ch 8（AI / 新能力映射）+ Ch 9（机会）+ Ch 10（roadmap 下注路径）+ Ch 12（风险 / pre-mortem）重点加重**；Ch 6（功能架构）裁为"现状对该新能力的承载力"；Ch 5（竞品图谱）裁为白地图；do_not_drop = Ch 1/2/8/9/10/12/13 |
| **5. persona_tm_weighting** | Strategist = **重**（`[TM-3, TM-5, TM-7, TM-8, TM-9, TM-13]` — 四风险 + 显性权衡 + 影响层级 + pre-mortem + 杠杆 + 前瞻）；EA = **轻**（`[TM-1, TM-6]` — 看 unmet 与弦外之音）；跨人格门 = `[TM-4, TM-11]` 全员（**TM-11 可证伪性对下注尤其关键**）|
| **6. capability_specific** | aspect_fields: `trend_scan` / `unmet_outcomes` / `whitespace_canvas` / `future_capability_map` / `disruption_threats` / `defensibility` / `pre_mortem_top3` / `build_cost_estimate` / `recommended_bets` / `validation_conditions`；gap_checks: §3.1；floor_items: §3.2 |

---

## 2. 8 段骨架（报告主干）

每段 = 主用方法（引通用 §4）+ 证据标准 + 报告落点。

### 段 1 · 趋势扫（市场 / 技术 / 竞争）
- **方法**：[M-Porter](../pm-deep-research-spec.md#m-porterporter-五力)（仅 M1 行业层；判长期供需 / 替代品 / 新进入者）+ 技术成熟度曲线 + 竞争节奏（来自 [M-Changelog](../pm-deep-research-spec.md#m-changelogchangelog--版本时间线--实际动作证据)）
- **证据标准**：每条趋势附 Tier 1/2 来源 + 时间窗（12-36 月）
- **报告落点**：Ch 2（边界）+ Ch 8 序
- **人格 / TM**：Strategist / TM-13

### 段 2 · 未满足 job + outcome（ODI underserved）
- **方法**：[M-JTBD](../pm-deep-research-spec.md#m-jtbdjobs-to-be-done) + [M-ODI](../pm-deep-research-spec.md#m-odiopportunity-algorithm)
- **证据标准**：ODI 重点放在 **underserved (>10)** outcome；估算 Imp/Sat 时强标 TM-4
- **报告落点**：Ch 4 + Ch 9（机会矩阵）
- **人格 / TM**：EA / TM-1/6 + Strategist 收敛

### 段 3 · 白地 + 定位 canvas
- **方法**：[M-Positioning](../pm-deep-research-spec.md#m-positioningstrategy-canvas--感知图) — 用 buyer-validated 轴或**未来 emerging** 轴画 value curve，标白地
- **证据标准**：白地须给"为何无人占据" + "未来 12-36 月谁可能占据"
- **报告落点**：Ch 5（裁为白地图）
- **人格 / TM**：Strategist / TM-9/13

### 段 4 · 未来能力映射（AI / 硬件 / 数据 / 社区）
- **方法**：跨候选能力类型（AI / 硬件 / 内容 / 社区 / 数据）逐项映射"能干什么 + 我方现状承载力 + 与段 2 的 unmet 是否对位"
- **证据标准**：能干什么部分必须有 Tier 1/2 技术 / 产品依据，禁止纯臆测
- **报告落点**：Ch 8（核心）
- **人格 / TM**：Strategist / TM-9/13 + EA / TM-1 看 job 对位

### 段 5 · 颠覆威胁与可防御性
- **方法**：[M-Disruption](../pm-deep-research-spec.md#m-disruptionchristensen-颠覆理论)（Christensen 维持性 vs 颠覆性）+ [M-Cagan-4Risks](../pm-deep-research-spec.md#m-cagan-4riskscagan-四大风险) 商业可行性维（护城河 / 锁定 / 规模效应）
- **证据标准**：每威胁附"为何这是颠覆 / 仅维持"的判定依据
- **报告落点**：Ch 5 末 + Ch 12 头
- **人格 / TM**：Strategist / TM-8/13

### 段 6 · pre-mortem
- **方法**：[M-PreMortem](../pm-deep-research-spec.md#m-premortempre-mortem) — 假设 12-18 月后已失败，列三大死因（Tigers / Paper Tigers / Elephants）
- **证据标准**：每死因附"为何这会发生"的依据（不是泛泛风险列举）
- **报告落点**：Ch 12（核心）
- **人格 / TM**：Strategist / TM-8（强制）

### 段 7 · build-cost & 可行性
- **方法**：[M-Changelog](../pm-deep-research-spec.md#m-changelogchangelog--版本时间线--实际动作证据)（同能力域对手历史迭代节奏 = build-cost 下界代理）+ [M-Cagan-4Risks](../pm-deep-research-spec.md#m-cagan-4riskscagan-四大风险) 可行性 / 商业可行性维
- **证据标准**：build-cost 估算必须显式区间（如"6-12 月达成基础能力"）+ TM-4 标证据等级
- **报告落点**：Ch 9（复杂度列）+ Ch 10
- **人格 / TM**：Strategist / TM-9 + TM-12 借自 EA 看 changelog 实际动作

### 段 8 · 推荐下注 + 验证条件
- **方法**：综合段 2-7，给 1-3 个 **可证伪的下注方向**，每个附"什么条件下进 / 什么条件下停"
- **证据标准**：每下注方向给 [M-Cagan-4Risks](../pm-deep-research-spec.md#m-cagan-4riskscagan-四大风险) 四风险评级 + 显性权衡（TM-5："选 X = 放弃 Y"）
- **报告落点**：Ch 1（结论）+ Ch 10（roadmap 下注路径）+ Ch 11（验证实验与指标）
- **人格 / TM**：Strategist / TM-3/5/7/9/11/13（**TM-11 可证伪强制**）

---

## 3. Innovation-direction-specific Gap / Floor

通用 gap / floor 见 [通用规格 §9](../pm-deep-research-spec.md#9-gap-检测清单--quality-floor通用部分)；以下为本 profile 追加项。

### 3.1 Gap 检测

| 检测项 | 不合格条件 | 处理 |
|---|---|---|
| 趋势时间窗 | 趋势无 12-36 月时间窗 | 补时间窗 |
| Underserved outcome | 段 2 ODI 无 underserved 项 / 全 <10 | 标"无明显机会"或换问题 |
| 白地解释 | 白地无"为何无人占据"+"谁可能占据" | 补，否则降置信 |
| 未来能力依据 | 段 4 能干什么纯臆测 | 补 Tier 1/2 技术依据或剔除 |
| 颠覆 / 维持判定 | 威胁无判定依据 | 补 Christensen 判定逻辑 |
| pre-mortem 死因 | 死因泛泛（"市场不接受"）| 补具体机制 + 触发条件 |
| 下注可证伪性 | 推荐下注无"什么条件下错"| **强制补**（TM-11）|

### 3.2 Quality Floor（Deep 模式追加项）

| 质量项 | 最低要求 |
|---|---|
| 趋势条数 | ≥3 条市场 / 技术 / 竞争趋势，每条 Tier 1/2 |
| Underserved outcome | ≥3 个 ODI >10 |
| 未来能力候选 | ≥2 个候选（AI / 硬件 / 数据等）|
| pre-mortem 死因 | 3 个，每个附机制 |
| 推荐下注 | 1-3 个，每个附 4 风险评级 + 验证条件 + 显性权衡 |
| 可证伪性 | 每下注方向**强制**附"什么条件下错" |

---

## 4. Aspect Schema · innovation-direction 扩展字段

通用扩展见 [通用规格 §8](../pm-deep-research-spec.md#8-aspect-report-schema通用扩展字段)；本 profile 追加：

```json
{
  "capability": "innovation-direction",
  "trend_scan": [],                  // 段 1：{trend, window, evidence_refs}
  "unmet_outcomes": [],              // 段 2：ODI underserved 子集
  "whitespace_canvas": {},           // 段 3：axes + value_curves + whitespace + why_unoccupied + future_occupants
  "future_capability_map": [],       // 段 4：{capability_type, can_do, our_carry_capacity, unmet_alignment}
  "disruption_threats": [],          // 段 5：{threat, type:sustaining|disruptive, basis}
  "defensibility": {},               // 段 5
  "pre_mortem_top3": [],             // 段 6
  "build_cost_estimate": {},         // 段 7：{capability, range_months, evidence_tier}
  "recommended_bets": [],            // 段 8：{bet, four_risks, tradeoff, falsifiability_condition, validation}
  "validation_conditions": []        // 段 8
}
```

---

## 5. 人格 / TM 详尽分配（8 段 × 人格）

跨人格质量门（TM-4 / TM-11）注入所有 aspect；本 profile **Strategist 重 / EA 轻**：

| 段 | 主人格 | TM |
|---|---|---|
| 1 趋势扫 | Strategist | TM-13 |
| 2 unmet + ODI | EA + Strategist | TM-1/6 + TM-9 |
| 3 白地 canvas | Strategist | TM-9/13 |
| 4 未来能力映射 | Strategist | TM-9/13（+ EA TM-1 看对位）|
| 5 颠覆威胁 + 可防御性 | Strategist | TM-8/13 |
| 6 pre-mortem | Strategist | TM-8（强制）|
| 7 build-cost | Strategist | TM-9 + TM-12 借 |
| 8 推荐下注 | Strategist | TM-3/5/7/9/11/13（**TM-11 强制**）|

---

## 6. 验证状态

- ✅ **端到端 golden 验证通过**（2026-05-30）—— 8/8 aspect 收敛，最终评分满分。
- **核心验证结果**：
  - **TM-11 hard gate 100% 通过**（3/3 推荐下注全有 leading indicator + 阈值 + 副 indicator + 触发响应）——profile §3.2 "可证伪性覆盖率 = 100%" 达成。
  - **视觉证据（C1）达标**：innovation-direction 报告类型偏战略图（trend chart / canvas / capability map / press / docs），Tier 1 official 来源天然 ≥7 张达标——**本 capability 报告类型自身不依赖 Layer-2 in-app capture**。
- **装配契约实测可行**（profile §0 表 5 + §1/§3 全过）：
  - **Strategist 重 / EA 轻 7:1 配比** ✅（段2 sole-EA；段4/段7 通过 prior_sources 引用 EA 数据）；
  - **段6 pre-mortem 三死因强制 floor** ✅（Tiger / Elephant / Paper Tiger Christensen 类型 + 机制 + 触发条件 + 早期信号 + 止损动作）；
  - **段8 TM-11 hard gate** ✅（3/3 bets 全过）；
  - **13-section narrative report template** ✅（Ch 8/9/10/12 加重 + Ch 5 裁为白地图 + Ch 6 裁为承载力评估 + Ch 11 加重 TM-11 验证实验 + do_not_drop Ch 1/2/8/9/10/12/13）；
- 黄金课题：海外运动/健身 AI Coach + 长流程自适应训练计划赛道未来 12-36 月下注方向。incumbent baseline = Strava+Runna (2025-04 收购后实体)。
- **Strategist-futurist 加重变体不必要**：通用 strategist persona 在 7 个 aspect 上承载充分。
- 8 段实跑成本：8 aspect + 62 evidence + ~50 min wall time（含 sequential retry）+ deep tier max_total_model_calls=50 / max_total_search_calls=42。
- 详 [golden report](../evaluation/golden/innovation-direction-ai-coach-bets.md) + [evaluation score](../evaluation/golden/innovation-direction-rubric-score.md)。
