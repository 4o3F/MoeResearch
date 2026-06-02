# Strava 是否应升级为 AI 跑步教练，以及如何与 Runna / Garmin / Nike Run Club 错位竞争

> PM DeepResearch 竞品深度研究报告 · decision_intent = ai_upgrade · 复杂度 = Deep（13 章）
> 目标产品锚点：Strava · 竞争集：Runna、Garmin Coach / Daily Suggested Workouts、Nike Run Club、adidas Running、通用 LLM、人类教练
> 生成日期：2026-05-30 · 语言：中文 · 引擎：Lapis deep_research（6/6 aspect，26 条证据，search=grok / model=openai）
> 整体置信度：**Medium**（无一手调研，证据为公开定位页 + 评测 + App Store/官方页；ODI / 价值曲线为标注 estimated 的专家估计）

---

## 1. 研究结论摘要

**应该升级，但不是"再做一个训练计划库"或"加一个聊天教练"——Strava 唯一能赢的位置，是把它已经独有的社交图谱和跨设备活动数据，变成一个"可信、可解释、能在你疲劳/缺课后自动重排"的日级教练。我们给这个定位起名「社交图谱日教练」（Social-Graph Daily Coach）。**

情境（S）：跑者已经在用 Strava 记录和社交，但当他们想"认真备赛、今天该怎么练"时，付费心智正流向 Runna（结构化 AI 计划）、Garmin（手表端恢复闭环）和 Nike Run Club（免费陪跑）。冲突（C）：Strava 的原生 AI 教练能力弱（价值曲线自评 adaptivity 仅 2/5，`positioning-whitespace:finding-2`），如果只复制 Runna 会进高价红海，只做通用聊天会输给 Garmin 的生理数据闭环。问题（Q）：升不升级、往哪升？回答（A）：升级，且把火力压在 ODI 最高、竞品最分散的"恢复感知的自适应日级教练 + 社交问责闭环"，分阶段发布而非一次性 all-in。

支撑这一结论的三个最强信号：

- **真实竞争是"训练计划主权"而非"有没有 AI 文案"。** ODI 排名前三全部是 underserved 且同一主题：疲劳/伤病后自动降载重排（Opportunity=14）、目标→可执行的每周自适应计划（13）、日级"今天该跑什么/为什么"（13）（`opportunity-gaps:finding-1`）。市场最缺的不是又一个 guided run 内容库。
- **Strava 刚收购了 Runna——这是整张竞争地图的枢轴。** 公开报道证实收购成立（`positioning-whitespace:ev-1-2`），意味着 Runna 既是外部标杆、又是可整合资产；"自建 vs. 整合"的建设成本判断因此完全改变。
- **窗口期真实存在但有限。** Garmin 占据严肃训练的生理闭环、Nike 用免费压低基础 coaching 价格、Runna 教育用户形成高质量自适应计划预期——三者夹击下，Strava 趁社交网络优势未被复制前定义"跨设备社交教练"类目，是 12–18 个月的窗口（`positioning-whitespace:finding-5`，TM-13）。

**最大的不确定性（决定成败）**：我们没有一手数据证明 Strava 的大众社交用户真的愿意为"训练处方"付费——他们雇佣 Strava 的 Job 也可能仍然是"记录与社交展示"。这是必须用第 11 章的实验最先证伪的假设；若证伪，本报告把 coaching 作为升级主轴的结论不成立（`job-and-competitive-set:counterargument-1`）。

---

## 2. 研究输入与边界

**这份研究回答一个 Build/Upgrade 决策，而不是做市场全景扫描。** 决策意图明确为 `ai_upgrade`：一个以跑步社交/记录见长的平台（锚点 Strava）要不要、以及如何进入 AI 跑步教练。

| 维度 | 设定 |
|---|---|
| 决策意图 | ai_upgrade（是否升级到 AI 跑步教练 + 如何错位） |
| 目标产品 | Strava（社交运动网络，新近收购 Runna） |
| 直接竞品 | Runna（coaching-first 订阅）、Garmin Coach / Daily Suggested Workouts（手表端自适应） |
| 间接 / 替代 | Nike Run Club（免费陪跑）、adidas Running、通用 LLM / 自建 AI coach、人类教练 |
| 受众 | Strava 产品 / 战略决策者 |
| 明确排除 | 力量训练专用、营养专用 App；非英文低可见度产品的能力可能被低估 |
| 数据边界 | 无一手调研、无留存/转化/完训数据；每 aspect 仅 2 次搜索；所有 Importance/Satisfaction/价值曲线为标注 estimated 的方向性估计（TM-4） |

