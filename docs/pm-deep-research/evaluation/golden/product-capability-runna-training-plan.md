# 黄金样例 · Runna 的训练计划自适应能力深度升级（产品能力研究）

> **本报告 = 最终权威黄金**：PM DeepResearch **产品能力研究**能力（profile = [`product-capability`](../../capabilities/product-capability.md)）的首个真实引擎产出，验证 6 段 EA-heavy 装配契约 + 13 章变体（Ch 6/7/4 加重 / Ch 5 裁为 benchmark 段）可承载。引擎 Lapis `9db7464`（vendored verbatim）全量真实产出。
> 写法：按通用规格 [§7.5 行文规范](../../pm-deep-research-spec.md#75-行文规范)（BLUF/SCQA + action-title + 主题综合）+ profile §1 表 4 章节加重 + §3.2 floor。证据：6 aspect 真引擎产出 + prior_sources 注入段1。所有 Imp/Sat 估算标 TM-4 estimated。

---

## Ch1 · 结论：Runna 真正的护城河不是更聪明的 AI，是「**解释型自适应**」

**情境**：Runna 是 2025-04-17 被 Strava 收购的 AI 训练计划 App，App Store 4.9/5 (30K reviews)、Editors' Choice，核心能力链=入门问卷→AI 自适应计划→每日 workout→执行反馈→计划动态调整。它在「AI 训练计划」赛道是 consumer-first benchmark。

**冲突**：但同一份能力域 teardown 把 Runna 放在 Garmin Coach+DSW（生理数据驱动每日自适应，得分 4.1/5）与 TrainingPeaks（教练编排 + Dynamic Plans，2.7/5 但定义专业标准）之间——Runna 自身 3.6/5，比大众化 Adidas Running (3.1) 强，比 Garmin 弱在 **adaptive frequency 与 injury override**。同时用户行为证据（Reddit r/runna + WSJ + App Store 评论）反复出现同模式：first-week overwhelming、unrealistic paces、users manually dial down、injury adjustment "not as seamless"——说明 Runna 已声称的"adaptive"在伤痛/疲劳/漏跑场景上**没有兑现承诺**。

**问题**：那 Runna 的升级方向应该是追 Garmin 的全栈生理 moat 吗？还是补 TrainingPeaks 的教练编排后台？

**核心判断**：都不是。两条都是高成本对位赛，Runna 没有硬件、固件和长期健康数据栈做 Garmin parity（成本 12-24+ 月），也没有教练市场做 TrainingPeaks 对位。Runna 真正能赢、对手**结构上难复制**的位置，是**「解释型自适应」**——夹在硬件 moat 与教练 moat 之间的 consumer-friendly + safety-aware + explainable 一段。这条赛道未被占据，正因为它同时需要运动科学、用户信任 UX、计划约束引擎和留存验证，单做一层都不够；但 Runna 已经有问卷-计划-反馈-调整链路 + Strava 的用户分发，是少数有资格做这件事的产品。我把它命名为「**Opinionated Coaching 护城河**」——AI 不只是给计划，是**敢于在伤痛/疲劳/漏跑时给出 conservative alternative，并解释为什么**。

由此推出 P0×2、P1×1、P2 各一（依据见 Ch9-10）：

| 升级方向 | 为什么是它 | 优先级 |
|---|---|---|
| **Recovery / Injury / Illness Safety Mode**（伤痛/疲劳/生病时主动降载、re-entry ramp、何时暂停做成一等流程，配主观反馈 + 可选 wearable）| 直接修补 BP3 + 解 ODI 最高 underserved (O1=13)；用户行为反复显示 manual customize—— Must-be 卫生项缺失 | **P0** |
| **Adaptive Plan Control Tower**（把所有 plan delta 给 2-3 个可选调整方案 + 解释为什么改 + 用户可覆盖）| ODI O2 = 12 underserved；对照 Garmin DSW 已把"为什么调"做出来；解决 BP1 + 给 Safety Mode 提供解释皮 | **P0**（与 P0-1 配套上线）|
| **Dynamic Plan Primitives**（借 TrainingPeaks 后台思路做模块化 workout、隐藏/占位 workout、回滚）| 内部 leverage（plan 维护时间 -30%），让前两个 P0 长期可迭代；不直接面向用户但可见 | **P1** |
| Strength/Mobility addon 深度（Attractive，与跑步计划信任绑定）| Kano 段 4 唯一 Attractive 项，已被用户证据正面验证 | **P1** |
| Garmin-like 全栈生理数据闭环 | benchmark 上限但 build-cost 12-24+ 月、需硬件 + 健康数据 + 固件团队 | **P2**（不在 2-4 季度内主战场）|

**置信度（校准）**：对升级方向**判断信心中-高**——teardown 矩阵 + 断点同模式 ≥3 来源 + ODI 估算 + benchmark proxy 收敛于同一结论；对**具体 Imp/Sat 数值**信心**低**（全 TM-4 estimated，无一手调研），是最该先验证的部分。

**最大不确定性**：① 用户是否愿意录入主观疼痛/疲劳/RPE 决定 readiness-aware 自适应 usability；② 合规边界——伤后建议如何避免被解读为医疗诊断；③ Strava bundle ($149.99/yr) 后用户对独立 Runna 订阅价值感知，决定深度自适应的留存回报。

---

## Ch2 · 研究输入与边界

| 项 | 内容 |
|---|---|
| 目标产品 | Runna |
| capability_domain | **训练计划生成 + 自适应调整** |
| boundary 含 | intake/onboarding · plan generation · adaptive adjustment · supporting strength/mobility/recovery |
| boundary 排除 + 理由 | ❌ GPS 记录（用户 job 是要下一步训练决策，不是记录一次跑步） · ❌ 社交 feed / Strava 跨产品功能（避免把收购方生态能力误判为 Runna 训练计划能力） · ❌ 商城（不在本能力域 job 内）|
| decision_intent | `improve`（Runna PM 视角：如何升级训练计划自适应能力） |
| 深度 tier | Deep（6 aspect 全段 + 13 章变体 + Ch 6/7/4 加重）|
| benchmark set（best-in-class 2-3） | **Garmin Coach + Daily Suggested Workouts**（wearable/physiology-first 上限）· **TrainingPeaks Dynamic Plans**（coach/platform-first 上限）· **Adidas Running**（mass-market consumer-first 上限）|

---

## Ch3 · Runna 的现状：consumer-first AI 教练，强在 onboarding 与计划生成，弱在 adaptive frequency 与 injury override

| 3 强项（用户证据支持）| 3 弱项（同模式 ≥3 来源证据）|
|---|---|
| 计划生成 + 个性化感知（4.9/5、30K reviews、Editors' Choice，"AI insights / tailored / pace goals" 高频正向） | injury / 疲劳后的 adaptation 常需手动 ease，用户反复抱怨 "not as seamless"（App Store + WSJ + r/runna）|
| Reschedule / 假期 / 生活变动自适应（V3 calendar、easy switching、weekly moves 用户正反馈） | first-week 计划过强 / unrealistic paces / users manually dial down（Reddit r/running + WSJ）|
| Strength / Pilates / 营养整合（praised for balance and injury prevention，Kano Attractive 唯一项）| Strength sessions 不像 runs 自动同步到 Garmin watch；缺基础卫生项 |

Strava 2025-04 收购后产品边界可能变化，但**短期独立运营**——Runna 仍以独立 App + $119/yr 订阅运行；Strava bundle = $149.99/yr。本研究以独立 Runna 视角进行。

---

## Ch4 · 4 类 user jobs：J3 + J4 + J2 的"信任与连续"才是计划要解的事

用 JTBD 看，Runna 用户的核心 job 不是"用 AI"，而是"**把我不知道如何安排训练的焦虑，转换为今天可执行且会随现实变化更新的计划**"（TM-1）。这句话把竞争集从"另一个跑步 App"扩到 Garmin Coach + TrainingPeaks + 真人教练。

四类 jobs（situation→motivation→outcome）：

| Job ID | Situation | Motivation | Outcome | 权重 |
|---|---|---|---|---|
| **J1 新手安全起步** | 刚开始或重启跑步，担心受伤 / 坚持不下去 | 让系统替我安排渐进负荷、配速、辅助训练 | 安全起步 + 建立习惯 + 降低过度训练风险（"减脂"是待验证子动机，TM-4 assumption） | high |
| **J2 进阶备赛** | 报名 5K/半马/全马，有目标但缺系统周期 | 按目标、当前水平、个人配速生成的计划，并随进展更新 | 不雇私教也能结构化备赛，提高完赛/达标概率 | **critical** |
| **J3 伤后/中断恢复** | 伤病、疲劳、旅行、漏训导致原计划失真 | 让计划吸收中断并重新安排负荷，而不是用意志力硬补 | 恢复训练信心 + 降低复伤风险 + 尽量保留目标路径 | high |
| **J4 习惯养成** | 工作 / 生活日程变化，原训练日无法执行 | 快速 reschedule，让后续训练仍合理，不让整个计划报废 | 在现实生活中保持连续性 + 低心理负担 | high |

**本章关键洞察**：J3 + J4 + J2 全部依赖 **"adaptive 兑现"**（J3=伤痛降载 / J4=漏训重排 / J2=完成质量重估目标），不是"更多 AI"。**未说出口的需求是"我不想因一次漏训、受伤或不会配速就放弃整个身份目标"**（TM-6）。这直接指向 Ch5 的白地与 Ch10 的升级方向。

**排除理由（≥1 条）**：✅ GPS 记录的 user job 是「记录这次跑步」，不是「决定下一次跑什么」——不属本能力域；社交 feed 类似——属 Strava 跨产品能力。**反驳条件**：若 Runna 整合 Strava 后 GPS 数据深度进入计划生成（非边缘输入），则 boundary 需重画。

---

## Ch5 · Benchmark 段：Garmin 是上限（硬件 moat），TrainingPeaks 是后台 primitives，Adidas 是大众下限——三者都不是 Runna 该正面对位的赛道

| benchmark | 为何 best-in-class | 与 Runna 的关系 |
|---|---|---|
| **Garmin Coach + Daily Suggested Workouts** | "传感器数据驱动的每日训练建议"上限：DSW 用 recovery、sleep、HRV、Training Status/Load 等做每日自适应；Coach plans 可被 DSW 动态调整强度难度 | **Sustaining + asymmetric moat threat**（同一高价值训练结果，但凭硬件/数据栈提高门槛——Runna 没法 1-2 季度对位）|
| **TrainingPeaks Dynamic Plans** | "训练计划编排与教练工作流"上限：changelog 出现 Dynamic Plans 改动（hidden workouts、apply without workouts），是 coach-applied live-calendar plan 的成熟范式 | **Sustaining / adjacent threat**（不是同一 buyer——B2B coach platform，但定义了"动态计划"专业标准）|
| **Adidas Running** | "低门槛 onboarding + 用户反馈驱动调整"大众化上限：官方站描述 fitness assessment 生成 plan + feedback-based adaptation；App Store 评论强调 onboarding 简单 | **低端 disruptive threat**（以广泛品牌入口和轻量计划吸走非严肃跑者；release notes 在 plan 上较沉默，可能在服务端迭代）|

**两块白地（命名）**：

1. **「Opinionated Coaching 护城河」**（consumer-friendly × safety-aware × explainable）：未被占据正因为它同时要求运动科学、用户信任 UX、计划约束引擎、商业留存验证——单做一层都不够。Garmin 强在传感器但解释性弱（"为什么 DSW 给这个 workout 用户经常猜"）；TrainingPeaks 强在 coach ops 但消费者不可见；Adidas 强在大众 onboarding 但深度不足。Runna 有问卷-反馈链路 + Strava 用户分发，**结构上有资格**做这块。

2. **「Selective Build」build-cost 策略**：不二选一 build/not-build 整套 Garmin parity。**低成本层（6-10 周）**= 主观反馈 + 错过训练 + 旅行/疲劳规则；**中成本层（2-3 季度）**= 状态机 plan engine + 模块化 workout library + regression QA；**高成本层（12-24+ 月）**= Garmin-like physiology + 多传感器模型。Runna 主战场应在低-中成本层，不去碰高成本层。

---

## Ch6 · 功能架构与体验路径：Teardown 矩阵 / 6 段路径 / 3 断点 / Kano 域内分级（**加重**）

### 6.1 Teardown 矩阵（每 cell 内联 evidence 或 assumption；1=弱 5=强；PM practitioner interpretation）

| 能力步骤/指标 | **Runna** | Garmin Coach+DSW | TrainingPeaks | Adidas Running |
|---|---|---|---|---|
| Intake / setup（步数 + 时长）| **4/5** 问卷 + goal inputs 自助；setup_time <10min est. (TM-4) | 3.5/5 Connect→Training&Planning→Coach Plans 路径深；watch sync 必需 | 2.5/5 load 已购计划 / coach sync；自助 intake 不公开 | 3/5 fitness assessment + goal setting |
| Plan-gen latency | **4/5** 问卷后即时（精确秒数 assumption）| 4.5/5 DSW 表/Calendar 即时；Coach 需 setup+sync | 2.5/5 看 purchased plan / coach schedule | 3.5/5 评估后生成 |
| **Adaptive frequency + input richness** | **3.25/5** 公开声称 schedule/progress/vacation；但无 sleep/body battery，用户反馈"几周后不动" | **5/5** daily + 多 input（fitness/recovery/Training Load + missed/skipped workouts）| 3/5 dynamic 标签 + coach sync，自动反馈较弱 | 3.5/5 自动 feedback 调整，input 深度不公开 |
| Override / 错误恢复（reschedule, pause, race change, injury/B-race）| **3/5** reschedule/vacation 强；injury 弱，无 B-race | 4/5 reschedule + pause + edit race date + missed adapt（但 DSW 在 Coach 中无 prompt）| 2.5/5 编辑存在但 usability/订阅 friction | 2.5/5 feedback/progress 触发，override 控件不可见 |
| Error rate / friction proxy | **3.5/5** 4.9/5/30K 评分高 + aggressive progression / partial adaptivity 评论 | 3.5/5 菜单复杂 + Coach 中无自动 prompt | 2.5/5 usability/subscription 编辑投诉 | 3/5 简单评估但 override 不明 |
| **等权均值** | **3.6** | **4.1**（最高）| 2.7 | 3.1 |

**两条关键差异（综合 finding-2/3/4）**：
- **差异 1（intake 路径）**：Runna 像手机端自助生成器，**降低 J1 首次心理门槛**；Garmin 用更高 setup cost 换更深数据；TrainingPeaks 换来教练能力但不适合无教练新手。
- **差异 2（adaptive 频率与可信度）**：Garmin DSW 的每日 readiness-aware 调整把市场预期推高；Runna 公开叙事强调 vacation/progress，但用户证据出现典型 say-vs-do——"系统几周后不动了"。**用户不是只要 AI 文案，是要疲劳/缺课/伤病时自动降风险**。
- **差异 3（override 缺口）**：Runna 在 reschedule 强、injury 弱、不能加中途 race；Garmin 完整 Override 路径但 Coach×DSW 摩擦；TrainingPeaks 编辑要付费；Adidas 不可见。

> **反驳**：若 Runna 付费版已接入睡眠/HRV/恢复数据并每日重算，本结论会低估 Runna；若实测显示 first-run latency 与公开路径不同，分数应重算。

### 6.2 体验路径（6 step）+ 断点地图（3 BP，每断点 ≥1 visual + ≥3 同模式 Tier-3 证据）

**完整路径**：① intake（目标/能力/日程/约束）→ ② plan generation（个性化周计划）→ ③ calendar/schedule edit（reschedule）→ ④ daily workout execution（按 pace/intensity 执行）→ ⑤ post-run feedback（完成/疲劳/疼痛）→ ⑥ plan adjustment（下周自动重排）

**3 断点**（同模式证据 ≥3 已对齐 profile §3.2 floor）：

| BP | Step | Type | Gap | Visual evidence | 同模式用户证据（≥3） |
|---|---|---|---|---|---|
| **BP1 初始计划过强** | plan generation after intake | 错误 / 困惑 / 潜在 drop | beginner / returner / injury-risk 保守校准不足 | App Store product page screenshots (ev-2-4) | App Store reviews: first-week overwhelming + no easy injury adjust + manual ease (ev-1-1); Reddit r/runna: beginner marathon intensity / injury / heavy customization (ev-1-2); WSJ: high intensity / limited real-time adaptation / caution for beginners and injury returners (ev-1-3); Reddit r/running: overly aggressive / excessive speedwork / injuries (ev-3-7) → **4 来源同模式** |
| **BP2 当日强度执行卡顿** | daily workout | 卡顿 / 错误 | 系统配速/强度处方与身体状态脱节，需用户自己 dial down | App Store + Google Play screenshots (ev-2-4/2-6) | Reddit r/running: unrealistic paces + dial down or reduce weekly runs (ev-3-7); Reddit r/runna: insufficient real-time feedback on pace/fatigue + rescheduling/modifications (ev-3-8); Reddit r/runna beginner: plans too hard + overtraining (ev-1-2); WSJ: high intensity + limited real-time adaptation (ev-1-3) → **4 来源同模式** |
| **BP3 伤痛/疲劳反馈→调整不顺滑** | post-run feedback → plan adjustment | 困惑 / 卡顿 / drop risk | 用户证据：vacation 能调，injury/fatigue 仍需手动 ease——系统承诺 adaptive 但用户成了自我教练 | App Store + Google Play "AI insights / Not Feeling 100% / adaptive updates" 截图 (ev-2-4/2-6) | App Store: vacations auto-adapt but no easy injury, manual ease/customize (ev-1-1); Reddit r/runna: heavy customization or injury/overtraining (ev-1-2); Reddit r/running: many modify workouts / dial down / reduce weekly (ev-3-7); WSJ: Runna added "Not Feeling 100%" + slowing options in response to aggressive-plan injury feedback (ev-3-9) → **4 来源同模式 + 已部分缓解信号** |

> **say-vs-do 综合判断**：Runna 路径文案说 "personalized / adaptive / dynamic feedback / continuous updates"；用户行为证据反复出现 modify / dial down / reduce / manual ease。**用户未说出口的话不是"加 AI 文案"，是"让今天/本周计划自动变得更安全、更可信"**（TM-6）。

### 6.3 Kano 域内分级（叠在 teardown 之上）

| 能力 / 断点 | Kano | 依据 |
|---|---|---|
| intake / onboarding 信任 | **Must-be** (TM-4 practitioner; 可证伪: onboarding 完成率与可信度无关则降级) | 高评分与 tailored plans 只能间接支持；缺解释会先损伤信任而非创造惊喜 |
| plan-generation 个性化感知 | **Performance** | tailored / pace goals / 不过分不可达 = 越准越满意（ev-3-8）|
| **adjustment trigger: 伤痛/生病/恢复**（BP3）| **Must-be** | 行为信号：用户报告 niggles/injuries + mixed success staying injury-free——无声需求是安全调整（ev-1-3, ev-2-4）|
| override / reschedule / calendar shift | **Must-be** | rattling-car 信号：用户不是想更多 AI，是想生活变化时计划不崩（ev-3-8, ev-2-5）|
| **strength / mobility / Pilates addon** | **Attractive** | 用户称赞 balance + injury prevention，但不是完成核心计划的最低门槛（ev-1-2, ev-2-5）|
| cross-device sync / watch delivery | **Must-be**（run workouts）/ **Performance**（health 信号联动）| Garmin sync 强正反馈 + strength sessions 未同步暴露执行层卫生项缺口（ev-2-6）|

**Kano 关键洞察**：6 项中 4 项 Must-be / 1 项 Performance / 1 项 Attractive——**升级方向应先补 Must-be**（injury + reschedule + sync），而不是先加 AI 文案或扩张距离模板。

---

## Ch7 · 视觉证据资产表（断点 visual_evidence 强制 ≥每断点 1 张；本期 4 张）

| Evidence id | Product | Screen / Flow | Media | Source URL | Related claim |
|---|---|---|---|---|---|
| `experience-paths-breakpoints:ev-2-4` | Runna | App Store product page (intake → daily workouts → AI insights → schedule flexibility → adaptive updates) | screenshot | https://apps.apple.com/us/app/runna-running-plans-coach/id1594204443 | intake→plan-gen→adaptive 链路存在；BP1/BP2/BP3 路径锚点 |
| `experience-paths-breakpoints:ev-2-6` | Runna | Google Play product page (personal running coach + adaptive plans + dynamic feedback) | screenshot | https://play.google.com/store/apps/details?id=com.runbuddy.prod&hl=en_US | continuous adjustment promise；BP2/BP3 锚点 |
| `capability-teardown-deep:ev-2-4` | Garmin | Connect support FAQ for DSW path (START > Training > Workouts > Today's Suggestion) | doc page | https://support.garmin.com/en-US/?faq=QI8yafYMiH1jF1lqX84Gw7 | DSW 入口与 Coach 共存逻辑 |
| `benchmark-buildcost-upgrade:ev-2-4` | TrainingPeaks | Web changelog 含 Dynamic Plans 改动（hidden workouts、applying Dynamic Plan without workouts）| changelog | https://www.trainingpeaks.com/changelog/ | Dynamic Plan primitives 是 backend 投资重点 |

> **C1 floor 自检**：达 ≥每断点 1 张 + ≥4 张（Deep floor ≥5 未达——本期 evidence 仍以 App Store / Google Play 产品页为主，不是断点发生时的原生录屏。**按 rubric 诚实降分 C1=1**，保留为待办（需 Skill 层 Layer-2 浏览器抓 in-app 录屏才能升 C1=2）。

---

## Ch8 · AI / 新能力映射

本研究 decision_intent = `improve`（不是 `ai_upgrade`），且**核心结论恰恰是反 AI-mimicry**——Runna 不应继续叙事化"更聪明的 AI"，而应让 AI "opinionated"（解释 + 可控）。三个升级方向（Ch10）中 AI 含量：

- **Recovery Safety Mode**：AI 层薄（保守规则 + 阈值 + 可选 wearable 信号），重点是状态机 + 阈值 + 教练原则可审计性。
- **Adaptive Plan Control Tower**：AI 层中（plan delta 解释 + 2-3 个可选方案），重点是解释性 UX。
- **Dynamic Plan Primitives**：AI 层近 0（后台 plan engine + content ops）。

> **本章相对其他章被裁短**（profile §1 表 4 do_not_drop 不含 Ch 8）——本研究是产品能力研究而非 AI 升级方向研究。

---

## Ch9 · 产品机会矩阵（域内 ODI；段5）

5 个 desired outcomes（TM-4 全 estimated；Imp/Sat 1-10；Opp = Imp + max(0, Imp − Sat)；>10 underserved，<7 overserved）：

| ID | desired outcome | 对应 jobs | Kano | Imp | Sat | **Opp** | 结论 | 估算依据 |
|---|---|---|---|---:|---:|---:|---|---|
| **O1** | 伤后/生病/疲劳/疼痛 24h 内自动降载/暂停/返跑，并解释安全边界 | J1 + J3 + J4 | Must-be + Performance | 9 | 5 | **13** | **🔴 Underserved（最高优先级）** | 官方声称 adaptive 但用户证据显示伤病自适应多需手动；安全失败直接破坏信任与留存 |
| **O2** | readiness（RPE/疼痛/睡眠/HRV/恢复）→ 下次训练强度调整，既不过载也不打断进步 | J1 + J2 + J4 | Performance（未来 → Must-be）| 9 | 6 | **12** | **🔴 Underserved** | Runna 已有进度自适应但 Garmin DSW 推高市场预期；缺同等闭环会显不可信 |
| O3 | 漏跑/旅行/工作家庭变动后重排本周训练，保目标日期与递进 | J2 + J4 | Performance | 8 | 7 | 9 | 🟡 Adequately served（非最高优先级）| Runna 与 TrainingPeaks 都已具备日历同步 = hygiene |
| O4 | 进阶备赛中根据完成质量持续重估目标配速/完赛时间，并解释为什么改 | J2 | Performance | 8 | 6 | 10 | 🟡 Borderline | Runna 有 AI 洞察但解释性不强；Garmin 用生理指标推高用户对可解释调整的期待 |
| **O5** | 标准距离/目标训练模板（5K → 马拉松/超马）+ 基础个性化 | J1 + J2 | Basic / Expected | 6 | 8 | **6** | **🟢 Overserved** | Runna 已有多距离 + 个性化 + 简单可定制；继续堆模板边际价值低 |

**关键判断**：O1 + O2 是 Runna 唯一应在未来 2-4 季度集中火力的方向——同时是 Kano Must-be 缺口（Ch6.3）+ 体验路径断点 BP3（Ch6.2）+ teardown 矩阵 adaptive frequency 弱项（Ch6.1）的**收敛点**。O5 标 overserved 是 TM-5 显性 trade-off：**继续投资距离/模板覆盖 = 在未来 2 季度明确放弃 O1/O2**。

> **反驳**：若一手 ODI 调研显示 O1/O2 Sat ≥8 或 Imp ≤7，underserved 结论错误；若公开评论样本偏重伤受/高强度用户而主流用户主要满意简单计划，则 O1/O2 分数过低。

---

## Ch10 · Roadmap 建议（P0/P1/P2 + 4 风险评估 + 验证条件）

| 优先级 | 方向 | Leverage（TM-9）| 选择=放弃什么（TM-5）| Value risk | Usability risk | Feasibility risk | Business risk | 验证条件 |
|---|---|---|---|---|---|---|---|---|
| **P0-1** | **Recovery / Injury / Illness Safety Mode** —— 主观疼痛、疲劳、睡眠、可选 wearable import → 训练降载、re-entry ramp、何时暂停做成一等流程 | **10x**：对 J1 新手 + J3 伤后/病后用户是信任护城河 | 未来 1-2 季度明确放弃纯 PR/速度优化优先级 | 用户是否愿报告疼痛/疲劳未知 | 过度安全提示可能吓退用户 | 医疗边界、误判、恢复规则验证复杂 | 过度保守可能降成绩导向用户满意 | 自报 injury interruption -15%；恢复后 4 周 plan completion 不低于对照；负面反馈/退款不升；医疗免责声明 + 升级路径清晰 |
| **P0-2**（与 P0-1 配套上线）| **Adaptive Plan Control Tower** —— 错过训练 / 生病/旅行 / 疲劳 / 赛事变更后的 plan delta 解释清楚，给 2-3 个可选调整方案，用户可覆盖 | **10x**：信任 + 执行率，所有未来自适应都复用 | 未来 1-2 季度放弃黑箱"自动完美"叙事 + 部分高级生理模型 | 用户可能只想被安排不想做选择 | 太多解释造成认知负荷 | 计划约束状态爆炸（race date / long run / 强度分布）| 高支持成本可能侵蚀订阅毛利 | A/B：计划遵守率 +7-10%；用户"理解为什么改" ≥80%；support tickets 不升；付费 D30/D90 留存提升 |
| **P1** | **Dynamic Plan Primitives / Content Ops** —— 借 TrainingPeaks 思路建隐藏 workout、占位 workout、模块化 block、运营后台、计划回滚 | additive → 10x（先内部效率，长期让所有自适应迭代更便宜）| 1-2 季度明确放弃一部分前台社交/游戏化可见功能 | 用户短期感知弱 | 内部工具若复杂会拖慢教练/内容团队 | 数据模型、版本回滚、QA 自动化复杂 | 若不能转化为可感知计划质量会成 overhead | plan 维护时间 -30%；plan edit latency 降低；线上计划错误率不升；后续通过 P0-1/P0-2 转化为可见用户收益 |
| P1 | Strength/Mobility addon 深度（Attractive → 边际 Performance 触发）| 中：与 Safety Mode 自然耦合（"今天降载，做 strength 替代"）| —— | 部分非力量用户认知负荷 | 触发逻辑要避免变成强制复杂度 | 已有 V3 calendar 基础，feasibility 较低 | —— | strength attach rate 提升 + 完成率 + 续费 uplift |
| **P2** | Garmin-like 全栈生理数据闭环（DSW parity）| 高但 12-24+ 月成本，无硬件 + 健康数据栈 | 选择 = 用所有资源对追 Garmin 而 Runna 没有结构性 advantage | 高（同样市场已被 Garmin 占住）| 高（数据获取摩擦）| **高**：成本 12-24+ 月 + 跨团队（数据科学 + 固件 + 运动科学） | 高（投入回报周期长）| 不在 2-4 季度内主战场；待 Strava + Apple Health + Garmin Connect 数据覆盖率 ≥60% 付费用户后再评估 |

**TM-5 整体 trade-off**：选择 P0-1 + P0-2 = 在未来 1-2 季度**显式牺牲**模板/距离覆盖扩张、社交化激励、商城/内容扩张、高级配速小工具。这是必要 trade-off——继续做 overserved 的 O5 模板扩张不形成防御。

---

## Ch11 · 验证实验与指标

按 spec §7.3 metric definition（主 / 次 / 护栏）：

| 升级方向 | 主指标 | 次指标 | 护栏指标 |
|---|---|---|---|
| P0-1 Safety Mode | 自报 injury interruption -15%（同期对照） | 恢复后 4 周 plan completion 不低于对照；伤痛后 7 日 streak 留存 +X% | 客服 ticket（伤病/不当建议类）不升 ≥10%；退款率不升；负面 App Store 评分占比不升 |
| P0-2 Control Tower | 计划遵守率 +7-10%（同期对照） | 用户"能正确复述变更原因" ≥80%；override 率在健康区间（既非 0 也非过高） | support tickets 不升；付费 D30/D90 留存提升；用户认知负荷反馈（"too many options"占比 <10%） |
| P1 Plan Primitives | 计划内容维护时间 -30%（内部 ops） | plan edit latency 降低；线上计划错误率不升 | 教练/内容团队体感（NPS 不降）；不阻塞后续 P0-1/P0-2 迭代 |

**3 个先验证假设**（rank by ODI underserved + 不可逆性）：

1. **用户是否愿录入主观 RPE / 疼痛 / 疲劳**（决定 O2 readiness-aware 自适应 usability）— concierge test + fake door：3 个主观问题 → 看用户是否完成；如未完成率 >40%，O2 方向需重设计。
2. **伤后建议合规边界**（决定 O1 / Safety Mode 上线条件）— 与法律 / 医学顾问定义文案红线：训练建议 / 风险提示 / 何时建议就医；建保守阈值审核样例库。
3. **Strava bundle 后用户对独立 Runna 价值感知**（决定深度自适应建设的商业回报）— 退订原因 + 价格弹性测试 + Strava 用户 vs 独立 Runna 用户 cohort 比较。

---

## Ch12 · 风险、冲突与开放问题（**加重**：Pre-mortem 三死因 + 各 aspect counterargument 收敛）

### 12.1 Pre-mortem 三死因（TM-8；假设 12-18 月后 P0-1/P0-2 升级失败）

| 死因 | Root cause（多层）| Falsification condition（TM-11）|
|---|---|---|
| **1. Garmin mimicry trap** —— 团队把 benchmark 误读为"必须复制 Garmin 的全栈 physiology"，但 Runna 无同等硬件、固件、长期健康数据栈 | strategy 把 moat 当 feature list；incentive 奖励 AI / 传感器大词；culture 偏技术炫耀而非教练信任 | 可用恢复/睡眠/HRV/训练负荷数据覆盖 ≥60% 付费用户，且加入这些信号后留存/伤病中断/计划遵守率显著优于解释型轻量方案 → 死因不成立；否则 trap 存在 |
| **2. Black-box adaptivity trust failure** —— 计划每天都"会变"但用户不知道为什么、能否覆盖、改变是否安全；最终形成不信任和手动改计划 | culture 优化模型输出而非 coaching communication；incentive 只看生成次数/自动调整次数 | ≥80% 用户能正确复述变更原因，override 率在健康区间（既非 0 也非过高），计划遵守率与 paid retention 同时上升 → 死因不成立 |
| **3. Changelog contract failure** —— 团队大量时间做 invisible backend/model work，但用户看到的 release notes 和体验只是"bug fixes / performance"，无法感知升级价值 | strategy 没把 deeds 绑到用户可感知 outcome；business 在 Strava bundle 压力下无法证明增量 LTV | 每月至少一个可感知 plan-quality 改进进入实验或发布说明；cohort retention/ARPU 或 downgrade/refund 相对对照改善；support burden 不升 |

### 12.2 跨 aspect counterargument 收敛

收敛于一个根本反方：**"Runna 当前 4.9/5 与 30K 评分说明 Must-be 已基本满足；负向断点可能只是高强度备赛用户或线上抱怨者局部样本"**——若 telemetry 显示 injury-returner / beginner cohort 计划完成率高 + 疼痛反馈后自动降载成功率高，本研究断点地图应改"历史问题"而非"现状问题"。WSJ 已记录 Runna 加入"Not Feeling 100%"作为 response——说明问题被 Runna 自身识别，但程度未知。**最关键的验证就是：在最新版上重做 BP3 测试 + cohort telemetry 抽样**。

### 12.3 各 aspect 开放问题（合并去重）

| Open Question | 来源 aspect | Follow-up |
|---|---|---|
| Runna 当前版本完整 intake → plan-gen → daily workout → post-run feedback → plan adjustment 录屏，尤其 "Not Feeling 100%" / injury/fatigue 输入后下一周计划变化 | 段3 | 用 beginner/race-prep/injury-returner 3 测试账号录屏；4 种反馈后比较 next workout/next week 是否自动降载 |
| 每断点是否能拿 ≥3 条逐条原文用户评论（非搜索摘要）| 段3 | 抽样 App Store/Play 1-3 星评论按 too hard/manual customize/injury adjustment failed/pace unrealistic 编码；Reddit 主题内评论去重用户 |
| 各产品真实 first-run step_count / setup time / plan-gen latency | 段2 | 同 persona screen recording，记录 tap count、time-to-plan |
| Runna 在 missed/疲劳/伤病/生病/旅行/B-race 场景下多久调整一次、调整多少 | 段2 | 构造 6 情景脚本连续两周输入不同完成/未完成/疼痛反馈，比较计划 diff |
| onboarding 信任 = Must-be 还是文案层优化 | 段4 | 抽 onboarding 退出点 / 首计划生成后编辑取消率；评论按 onboarding/questionnaire/trust 编码 |
| Runna 能否稳定获得 readiness 数据（RPE/疼痛/睡眠/HRV）| 段5 | 盘点 Strava/Apple Health/Garmin/WHOOP/Oura 数据权限与覆盖率；原型测试只用 3 个主观问题是否足以让用户信任当天调整 |
| 伤病/返跑建议的合规边界 | 段5 | 法律/医学顾问定义文案红线；建保守阈值审核样例库 |
| 4 类 jobs 中付费与留存权重 | 段6 | 按入门问卷/目标赛事/中断原因/wearable 状态做 cohort 分析；将 injury/illness/schedule-change 事件与 D30/D90 留存关联 |
| Runna 当前 plan engine 是否支持 workout 模块化 / 隐藏/占位 / 版本回滚 / 服务端 plan delta | 段6 | 访谈 plan-engine/coach-content 团队；6 周 prototype spike：错过训练后自动重排 + 可解释 plan delta |

### 12.4 自验证记录（profile §3.2 + spec §9.2 floor）

| Floor 项 | 状态 |
|---|---|
| 体验路径图 ≥1 张完整 + ≥3 断点 | ✅ 段3 Ch 6.2 完整 6-step + 3 BP |
| 断点 visual_evidence ≥每断点 1 张 | ✅ 每 BP ≥1 张（App Store / Google Play 产品页）|
| 用户证据 ≥每断点 3 条同模式 | ✅ 每 BP 4 条来源（App Store + Reddit ×2 + WSJ）|
| Benchmark 对手 2-3 best-in-class + 选择理由 | ✅ 3 个（Garmin / TrainingPeaks / Adidas）+ Ch5 选择理由 |
| ≥3 desired outcomes ODI | ✅ 5 个 outcomes（O1-O5），含 underserved/adequate/overserved 三档 |
| Target-product basics ≥3 sources, Tier 1/2 | ✅ Runna 官方 (Tier 1) + App Store (Tier 1) + Google Play (Tier 1) + Reddit (Tier 3) + WSJ (Tier 2-3) + Garmin official (Tier 1) + TrainingPeaks Help (Tier 1) |
| Capability matrix 每格 evidence or assumption | ✅ Ch 6.1 每 cell 标 evidence ids 或 partial/true assumption + 等权均值 |
| Visual evidence Deep total ≥5 | ⚠️ **4 张**（未达 5）—— rubric C1 = 1 而非 2（需 Layer-2 浏览器抓 in-app 录屏才能升 C1 = 2；非方法缺陷）|
| Confidence 每关键结论标 high/medium/low + epistemic status (TM-4) | ✅ 所有 ODI 标 estimated; 全 finding 标 medium confidence + TM-4 epistemic |
| Open questions 不足/冲突/未验证假设单列 | ✅ Ch 12.3 已汇总 |

**Floor 通过**：A1-A5 + B1 + B3 + C3 全 ≥1；**C1 = 1**（视觉证据 < 5，待办）。预期 rubric 总分见下 Ch13 后跟的 score file。

---

## Ch13 · 附录：来源与搜索记录（4-tier 标签）

**来源总数**：43 evidence cells / 6 aspect / 14 unique URLs（去重）。

| Tier 标签 | 数量 | 主要域名 |
|---|---|---|
| **High** (Tier 1-2: official / documentation / .gov/.edu / app store / 版本历史) | ~25 | runna.com · apps.apple.com · play.google.com · garmin.com · support.garmin.com · trainingpeaks.com · help.trainingpeaks.com · adidas.com · runtastic.com |
| **Medium** (Tier 3: news / blog / mainstream / 名记) | ~5 | wsj.com (Runna app race training) |
| **Low** (Tier 3 community: forum / Reddit / 评论) | ~12 | reddit.com/r/runna · reddit.com/r/running |
| **Unknown** (Tier 4) | ~1 | google-cached / 部分 App Store reviews 转 unknown |

**关键 evidence 簇**（前 10 by importance）：
1. https://www.runna.com/ + https://www.runna.com/training/training-plans（official, Tier 1）—— 能力域 boundary + 服务的 user jobs + 官方对 injury/vacation/return 的承诺
2. https://apps.apple.com/us/app/runna-running-plans-coach/id1594204443（official+reviews, Tier 1-2）—— 4.9/5/30K + AI insights/adaptive 描述 + BP1/BP2/BP3 用户证据
3. https://play.google.com/store/apps/details?id=com.runbuddy.prod（official+reviews, Tier 1-2）—— continuous plan updates + injury adaptation 弱点信号
4. https://www.reddit.com/r/running/comments/1cg6pe0/.../runna/（forum, Tier 3）—— Runna vs 不戴表用户的真实 job + 同模式 unrealistic paces / dial down
5. https://www.reddit.com/r/runna/comments/1j3z4r4/.../beginner_marathon_plan/（forum, Tier 3）—— BP1 beginner marathon intensity / injury
6. https://www.wsj.com/tech/personal-tech/runna-app-race-training-09c1a723（news, Tier 2-3）—— "Not Feeling 100%" 已加入 + injury 反馈
7. https://www.garmin.com/en-US/garmin-coach/overview/ + https://support.garmin.com/.../?faq=oYknGZ910l1pfBNzkDHX6A（official, Tier 1）—— DSW + Coach + adaptive 闭环 benchmark
8. https://www.trainingpeaks.com/changelog/ + https://help.trainingpeaks.com/.../Dynamic-Training-Plans（official/doc, Tier 1）—— Dynamic Plans coach workflow + changelog 显示 hidden workout / apply without workouts
9. https://www.runtastic.com/training-plans/running + https://www.adidas.com/us/running-app（official, Tier 1）—— Adidas plans by fitness assessment + feedback adaptation；release notes 沉默
10. https://apps.apple.com/sg/app/garmin-connect/id583446403 + https://apps.apple.com/us/app/trainingpeaks-plan-train-lift/id408047715（official, Tier 1）—— App Store version history + changelog 投资优先级推断

**搜索查询**（grok）：6 aspect × 3 queries avg ≈ 18-20 queries 跨 Runna / Garmin Coach / TrainingPeaks / Adidas / 训练计划 自适应 / injury adjustment / changelog 等关键词。

**Provenance 校验**：6/6 aspect 通过 Lapis byte-equal evidence check；merge 后 43 evidence + 28 findings 0 dangling refs（引擎 merge 输出）。

---

*本报告由 PM DeepResearch [`product-capability` profile](../../capabilities/product-capability.md) + [`final-report-product-capability.md`](../../../../prompts/layer1/pm-deep-research/final-report-product-capability.md) 装配；6 aspect 真引擎产出（Lapis `9db7464` vendored verbatim）。Rubric 自评见 [`product-capability-rubric-score.md`](product-capability-rubric-score.md)。*
