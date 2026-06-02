# v2.3 Golden Report (R4-b · #9 engine) · Endurance-athlete Explainable Biometric Coach — PR-FAQ 前置物 (PRD Input Deck)

> Profile: **product-requirements** (v2.3) · **Family B 8 段 PR-FAQ 模板** · decision_intent: **build** · audience: PM/TPM/Eng/Design leads
> Subject: 新创 endurance-athlete-focused AI biometric coach — HRV/strain/sleep explainability + optional human-in-loop trainer review + cross-wearable data fusion
> **本报告 = v2.3 在 #9 最终引擎（vendored Lapis `9db7464`，verbatim、零本地修改）上的全量真实复跑权威黄金**（取代 M7.5 design-validation 预演 [`m75-golden-report-explainable-biometric-coach-prfaq.md`](m75-golden-report-explainable-biometric-coach-prfaq.md)）。
> Lapis 运行：deep_research 11 aspect → 直接 4/11（3 cagan micro value/usability/feasibility + open-questions）；6 aspect 经 aspect_research 补跑收敛（pr-faq/jtbd-odi/cagan-business/ost/requirements/metrics）；evidence-table = R4-f Phase B 跨段聚合。**最终 10/10 dedicated aspect 全 ok（cagan 4/4 全收敛）**。
> 证据：10 dedicated aspect / 48 findings / **79 evidence** / dangling=0（详 [`rerun9-merged-10of10.result.json`](rerun9-merged-10of10.result.json) + [`rerun9-bundle.md`](rerun9-bundle.md)）。
> 自评：详 [`rerun9-rubric-score-v23.md`](rerun9-rubric-score-v23.md)（R1 回归闸门：vs M7.5 锚点 24/24）。
> **段3 由 4 个 dedicated cagan micro-aspect 装配**（`cagan-risk-value` 5 findings/5 evidence + `cagan-risk-usability` 5/5 + `cagan-risk-feasibility` 6/7 + `cagan-risk-business` 5/5）。R4-e 核心在 #9 引擎复现：M7 单 cagan-4risks aspect search-saturation 5 retries 全败 → 拆 4 micro 后 4/4 收敛。

---

## 段 1 · Press Release Frame (PR-FAQ) — 加重 (family B BLUF 等价)

来源：`pr-faq-frame:finding-1`~`finding-5`. 价值主张语句 trace 回段2 ODI outcomes (`jtbd-odi-kano:finding-1`/`finding-3`).

### PR (≤300 字)

**Headline**：让耐力训练从"照做"变成"明白后再做"。

**Sub-headline**：面向 5K 至马拉松跑者，把睡眠、恢复和训练负荷转成可追问、可调整、可安心执行的计划；高风险调整可请真人教练把关。

**PR 正文**：今日推出 Endurance Coach。当训练计划、身体感受与穿戴提示互相冲突时——长跑前睡眠差、间歇日前恢复低、伤后复训不确定——它不再让跑者盲从绿/黄/红分数，而是给出"为什么改"的可追问理由（哪个信号触发、置信度、可替代训练），用户可一键"我不同意"展开辩论；当 readiness 显著下行或上报伤痛/缺训时自动建议安全降级；高风险升级方案（如改重练日）可请教练异步复核。跨 Apple Watch / Garmin / WHOOP / Strava / TrainingPeaks 汇集恢复与训练数据。订阅价 anchor 在 Runna $119.99/yr 区间（仍在校准）。

**客户引言**："当半马倒计时三周、手表说我恢复差但课表要求间歇时，我以前只能猜该硬撑还是休息；现在我能看到理由、提出异议，并在高风险调整前获得教练把关。" — 目标 1:35 半马跑者 (`pr-faq-frame:finding-1`)

Reach refs: `pr-faq-frame:ev-1-1` `ev-7-20` `ev-9-24` · `jtbd-odi-kano:ev-1-1` `ev-4-10`.

### 内部 FAQ (≥5；`pr-faq-frame:finding-2`)