边界诚实声明：本研究的结论强度受限于"公开证据 + 产品策略推断"，所有数值是给决策排序用的相对刻度，不是统计测量。第 12 章逐项列出由此带来的置信下调。

---

## 3. 目标产品定位与现状：Strava 的资产在"网络与中立"，短板在"教练可信度"

**Strava 的战略现状可以用 Cagan 3+3 速写：它的力量全在社交与数据网络，它的弱点全在"把数据变成可信训练决策"——而 AI 升级正好踩在它最弱的那条轴上。**

| 维度 | Strava 速写（基于公开定位证据，medium 置信） |
|---|---|
| 优势 1 | **社交图谱 + 问责机制**：kudos、leaderboards、challenges、俱乐部——这是竞品最难复制的网络效应资产（`job-and-competitive-set:finding-1`） |
| 优势 2 | **跨设备中立的数据聚合**：不绑定单一硬件生态，天然是 device-neutral 的活动数据中枢（`positioning-whitespace:finding-2`，curve summary） |
| 优势 3 | **刚收购 Runna**，获得现成的 coaching IP 与计划模板（`positioning-whitespace:ev-1-2`） |
| 弱点 1 | **原生自适应 coaching 弱**：价值曲线 adaptivity 自评仅 2/5；现有 AI（Athlete Intelligence / Instant Workouts）偏描述性反馈而非完整处方（`opportunity-gaps:finding-2`，`positioning-whitespace:finding-2`） |
| 弱点 2 | **缺手表端执行触达**：在"跑步进行时给 cue"的场景上弱于 Garmin 手表闭环（`experience-paths:finding-3`） |
| 弱点 3 | **首页以 feed / 活动回顾为主**：AI coaching 入口容易被埋没，用户行为上仍可能去 Garmin/Runna 决定"今天跑什么"（`experience-paths:finding-1` daily_workout 断点） |

**所以**：Strava 的 AI 升级不该是"补一个我没有的功能"，而该是"把我独有的网络 + 中立数据，接到我刚买来的 coaching 能力上"。这条主线贯穿后续所有章节。

---

## 4. 用户人群与 JTBD：跑者雇佣 AI 教练，是为了"今天该怎么练"的可信决策，不是更多图表

**核心 Job Statement（TM-4 evidenced，medium）**：

> 当我已经决定为比赛/健康/速度目标认真训练、但不确定今天该跑什么、又担心过度训练或计划失效时，我想把"目标 + 历史表现 + 身体状态 + 时间约束 + 动机缺口"转换成一个**可信、可执行、会调整**的训练决策——以便持续进步、少受伤、保持责任感。（`job-and-competitive-set:finding-1`）

这个 Job 拆成五个 desired outcome 后的 ODI 估计揭示了真正的痛点分布——它们不是"想要更多社交"，而是"想要训练决策被托管且可信"：

| Desired outcome | Importance | Satisfaction | Opportunity | 状态 |
|---|---:|---:|---:|---|
| 计划能随表现/恢复/睡眠/HRV/伤病风险调整 | 9 | 5 | 13 | Underserved |
| 每天知道"今天该跑什么 / 是否该休息" | 9 | 6 | 12 | Underserved |
| 对建议有信任：解释为什么这样练 | 8 | 5 | 11 | Underserved |
| 跨设备低摩擦接入历史数据并自动更新 | 8 | 6 | 10 | Latent |
| 训练过程有责任感、动机与社会强化 | 7 | 7 | 7 | Served |

（来源 `job-and-competitive-set:finding-1`；全部 estimated:true）

**TM-6 听弦外之音**：用户下载 Runna / Garmin / NRC，行为上表达的不是"给我更多 AI"，而是"减少今天该怎么练的决策成本"（`experience-paths:finding-3`）。"社交/记录"在 ODI 上是 Served 甚至 Overserved（社交展示/排行榜 Opportunity 仅 6，`opportunity-gaps:finding-1`）——这恰恰是 Strava 最强、却最不稀缺的环节。

**最强反驳（必须正面回应）**：跑者雇佣 Strava 的 Job 可能始终是"记录与社交展示"而非 coaching。若一手调查显示付费用户选择 AI 功能的主因是分享/排行榜，则本章把 coaching 作为升级主轴的判断错误（`job-and-competitive-set:counterargument-1`）。我们的处理：不假设，而是把它列为第 11 章最优先证伪项——这是一个判断（likely 成立），但置信度只到 medium。

---

