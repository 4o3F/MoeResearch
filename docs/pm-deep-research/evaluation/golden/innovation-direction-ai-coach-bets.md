# M6 Golden Report · 海外运动/健身 AI Coach + 长流程自适应训练计划赛道：未来 12-36 月下注方向

> Profile: **innovation-direction** (v2.2) · 13-章 family A 变体 · do_not_drop = Ch 1/2/8/9/10/12/13
> Subject domain：海外运动/健身 app AI Coach + 长流程自适应训练计划赛道
> Time window：12-36 月（2026-2028）
> Target actor（incumbent baseline）：Strava+Runna（2025-04 收购后实体，作现状承载力观察基线；非单产品评测）
> decision_intent：`ai-upgrade` + `enter`
> Lapis 引擎：8 aspect deep + 4 retry sequential，本地 SSE 补丁 `MAX_SSE_EVENTS=65536` + `MAX_SSE_DATA_BYTES=256 KiB`（[4o3F/Lapis#8 ✅ 修复 / #9 已提](https://github.com/4o3F/Lapis/issues/9)）
> 证据：8 aspect / 38 findings / 62 evidence（详 [`m6-merged-8of8.result.json`](m6-merged-8of8.result.json) + [`m6-report-bundle.md`](m6-report-bundle.md)）
> 自评：详 [`m6-rubric-score.md`](m6-rubric-score.md)

---

## Ch 1 · 研究结论摘要（**加重** — TM-11 leading indicator 直接嵌入 thesis 行）

**核心论断**：AI Coach + 长流程自适应训练计划赛道 2026-2028 的真正机会**不是"再造一个聊天教练"**，而是 W1 **explainable biometric long-flow coach** —— 跨设备可执行、生物指标闭环、每次计划变更带原因解释、可承担伤病/恢复 guardrail 的耐力训练 AI 教练。当前 5 类 incumbent 在 buyer-validated 5 轴（自适应深度 / 生物闭环 / 解释性 / 跨设备执行 / 社群分发）中**无人同时达到 4+/5+ 的 W1 全占位**（段3 canvas 实测）。

赛道在 12-36 月会从"有没有 AI"上移到"AI 能否安全解释并执行长期计划"。下面三注按 P0/P1/P2 收敛（段8 整段 TM-11 hard gate 全过）：

### P0 — incumbent Strava+Runna：可信 adaptive training loop（非泛 AI Chat）

**Bet**：2026H2-2027H1 把 Runna 计划引擎 + Strava 社交/活动历史 + 用户授权的 Garmin/Apple/WHOOP readiness 信号，做成**可解释的恢复/伤病门控 + 自动重排 + 必要时 human coach escalation**。只覆盖跑步计划和赛前周期，不扩张全健康助理。

**TM-11 hard gate (leading indicator + 阈值)**：
- A：Beta 上线 12 周内，≥25% weekly active plan users 接受至少 1 次 recovery/injury-driven plan change，且 4-week adherence 相对 holdout +5pp，自报伤痛/客服升级率 ≤ holdout +1pp。任一不达标 → P0 假设失败。
- B：上线 6 月内 Strava+Runna bundle attach 或 renewal uplift <10%（vs 未触达 cohort）→ "付费理由"假设失败。

**TM-5 显性权衡**：选 P0 = 2026H2-2027H1 明确放弃"全健康 AI 助理、泛营养 agent、非跑步力量内容"主航道，把 KPI 锁到跑步计划 adherence + injury-safe adaptation + bundle retention。

**TM-3 4 风险评级**：value=中 · usability=中高 · feasibility=中高 · business_viability=中。

### P1 — 新创入场：human-in-the-loop injury/life-stress coach ops（非通用 wearable AI）

**Bet**：不从硬件/OS/通用 biometric chat 切入；从 comeback runners、易伤跑者、首马/半马、俱乐部/物理治疗合作切入，**AI 生成 plan diff + 教练/physio 24h 审核**。商业先 B2B2C / coach tooling / club cohorts。

**TM-11 hard gate**：
- A：90 天 concierge MVP 目标人群（近 6 月伤病/复出或首马）free-to-paid ≥20%，8-week retention ≥55%，human review median turnaround <24h。任一不达标 → wedge 不成立。
- B：6 月内每名 coach 同时管理 ≥40 活跃运动员，gross margin after coach payouts ≥50%。低于阈值 → hybrid ops 不可规模化。

**TM-5 显性权衡**：选 P1 = 2026-2027 明确放弃纯软件 90%+ 毛利和大众 viral 增长叙事，换取平台巨头短期不愿深做的高信任边界案例。

### P2 — 条件性平台对冲：agent/API readiness + plan compiler（非 2026 全量 super health agent）

**Bet**：把 10-15% R&D 做成可插拔 **coach memory + plan compiler + safety/eval harness**，能把训练意图编译为 Apple/Garmin/Strava/FIT structured workouts。OS/wearable agent SDK 打开时即接入，否则继续做 vertical coach moat。

**TM-11 hard gate**：
- A：到 Apple WWDC 2027 / Google I/O 2027，若主流平台仍未公开满足三条件的 health/workout agent SDK（实时 workout context + 可写 structured plan/action + 第三方可用授权 health signals）→ 继续押 vertical coach；若任一平台开放且 6 月内 ≥20% 目标用户每周用 OS agent coaching → P0/P1 必须转插件/分发策略。
- B：内部 2 季度若不能完成 ≥3 设备/平台 plan compile + sync 原型且 sync success ≥80% → 砍 P2。

**TM-5 显性权衡**：选 P2 = 2026 明确放弃"自建跨平台全健康 super-agent"大规模投入；只买期权，不把核心团队从 P0/P1 拉走。

**最大不确定性**（详 Ch 12）：(a) Apple Intelligence 是否在 18-36 月延伸到 long-flow training；(b) 跨平台 wearable readiness 数据权限稳定性；(c) Strava+Runna bundle 真实 attach / renewal uplift（公开仅有价格，无 retention 数据）。

---

## Ch 2 · 研究输入与边界

- **decision_intent**：`ai-upgrade` + `enter`（双 intent；incumbent 视角与新创视角都覆盖）
- **subject_domain**：海外运动/健身 app AI Coach + 长流程自适应训练计划赛道
- **time_window_months**：24（2026-2028）
- **target_actor**（incumbent baseline）：Strava+Runna（2025-04-17 收购后实体）
- **audience**：incumbent 策略团队 + 新创/PM 入场决策
- **明确排除**：
  - 不做单产品 teardown（那属 v2.1 product-capability）；现状能力仅"承载力评估" lens
  - 不做完整竞品图谱（那属 v2.0 competitive）；段3 canvas 仅作白地定位
  - 不做硬件设备评测（仅作数据源/竞争威胁分析）
  - 不延伸到泛健康 wellness / 力量训练 / 营养赛道
  - 时间窗外（>36 月）的远期判断只作背景

---

## Ch 3 · 目标产品定位与现状（简）

Strava+Runna 现状（incumbent baseline）：
- Strava：海外运动 social platform，2024-2025 ARR 增长来源主要靠订阅/付费跑步分析（M-Subscribers/Sponsored 等）。Bundle 定价 $11.99/mo。
- Runna：UK-based AI 个性化训练计划 App，主打"训练计划生成 + 自适应调整"。$19.99/mo。
- 收购后：2025-04-17 宣布 Strava 收购 Runna；2025-07-02 推出 **Strava + Runna $149.99/yr 捆绑**（vs 单订阅合算 ~60%）；初期独立运行，承诺投资 Runna 后续整合。
- 承载力评估见 Ch 6。
- Source：`trend-scan:ev-1-1` / `trend-scan:ev-1-3`（Tier 1 official press）。

---

## Ch 4 · 用户人群与 JTBD（段2 unmet outcomes 段落级嵌入）

赛道级 user jobs 拆 ≥3 desired outcomes（ODI underserved）：

| Desired outcome | Imp | Sat | Opp | Estimated | 段2 finding 来源 |
|---|---:|---:|---:|---|---|
| 在疲劳/伤痛/睡眠差时自动降低训练风险且保住目标 | **9** | 4 | **14** | TM-4 | unmet-outcomes / forum+official benchmark |
| 解释为什么改计划、让用户信任不是偷懒/乱改 | **8** | 5 | **11** | TM-4 | unmet-outcomes / review-derived |
| 伤病/复出/生活压力下得到可信、可追责的计划调整 | **9** | 4 | **14** | TM-4 | unmet-outcomes / forum-derived |
| 不被锁进单一表/OS/订阅生态 | 7 | 5 | 9 | TM-4 | unmet-outcomes / official platform trend |
| 将 Strava 社交激励转成付费训练结果 | 8 | 6 | 10 | TM-4 | unmet-outcomes / official bundle interpretation |

**关键 underserved**（>10）= 上面 3 个（≥14, 14, 11），全部围绕"**可解释 + 安全 + 可追责的自适应**"，不是"AI 更聪明"。

**"为何未满足"**（段2 finding 段落综合）：
- 大众用户要简单 / 严肃训练者要可信 → 单一产品同时满足"少打扰"与"可审计训练逻辑"很难；
- 真正自适应会**频繁改计划**，若不解释为什么今天降载，用户会判定 AI 反复无常；
- 数据碎片化 → 软件 app 难复制 Garmin/WHOOP 的传感器闭环（feasibility）；
- 硬件玩家有动力把最强指标留生态内（business viability blocker）。

---

## Ch 5 · 白地图段（**裁** — 段3 canvas + 段5 颠覆威胁列，≤1 页）

**段3 buyer-validated canvas**（5 轴 estimated TM-4）：

| Player | A1 自适应 | A2 生物闭环 | A3 解释/信任 | A4 跨设备 | A5 社群/分发 |
|---|---:|---:|---:|---:|---:|
| Strava+Runna | 4 | 2 | 3 | 3 | **5** |
| Garmin | **5** | **5** | 4 | **5** | 3 |
| TrainingPeaks | 4 | 3 | **5** | 4 | 3 |
| Apple Fitness+ | 2 | 2 | 3 | 3 | **5** |
| WHOOP | 3 | **5** | 3 | 2 | 3 |

**白地 W1**（无人占据）：A1≥4 + A2≥4 + A3≥4 + A4≥4 + A5≥4 的"可解释、可执行、跨设备、长周期、带社交问责的 AI endurance coach"。

**段5 颠覆威胁列**（time-limited，Christensen 判定）：

| 威胁 | sustaining/disruptive | 依据 | 时间窗 | TM-11 推翻条件 |
|---|---|---|---|---|
| Apple Intelligence + WatchOS Workout Buddy → long-flow training | **disruptive distribution** | 系统/OS 默认分发 + Private Cloud Compute 隐私 trust | 18-36 月 | 若 2027 仍仅实时鼓励/总结、未进入 plan adaptation |
| Garmin Coach + DSW + Forerunner 闭环 | **sustaining hardware moat** | Garmin Connect + 设备执行 + readiness 完整闭环 | 12-24 月 | 若 Garmin 不进入对话式解释 + 跨品牌中立性 |
| WHOOP Coach（OpenAI 合作）→ 结构化训练计划 | **sustaining-to-disruptive biometric moat** | recovery/sleep/strain 一手数据 + LLM | 12-24 月 | 若 WHOOP 不延伸 plan execution 与跨设备 |
| OpenAI Startup Fund / Thrive AI Health → consumer fitness | **disruptive adjacent-entry** | broad health agent 切入 fitness 是邻近赛道 | 18-36 月 | 若 2027 仍停留 wellness B2B / 未做消费者 sports |
| Strava+Runna bundle 价格挤压 | **sustaining aggregator threat**（对软件 app）| $149.99/yr bundle 把 AI plan 变低边际订阅权益 | 12-18 月 | 若 bundle 不带来 attach/renewal 改善 → 价格挤压不成立 |

---

## Ch 6 · 现状承载力评估（**裁** — 段4 future_capability_map 的 `our_carry_capacity` 列）

**baseline = Strava+Runna**。能力候选 × 承载力（5 类 × ready/partial/gap/unable + 依据）：

| 能力候选 | Can do（Tier 1/2 依据） | Strava+Runna 承载力 | 与段2 unmet 对位 |
|---|---|---|---|
| **AI**（LLM personalization + reasoning） | OpenAI/WHOOP Coach 已证明 GPT-4 + biometric + on-demand coaching（`trend-scan:ev-5-13`）；Apple Intelligence on-device + Private Cloud Compute（`trend-scan:ev-2-4`）| **partial**：Runna 有 plan engine，但**无 LLM 解释层**；Strava 无 AI coach 主张 | 直接对位 underserved "为什么改计划解释" |
| **硬件传感器** | Garmin DSW + Apple Watch + WHOOP 已闭环（`trend-scan:ev-2-4` / `recommended-bets:ev-3-7` / `recommended-bets:ev-4-10`）| **gap**：无自有硬件；依赖 user-authorized third-party（HealthKit/Garmin/WHOOP API）| 间接对位 "在疲劳/伤痛降险" |
| **数据 / readiness 信号** | HRV、sleep、strain、training load 各家 API 已公开（HealthKit / Garmin Connect IQ / WHOOP API）| **partial**：技术可行，但权限/限流/字段一致性是 oq-2 风险 | 直接对位 "降险" + "伤病调整" |
| **社区 / 分发** | Strava 自带 100M+ social graph 是 incumbent 最大资产 | **ready**：A5=5（canvas） | 部分对位 "社交问责"；oq-1 待验证社交→付费转化 |
| **内容 / 教练 marketplace** | TrainingPeaks 已证明 coach marketplace（`build-cost-feasibility` evidence）；Runna 早期含 human coach element | **partial**：Strava+Runna 历史 hybrid 双轨，整合后 marketplace 战略未公开 | 部分对位 "可追责调整"（hybrid wedge）|

**关键结论**：Strava+Runna 在 W1 5 轴中 A5（社群分发）= 5 ✅，A1（自适应深度）= 4 partial；**A2 / A3 / A4 是真正 gap**。P0 下注（Ch 1）= 用 A5 + A1 已有承载力，攻 A3（解释）+ A2/A4（biometric + 跨设备闭环）的 partial→ready 升级。

---

## Ch 7 · 视觉证据资产表（≥5；类型偏战略图，非 in-app）

| Artifact | Type | Source | Timestamp | Observed signal | Confidence | Related claim |
|---|---|---|---|---|---|---|
| Strava-Runna acquisition press | official text + brand graphic | https://press.strava.com/articles/strava-to-acquire-runna-a-leading-running-training-app | 2025-04-17 | acquisition fact + plan to integrate | high | Ch 1 P0 / Ch 3 baseline |
| Strava+Runna bundle launch | official text + price | https://press.strava.com/articles/strava-runna-launch-combined-subscription-bundle | 2025-07-02 | $149.99/yr bundle, ~60% savings | high | Ch 1 P0 / Ch 9 ODI |
| Apple Workout Buddy newsroom | official screenshot + workflow | https://www.apple.com/newsroom/2025/06/watchos-26-delivers-more-personalized-ways-to-stay-active-and-connected/ | 2025-06 | on-device + Private Cloud Compute Workout Buddy | high | Ch 5 disruption / Ch 8 AI 候选 |
| Garmin DSW + Training Readiness | official documentation | https://www.garmin.com/en-US/garmin-technology/running-science/physiological-measurements/daily-suggested-workouts-feature/ | docs current | adaptive plans + readiness | high | Ch 6 硬件闭环 baseline |
| WHOOP Coach by OpenAI | official text + integration page | https://www.whoop.com/us/en/thelocker/whoop-unveils-the-new-whoop-coach-powered-by-openai/ | 2023-2025 ongoing | GPT-4 + biometric coach | high | Ch 5 颠覆威胁 / Ch 8 AI 候选 |
| Strategy canvas matrix (5×5) | inline markdown table | 本报告 Ch 5 | 2026-05-30 (interpretation) | W1 白地 NE corner | medium (TM-4) | Ch 5 白地图 |
| Future capability map matrix | inline markdown table | 本报告 Ch 6 | 2026-05-30 (interpretation) | partial/gap by capability | medium (TM-4) | Ch 6 承载力 |

**类型差异说明**（vs v2.0/v2.1）：v2.0/v2.1 偏 in-app screenshot；v2.2 偏战略图（canvas / threat matrix / capability map）。Tier 1 official 来源 ≥5 张（press / Apple newsroom / Garmin docs / WHOOP / Strava bundle），无需 Layer-2 backfill；本期 C1 ≥5 达成（rubric §6.3 floor 通过）。

---

## Ch 8 · AI/新能力映射（**核心加重** — 段4 future_capability_map）

5 类候选能力 × （能干什么 / Tier 1/2 依据 / 承载力 / 对位段2 unmet / 适配 ai-upgrade+enter 评级）：

### 候选 C1 · AI（LLM + 推理 + 解释层）— **best-fit for W1**

- **Can do**：基于 user 历史训练 + 当日 readiness + 主观 RPE 输入，生成 plan diff + **自然语言解释** "今天为什么降量"；Apple Intelligence 已证明 on-device + Private Cloud Compute 能跑 Workout Buddy 类（`trend-scan:ev-2-4`）；OpenAI×WHOOP 已证明 GPT-4 + biometric on-demand coach（`trend-scan:ev-5-13`）。
- **承载力**：partial（Runna 有 plan engine 但无 LLM 解释层；段4 finding）
- **对位 unmet**：✅ "为什么改计划解释"（Imp 8/Sat 5/Opp 11）+ "在疲劳/伤痛降险"（Imp 9/Sat 4/Opp 14）
- **TM-11 推翻**：若 LLM-generated plan diff 解释在 8 周 holdout 中 adherence 提升 <5pp，AI 解释层假设错误

### 候选 C2 · 硬件 / 传感器（wearable AI）— **gap for non-hardware incumbents**

- **Can do**：HR/HRV/sleep/strain 实时采集 + on-device 预处理 → 给 cloud / 第三方 app 用（HealthKit / Garmin Connect IQ / WHOOP API）
- **承载力**：gap（Strava+Runna 无自有硬件 → 依赖 user-authorized API）
- **对位 unmet**：间接对位 "降险" + "伤病调整"
- **TM-11 推翻**：若 OS/wearable 平台到 2027 限制 third-party 实时 health signals 权限 → C2 不可行

### 候选 C3 · 内容 / 教练 marketplace（human + AI hybrid）— **W2 次级白地**

- **Can do**：TrainingPeaks 已证明 coach marketplace 可承载严肃训练计划 + 人工解释（`build-cost-feasibility` evidence baseline）；Runna 历史含 human coach 双轨
- **承载力**：partial（Strava+Runna 整合后 marketplace 战略未公开；P1 新创可填）
- **对位 unmet**：✅ "可信、可追责的计划调整"（Imp 9/Sat 4/Opp 14）
- **TM-11 推翻**：若 incumbent 在 2 训练周期内直接 ship human review，新创入场窗口压缩

### 候选 C4 · 社区 / 社交问责（network effect）— **ready for Strava+Runna**

- **Can do**：Strava 100M+ social graph 已是 incumbent 最大资产；可把训练 adherence 转社交问责（"今天为何降量" → 推送给好友 / coach 圈）
- **承载力**：ready（A5=5）
- **对位 unmet**：部分对位 "社交激励转付费结果"（Imp 8/Sat 6/Opp 10）
- **TM-11 推翻**：若用户行为证明社交问责对付费**无边际**提升（oq-1）→ C4 不是 wedge 而是 hygiene

### 候选 C5 · 数据 / training-outcome metric layer

- **Can do**：定义跨平台 outcome metric（adherence / injury-free rate / race time achieved / readiness trend）→ 用作所有 AI 决策的 ground truth + 商业 KPI
- **承载力**：partial（Strava 有 segment / activity 数据；Runna 有 plan completion；缺统一 outcome 层）
- **对位 unmet**：所有 underserved outcomes 都需要这层 metric 来衡量"做得好/不好"
- **TM-11 推翻**：若 outcome metric 与商业 KPI（renewal / ARPU）无相关性 → C5 不是杠杆

**Ch 8 综合**：W1 = C1（AI 解释）+ C4（社交问责）+ C5（outcome metric）的**复合**；C2/C3 是**补强**或**hedging**。

---

## Ch 9 · 产品机会矩阵（**加重** — 赛道级 ODI underserved 段2 + 段3 白地交叉）

ODI underserved >10（赛道级，跨 incumbent 视角；非 v2.1 域内）：

| Outcome | Opp | 对应白地 / 候选 | Kano（推断） |
|---|---:|---|---|
| 在疲劳/伤痛/睡眠差时自动降低训练风险且保住目标 | **14** | W1 + C1 + C5 | Performance / Must-be（核心训练 outcome）|
| 伤病/复出/生活压力下可信可追责的计划调整 | **14** | W1 + W2 + C1 + C3 | Attractive（差异化 wedge）|
| 解释为什么改计划，让用户信任 | **11** | W1 + C1 | Performance（信任地基）|
| 将社交激励转为付费训练结果 | 10 | C4 | Attractive（仅 Strava+Runna 有此资产）|
| 不被锁进单一表/OS/订阅生态 | 9 | C2 negotiation / P2 平台对冲 | Performance（卷不动 = 退路）|

**与段3 白地交叉**：W1 同时承载 Opp 14×2 + Opp 11，是**唯一覆盖 3 个 underserved outcomes 的白地**；W2 仅覆盖 Opp 14×1（hybrid coach）；其余 outcome 由单一候选承担。

**Ch 9 推论**：W1 是 P0 / P1 共同争夺的高 ROI 目标（不同切入：incumbent 用 A5+C4 / 新创用 C3+W2）；W2 是 P1 wedge 边界；C5（outcome metric）是 P0/P1 共享 infrastructure。

---

## Ch 10 · Roadmap 建议（**核心加重** — 段8 推荐下注 + TM-11 leading indicator 行内列）

| Bet | Tier | 启动时间 | 依赖 | TM-11 leading indicator + 阈值 | TM-5 显性权衡 | TM-3 4 风险 |
|---|---|---|---|---|---|---|
| **P0** Strava+Runna adaptive training loop | **P0** | 2026H2-2027H1 | Runna plan engine + Strava social + user-authorized HealthKit/Garmin/WHOOP API + human coach escalation channel | A：12 周 ≥25% WAU 接受 ≥1 次 recovery/injury plan change + 4-week adherence vs holdout +5pp + 自报伤痛 ≤ +1pp；B：6 月 bundle attach/renewal uplift ≥10% | 放弃全健康 helper + 营养 agent + 力量内容 | value=中 / usability=中高 / feasibility=中高 / business=中 |
| **P1** 新创 human-in-loop injury coach ops | **P1** | 2026Q3-2027Q2 | Coach/physio 招募（10-50 人） + AI plan diff engine + B2B2C 渠道（俱乐部 / 物理治疗诊所） | A：90 天 MVP free-to-paid ≥20% + 8-week retention ≥55% + human review <24h；B：6 月每 coach ≥40 athletes + margin after payouts ≥50% | 放弃 90%+ 软件毛利 + 大众 viral 增长叙事 | value=中 / usability=中 / feasibility=中低 / business=中高 |
| **P2** 平台 agent/API readiness + plan compiler（cap 10-15% R&D） | **P2** | 2026-2028 持续 option | Apple / Garmin / Strava / FIT structured workout schema + 跨平台 sync 原型 | A：WWDC 2027 / Google I/O 2027 主流平台**未**公开三条件 health agent SDK → 继续 vertical；若开放且 6 月 ≥20% 用户每周用 OS coaching → P0/P1 转插件；B：2 季度 ≥3 平台原型 + sync ≥80% | 放弃 super health agent 自建大规模投入；只买期权 | value=中低 / usability=低中 / feasibility=中高 / business=高（机会成本）|

**Roadmap 假设依赖**：
- P0 ↔ Strava + Runna 整合不延迟 + bundle data 可分析 retention（oq-1）
- P1 ↔ Coach supply 可规模化 + 法律边界可清晰（oq-3）
- P2 ↔ 平台 SDK 在 12-18 月内有动向（WWDC 2026 / 2027 watch）

---

## Ch 11 · 验证实验与指标（**加重** — 段8 每下注 TM-11 转可观察实验）

### P0 实验

- **Beta 设计**：8-12 周 randomized holdout（200-500 active Runna+Strava users）
- **输入**：wearable readiness（HRV/sleep/strain）+ post-run RPE + "not feeling 100%" 问答
- **输出**：plan diff + LLM 解释；高风险变更走 human approval；低风险自动应用
- **Primary metric**：adherence rate（plan completion %），accepted plan diff %，injury complaints rate，4-week renewal intent
- **触发响应**：
  - adherence vs holdout +5pp ✅ → 扩量到 5K
  - adherence +0-5pp + injury 不增 → 继续优化解释层 + 再 8 周
  - adherence ≤ +0 或 injury 显著 → **停 P0**，回 P1 wedge

### P1 实验

- **Concierge MVP**：90 天，10 名 coach/physio + 300-500 runner（comeback / first-marathon / injury history）
- **流程**：AI 生成 plan diff + 风险原因 → 人审（<24h SLA）→ 用户接收
- **对照**：静态 Runna/Garmin-like plan cohort
- **Primary metric**：free-to-paid ≥20%，8-week retention ≥55%，human review median <24h，每 coach 同时管理 ≥40 athletes
- **触发响应**：
  - 全部达标 → 招更多 coach + B2B2C 渠道扩张
  - free-to-paid 在 10-20% + retention 达标 → 收缩到更细 segment（如纯 comeback）
  - 任一未达标 → **P1 wedge 不成立**，转 B2B coach tooling-only

### P2 实验

- **2-quarter prototype**：把同一训练周编译到 Apple Health/Watch、Garmin/FIT、Strava/Runna calendar
- **Metric**：sync success rate（≥80%），plan diff error rate，coach approval time
- **季度 SDK trigger review**：监控 WWDC / Google I/O / Garmin SDK / WHOOP API 公告
- **触发响应**：
  - SDK 未开放 + 原型 ≥3 平台 ≥80% sync → 维持 cap 10-15% R&D
  - SDK 开放 + 6 月 ≥20% 用户用 OS coaching → P0/P1 立即转插件/分发
  - SDK 不开放 + 原型 <80% sync → **砍 P2**，把 R&D 回 P0

---

## Ch 12 · 风险、冲突与开放问题（**核心加重** — 段6 pre-mortem 三死因主体）

### 段6 Pre-mortem 三死因（TM-8 强制）

**死因 1 — Tiger**：**Apple/Garmin 等 wearable-native 平台把 AI Coach 变成"系统默认能力"，独立 AI Coach 的付费理由被压扁**。
- 机制：Apple Intelligence + WatchOS Workout Buddy 在 18-36 月延伸到 long-flow plan adaptation；Garmin DSW + Forerunner 闭环再加 LLM 解释层。
- 触发条件：WWDC 2027 demo plan adaptation；watchOS 27 ship 长周期 race plan；Garmin Connect IQ 加 coaching agent SDK。
- 早期可观察信号：OS 厂商 health agent SDK 公告 / 主流 wearable 单产品月活 >50M 使用 AI coach。
- 止损动作：触发 → P0/P1 立即转 P2 插件/分发策略；不烧 P0 余力。
- Christensen 类别：Tiger（Apple/Garmin 已知大敌）+ Garmin 兼具 Elephant（既有闭环难撼）。

**死因 2 — Elephant**：**Strava+Runna 把社区 + 训练计划 + 价格捆绑合并成默认"运动身份层"，新进入者即使 AI 更强，也变成可替换的训练插件**。
- 机制：Strava 100M+ social graph + $149.99/yr bundle + Runna plan brand → 用户**默认在 Strava ecosystem**，新创 CAC 被压制。
- 触发条件：Bundle attach >40% + renewal uplift >15% → Strava+Runna sustaining moat 形成。
- 早期可观察信号：Strava+Runna 投资者会议公布 bundle metric；第三方 app intelligence 数据。
- 止损动作：触发 → P1 新创**不打消费者市场**，转 B2B2C / coach tooling / club 渠道。
- Christensen 类别：Elephant（visible but 难撼动；sustaining aggregator moat）。

**死因 3 — Paper Tiger**：**被"通用全人健康 AI coach / ChatGPT for fitness"叙事带偏，团队追求可演示的聊天和 broad wellness，丢掉长流程训练结果验证**。
- 机制：组织优化 demo / engagement / 聊天次数，而非 plan completion / renewal；KPI 走偏 → 产品堆叠泛功能 → 失去 W1 vertical 焦点。
- 触发条件：内部 OKR 出现 "broad health coach" / "全场景 AI assistant" / "聊天日活" 类指标；产品上线泛健康 Q&A 而非 plan adaptation。
- 早期可观察信号：1-2 个 PR review 出现 "expand to nutrition / sleep coaching / general wellness" 主题；roadmap discussion focus shift。
- 止损动作：触发 → CTO/VP-Product 强制回归 W1 vertical 定义；KPI 重新挂到 adherence / renewal。
- Christensen 类别：Paper Tiger（叙事吓人但本质不是 endurance training 颠覆者）。

### 段5 颠覆威胁 + 防御性弱点（已 Ch 5 列）

防御性 3 维（Strava+Runna 视角）：
- 护城河（network effect + data moat）：medium（Strava graph 强 / Runna plan data 中 / readiness signal 弱）
- 锁定（subscription stickiness）：medium-high（bundle 锁 + social identity 锁，但 oq-1 待验证）
- 规模效应（community size）：high（A5=5 canvas）

**主要脆弱点**：A2/A3/A4（biometric / 解释 / 跨设备）是 partial→gap，正是 W1 + 死因 1 主战场。

### 开放问题（汇总段1/段2/段8 oq）

- **oq-1**：Strava+Runna bundle 真实 attach / renewal / ARPU uplift + Runna 单订阅 cannibalization？→ matched holdout cohort 分析 + 分层比较
- **oq-2**：跨平台 wearable readiness 数据权限稳定性（HealthKit / Garmin / WHOOP API 字段 / 延迟 / 商用限制）？→ data sufficiency test
- **oq-3**：human coach / physio 审核的合规边界 + 责任分配 + unit economics？→ 10-coach concierge pilot + SOP

### 冲突 / 反驳（counterargument 吸收）

| Bet | Strongest counter | 升级触发 |
|---|---|---|
| P0 | CA-P0-free-platform-coaching：Apple/Garmin/WHOOP 用免费/系统级 coaching 覆盖日级训练重排，付费差异化被压缩 | 2027H1 前平台方提供 injury/recovery 模式 + ≥20% 目标用户每周用 → P0 10x multiplier 错误 |
| P0 | CA-P0-safety-liability：injury-safe adaptation 在法律 / 客服 / 信任成本过高 | beta 自报伤痛 vs holdout +1pp 以上 → P0 降级或停 |
| P1 | CA-P1-hybrid-margin：human-in-loop 只能做低毛利服务 | 6 月每 coach <40 athletes 或 margin <50% → P1 错误 |
| P1 | CA-P1-incumbent-clones-human-loop：Runna/TrainingPeaks/Future 直接加教练审核 | incumbent 2 训练周期内 ship + 价格低 30% 以上 → P1 转 B2B tooling |
| P2 | CA-P2-platform-opens-fast：Apple/Garmin/Google 快开放 agent SDK | SDK 开放 → P2 从防守转分发/插件 |
| P2 | CA-P2-option-overhead：内部 plan compiler 不能被 P0/P1 复用 | 2 季度内 <3 平台原型 + sync <80% → 砍 P2 |

### 自验证记录（profile §3.2 floor）

| Floor item | 实测 | 通过？ |
|---|---|---|
| 趋势条数（≥3 market/tech/competition + Tier 1/2 + 时间窗） | 6 trends (T1-T6, 段1) | ✅ |
| Underserved outcome (≥3 ODI >10) | 3 outcomes Opp ≥11 (段2) | ✅ |
| 未来能力候选 (≥2，Tier 1/2 依据 + unmet 对位) | 5 candidates C1-C5 (段4/Ch 8) | ✅ |
| pre-mortem 死因 (≥3，机制+触发条件+止损) | 3 deaths (Tiger/Elephant/Paper Tiger) | ✅ |
| 推荐下注 (1-3 + TM-11 + 4 风险 + TM-5) | 3 bets P0/P1/P2 | ✅ |
| **可证伪性覆盖率** | 3/3 bets, **100%** (每注 ≥1 leading indicator + 阈值 + B 副 indicator) | ✅ |
| Subject-domain basics (≥3 sources, Tier 1/2) | 62 evidence, Tier 1/2 ≥30 | ✅ |
| 视觉证据 (Deep ≥5) | 7 artifacts (Ch 7) | ✅ |
| Confidence (high/medium/low + TM-4) | 全 finding 标 confidence + TM-4 | ✅ |
| Open questions | 3 oq + 段1/段2/段4 各自 oq | ✅ |

**TM-11 hard gate 通过率 = 100%**（3/3 推荐下注全过；profile §3.2 floor "可证伪性覆盖率 = 100% 非 ≥80%" 达成）。

---

## Ch 13 · 附录：来源与搜索记录 + "还需要查什么"

### Evidence summary（4-tier credibility labels）

- **High** (Tier 1-2: official / documentation / press / .gov-.edu)：≥30 evidence
  - 代表：Strava press（acquisition / bundle）/ Apple newsroom + support docs / Garmin official / WHOOP official + OpenAI x WHOOP / TrainingPeaks ATP / Yahoo Finance 市场报告
- **Medium** (Tier 3: news / blog)：~15 evidence
  - 代表：Bloomberg Apple Intelligence 路线 / PR Newswire Thrive AI Health
- **Low** (Tier 3 community: forum / app store reviews)：~12 evidence
  - 代表：r/Runna / r/AdvancedRunning / App Store reviews
- **Unknown** (Tier 4 undated)：<5；已标 → 仅 flagged，未进 Ch 1/10/11 主结论

详 [`m6-report-bundle.md`](m6-report-bundle.md) 段尾 Evidence sections（62 evidence × 8 aspects）。

### 每下注 "还需要查什么才能更确信"

| Bet | 待查 | 来源建议 |
|---|---|---|
| P0 | Strava+Runna bundle 真实 attach + renewal + ARPU uplift；Runna 单订阅 cannibalization | Strava Q4 2026 / Q1 2027 earnings call；data.ai / Sensor Tower app intelligence；matched cohort analysis |
| P0 | injury-safe adaptation 法律/客服 cost 基线 | Runna / Garmin Coach 客服数据 (NDA)；类似 hybrid coach 公司公开 lawsuit |
| P1 | coach supply 增长速率 + 教练对 AI tooling 接受度 | TrainingPeaks coach marketplace data；US/UK physio 行业报告；10-coach concierge pilot |
| P1 | 真实 free-to-paid + retention（comeback / first-marathon segment） | 自建 90-day concierge MVP；conjoint pricing study (300-500 人) |
| P2 | Apple WWDC 2027 / Google I/O 2027 SDK 信号 | 季度跟踪：WWDC / iOS 27 HealthKit sessions / watchOS release notes / Apple Health app / Fitness+ |
| P2 | 跨平台 plan compile 技术可行性 | HealthKit Workouts / Garmin Connect IQ Workout / Strava API Workouts / FIT structured workout schema 字段对比 |
| 通用 | OpenAI/Thrive AI Health 是否进入 consumer fitness | 跟踪 Thrive AI Health consumer launch + OpenAI health partnerships |

### 搜索查询记录

8 aspect × 5-6 search calls = 42 searches（引擎合并输出）。
