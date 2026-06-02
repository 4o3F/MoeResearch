# Capability Profile · 产品能力研究 (product-capability)

> 通用 frame：[`../pm-deep-research-spec.md`](../pm-deep-research-spec.md)（人格 / 13 TM / 4-tier 证据 / 视觉证据 / 反幻觉 / 行文 floor / 优雅降级 / Lapis 接口边界等**所有跨能力机制**以通用规格为准）。
> 上游： plan §3.2、ROADMAP §1（4 项能力之 ②）。

---

## 0. 核心问题

在某**能力域**里，我方做得多好、断点在哪、补什么能赢？比 [competitive](competitive.md) profile 更"内向 + 体验纵深"：单产品（或 1 + 2-3 个 benchmark 对手）内的功能 / 流程 / 体验 / 数据能力深度剖析，目的是补齐能力或在能力域上建立差异化。

> 与 competitive 的区别：competitive 关心"竞争集与差异化"（跨竞品的广度），product-capability 关心"某能力域的成熟度与体验"（单产品 / 少数对手的纵深）。

---

## 1. 装配契约（§通用规格 §11 六字段）

| 字段 | 值 |
|---|---|
| **1. decision_intent_affinity** | `improve` / `build` / `differentiate`（聚焦在某能力域）|
| **2. mece6_emphasis** | primary = `M4` Product & Experience Capabilities（纵深）+ `M2` User & JTBD（该能力服务的 job 与 outcome）；supporting = `M3` Competitive Landscape（benchmark 段，**不必图谱**）+ `M6` Future Capability（升级方向）；contextual = `M1` Market Context + `M5` Business & Growth Model |
| **3. skeleton** | 6 段：①能力域定义 + 服务的 user jobs 分解 → ②**单能力域** feature teardown（深度版）→ ③体验路径 + 断点地图（first-class）→ ④Kano 在该能力域内分级 → ⑤ODI 在能力 outcome 上打分 → ⑥竞品 benchmark + build-cost & 升级方向（详 §2）|
| **4. report_template** | family = **A（13 章变体）**；weighting = **Ch 6（功能架构与体验路径）+ Ch 7（视觉证据）+ Ch 4（JTBD）重点加重**；Ch 5（竞品图谱）裁为 benchmark 段；Ch 8 视升级方向决定是否展开；do_not_drop = Ch 4/6/7/9/11/12/13 |
| **5. persona_tm_weighting** | EA = **重**（`[TM-1, TM-2, TM-6, TM-10, TM-12]` — Job→Feature→Gap + metrics-informed + 听弦外之音 + 5Qs + 言行分离）；Strategist = **轻**（`[TM-9, TM-13]` — 杠杆点 + 前瞻仅用于升级方向）；跨人格门 = `[TM-4, TM-11]` 全员 |
| **6. capability_specific** | aspect_fields: `capability_domain` / `experience_path` / `breakpoint_map` / `kano_grades_in_domain` / `opportunity_scores_in_domain` / `benchmark_set` / `upgrade_directions`；gap_checks: §3.1；floor_items: §3.2 |

---

## 2. 6 段骨架（报告主干）

每段 = 主用方法（引通用 §4）+ 证据标准 + 报告落点。