## 5. 竞品与替代方案图谱：真正的竞争是"训练决策主权"，威胁来自三方夹击

**按 Job 而非 App 品类框定，真实竞争集是 5 类玩家，且最大威胁不是任何单点功能，而是"Garmin 锁高端生理闭环 + Nike 压低价格锚 + Runna 教育用户预期"的合围。**

### 5.1 真实竞争集（按 Job 重新框定，`job-and-competitive-set:finding-2`）

1. **Coaching-first 订阅 App**：Runna——直接替代"制定并调整计划"（现已被 Strava 收购）。
2. **Watch-native 自适应教练**：Garmin Coach / DSW——替代"今天怎么练"，靠生理数据 + 手表触达形成 friction advantage。
3. **社交/进度网络**：Strava 自身——擅长记录、比较、accountability；AI 升级让它从相邻 Job 扩入训练决策 Job。
4. **通用 / 自建 AI 教练**：通用 LLM——非显而易见的低端替代，短板是生理数据接入与安全约束。
5. **人类教练**：高信任、高责任、高价的上游替代，定义"好教练"的期望上限。

> Nike Run Club / adidas Running 在本轮证据中信号偏弱，应先列为 tracking/brand/community 相邻替代，除非补证显示其已有强自适应 coaching（`capability-and-importance:finding-4`，避免把"搜索可见度低"误判为"能力缺失"）。

### 5.2 价值曲线：四条买方可见轴上，没有人同时占住"自适应 + 社交 + 中立 + 可负担"

买方验证的 4 条定位轴（来自官方定价/功能页 + 第三方购买比较页，非凭空发明）：adaptivity/AI coaching、community/social、hardware integration、price accessibility（`positioning-whitespace:finding-1`）。各玩家相对强度（1–5 估计，medium）：

| Player | 自适应/AI | 社交 | 硬件整合 | 价格可达 | 曲线概括 |
|---|---:|---:|---:|---:|---|
| Strava core | 2 | 5 | 4 | 3 | 社群/路线/数据网络强，原生 coaching 弱 |
| Runna | 5 | 3 | 3 | 2 | 结构化自适应强，价格高、社群弱 |
| Garmin Coach/DSW | 4 | 2 | 5 | 3 | 绑定硬件与生理数据，跨设备弱 |
| Nike Run Club | 2 | 3 | 3 | 5 | 免费陪跑 + 品牌动机强，自适应弱 |
| adidas Running | 2 | 3 | 3 | 4 | freemium tracking 型，AI 证据弱（低置信） |

（`positioning-whitespace:finding-2`）

**白地（whitespace）**：「社交图谱驱动的、跨设备、低摩擦 AI running coach」——把 Strava 的社群问责 + Runna 级计划 + Garmin 级恢复信号 + Nike 级低门槛组合起来。**为什么至今无人占据**：Runna 优先优化付费结构化训练（价格高）、Garmin 优先优化自家硬件（中立性弱）、Nike 优先免费内容（自适应深度弱）、Strava 历史强项是社交记录而非教练处方（`positioning-whitespace:finding-3`）。

### 5.3 威胁分级（Christensen，medium）

| 竞品 | 威胁类型 | 为什么 |
|---|---|---|
| Garmin Coach / DSW | **Sustaining 高端在位者** | 用硬件/恢复/表现数据服务高要求跑者；威胁是高端性能闭环，不是价格破坏 |
| Nike Run Club | **低端/新市场破坏者** | 免费 guided plans 降低入门门槛，教育大众"基础 coaching 应免费" |
| Runna | **整合则 sustaining，长期独立则 disruptive** | 强化 Strava 进入 coaching 的价值主张；但若独立品牌持续吸走高意图用户，会从"严肃备赛"楔入 |
| adidas Running | Sustaining / 弱破坏证据 | 更像 freemium tracking 竞争，证据不足 |
| **Strava 不升级** | **自我破坏风险** | 不把 Runna 能力转为核心体验，用户会继续只把 Strava 当记录层，把 coaching 预算给别人 |

（`positioning-whitespace:finding-4`、`job-and-competitive-set:finding-3`）

**所以**：竞争的胜负手是"谁能持续读取多源生理/行为上下文、并负责任地改变今天的训练"，而 Strava 唯一不可复制的筹码是社交图谱 × 跨设备数据 × 现在到手的 Runna IP。

---

## 6. 功能架构与体验路径：差距不在"是否冠名 AI"，而在"数据→计划→执行→反馈→重排"是否闭环