| # | Question | Answer (价值导向) | Confidence |
|---|---|---|---|
| 1 | **为何现在做** | 跑者已为训练计划/恢复洞察付费，官方竞品已把"自适应计划""恢复/训练就绪"教育成熟；下一步竞争从"给计划/给分数"转向"让我理解并安全调整" | medium |
| 2 | **4 大风险如何应对** | **value**：不把"解释"当卖点本身，而把"避免一次错误硬撑/盲目休息/错过比赛目标"作为价值；**usability**：解释降焦虑而非加仪表盘（progressive disclosure）；**feasibility**：只在足够可信时给建议，不确定时说不确定 + 保守/教练复核；**business**：价格以 Runna/TrainingPeaks/WHOOP 锚定，先证明降决策成本再谈高价。**4 风险由段3 的 4 个 dedicated cagan micro-aspect 评估** | medium |
| 3 | **度量怎么算成功** | 北极星 = 用户每周完成并信任的关键训练决策数（P2 Explained Adaptive Training Loops）+ D14 激活 + D30/D90 coached 留存；护栏 = 伤痛/过载自报不升、解释不增焦虑、续费高于计划 App 锚点 (`metrics-tree:finding-2`) | high |
| 4 | **与 Runna/TrainingPeaks/WHOOP/Garmin 区别** | Runna=个性化跑步计划，TrainingPeaks=严肃训练工作台，WHOOP=恢复/睡眠/压力洞察，Garmin=设备内训练建议；**本产品在"为什么今天这样练 / 我不同意怎么办 / 风险高时谁把关"三轴占位** (`pr-faq-frame:finding-4`) | high |
| 5 | **下一步** | Phase B 用段2 ODI evidence 替换占位价值主张；先验证半马/马拉松备赛高风险调整场景的付费门槛 (`open-questions-experiments:finding-1`) | high |

### 外部 FAQ (≥3；`pr-faq-frame:finding-3`)

| # | Question | Answer |
|---|---|---|
| 1 | 何时用 | 训练计划、身体感受、穿戴提示冲突时——长跑前睡眠差、间歇日前恢复低、伤后复训不确定；目标是少猜、少逞强、少错过关键训练 |
| 2 | 收费 | 价格未定；参考 Runna $119.99/yr、TrainingPeaks $134.99/yr、WHOOP $199/yr 续费 + 健康健身年度订阅锚点，只在用户确认它降低错误训练决策成本后收费 (`cagan-risk-business:ev-1-1`) |
| 3 | 与友商不同 | 不拼计划库深度（Runna/TrainingPeaks 更全），不拼生物指标精度（WHOOP/Garmin 更准）；拼"训练日决策可解释 + 可辩论 + 高风险可请真人复核"的端到端闭环 |

> **PR-FAQ 输出标记**：段1 结构完整；价值主张 trace 回段2 Opp>10 outcomes；4 风险 FAQ 由段3 **4 个 dedicated cagan micro** 提供（confidence medium = 证据基础限制，非方法缺失）。

---

## 段 2 · 机会验证 (JTBD + ODI + Kano + Opportunity Landscape) — 加重

### Job Statement (TM-1 Job→Feature→Gap；`jtbd-odi-kano:finding-1`)

**When** I am 6–16 weeks into a 5K/10K/half/marathon block and my plan collides with poor sleep, HRV/HR anomalies, missed sessions, injury niggles, or pace targets that feel wrong,
**I want to** know whether to execute, modify, or skip the next workouts — and *why* —
**so I can** improve race performance without overreaching, losing consistency, or blindly trusting an opaque score.

> Existing path = Runna 计划/配速 + WHOOP 恢复/strain + Strava HR zones + Reddit 自解读；Gap = 跑者仍须在割裂、半黑盒的工具间自己调和"今天到底练不练"。Unsaid = 不是更多生物指标图表，而是**一个可信、可质疑、可安全覆盖的下一步训练动作**。

### ODI Outcomes (estimated_flag=true; TM-4; Opp = Imp + max(0, Imp − Sat)；`jtbd-odi-kano:finding-2`)

| ID | Outcome | Imp | Sat | Opp | Kano | Underserved? |
|---|---|---:|---:|---:|---|---|
| O1 | 关键训练前把累计疲劳/过度训练风险提前识别并转成 next-7-days 调整 | 9 | 5 | 13 | Must-be | ✅ Opp>10 |
| O2 | 计划被改时把原因解释为可验证 top drivers（HRV/睡眠/负荷/RPE/伤痛）并允许辩驳/覆盖 | 8 | 4 | 12 | Attractive | ✅ |
| O3 | 把 pace/强度/质量课频率限制在安全边界（尤其 workout 过激或伤痛时） | 9 | 4 | 14 | Must-be | ✅ |
| O4 | biometric 数据可信度（缺失/过期/冲突时仍能正确决策） | — | — | >10 | Performance | ✅ |
| (overserved 对照) O7 | opaque readiness/strain/fitness badge | 低 | 高 | <0 | Indifferent | ❌ overserved → 不做候选 |

**核心机会** (`jtbd-odi-kano:finding-3`)：underserved（Opp>10）集中在 O1 fatigue/overreaching detection、O2 explainable/debatable plan change、O3 safe load/pace boundary、O4 biometric data confidence — 用户不缺训练日历，缺**可解释、可辩论、带风险闸门的 next-run/next-week 调整**。

**Overserved / no-do** (`jtbd-odi-kano:finding-4`)：opaque readiness/strain badge 已饱和；TM-12 say-vs-do — 用户口头称赞 structure/flexible scheduling/Garmin integration，行为上 tone down suggestions / add easy mileage / ignore pace alerts。**v1 不做 louder motivation / status score**。