### 段 1 · 能力域定义 + 服务的 user jobs 分解
- **方法**：[M-JTBD](../pm-deep-research-spec.md#m-jtbdjobs-to-be-done) — 写清能力域**边界**（哪些功能 / 流程算"该能力域内"）+ 该能力服务的 ≥3 个 user job
- **证据标准**：边界须给"为何排除 X"理由；每 job 给 situation→motivation→outcome
- **报告落点**：Ch 4
- **人格 / TM**：EA / TM-1

### 段 2 · 单能力域 feature teardown（深度版）
- **方法**：[M-Teardown](../pm-deep-research-spec.md#m-teardown功能对位矩阵--feature-teardown) — 单产品（或 1 + 2-3 个 benchmark 对手），按能力域内功能 / 步骤逐项评分（含步数 / 错误率 / 时延 / 错峰）
- **证据标准（强制）**：**每行必须附 visual_evidence 或操作步数**——纯文字描述不足以支撑 product-capability 的纵深判断
- **报告落点**：Ch 6（核心）
- **人格 / TM**：EA / TM-2/12

### 段 3 · 体验路径 + 断点地图（first-class，远胜 competitive profile）
- **方法**：体验路径图（user journey）+ 断点（drop / 卡顿 / 错误 / 困惑点）
- **证据标准**：每断点附 visual_evidence + Tier 3 用户评论（≥3 条同模式 = 不是孤例）
- **报告落点**：Ch 6（与 teardown 并列）+ Ch 7（视觉证据资产表）
- **人格 / TM**：EA / TM-1/6/10

### 段 4 · Kano 在该能力域内分级
- **方法**：[M-Kano](../pm-deep-research-spec.md#m-kanokano-模型) — 把段 2 的功能 / 段 3 的体验断点按 Must-be / Performance / Attractive 分级
- **证据标准**：分级须有用户证据或明确标 practitioner 诠释（TM-4）
- **报告落点**：叠在 Ch 6 之上
- **人格 / TM**：EA

### 段 5 · ODI 在能力 outcome 上打分
- **方法**：[M-ODI](../pm-deep-research-spec.md#m-odiopportunity-algorithm) — 把段 1 的 user job 拆出的 desired outcomes 跑 ODI
- **证据标准**：Imp / Sat 估算时标 TM-4；underserved (>10) outcome 直接喂入段 6
- **报告落点**：Ch 9（机会矩阵）
- **人格 / TM**：Strategist（结论）+ EA（数据）

### 段 6 · 竞品 benchmark + build-cost & 升级方向
- **方法**：[M-Disruption](../pm-deep-research-spec.md#m-disruptionchristensen-颠覆理论)（该能力域 best-in-class 的竞品 benchmark）+ [M-Changelog](../pm-deep-research-spec.md#m-changelogchangelog--版本时间线--实际动作证据) + [M-PreMortem](../pm-deep-research-spec.md#m-premortempre-mortem)
- **证据标准**：benchmark **不做竞品图谱**，只取 best-in-class 2-3 个对手作"该能力最佳实践参照"；build-cost 用 changelog 估算自建成本下界
- **报告落点**：Ch 5（裁为 benchmark 段）+ Ch 8（升级方向）+ Ch 10（roadmap）+ Ch 12（pre-mortem）
- **人格 / TM**：Strategist / TM-9（杠杆点）+ TM-13（前瞻）

---

## 3. Product-capability-specific Gap / Floor

通用 gap / floor 见 [通用规格 §9](../pm-deep-research-spec.md#9-gap-检测清单--quality-floor通用部分)；以下为本 profile 追加项。

### 3.1 Gap 检测

| 检测项 | 不合格条件 | 处理 |
|---|---|---|
| 能力域边界 | 边界模糊 / 无排除理由 | 补段 1 边界论证 |
| 体验路径 | 无路径图 / 无断点标注 | 补段 3（必填，本 profile 核心证据）|
| 断点 visual_evidence | 断点描述无截图 / 视频证据 | 标缺口，**不得给"用户痛点"强结论** |
| 用户证据样本量 | 单条评论支撑断点 | 补 ≥3 条同模式或标假设 |
| Benchmark 选择 | benchmark 对手非 best-in-class（无 best-in-class 理由）| 补选择理由或换 |
| Build-cost 缺失 | Build 意图但无 changelog 估算 | 补 changelog 时间线 |

### 3.2 Quality Floor（Deep 模式追加项）

| 质量项 | 最低要求 |
|---|---|
| 体验路径图 | ≥1 张完整路径图 + ≥3 断点标注（核心证据，**强制**）|
| 断点 visual_evidence | 每断点 ≥1 张截图 / 视频帧 |
| 用户证据 | 每断点 ≥3 条同模式 Tier-3 证据（或标假设）|
| Benchmark 对手 | 2-3 个 best-in-class（非随机选）|
| 能力域 outcome | 至少 3 个 desired outcome 跑过 ODI |

---

## 4. Aspect Schema · product-capability 扩展字段

通用扩展见 [通用规格 §8](../pm-deep-research-spec.md#8-aspect-report-schema通用扩展字段)；本 profile 追加：

```json
{
  "capability": "product-capability",
  "capability_domain": { "name": "", "boundary": "", "excluded_with_reason": [] },
  "user_jobs": [],                  // 段 1
  "feature_teardown_deep": [],      // 段 2：含 step_count / error_rate / latency / visual_refs
  "experience_path": [],            // 段 3：journey steps
  "breakpoint_map": [],             // 段 3：每断点 {step, type, visual_refs, user_evidence_refs}
  "kano_grades_in_domain": [],      // 段 4
  "opportunity_scores_in_domain": [], // 段 5
  "benchmark_set": [],              // 段 6：best-in-class 2-3 对手
  "upgrade_directions": []          // 段 6
}
```

---

## 5. 人格 / TM 详尽分配（6 段 × 人格）

跨人格质量门（TM-4 / TM-11）注入所有 aspect；本 profile **EA 重 / Strategist 轻**：

| 段 | 主人格 | TM |
|---|---|---|
| 1 能力域 + JTBD | EA | TM-1 |
| 2 单域 teardown 深度 | EA | TM-2/12 |
| 3 体验路径 + 断点 | EA | TM-1/6/10 |
| 4 Kano 域内分级 | EA | TM-1/6 |
| 5 ODI 域内打分 | Strategist + EA | TM-9 + TM-2 |
| 6 benchmark + build-cost + 升级 | Strategist | TM-9/13（+ TM-12 借自 EA 看 changelog 实际动作）|

---

## 7. De-AI Voice Pass 哨兵（product-capability-specific）

通用 voice pass 见 [`skills/pm-deep-research/prompts/layer1/phase-d-voice-pass.md`](../../../skills/pm-deep-research/prompts/layer1/phase-d-voice-pass.md)。本 profile 追加 product-capability-specific 加权项：

| 哨兵 | 触发条件 | 不达标动作 |
|---|---|---|
| TM-4 全员 | ODI 公式 outcome 列未标 TM-4 | 补 fact/interpretation/assumption/speculation 标注 |
| ODI underserved (>10) 公式 | finding 引 "underserved" 无公式回链 | 补 `(importance + max(0, importance - satisfaction)) = X` |
| Kano 分级 user evidence | Kano grades 单源 | 补 ≥3 一手或标 practitioner 诠释（TM-4）|
| 体验路径 + 断点 visual | 段3 断点描述无 visual_refs | 标缺口；不得给 "用户痛点" 强结论 |

voice pass **不准**洗去：v2.1 13-section narrative report 的 "4-tier 来源标签 + estimated flag" 双轨 / 段6 build-cost overlay / 体验路径图标注 / Ch 12 风险与开放问题段。