**把竞品能力对位后会发现：Runna 在"计划+执行+支持"上最完整，Garmin 在"设备/恢复闭环"上最强，而 Strava 现有 AI 更像入口层——真正的 gap 是闭环深度，不是有没有 AI 标签。**

### 6.1 能力对位矩阵（每单元含 inline 证据或标 assumption，`capability-and-importance:finding-1`）

| 能力 | Strava | Runna | Garmin | NRC | adidas |
|---|---|---|---|---|---|
| 个性化/自适应计划 | 部分（Instant Workouts，或偏静态）`ev-1-1/1-2` | **强**（专家计划+adaptive）`ev-1-3/2-4` | **强**（按表现/恢复，偏设备）`ev-1-3/2-6` | 未确认 *assump* | 未确认 *assump* |
| AI 生成单次训练/路线 | **确认**（Instant Workouts 生成 workouts/routes）`ev-1-1` | 部分 `ev-1-3` | 部分（DSW，非生成式）`ev-1-3/2-6` | 未确认 *assump* | 未确认 *assump* |
| 设备/恢复指标闭环 | 未确认 *assump* | 确认（Strava/Garmin sync）`ev-2-4` | **强**（恢复/表现+watch）`ev-2-6` | 未确认 *assump* | 未确认 *assump* |
| 执行层 cue（配速/音频） | 部分 `ev-1-1` | **强**（audio/pace cues）`ev-1-3/2-4` | 中强（watch）`ev-2-6` | 未确认 *assump* | 未确认 *assump* |
| 配套（力量/营养/教练支持） | 未确认 *assump* | **强**`ev-1-3/2-4` | 未确认 *assump* | 未确认 *assump* | 未确认 *assump* |

> 诚实标注：NRC / adidas 多格为 assumption（本轮证据未见，给了 falsifiable test），不能判定为"缺失"——只能判定为"未确认"（`capability-and-importance:finding-4`）。

### 6.2 Kano 分级（TM-4：公开证据 + 实践解释，无一手调研，`capability-and-importance:finding-2`）

- **自适应计划 + 缺课后重排** → Performance，逼近 Must-be 门槛（严肃训练场景下静态计划会变不满点）。
- **设备/手表同步 + 恢复闭环** → Must-be for serious runners（没同步会破坏 coaching 可信度）。
- **执行层 cue（配速/音频）** → Performance（提示越清晰，认知负荷与训练失败风险越低）。
- **力量/营养/交叉训练/教练支持** → Attractive（部分付费用户视为 Performance）。
- **AI 生成即时训练/路线** → 当前 Attractive，竞品普及后转 Performance。

TM-2 读法：评分/提及是量化线索，定性含义是用户在奖励"少规划、少手动同步、少跑错配速"的减摩擦体验。

### 6.3 体验路径四断点：闭环断在 Job 之间，而非单点功能

强训练导向产品把 coaching 拆成 4 个连续 Job，Strava 的 AI 风险都在 Job 间的断点上（`experience-paths:finding-1`）：

| 路径 | 用户 Job | 竞品已建立的模式 | Strava AI 的断点 |
|---|---|---|---|
| Onboarding/计划设置 | 把目标/水平/日程转成计划 | Runna 显式采集水平/频次/长跑日/赛事目标→生成计划 | 不前置采集目标与约束，AI 只能给泛化建议（`finding-2`） |
| Daily workout | 今天跑什么/为什么/配速 | Runna Train 屏训练卡、NRC Guided Runs、Garmin DSW 同步手表 | 首页仍以 feed 为主则入口被埋没（`finding-3`） |
| Feedback/适应 | 这次对不对，下一步改不改 | Runna AI post-run insights、Garmin 按恢复调整 | 只总结历史不更新未来处方→"懂我但不带我练"（`finding-4`） |
| Retention | 相信计划仍适合我 | Runna 计划可见到比赛日、日程灵活 | 调整原因不可见则 AI 被当黑盒（`finding-5`） |

**关键诚实缺口**：本轮没拿到足够的 Strava 现有 UI 截图，因此**不对 Strava 当前 UI 下强断言**，只把上述表述为"AI 升级设计风险"（`experience-paths:finding-6`）。补图是第 12 章列出的首要 gap。

---

## 7. 视觉证据资产表

**Deep 层要求 ≥5 条视觉证据；本轮拿到 3 条 URL 级视觉证据，未达标——因此第 6 章的 UI 结论保持"设计风险"措辞，不升级为高置信 UI 判断。** 这是按 floor 主动 abstain，而非隐瞒。