**Falsifiability (TM-11)** (`jtbd-odi-kano:finding-5`)：Imp/Sat 全 estimated；最大风险 = 把 forum/review pain 误读成普遍需求。对高风险调整设 human-in-loop / conservative default（injury、large HRV drop、sensor anomaly、rapid load increase、repeated failed workouts）。

### 段尾 prose

核心机会 = 可解释、可辩论、带风险闸门的训练调整，因 ODI 中 Opp>10 的 outcomes 都不是缺训练日历，而是缺对"身体信号如何改变训练决策"的可信解释。**Confidence: medium**（估算，待首批 cohort 校准）。

---

## 段 3 · Cagan 4 大风险 (hard gate ✅ — **R4-e: 4 个 dedicated micro-aspect 装配**)

> **Phase A 实施记录 (R4-e on #9)**：段3 cagan-4risks 在 M7 是单 aspect、5 次 backfill 全失败（search-saturation pathology）。R4-e 拆为 **4 个 micro-aspect (1 risk class / aspect, max_search=4)**。#9 全量复跑：value/usability/feasibility **deep run 直接收敛**，business 1 次 aspect_research 补跑收敛 → **4/4 全收敛**。每类独立证据 + 独立 confidence + TM-3 边界。

| 风险类 | 描述 | 证据等级 | 来源 refs | 应对策略 |
|---|---|---|---|---|
| **value** | 相邻付费锚点（Runna **$119.99/yr**、TrainingPeaks **$134.99/yr**、WHOOP **$199/yr 续费**）证明 serious/sub-elite 跑者已为"训练计划/严肃平台/生物指标恢复"付费；但**用户更可能买"更安全变快/少受伤/按时完赛"，不是买解释本身**——explainability-specific WTP 无直接证据；高于 Runna anchor 时 churn 风险高 | medium（相邻 WTP）；**low**（explainability-specific WTP / churn） | `cagan-risk-value:ev-1-1` `ev-2-3` `ev-2-4` `ev-3-6` `ev-3-7` | MVP 定价 **$99–$119.99/yr（Runna-parity）**；核心实验 = "解释+debate+risk review 是否提高付费转化/plan acceptance/missed-workout 留存"（**不是"用户喜欢解释吗"**）；human review 做 opt-in quota/per-risk add-on，不默认抬全量订阅价 |
| **usability** | 最危险**不是"不够透明"，而是把低信噪比、可能与主观体感冲突的 readiness 信号包装成确定性解释**（"感觉好但系统说休息"→困惑/反复 debate/直接忽略）；解释做成 SHAP/特征重要性 panel = 认知负担（mHealth/XAI/CDS 实证）| medium（相邻 mHealth/XAI/CDS 迁移；无一手原型）| `cagan-risk-usability:ev-1-*` `ev-2-*` `ev-3-*` | **progressive disclosure（L0 一句话决策 / L1 2–3 驱动因素 / L2 confidence badge / L3 可展开）**，非 dashboard wall；debate 默认折叠 + 1-tap "这不对" + 只让用户改少数高影响输入；**build 前先做 micro-prototype test** |
| **feasibility** | **12–24 月 MVP 可行但风险高**：HealthKit 可读 HRV SDNN、WHOOP API 有 OAuth scopes、**Garmin Health API 须 partner 审批 + 非 self-serve**；关键风险 = 多厂商数据权利/语义/延迟/缺失能否形成稳定 coaching 输入；FM/LLM 加速应用层 2–4x 但**不加速 SDK 权限/HRV 生理校准/安全验证** | medium（官方 SDK 文档 evidenced + 专家估计）| `cagan-risk-feasibility:ev-1-1` `ev-1-2` `ev-2-4` `ev-3-7` `ev-4-9` | **tiered connector（T0 HealthKit+Strava → T1 WHOOP → T2 Garmin after approval）+ canonical signal subset + 缺失核心信号时禁自动 risky change**；架构 = rules + 轻量 risk/load model + LLM explanation wrapper（**LLM 不直接改计划**）；staged hard gates |
| **business viability** | **三重挤压**：① 订阅 ARPU 被 Runna $119.99/yr + **Strava+Runna bundle $149.99/yr** 锚定；② **CAC 被 Strava-owned Runna 抬高**（Strava 收购 Runna，150M+ athletes 社区 + bundle 折扣降获客成本，新创无独立 wedge）；③ trainer review 抽成（15–40% take-rate）是 **trust add-on + ops overhead 非 launch-stage 10x 引擎** | medium（订阅经济：官方价格 + RevenueCat）；**low**（trainer marketplace：行业 blog，缺一手 CAC/attach/handle-time）| `cagan-risk-business:ev-1-1` `ev-1-2` `ev-2-3` `ev-3-6` `ev-3-7` | **subscription-first + curated human review，不开放 trainer marketplace**；trainer review 只做高风险调整付费 add-on；商业 gate = **CAC payback <6–9 月 + annual renewal 支撑 LTV/CAC + trainer 单审毛利达标**才扩供给 |

### 段尾 prose

**最大风险 = feasibility（high）+ business viability（分发/CAC 被 Strava-owned Runna 结构性挤压）双重**。R4-e dedicated 4-risks 给出各类独立证据深度：value 锚三家官方定价 + RevenueCat、usability 引 mHealth/XAI/CDS 实证、feasibility 锚 HealthKit/WHOOP/Garmin 官方 SDK 约束、business 锚 Strava 收购 Runna。**应对 = 把 4 类 risk 收敛为段8 的 OQ1(usability)/OQ2(value+business)/OQ3(feasibility 取数)/OQ4(routing) 实验，3–6 月内分别验证或砍。不为分数注水：value 的 explainability-specific WTP + business 的 trainer unit economics 仍标 low confidence。**

---

## 段 4 · Torres OST 解空间 (hard gate: ≥3 候选 / outcome) — 加重

来源：`ost-solution-space:finding-1`~`finding-4`. **3 underserved outcomes × ≥3 候选 = 9 candidates ✅**（每候选附最危险假设 + 现状/不同 + 新建 vs 复用）。

### O1 — HRV/strain/sleep 或负荷异常时，知道为什么改训练并能反驳

| 候选 | 名称 | 可行性 | 用户价值 | 最危险假设 |
|---|---|---|---|---|
| **O1-C1** | **解释型调整理由引擎**（摄取 HRV/睡眠/恢复/负荷 → reason chain：哪个信号触发/置信度/可替代训练）| 中高（数据入口+竞品机制已存在，解释质量需自建）| 高但待证 | 生理信号↔训练变更可稳定映射为可理解理由，否则解释层=科学噪音 |
| O1-C2 | 反驳/约束输入回路（用户输入腿痛/旅行/没睡好 → 系统重排并记录采纳/拒绝原因）| 中 | 中高（TM-6/12 约束前置）| 用户愿在关键时刻提供真实约束 |
| O1-C3 | What-if 训练变更解释器（同日给 2-3 个调整：照跑/降强度/休息 + 各自对恢复/周量/比赛影响）| 中 | 高但待证 | 目标用户愿比较方案并理解 trade-off |

### O2 — 风险训练决策可被人类教练复核

| 候选 | 名称 | 可行性 | 用户价值 | 最危险假设 |
|---|---|---|---|---|
| **O2-C1** | **Red-flag 教练复核队列**（仅低恢复+高强度/临赛大改/连续缺训命中规则时异步发教练）| 中 | 高但待证 | 高风险调整足够少 + 用户愿为复核付费，否则单位经济失败 |
| O2-C2 | Ask-a-coach second opinion 按钮（一键提交 7-14 天训练/恢复 + AI rationale 获异步短评）| 中高 | 中高 | 用户在不确定时愿等待异步回答 |
| **O2-C3** | **Safety guardrails 先行**（规则拦明显不安全加量/强度，冲突或坚持时升级人审）| 高 | 中高 | 低恢复/高负荷信号能识别足够多风险场景 |

### O3 — 跨设备（Garmin/Apple/WHOOP/Strava）不被单源锁定

| 候选 | 名称 | 可行性 | 用户价值 | 最危险假设 |
|---|---|---|---|---|
| **O3-C1** | **HealthKit-first + Garmin Health API 双入口 MVP**（iOS HealthKit 读 workouts/sleep/HRV；Garmin 经 approved partner 路径 → 统一 athlete timeline）| 中（HealthKit 清晰，Garmin 需 partner approval）| 高但待证 | Apple/Garmin 数据足够覆盖首批用户，否则 cross-wearable 承诺落空 |
| O3-C2 | 指标置信度与口径归一化层（HRV/sleep/load → 带来源/时间窗/可信度 features）| 中低到中 | 高但待证 | 跨设备指标可归一化到足以支持训练建议 |
| O3-C3 | Existing-plan overlay（不第一天替换 Runna/TrainingPeaks/Garmin Coach，只读计划意图叠加解释/风险/what-if）| 中 | 中高 | 用户愿让新创作为 overlay 存在而非完整替代 |

### 段尾 recommendation (build intent；`ost-solution-space:finding-4`)

**首选 = O3-C1 HealthKit-first + Garmin Health API 双入口为数据地基 → 上线 O1-C1 解释型调整理由引擎 → O2-C3 safety guardrails 作默认安全层 + O2-C1 人类复核只用于 red-flag case**。最危险假设 = 用户确实愿为可解释/可复核/跨设备的训练调整付费或改工作流；否则产品会被视为 Garmin/WHOOP/Runna 之外又一层通知噪音。**Hard gate ✅**：9 candidates，每个含"新建 vs 复用既有平台"对照（如 O3-C2 vs O3-C1）。

---

## 段 5 · 需求 (功能 / 非功能 / **非目标**) (hard gate: 非目标显式) — 加重

来源：`requirements-fn-nfn-nongoals:finding-1`~`finding-4`. 每条 inline grounding (e=证据/a=assumption)。

### 功能需求 (FR-1~FR-8；每条 trace 回段2 outcome + Kano)

| FR | 功能 | Outcome | Kano |
|---|---|---|---|
| FR-1 | 跑步目标与当前能力建模（比赛/日期/可训练时间/近期跑量/伤病限制 → adaptive plan）| O1 | 基本→期望 |
| FR-2 | Cross-wearable 数据连接 + 数据质量地图（每源 last-sync/可用/缺失/权限/降级）| O3,O4 | 期望 |
| FR-3 | HRV/strain/sleep/readiness 可解释建议（why-now + 正/负因子 + 新鲜度 + 置信度 + progressive disclosure）| O2,O4 | 期望 |
| FR-4 | Debate/override（自然语言质疑 + 补 soreness/illness + 2-3 可选路径 + override 入学习）| O2 | 魅力 |
| FR-5 | Biometric-informed adaptive plan update（睡眠差/HRV↓/负荷过高 → proposed change，重大变化需确认）| O1 | 基本→期望 |
| FR-6 | 高风险调整可选 human trainer review（定义 high-risk/何时建议/可解释依据/责任边界；v1 不要求公开 marketplace）| O2 | 魅力/信任 |
| FR-7 | 健康数据 consent/privacy control center（分类型授权/purpose/trainer 可见性/撤销/导出/删除）| O4 | 基本 |
| FR-8 | Data freshness/confidence alert（过期/缺失/冲突提示；关键数据缺失不输出高置信风险建议）| O4 | 基本 |

### 非功能需求 (NFR；需求层不指定实现)

| NFR | 类别 | 要求 |
|---|---|---|
| NFR-P1 | 性能-latency | Today dashboard/建议（已缓存）p95 ≤2s；新 check-in/训练后重算 p95 ≤10s；超时显 stale/partial 非空白 |
| NFR-P3 | 数据新鲜度 | 每建议显 source last-sync；关键数据刷新目标 ≤15min；HRV/sleep/recovery >12h 未更新或权限未知 → 置信度降级 + 提示 |
| NFR-S1 | 安全-least privilege | wearable/health 连接用户授权 + scope 最小化 + 可撤销；无权限用缺失状态而非 silent failure |
| NFR-S2 | 合规-privacy | HRV/睡眠/recovery/injury notes 按敏感健康数据；EU GDPR Art.9 explicit consent；US HIPAA-grade safeguards |
| NFR-C1 | 安全边界 | 定位 training guidance/education，不做诊断/治疗/处方/医疗器械 claim；red flags 建议寻求医疗意见 |
| NFR-O1 | 可观察性 | 每 recommendation 记录 input timestamps/权限/模型版本/解释版本/debate-override/human-review/训练完成；不存不必要 raw notes |

### **非目标** (hard gate ✅ 显式 + "为何不做"；NG-1~NG-7)

| Non-goal | 类型 | 为何不做 |
|---|---|---|
| NG-1 不做 strength/bodybuilding/gym/casual fitness | 战略 | 数据模型/内容/Kano 验证会被稀释；跑步 PMF 优先 |
| NG-2 不做 cycling/swim/triathlon 完整计划 | 战略 | 5K-marathon 训练负荷/配速/伤病已够复杂；防 scope creep（vNext placeholder）|
| NG-3 不做硬件/wearable OS/单厂商 exclusive | 战略 | 差异化在 cross-wearable fusion + 解释/人审，不在卖设备 |
| NG-4 不做疾病诊断/治疗/处方/return-to-play clearance | 合规/安全 | 健康数据高敏感；medical claim 显著抬合规/责任/安全风险 |
| NG-5 不做开放教练 marketplace（排班/支付/评级/获客）| v2 placeholder | v1 只验证 risky-adjustment review 的信任价值；marketplace 供给/质量/责任复杂 |
| NG-6 不做无确认 fully-autonomous 高风险变更 | v2/安全 | 解释/信任证据有限；高风险自动化放大伤害与责任 |
| NG-7 不做社交 feed/跑团/挑战赛/内容/广告 | scope creep 防御 | 会拉向 Strava/social 竞争，干扰核心留存信号 |

### 段尾 prose

需求范围 = 5K/10K/half/marathon 跑者的 cross-wearable biometric ingestion + 可解释可辩论训练/恢复建议 + adaptive plan + 数据新鲜度/置信度 + 高风险 human review。**不做 strength/cycling/硬件/医疗/marketplace/自动化/社交**，因 v1 必须验证 explainable biometric coach 的 running PMF，同时控合规、数据可得性、教练供给、scope creep 风险。

---

## 段 6 · 成功度量 (主 / 次 / 护栏 三套) (hard gate ✅) — 加重

来源：`metrics-tree:finding-1`~`finding-5`. 三套全有，每主指标 5 字段全。

### 主指标 leading (TM-9 LNO；放弃 DAU/解释浏览量/AI 对话数作北极星)

| 指标 | 杠杆 | 定义 | 成功标准 |
|---|---|---|---|
| **P1. D14 Explainable Biometric Activation** | 10x | 14 天内：连 ≥1 wearable + ≥5 天 HRV/睡眠数据 + 看 ≥3 条 explanation + ≥1 调整 accept/debate/ask-human + 完成/安全改排 ≥1 训练 | MVP gate ≥35%（任一 wearable cohort ≥25%）|
| **P2. Explained Adaptive Training Loops / athlete-week** | 北极星 | 1 loop = biometric 被解释 → accept/质疑/human-review → plan adjust 确认 → 训练完成/降载/安全取消 | Week4 起 P50 ≥2 loops/week；activated ≥60% 28 天内 ≥4 loops（不计 pure explanation views）|
| **P3. D30/D90 Coached Retention** | leading→lagging | D30/D90 仍有 coached activity（sync + 回应 guidance + 完成/改排训练，不只开 App）| stop-scale floor：开放渠道 D30 ≥8.48-12.1% / D90 ≥6.4%；PMF target activated D30 ≥30% / D90 ≥18%；human-review cohort 不低于 matched |

### 次指标 secondary (S1-S5；解释主指标，不单独作成功)

S1 Week-3 Plan Continuity（借 Runna "week three" KPI，≥65% D14-activated）· S2 Wearable Data Completeness（≥5 天数据 + ≥80% workouts synced，≥70%）· S3 Explanation Usefulness & Debate Resolution（≥60% useful + ≥80% debate 24h resolved）· S4 Human Review Leverage Ratio（高风险 review coverage ≥95%，trainer changed/confirmed 10-40%）· S5 Segment Retention Delta（race distance/wearable/paid/human-review exposure 分层）。

### 护栏 guardrails (G1-G4；≥1 关联 Runna 留存 baseline)

| 指标 | 不能让什么变差 | 阈值 |
|---|---|---|
| **G1. Runna-linked Paid Retention / Channel** | 不为 explainability/human-review 牺牲付费留存（Runna proxy：web subscribers retention 比 in-app 高 15%）| D30/D90 不低于同渠道 pre-AI cohort + 行业地板；web-paid uplift 目标 ~+15% |
| G2. Revenue / Subscription Conversion | 不因免费解释/人审成本/bundle 压力破坏商业可行性 | 90 天 gross margin 为正；WTP 不低于 Runna/Strava-Runna 锚点可解释折扣区间 |
| G3. Safety / Biometric Error | 不让错误解释/危险调整增加受伤/过载/信任损失 | 高风险未审核 <1%；explanation factual error ≤3%；severe incident → release freeze |
| G4. Existing Plan / Sync Reliability & Complaint | 不让基础计划/sync/客服因 AI 变差 | sync success ≥98%；crash-free ≥99.5%；AI/confusing-coach complaints <5/1k active |

### 段尾 prose

用 P2 explained-loop 周闭环 leading 驱动 adoption & retention，不让 G1-G4（付费留存/商业/安全/sync 可靠性）恶化。**TM-3 四风险一一对位**：value→P1/G2、usability→P2/S3、feasibility→S2/G3、business→P3/G1/G2。**主/次/护栏三套全、每主指标 5 字段全，hard gate ✅**。

---

## 段 7 · 证据与来源 (evidence-table) — **跨段聚合 (R4-f, profile §1 "允许 appendix")**

> **说明 (R4-f)**：段7 由 final-report Phase B 跨段聚合 **#9 run 的 10 个 dedicated aspect 共 79 evidence**（段7 aspect 按 R4-f 标 OPTIONAL：Lapis `evidence_refs` 不许 cite prior_sources by id，单独 spin meta-aggregation aspect 会 schema_validation_failed → 实测在 #9 上 evidence-table deep 阶段亦 network_failed，走 Phase B 聚合）。

### 4 cagan micro evidence（按命名空间）

| micro-aspect | evidence | 代表来源 (Tier) |
|---|---:|---|
| cagan-risk-value | 5 | Runna pricing (official/T1) · TrainingPeaks pricing (T1) · WHOOP pricing (T1) · RevenueCat (blog/T3) · fitness churn benchmark |
| cagan-risk-usability | 5 | mHealth XAI · HRV-readiness mismatch · cognitive overload self-tracking · contestable AI CDS · XAI healthcare CDS（全 medium）|
| cagan-risk-feasibility | 7 | HealthKit HRV SDNN docs (T2) · HealthKit overview · Garmin Health API · WHOOP API · WHOOP getting-started · Runna App Store cadence (T1) |
| cagan-risk-business | 5 | Runna pricing (T1) · Strava+Runna bundle (T1) · **Strava 收购 Runna (T1)** · RevenueCat (T3) · coaching marketplace take-rate (T4) |

### 全表 Tier (10 aspect 聚合，79 evidence)

| Tier | 代表 |
|---|---|
| Tier 1 (official) | Strava 收购 Runna press、Runna/TrainingPeaks/WHOOP 官方定价、Strava+Runna bundle、WHOOP API docs |
| Tier 2 (documentation) | HealthKit HRV SDNN/overview、Garmin Health API、WHOOP getting-started、Runna App Store version history、TrainingPeaks help |
| Tier 3 (community/news/blog) | Reddit r/AdvancedRunning、Strava community、Runna App Store reviews、RevenueCat 订阅基准、媒体评测 |
| Tier 4 (research / unknown) | mHealth/XAI/CDS 学术（source_type=unknown 但 peer-reviewed，confidence medium）、coaching marketplace 行业 blog |

- 来源类型 ≥5 ✅；≥1 Tier 1 ✅ → **A4 来源质量保持 2**。source_type 分布：official 39 / documentation 15 / forum 6 / news 4 / blog 4 / unknown 11；distinct domains 36。
- **provenance ✅**：79 evidence ids 全 cross-referenced；**0 dangling refs**（merge_rerun9.py 验证）。

详 [`rerun9-merged-10of10.result.json`](rerun9-merged-10of10.result.json) `evidence_index`。

---

## 段 8 · 未决问题 & 下一步 (TM-11 hard gate: falsification 100%) — 加重

来源：`open-questions-experiments:finding-1`~`finding-3`. 6/6 OQ 均有 question + why_open + how_to_resolve(who/when/pass/fail) + owner + target_date。

| OQ | Question | How to resolve (pass/fail) | Owner | Target |
|---|---|---|---|---|
| **OQ1** | 解释性 UI 让目标跑者更理解/信任还是造成困惑 | 5-user prototype usability（black-box vs explainable+debate）；pass ≥4/5 正确复述+找到 debate 入口+愿再用+0 critical confusion+≤3min；fail → 收起解释改 3 条因果摘要 | Design-A+PM-A | 2026-06-26 |
| **OQ2** | $X/月订阅 + trainer 抽成可规模化否（含 explainability WTP）| 4-week Concierge MVP + pricing smoke（$12.50/$19/$29，n=30）；pass conversion ≥20% + trainer payout 后 margin ≥60% + median review ≤8min；fail → trainer 降级 premium add-on | PM-A+Finance/CoachOps | 2026-08-14 |
| **OQ3** | cross-wearable SDK 限制能否形成可靠 MVP 数据层 | 3-week Eng dogfood API spike（HealthKit+Garmin+WHOOP，15 multi-device）；pass ≥90% workouts 15min 内可用 + ≥85% sleep/HRV by 08:00 + 缺字段 <10% + legal green；fail → v1 仅 Apple Health + Strava import | Eng-A+TPM-A | 2026-07-17 |
| **OQ4** | trainer review 应 risk-triggered 还是 broad coaching | 2-week dogfood + coach rubric（50 scenarios，classifier 分流）；pass ≥90% risky 路由 + false-positive ≤25% + κ≥0.7；fail → v1 只做 explanation + recommendation 不自动改计划 | PM-A+CoachOps+Eng | 2026-08-07 |
| **OQ5** | 目标跑者会否从 Garmin/Runna/Whoop 切换 | 5-day GV discovery sprint（12 interviews + clickable PR-FAQ + $5 deposit）；pass ≥7/12 排 top-1 + ≥5 付 deposit/连数据；fail → 缩 ICP 到 injury-comeback/coach-supported | PM-A+Design-A | 2026-06-21 |
| **OQ6** | 允许 debate/rebut 会否制造 overtraining bypass | 5-user adversarial prototype + 1-week red-team（用户试图说服系统在 low HRV/sleep 后加强度）；pass 0 unsafe high-risk 在无 trainer gate 下被接受 + ≥4/5 理解 gate + 0 hallucinated medical；fail → 移除 free-form debate 改 constrained reason chips | Design-A+Eng-Safety | 2026-07-31 |

**TM-11 hard gate 覆盖率 = 6/6 = 100% ✅**（每 OQ 关联段3 对应 cagan 风险类：usability→OQ1/OQ6、value+business→OQ2、feasibility→OQ3/OQ4、switching/ICP→OQ5）。

### 下一步排序 (TM-9 LNO；`open-questions-experiments:finding-2`)

- **10x**：OQ5（切换意愿）+ OQ1（解释 UI 真用）+ OQ3（cross-wearable 合法取数）— 任一失败削弱 thesis；Week 1-3 先跑 OQ5/OQ1。
- **Additive**：OQ2（pricing+WTP）+ OQ6（debate 安全）— Week 3-7 跑 OQ3/OQ6。
- **Overhead**：OQ4（trainer routing）— Week 5-11，不在前 90 天关键路径（与段3 "subscription-first 不开放 marketplace" 一致）。

### TM-8 pre-mortem (`open-questions-experiments:finding-3`)

12-18 月失败前三死因：① explainable/debatable UI 成认知负担，用户回 Garmin/Runna 直接处方（falsifier OQ1/OQ6 同时 pass）；② human-in-loop 变高 COGS/低一致性/慢响应运营泥潭（falsifier OQ2/OQ4 pass）；③ cross-wearable 数据不足，承诺 non-lock-in 实际像低保真聚合器（falsifier OQ3 pass + 证缺失字段不改关键建议）。time-limited：2026-Q3 后须重验。

### 段尾 prose

**最关键未决 = OQ5（切换意愿）+ OQ1（解释 UI 真用）+ OQ3（合规取数）**，任一失败 = thesis 不成立；靠 discovery sprint + prototype + Eng spike，**最迟 2026-06-21(OQ5)/06-26(OQ1)/07-17(OQ3) 内决**。**TM-11 100% (6/6) hard gate ✅；6 OQ 与段3 4 个 dedicated cagan 风险类一一对位。**

---

## 自验证记录 (Phase C self-verification per profile §3.2)

| Floor item | Status | 详情 |
|---|---|---|
| PR-FAQ 价值导向 + 无实现细节 + 客户引言具体场景 | ✅ pass | 段1 PR ≤300 字，客户引言 "半马倒计时三周... 所以能更稳到起跑线" |
| ODI ≥5 outcomes 完整 5 字段 | ✅ pass | 段2 O1-O4 underserved + O7 overserved 对照，Opp 公式 + Kano + estimated |
| **Cagan 4-risks 全覆盖 (hard)** | **✅ pass (R4-e)** | 段3 4 dedicated micro 全收敛（value/usability/feasibility/business）；4 类齐 + 各独立证据等级 + 应对；不跨段代偿 |
| OST ≥3 候选 / outcome (hard) | ✅ pass | 段4 3 outcomes × 3 候选 = 9，每候选附最危险假设 + 新建 vs 复用 |
| 非目标显式 + "为何不做" (hard) | ✅ pass | 段5 NG-1~NG-7 + 每项类型 + 为何 + 后续处理 |
| Trace 回 outcome (功能→段2) | ✅ pass | 段5 FR-1~FR-8 每条 trace 回 O1~O4 |
| 三套指标全 (主/次/护栏) (hard) | ✅ pass | 段6 P1-3 + S1-5 + G1-4 = 12 indicators，主指标全 5 字段 |
| TM-11 falsification 100% (hard) | ✅ pass | 段8 6/6 OQ 全 how_to_resolve + pass/fail + owner + date |
| 视觉证据 ≥5 (family B 适配) | ✅ pass | 7 张语义表（ODI / 4-risks / OST 9-cand / FR / NFR / metrics 3-set / OQ）= family B 天然视觉类型（rubric §6.4 R4-g 判据）|
| Subject basics ≥3 sources Tier 1/2 | ✅ pass | Strava press + Runna/TrainingPeaks/WHOOP 官方定价 + HealthKit/WHOOP/Garmin API docs |
| Confidence labels + TM-4 | ✅ pass | 每段段尾标 confidence；全 finding TM-4 epistemic_status；段3 各类独立 confidence（value/business 含 low）|
| Open questions 列 separately | ✅ pass | 段8 6 个产品 OQ |

**整体 floor**：5 hard gates 全 pass（段3 由 4 dedicated micro 收敛）；通用 floor 全 pass；视觉按 family B 适配（7 张语义表）pass。

---

> 本报告由 PM DeepResearch v2.3 (family B 8 段 PR-FAQ) + R4-e cagan 拆分，在 **#9 最终引擎（Lapis `9db7464` verbatim）** 上全量真实复跑端到端产出。
> **R4-b 回归结论**：v2.3 在 #9 引擎复现 24/24（段3 4-risks 由 4 dedicated cagan micro 输出，B1/B3 各达 2；C1 family B 语义表达标）；无任一维跌破 M7.5 锚点 → R1 引擎漂移闸门 PASS。详 [`rerun9-rubric-score-v23.md`](rerun9-rubric-score-v23.md)。