| 产品 | 屏/流程 | media_type | source_url | observed_feature | related_claim | 置信 |
|---|---|---|---|---|---|---|
| Runna | listing / onboarding / 计划 / 反馈 | app_store_page | apps.apple.com/.../id1594204443 (`experience-paths:ev-1-1`) | 截图含 onboarding、计划可见性、AI insights、日程灵活 | Runna 用显式设置 + 可见计划/反馈面闭环 | medium |
| Nike Run Club | Training Plans / Guided Runs | app_store_page | apps.apple.com/.../id387771637 (`experience-paths:ev-2-4`) | listing 强调 Training Plans 与 Guided Runs 截图 | NRC 用计划+陪跑面降低日常执行摩擦 | medium |
| Garmin | Coach overview | official_page | garmin.com/.../garmin-coach/overview/ (`experience-paths:ev-2-5`) | 官方页描述自适应计划 + DSW 同步手表 | Garmin 把适应与日常执行做成 watch-native | medium |

**缺口**：缺 Strava 自身的 onboarding / 计划设置 / 日训处方 / 跑后 AI 反馈截图（≥2 条），以及 Runna/Garmin 的实机帧核验。Deep 层应由 Skill 触发 Layer-2 浏览器抓取补齐（外部步骤，非 aspect agent）。补齐前不输出 Strava UI 强结论。

---

## 8. AI / 新能力映射（ai_upgrade 专章）：把 AI 放在"训练决策"而不是"内容生成"上

**决策意图是 ai_upgrade，所以这一章是重点：Strava 的 AI 不该堆在赛后文案和聊天框里，而该作为日级训练决策层 + 社交问责层。** 用 TM-9 杠杆分级看清楚把工程投到哪：

| 层级 | 内容 | 判断 |
|---|---|---|
| **10x multiplier** | Strava 历史活动数据 × 社交问责 × 外部可穿戴状态 → 训练遵从闭环；Runna coaching IP | 火力集中于此（`job-and-competitive-set:finding-5`、`build-cost-version-history:finding-3`） |
| Additive | 单独的训练计划模板、聊天问答、跑后总结、guided audio | 做，但不是差异化来源 |
| Overhead | 无行动出口的 AI 文案、泛化鼓励、无限制聊天、不可验证的 performance insights、过度个性化 UI | 明确不做 |

能力优先级（由 ODI 驱动，`opportunity-gaps:finding-3`）：**恢复感知的自适应计划 + 日级训练决策 + 可解释反馈闭环**，而不是再做一层泛聊天/泛总结/社交包装。AI 必须拥有"训练计划主权"——能改写下一次 workout，而不只是解释上一次（`opportunity-gaps:finding-2`）。

安全边界（这是信任的命门）：先用"规则/训练负荷模型约束 + LLM 解释层"，把 LLM 当解释器而非医疗诊断器；明确定义"哪些建议永远不能自动化"（伤病/疼痛处理），否则 12–18 个月后最可能的死因就是信任崩塌（`build-cost-version-history:finding-5`，`opportunity-gaps:finding-4`）。

---

## 9. 产品机会矩阵：恢复感知自适应教练是最高优先级，建设成本用竞品迭代节奏校准

**ODI 排序 + Kano 类型 + 用竞品版本节奏校准的建设成本，三者叠加后，P0 清晰指向"恢复感知的自适应日级教练"。ODI 只是主排序，价值/复杂度/风险仍会调整最终优先级。**

| Rank | 机会（desired outcome） | Imp | Sat | Opp | Kano | 建设复杂度（竞品节奏校准，TM-4） | 优先级 |
|---:|---|---:|---:|---:|---|---|---|
| 1 | 疲劳/伤病/缺课后自动降载、重排、安全回归 | 10 | 6 | 14 | Performance→Must-be | 高：需疲劳/伤病 guardrails + 可解释；属"可订阅 adaptive coach"档，约 4–6 个版本/2–3 季度 | **P0** |
| 2 | 目标→每周自适应可执行计划 | 9 | 5 | 13 | Performance | 中高：可复用 Runna 计划模板（已收购）显著降本 | **P0** |
| 3 | 日级"今天跑什么/为什么/配速/可重排" | 9 | 5 | 13 | Performance | 中：需嵌入日常启动路径，非 feed | **P0** |
| 4 | 跑后反馈→下一步训练动作与风险调整 | 8 | 5 | 11 | Performance | 中：把已有 Athlete Intelligence 从描述性升级为处方性 | P1 |
| 5 | 跨设备统一建模（Garmin/Apple/Strava 同一 coach） | 8 | 6 | 10 | Latent | 高：数据接入与一致性工程 | P1 |
| 6 | 个性化动机/陪伴（语音、提醒、轻问责） | 7 | 7 | 7 | Attractive | 低：可复用社交资产 | P2 |
| 7 | 社交展示/排行榜/AI 文案 | 6 | 8 | 6 | — | 低（Overserved，勿投） | 不投 |

（ODI 来自 `opportunity-gaps:finding-1`；复杂度来自 `build-cost-version-history:finding-3` 的版本节奏估算）

**建设成本三档（用 App Store 版本历史作 revealed-strategy 代理，TM-12，`build-cost-version-history:finding-3`）**：

- **低成本 MVP（1–2 个版本）**：计划解释 + 跑后总结 + 简单问答。价值风险高，易被当 gimmick。
- **可订阅 adaptive coach（4–6 个版本 / 2–3 季度）**：目标输入 + 计划调整 + 疲劳/伤病 guardrails + 跑后反馈 + 失败恢复 + 可解释性 + 客服兜底。**← 推荐目标档**
- **Garmin-like 生理闭环（12–18 个月）**：稳定获取负荷/恢复/睡眠/HR-HRV + 模型评估 + 安全体系。

**关键校准（避免低估成本）**：Runna / Garmin 的公开 changelog 高度泛化（"bug fixes & performance improvements"），不能据此判断 AI coaching 是低成本——真实投入沉在计划生成、适应性规则、伤病/负荷、可解释性、QA 和客服（`build-cost-version-history:finding-1/2`）。但反向也成立：Strava 若能复用既有 ML/排序/segments/支付数据管线，边际成本可能低于 Runna 从零起家（`build-cost-version-history:counterargument-2`）——所以"2–3 季度"是带不确定性的下限估计。

---

## 10. Roadmap 建议：分层发布，而非一次性 all-in 完整 AI 教练

**核心建议：采用"三阶段分层发布"，每一层都先回答一个去险问题，再决定要不要进下一层。一次性发布完整 AI coach = 同时承担价值、信任、成本三重风险。**（`build-cost-version-history:finding-4`、`job-and-competitive-set:finding-4`）

| 阶段 | 时间 | 交付 | 去险目标 | 验证条件（过则进下一阶段） |
|---|---|---|---|---|
| **P0-A：AI 训练洞察层** | 0–8 周 | 解释已有训练、目标进度、恢复风险；**不自动改计划** | Value：用户是否愿为"更懂我为什么这样练"付费（而非聊天新奇感） | 洞察层带动订阅试用→留存提升显著 |
| **P0-B：受约束的自适应计划层** | 8–24 周 | 在明确规则内调整跑量/强度/恢复日，**记录每次调整原因**；复用 Runna 计划模板 | Usability：建议嵌入跑后/周计划/目标进度，不另建 chatbot | 计划坚持率、缺课后重排接受率、付费转化达标 |
| **P1：跨设备 + 评估是否进生理闭环** | 24 周后 | 跨设备统一建模；再评估是否做 Garmin-like 闭环 | Feasibility/Business：推理成本 vs 留存收益 | 跨设备遵从率 + 单位经济性 |

**四风险全覆盖（TM-3，每条建议都必须过）**：

| 风险 | 处理 |
|---|---|
| Value | 先面向"已有 Strava 社交关系 + 正在备赛 5K/10K/半马/全马"的高意图人群，不是所有休闲跑者 |
| Usability | AI 建议嵌入训练日历/跑后复盘/俱乐部挑战/好友问责，**不做独立 chatbot** |
| Feasibility | 先用活动历史 + 配速 + 心率 + 恢复 proxy + Runna 模板；不一开始承诺医疗级伤病预测 |
| Business viability | 基础 adaptive insights 进订阅；完整计划/Runna 深度 coaching 作更高价 bundle，避免组合内互相蚕食 |

**TM-5 取舍（明牌说出放弃了什么）**：选择"device-neutral 社交 AI 教练" = 在 6–12 个月内明确放弃与 Garmin 同等深度的硬件专属生理模型；选择分层发布 = 放弃"一次发布完整 AI coach"的传播爆点，换更低安全风险、更可控成本、更清晰的付费测试（`positioning-whitespace:finding-3`、`build-cost-version-history:finding-4`）。

**整合决策（因 Strava 已收购 Runna 而成为最高杠杆动作）**：把 Strava 端做成"低摩擦入口 + 轻量 coach"，把深度计划导向 Runna 能力或更高阶订阅，避免组合内自相蚕食（`opportunity-gaps:finding-3`）。整合边界（数据互通/统一日历/统一订阅/完整并入）是第 12 章的开放问题。

---

## 11. 验证实验与指标：先证伪"大众用户愿为训练处方付费"

**在写任何一行 coaching 代码前，先用最便宜的实验证伪第 1 章那个最大不确定性。** 验证顺序严格对应 roadmap 的去险问题。

| # | 待验证假设 | 实验 | 主指标 | 失败阈值（则停止/转向） |
|---|---|---|---|---|
| 1（最优先） | Strava 大众用户愿为"训练处方"付费，而非只要社交/记录 | 对高意图分层放出洞察层付费墙 A/B | 付费转化、付费意愿调研 | 转化 ≤ 对照社交功能边际，则 coaching 非升级主轴（推翻第 1/4 章） |
| 2 | AI 日级决策提升训练执行，而非只增好奇试用 | 自适应计划 vs 静态计划 A/B | 训练完成率、计划坚持率、缺课后重排接受率 | 完成率无显著提升则 daily decision 定位失败 |
| 3 | 社交问责确实提升训练遵从 | 社交问责 on/off 实验 | 周活、连续训练周数、留存 | 无提升则 social multiplier 假设不成立（`job-and-competitive-set:finding-5`） |
| 4 | 恢复感知建议被信任并被执行 | 恢复 guardrail 建议接受率追踪 | 建议接受率、过载投诉率 | 接受率低/投诉升则收紧自动化边界 |

指标定义须区分**可能性**（likely/highly likely）与**置信度**（基于证据量与稳健性）。本研究对上述假设的判断均为 "likely 成立 / 证据 medium"——必须用一手实验把 medium 抬到 high 后再扩张。

---

## 12. 风险、冲突与开放问题

### 12.1 TM-8 预演：假设 18 个月后失败，三个最可能死因（跨 aspect 收敛）

1. **战略层（Value death）**：把 AI 当内容/聊天功能而非训练决策系统，既赢不了 Garmin 的数据闭环，也夺不走 Runna 的 coaching 心智（`positioning-whitespace:finding-5`、`build-cost-version-history:finding-5`）。
2. **激励层（Incentive death）**：增长团队优化 feed engagement / 订阅转化，产品团队优化 engagement，coaching 团队优化训练质量——三者指标冲突，AI 建议要么打断社交体验，要么为留存牺牲训练可信度（`opportunity-gaps:finding-4`、`job-and-competitive-set:finding-6`）。
3. **信任/文化层（Trust death）**：消费 AI 团队追求"惊喜与生成感"，用泛化 LLM 文案替代可解释训练处方；一旦建议不准或过度自信，严肃跑者回到 Garmin/Runna/真人教练（`build-cost-version-history:finding-5`）。

TM-7 根因：表面看是 prompt/UX 问题，深层是战略选择没定清——Strava 到底要做社交记录平台、训练订阅产品，还是 Garmin/Runna 式教练替代品。

### 12.2 已吸收的最强反驳（calibrated）

- **"大众用户其实只要社交记录"**：likely 成立的反驳，已列为第 11 章 #1 优先证伪项；当前判断 coaching 是主轴，置信 medium。
- **"Garmin 硬件整合不是 Strava 可竞争轴"**：若高价值跑者从不愿把训练决策迁到 Strava，则白地变小——这正是为何推荐 device-neutral 而非硬件深度（`positioning-whitespace:counterargument-2`）。
- **"AI coaching 边际成本可能远低于 Runna 历史成本"**：若 Strava 能复用既有数据管线，"2–3 季度"下限可能偏高——已在第 9 章标注为带不确定性的估计（`build-cost-version-history:counterargument-2`）。
- **"NRC/adidas 能力被低估"**：搜索可见度低 ≠ 能力缺失；矩阵中相关格已标 assumption + falsifiable test（`capability-and-importance:finding-4`）。

### 12.3 开放问题（需一手数据/补研究）

- Strava 哪类用户愿为 AI coaching 付费：备赛者、PB 追求者，还是需要动机的新手？哪个细分 Satisfaction 最低且可规模化？
- Runna 与 Strava 的整合边界应到哪：数据互通 / 统一日历 / 统一订阅 / 完整并入？
- Garmin 用户是否愿意把训练决策迁到 Strava，还是只接受 Strava 作展示层？
- AI 教练的安全边界应停在训练建议、恢复建议，还是伤病/疼痛处理？
- Strava 内部已有多少训练负荷/恢复/目标预测模型可复用（直接决定建设成本）？

### 12.4 自验证记录（Phase C 质量 floor）

| Floor item | 状态 | 说明 |
|---|---|---|
| 目标产品基础（≥3 源，偏 Tier1/2） | **通过** | 多条 official（App Store/官网/Garmin 官页）+ news（收购报道） |
| 竞品数 ≥3，覆盖直接/间接/替代 | **通过** | 5 类玩家：Runna/Garmin（直接）、NRC/adidas（间接）、LLM/人类教练（替代） |
| 视觉证据 ≥5（Deep） | **未达标** | 仅 3 条 URL 级；已 abstain UI 强结论，列为首要补研究（第 7 章） |
| 用户证据 ≥20 条评论/社交 | **未达标** | 评论级证据有限（每 aspect 2 次搜索）；Kano/ODI 已标 estimated 并降置信 |
| 能力矩阵每格有证据或标 assumption | **通过** | NRC/adidas 多格标 assumption + falsifiable test |
| 机会矩阵 ≥5，含价值/复杂度/证据/优先级 | **通过** | 7 项，含 ODI + Kano + 建设复杂度 + 优先级 |
| 关键结论有 high/med/low + TM-4 标注 | **通过** | 全篇 medium，逐项标 estimated/evidenced |
| 开放问题/冲突单列 | **通过** | 见 12.2 / 12.3 |

**结论**：13 章结构与论证 floor 通过；两项数据量 floor（视觉证据、用户评论量）未达标，已对相应结论主动降置信或 abstain，并列入补研究清单——未隐瞒、未灌水。

---

## 13. 附录：来源与搜索记录

### 13.1 来源信用分级（4 档，映射自 source_type + 域名）

**Tier 1–2 / High（可支撑事实断言）**：
- Runna 官网 / App Store / features：runna.com, runna.com/features, apps.apple.com/.../id1594204443, play.google.com/.../com.runbuddy.prod（`*:ev` 多条）
- Garmin Coach 官页 / Garmin 训练博客：garmin.com/.../garmin-coach/overview/, garmin.com/.../garmin-training-plans-for-runners/
- Nike Run Club / Garmin Connect App Store：apps.apple.com/.../id387771637, apps.apple.com/.../id583446403
- 收购报道（news）：frontofficesports.com/strava-runna-fitness-app-purchase-deal/（`positioning-whitespace:ev-1-2`，支撑"Strava 收购 Runna"事实）

**Tier 3 / Medium（分析判断）**：
- 评测/分析博客与媒体：therunnerbeans.com（Runna 评测）、lifehacker.com、therunninggenie.com（Strava Athlete Intelligence vs AI coaches）、tech.yahoo.com（Strava Instant Workouts）、gowod.app、rouvy.com（best running apps 比较）、tatocaster.medium.com（自建 AI coach）

**Tier 3 社区 / Low（仅情绪/线索，不作事实）**：
- YouTube 评测：youtube.com/watch?v=eMLm73cf2II, watch?v=WJp1tpWYVmQ（source_type=unknown，已降置信）

### 13.2 搜索记录（query × provider=grok）

| Aspect | 代表性 query |
|---|---|
| job-and-competitive-set | "why runners use running apps training plans coaching motivation … Garmin Coach DSW Strava" |
| capability-and-importance | "Strava AI run coaching Runna Garmin Coach DSW NRC adidas official features" |
| opportunity-gaps | "AI running coach app adaptive training plan feedback … Strava Athlete Intelligence" |
| positioning-whitespace | "Strava Runna Garmin Nike adidas AI coaching adaptive community hardware price official" |
| build-cost-version-history | "Runna / Garmin Connect app version history release notes App Store" |
| experience-paths | "Runna / Strava / Garmin Coach / NRC screenshots onboarding training plan daily workout feedback App Store" |

### 13.3 证据计量

26 条证据（official 14 / blog 6 / news 2 / unknown 2，按 source_type）；覆盖 6 个 aspect；无悬空引用（合并后校验 dangling=0）；全部 retrieved 2026-05-29，published_at 多为空（聚合摘要，已据此保持 medium 置信）。

---

> 方法论标注图例：TM-1 Job→Feature→Gap · TM-2 metrics-informed · TM-3 四风险去险 · TM-4 认知状态标注 · TM-5 显式取舍 · TM-6 听弦外之音 · TM-7 影响层级 · TM-8 预演失败 · TM-9 杠杆点 · TM-11 可证伪 · TM-12 言行对照 · TM-13 面向未来。本报告最终判断与行文由 Skill 层（PM DeepResearch）综合，Lapis 引擎提供结构化 aspect 研究与证据。
