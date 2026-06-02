# PM DeepResearch 通用产品深度研究规格（canonical · 通用层）

> Status: canonical · 通用层 SSOT。
> **单一事实源（通用层 SSOT）**：本文件定义 4 项能力（竞品深度研究 / 产品能力研究 / 创新方向研究 / 产品需求深度调研）共用的顶层框架、方法库、证据/可信度机制、人格-TM 体系、报告模板族、Lapis 接口承载契约。**能力差异**装配在 [`capabilities/`](capabilities/) 各 profile 文档中。
> 方法论来源：track B 产品方法论调研 + track A 编排/可信度调研（21 份 arxiv 一手核实；详 PM DeepResearch 源仓 ADR 链）。
> 引擎接口：[`orchestration-interface.md`](orchestration-interface.md)；评测：[`evaluation/rubric.md`](evaluation/rubric.md)。
> v2.0 竞品研究的全部已验证方法论由本文件 + [`capabilities/competitive.md`](capabilities/competitive.md) 共同承载（黄金 [Strava 23/24](evaluation/golden/competitive-strava-coach-upgrade.md)）。

---

## 0. 定位与产出标准

PM DeepResearch = 产品深度研究的 **Skill 层（Layer 1 业务编排）**，MCP 工具能力主要由 **Lapis** 引擎提供（必要时不局限于 Lapis）。本规格定义其**通用产品深度研究能力**，承载 4 项具体能力。

**4 项产出标准（所有能力共同最低线）**：

| 标准 | 定义 | 最低要求 |
|---|---|---|
| 准确 | 关键事实可追溯，判断不伪装成事实 | 核心结论绑定来源 + 时间 + 置信度（§6）|
| 有证据 | 结论来自资料/截图/视频/评论/竞品/release notes | 关键判断必须有 evidence 或 visual_evidence |
| 有产品思考 | 从用户任务→功能路径→体验断点→机会推导 | 至少有功能必要性分析 + 体验路径 / outcome 之一（按能力 profile） |
| 可落地 | 结论能进 Roadmap/实验/指标/PRD | 至少含机会矩阵或解空间 + 优先级 + 验证计划之一（按能力 profile） |

> 各能力 profile 在通用最低线之上**追加** capability-specific 必填项（见各 profile 文档）。

---

## 1. 能力路由模型（trigger → intent → capability → complexity → profile）

```
用户请求
  │
  ▼
1. 触发识别（关键词 / 决策线索）         ─ §1.1
  │
  ▼
2. decision_intent 推断                  ─ §1.2
  │
  ▼
3. capability 选择                       ─ §1.3
  │
  ▼
4. complexity 路由（Quick/Standard/Deep）─ §1.4
  │
  ▼
5. profile 装配（dim 强调 / 骨架 /        ─ §11
   报告模板 / 人格-TM 权重 / gap & floor）
```

### 1.1 触发条件
| 触发线索 | 候选能力 |
|---|---|
| 竞品对比 / 差异化 / 进入判断 / AI 升级（带竞品对照）| **competitive** |
| 某能力域内做得多好 / 体验断点 / 能力补齐 / 能力升级（不必图谱） | **product-capability** |
| 未来方向 / 白地 / 新能力下注 / AI 战略 / 颠覆评估 | **innovation-direction** |
| 已选定问题 / 出 PRD 前置物 / 验证方案 / 立 OKR / 写需求 | **product-requirements** |

> 一个请求可能命中多能力——优先按 decision_intent + 用户明示锁定单一 capability；模糊时由 Skill 询问或回到 competitive 默认（最常见）。

### 1.2 decision_intent 推断（先于拆解）
Orchestration 先推断 `decision_intent`——用户基于本次调研做什么决策。没有决策意图，agent 产泛信息罗列；有了它，每段分析锚定到决策。

| decision_intent | 研究目标 | 关键输出 | 亲和 capability |
|---|---|---|---|
| Enter / Not Enter | 是否进入某市场 / 方向 | 竞品格局、机会缺口、进入风险、推荐结论 | competitive / innovation |
| Differentiate | 如何形成差异化 | 差异、能力缺口、定位白地 | competitive / capability / innovation |
| Build / Not Build | 是否建设某功能 | 用户价值、能力对位、**建设成本**（竞品迭代节奏信号）、验证计划 | competitive / capability / requirements |
| Improve | 如何优化体验 | 体验路径、断点诊断、优化建议 | capability / requirements |
| Grow | 提升增长 / 留存 / 转化 | 漏斗问题、机制对照、实验方案 | capability / requirements |
| AI Upgrade | 用 AI 改造产品 | AI 能力映射、场景机会、风险边界 | innovation / competitive |

### 1.3 capability 选择
4 项能力 = 通用 frame 之上的 4 种**装配 profile**。每能力 = profile 文档定义 §11 装配契约六字段。详见 [`capabilities/`](capabilities/) 各 profile。

| capability | profile 文档 | 报告模板族 | 主要决策意图 |
|---|---|---|---|
| **competitive** 竞品深度研究 | [`capabilities/competitive.md`](capabilities/competitive.md) | 13 章 | Enter / Differentiate / Build / AI Upgrade |
| **product-capability** 产品能力研究 | [`capabilities/product-capability.md`](capabilities/product-capability.md) | 13 章变体（Ch6/7/4 加重） | Improve / Build / Differentiate（能力域）|
| **innovation-direction** 创新方向研究 | [`capabilities/innovation-direction.md`](capabilities/innovation-direction.md) | 13 章变体（Ch8/9/10/12 加重） | AI Upgrade / Enter（新方向） / Differentiate（未来）|
| **product-requirements** 产品需求深度调研 | [`capabilities/product-requirements.md`](capabilities/product-requirements.md) | **8 段 PR-FAQ 模板** | Build / Improve（已选定问题） |

### 1.4 复杂度路由

| 等级 | 适用 | 证据要求 | 输出 |
|---|---|---|---|
| Quick | 窄问题、快速方向判断 | 5–10 来源 | 简版判断（按 profile 裁剪到关键章/段） |
| Standard | 常规研究 | 10–25 来源 | 标准报告 |
| Deep | 战略 / 进入判断 / PRD 前置 | 25+ 来源，**含视觉证据** | 完整报告（按 profile 模板） |
| Deep + Evidence Pack | 需支持评审 | 完整来源表 + 截图 / 视频 URL + 评论样本 + 矩阵 | 完整报告 + 证据资产表 |

> Quick 是重要"短路"——避免对简单问题启动整个多 agent 编排。

---

## 2. MECE-6 顶层维度集

ADR-0006 决策 1：MECE-6 = 跨 4 能力的顶层维度集。具体能力的"骨架"（如竞品五维、需求八段）是 MECE-6 中部分维度的**装配产物**，不是平行替代。

| 维度 | 研究对象 | 典型 evidence 形态 |
|---|---|---|
| **M1 Market Context** 市场与场景 | 市场边界、场景、趋势、规模、监管 | 行业报告、统计、政策、技术趋势 |
| **M2 User & JTBD** 用户与待办任务 | 用户分层、动机、痛点、未满足需求、outcome | 一手访谈、评论、问卷、行为数据 |
| **M3 Competitive Landscape** 竞品与替代 | 直接 / 间接竞品、JTBD-同源替代、威胁 | 产品对比、teardown、版本时间线、定价 |
| **M4 Product & Experience Capabilities** 产品能力与体验 | 功能、流程、交互、视觉证据、路径断点 | 截图、视频、teardown、步数、错误率 |
| **M5 Business & Growth Model** 商业与增长 | 获客、激励、留存、转化、变现、单经济 | 财报、漏斗、激励机制、定价对比 |
| **M6 Future Capability & Strategic Opportunity** 未来能力与机会 | AI / 硬件 / 内容 / 社区 / 数据能力、定位白地、下注方向 | 趋势分析、ODI underserved、白地图、pre-mortem |

> **正交性 note（A3 MECE 工程）**：6 维边界由"主用方法"区分（§4 / §5）。若同一证据可入多维（如 release notes 既是 M4 能力证据，也是 M3 竞品 build-cost 代理），按"该证据**支撑哪条 finding**"归位，而不是按"证据本身长什么样"。

---

## 3. 2 研究人格 + 13 TM + 跨人格质量门

ADR-0006 决策 2：**2 核心人格** + **跨人格质量门**。其它 capability-specific 人格扩展（如 Innovation 的 Futurist 视角）= 通用人格 + 加重 TM 权重，**不**新增人格类别。

### 3.1 两个核心人格

**Product Experience Analyst（用户 / 体验 / 证据）** — 携 TM：
- **TM-1 Job→Feature→Gap**（最高杠杆）：评估功能前先定位它服务的 user job，追 job→现有功能路径→体验 gap，按"填补 gap 程度"赋权。
- **TM-2 metrics-informed 非 metrics-driven**：每个量化发现配定性解读。
- **TM-6 听弦外之音**：记录用户没说 / 用行为而非语言表达的东西（Horowitz "吵车要的不是更响音响而是更安静的车"）。
- **TM-10 5Qs 测试**、**TM-12 言行分离**（访谈≠行为数据，冲突点名）。

**Product Strategist（战略 / 权衡 / 前瞻）** — 携 TM：
- **TM-3 四风险去险**：推荐须覆盖价值 / 可用性 / 可行性 / 商业可行性，缺一不完整。
- **TM-5 显性权衡**：每个选择写出代价 "选 X = 在 [时段] 显式放弃 Y"。
- **TM-7 影响层级**：执行失败往下挖战略 / 激励 / 文化根因。
- **TM-8 pre-mortem**：假设 12–18 月后已失败，列三大死因。
- **TM-9 杠杆点**：区分 10x 乘数 vs 加法 vs overhead（Doshi LNO）。
- **TM-13 面向市场的未来**：锚在市场 / 技术 / 竞争前进轨迹上；纯现状分析标"时效受限"。

### 3.2 跨人格质量门（注入所有人格、所有能力）
- **TM-4 认识论状态标注**：每条重要声明标 (a) 实证—附来源 / (b) 专家观点—注出处 / (c) 假设—给可证伪版 / (d) 推测—显式标注。**这是 PM DeepResearch 可信度使命的提示词级落点。**
- **TM-11 可证伪**：每个主要结论附最强反论 + 写出"什么条件下它是错的"。

### 3.3 各 capability 的人格-TM 权重
每 profile 在装配契约（§11）里给"人格主次 + 重点 TM"清单。例：
- competitive：EA + Strategist 平衡（TM-1/2/12 + TM-9/13）
- product-capability：EA 重（TM-1/2/6/10/12）+ Strategist 轻（TM-9/13）
- innovation-direction：Strategist 重（TM-3/5/7/8/9/13）+ EA 轻（TM-1/6）
- product-requirements：EA + Strategist 平衡（EA 取价值证据 / Strategist 取可行性 + 商业可行性）

具体见各 profile。

---

## 4. 方法库（所有能力共享）

每方法 = `{别名 / 主用途 / 输入 / 输出 / 证据标准 / 服务的 MECE-6 维度 / 关联人格-TM / 真实案例 / 反误用 note}`。**新增方法须经 Phase 1 同款审计**（真实可追溯 + 反误用 note）。

### 4.1 用户与机会方法

#### M-JTBD（Jobs-To-Be-Done）
- **主用途**：定义 job statement（situation→motivation→outcome）；找出"完成同一 job"的非显性替代者
- **输入**：用户访谈 / switch interview / 行为数据
- **输出**：job statement + desired outcomes 清单 + 真实竞争集
- **证据标准**：明确 job statement + ≥1 非显性替代者并给纳入理由
- **服务维度**：M2（首要）、M3（推出真实竞争集时）
- **人格-TM**：Experience Analyst / TM-1
- **真实案例**：Christensen 奶昔（对手是香蕉 / 百吉饼 / 无聊）；Facebook vs MySpace（两个不同 job）
- **反误用**：不要把 job 写成功能描述

#### M-Kano（Kano 模型）
- **主用途**：把功能 / outcome 分为 Must-be / Performance / Attractive
- **输入**：用户评论 / 问卷 / 一手调研
- **输出**：每功能 Kano 类型
- **证据标准**：分级须有用户证据或明确标 practitioner 诠释（TM-4）
- **服务维度**：M2、M4
- **人格-TM**：Experience Analyst
- **真实案例**：Kano 1984 原始论文（J-STAGE 核实）；GitLab 对 12 功能跑过正式 Kano 调研
- **反误用**：单条用户评论不能定 Kano 类型——要量

#### M-ODI（Opportunity Algorithm）
- **主用途**：把 outcome 排序为机会清单
- **公式**：`Opportunity = Importance + max(0, Importance − Satisfaction)`，1–10 量表
- **输出**：每 outcome 的 Opportunity Score；**>10=欠服务，<7=过度服务**
- **证据标准**：Imp / Sat 来自一手问卷优先；无一手时用研究证据 / 市场代理估算 + **TM-4 标注证据等级**（ADR-0006 决策 3）
- **服务维度**：M2（outcome 排序）、M6（underserved=机会下注）
- **人格-TM**：Strategist（结论收敛）+ Experience Analyst（数据准备）
- **真实案例**：Cordis 心脏支架"最小化再闭塞概率"未满足缺口 → 市占 1%→20%，J&J $109/股收购
- **反误用**：不是 RICE；不是团队内部分；估算值必须标证据等级

#### M-OST（Torres Opportunity Solution Tree）
- **主用途**：从 outcome 推 opportunity 推 solution 推 assumption test，组织解空间
- **输入**：desired outcome
- **输出**：每目标 outcome ≥3 候选方案 + 最危险假设清单
- **证据标准**：每候选方案附**既有 / 竞争方案**对照
- **服务维度**：M4（解空间）、M6（升级路径）
- **人格-TM**：Experience Analyst / TM-1 + Strategist / TM-3
- **真实案例**：Grailed LTV +20% 案例（producttalk.org）

#### M-PR-FAQ（Amazon Press Release + FAQ）
- **主用途**：用 PR 框定价值，倒推 FAQ 暴露关键决策与风险
- **输出**：≤300 字 Press Release + Internal/External FAQ
- **服务维度**：M2、M4、M5
- **人格-TM**：Strategist + Experience Analyst 双签
- **真实案例**：Amazon Prime / Kindle；Bryar & Carr《Working Backwards》(2021)
- **反误用**：不是营销稿——先讲价值再讲功能，禁止描述实现细节

### 4.2 竞争与定位方法

#### M-Teardown（功能对位矩阵 / Feature Teardown）
- **主用途**：跨竞品 / 跨产品功能对位评分（12 维 rubric，每格 1–5 分）
- **证据标准（强制）**：**每格必须附证据**——截图 / 评论数 / 操作步数 / 视频时间点。无证据的格标为假设
- **服务维度**：M3（跨竞品）、M4（深度版用于单产品能力域纵深 teardown）
- **人格-TM**：Experience Analyst / TM-1/2/12
- **真实案例**：Loom（异步视频，新市场颠覆，$975M 被 Atlassian 收购）；Built for Mars 系列

#### M-Positioning（Strategy Canvas / 感知图）
- **主用途**：以 buyer-validated 轴画 value curve，标白地
- **证据标准**：轴必须是买家验证过的购买维度（非臆造）；白地须给"为何无人占据"解释或假设
- **服务维度**：M3、M6
- **人格-TM**：Strategist / TM-9/13

#### M-Disruption（Christensen 颠覆理论）
- **主用途**：把竞品 / 趋势标"维持性 vs 颠覆性"威胁；判断是否需要自我颠覆
- **服务维度**：M3、M6
- **人格-TM**：Strategist / TM-8/13

#### M-Porter（Porter 五力）
- **主用途**：行业层结构分析
- **服务维度**：M1（**仅 M1，禁止下沉到 M4**）
- **反误用（强制）**：**禁止在产品 / 能力层使用**——Porter 是行业结构工具，不是产品工具（Phase 1 B1 系统性结论）

#### M-SWOT（仅沟通层）
- **主用途**：证据齐备后的**沟通**总结
- **使用约束（强制）**：**不作发现工具**——Hill & Westbrook 1997 研究 50 家英国公司发现 SWOT 产出极少转化为可行动战略；每格必须转成带证据的具体含义
- **服务维度**：沟通层（不归任何 MECE-6 维度）

### 4.3 价值与风险方法

#### M-Cagan-4Risks（Cagan 四大风险）
- **主用途**：评估推荐是否完整覆盖去险
- **4 类**：价值 / 可用性 / 可行性 / 商业可行性，每类给证据等级 + 来源
- **服务维度**：M2/M4/M5/M6 综合去险层
- **人格-TM**：Strategist / TM-3（强制，缺一不完整）
- **真实案例**：svpg.com/four-big-risks

#### M-PreMortem（pre-mortem）
- **主用途**：假设 12–18 月后已失败，列三大死因（Tigers/Paper Tigers/Elephants）
- **服务维度**：M3（威胁）、M6（下注风险）
- **人格-TM**：Strategist / TM-8（强制）

#### M-RiskFirst（风险优先 / pre-mortem 切入）
- **主用途**：先问"什么会杀死这个策略"——作为 Strategist 的 pre-mortem 切入
- **服务维度**：M3、M6
- **真实案例**：Reforge / Sachin Rekhi "Connected"

### 4.4 建设成本与实际动作方法

#### M-Changelog（Changelog / 版本时间线 = 实际动作证据）
- **主用途**：**判 "该不该建某功能" 不能只看用户价值，必须研究建设成本**——竞品 changelog / release notes / 版本时间线是最硬的外部代理
  - **TM-12 言行分离的直接落点**：营销说什么是"言"，版本里真正发了什么、发了几版、隔多久发是"行"
  - **build-cost 外部代理**：某能力对手迭代多少版才稳定 ≈ 我方自建复杂度 / 成本下界
- **数据源**：App Store Version History（最丰富免费档）/ Google Play / 官方 release notes / AppFigures
- **证据标准**：给可追溯版本时间线；陷阱：营销化 release notes 会隐藏真实工作量、捆绑发布、灰度未写入——拿不到可靠时间线时**标假设，不臆断节奏**
- **服务维度**：M3（实际动作 / 优先级）、M4（能力成熟度）、M6（build-cost 喂入下注判断）
- **人格-TM**：Strategist / TM-12 + Experience Analyst（取证）

### 4.5 备选方法（明确弱势但适用场景）

| 方法 | 适用 | 弱势 |
|---|---|---|
| RICE | 无客户调研数据时的团队 backlog 排序（Sean McBride/Intercom 2016）| "披着数学外衣的猜测"——不取代 ODI |
| WSJF | 强时间敏感 / 风险削减场景（Reinertsen Cost of Delay + SAFe）| 多变量、需团队对齐 |

> **明确剔除的非方法**："Value-vs-Effort 2×2" 无可追溯一手出处；Crayon/Klue 引语无法核实（Phase 2 子代理审计已剔）——禁止入库。

---

## 5. 方法 ↔ MECE-6 维度映射矩阵（A3 正交性骨架）

每维度的**主用方法**（粗）+ 支撑方法（细）+ 反误用。Profile 装配时按此选取。

| 维度 | 主用方法 | 支撑方法 | 反误用 |
|---|---|---|---|
| **M1 Market Context** | M-Porter（仅本维度合法）、市场规模 / 趋势分析 | — | Porter 不可下沉到 M3/M4 |
| **M2 User & JTBD** | M-JTBD、M-Kano、M-ODI（outcome 排序） | switch interview、TM-6 行为>语言 | 单 JTBD 不能写成功能描述 |
| **M3 Competitive Landscape** | M-Teardown（跨竞品）、M-Disruption、M-Changelog（实际动作）| M-Positioning（白地）、M-JTBD（推真实竞争集）| Porter 误用、SWOT 当发现 |
| **M4 Product & Experience Capabilities** | M-Teardown（深度版，单产品 / 能力域）、体验路径 + 断点地图、M-Kano | 视觉证据 first-class（§6.2）、M-Changelog（能力成熟度）| 矩阵无证据格 |
| **M5 Business & Growth Model** | 漏斗分析、激励机制对照、单经济、定价对比 | M-Cagan-4Risks 商业可行性维 | 把渠道当变现机制 |
| **M6 Future Capability & Strategic Opportunity** | M-ODI（underserved >10）、M-Positioning（白地）、AI/硬件/数据能力映射、M-PreMortem | M-Disruption、M-Changelog（build-cost 下界）、M-OST（解空间）| 未来话题误用 Porter |

> **跨维度证据 / 方法的归位规则**：按"该证据 / 方法**支撑哪条 finding**"归位（finding 在哪维就归哪维），不按"看起来像哪维"归位。同一方法可跨维使用，但 profile 装配时必须给"在本能力本 finding 下的主用 / 支撑角色"。

---

## 6. 证据完整性 — 一等支柱（R2，所有能力共享）

**所有能力共享同一证据纪律**——PM DeepResearch 「可信度远超普通 LLM」承诺的核心。

### 6.1 来源可信度：4-tier 逻辑底座 + 展示标签

**摄入 / 校验逻辑用 4-tier**（Phase 1 A1）；**报告展示用标签**。映射：

| 4-tier（逻辑）| 定义 | 展示标签 | 使用方式 |
|---|---|---|---|
| Tier 1 | 同行评审论文 / 会议、权威数据库 | **High** | 可支撑事实性结论 |
| Tier 2 | 官方文档、**版本更新 / release notes**、一手工程博客（具名）、政府 / 机构报告、应用商店、上市公司财报 | **High** | 可支撑事实性结论 |
| Tier 3 | 可靠新闻、二手分析、可信评测、公开访谈、开发者博客（注日期、标二手）| **Medium** | 可支撑分析性判断 |
| Tier 3（社区子类）| 应用商店评论、社媒、论坛、问答 | **Low** | 只作用户情绪 / 线索 / 假设，**不写成事实** |
| Tier 4 | 无日期 / 匿名 / 无一手出处的 LLM 摘要 / 未录用投稿 | **Unknown** | **不进核心结论**，只进开放问题，flag 人工复核 |

> arxiv 来源须**确认录用状态**（非 "under review"/desk-rejected）才入 Tier 1（先验真伪纪律：本规格 21 份 arxiv 引用全部一手核实通过录用状态）。

### 6.2 视觉证据 `visual_evidence`（first-class）
涉及功能设计、体验路径、竞品对比、页面对比、AI 功能体验、能力域 teardown 的结论，**必须**输出 visual_evidence；无法获得图片 / 视频 URL **必须说明缺口**，且不得给强结论。

| 字段 | 说明 |
|---|---|
| product | 产品名称 |
| screen_or_flow | 页面 / 流程 / 功能名 |
| media_type | screenshot / video / app_store_image / official_page / social_post |
| source_url | 图片 / 页面 / 视频 URL |
| timestamp | 视频时间点（非视频可空）|
| observed_feature | 观察到的功能 / 交互 |
| related_claim | 支撑的结论 |
| confidence | high / medium / low |

> **获取路径（接口 [§Step 7](orchestration-interface.md)）**：纯 API 搜索常拿不到真实截图——Deep 模式下由 Layer 2 浏览器抓取（agent-browser / browser-use 走系统 Chrome）补截图 / teardown 帧；这是 **Skill 层外部步骤**（非 Lapis aspect agent 能力，aspect agent 只有 `search`），获取成本计入预算。
> **承载机制（重要）**：本表是 **Skill 后处理产物**。底层 Lapis `Evidence` 的 provenance 字段（url/summary/snippet…）保持 **byte-equal 不可改写**；agent 不得把 media_type/observed_feature 写进 `Evidence.summary`，而是写进**引用该证据的 `Finding.claim`** 标注块，Skill 据此装配本表（见 [接口 §3/§4](orchestration-interface.md)）。

### 6.3 逐声明 provenance + 多层核验
1. **原子声明核验（FActScore 范式）**：报告关键结论拆成原子声明，逐条对照其引用源核验。
2. **语句级引用审计（DeepTRACE 8 维）**：核到语句级（生产级深研系统引用准确率仅 40–80%，必须显式审计）。
3. **引用忠实性 ≠ 正确性（CiteEval）**：声明必须真能从被引源推出（高达 57% 引用是"事后合理化"）。

### 6.4 反幻觉机制（落地 3 个，优先级从高到低）
1. **citation-grounding + abstention（宁少但真）**：无可靠来源时**弃权 / 标"未找到"**，不编。PM DeepResearch 第一天就自我执行（本规格自身的 21 份 arxiv 引用先验真伪审计为示范）。
2. **verification chain（rubric 验证器）**：用"验证比生成简单"的不对称性，对产出做 rubric 引导自验证。
3. **有限 self-refine**：Gap 检测→补充→再检测，**有终止条件**（边际收益递减即停，过多迭代引噪）。

---

## 7. 报告模板族 + 裁剪规则

每 capability 选定**一种**报告模板族；模板族 = profile 装配产物，本通用规格定义"族 + 通用裁剪规则"。

### 7.1 模板族 A · 13 章（适用 competitive / product-capability / innovation-direction）

| 章 | 标题 | 共同语义 | profile-specific 加重 |
|---|---|---|---|
| 1 | 研究结论摘要 | 核心判断 + 推荐 + 置信度 + 最大不确定性 | — |
| 2 | 研究输入与边界 | decision_intent、目标、人群、排除范围 | — |
| 3 | 目标产品定位与现状 | 速写（Cagan 3 强 3 弱） | — |
| 4 | 用户人群与 JTBD | job statement、outcome | capability profile 加重 |
| 5 | 竞品与替代方案图谱 | 真实竞争集、定位图、威胁分级 | innovation 裁为白地图；capability 裁为 benchmark 段 |
| 6 | 功能架构与体验路径 | 能力对位、Kano、路径 + 断点 | capability profile **重点加重** |
| 7 | 视觉证据资产表 | §6.2 visual_evidence | capability profile 重点加重 |
| 8 | AI / 新能力映射 | 升级方向、新能力下注 | innovation profile **重点加重** |
| 9 | 产品机会矩阵 | ODI 打分 + Kano + 复杂度 | innovation 重点加重 |
| 10 | Roadmap 建议 | P0/P1/P2 + 依赖 + 验证条件 | innovation 重点加重（下注路径） |
| 11 | 验证实验与指标 | 指标定义 + 实验设计 | — |
| 12 | 风险、冲突与开放问题 | TM-8 pre-mortem + 低置信 / 冲突证据 | innovation 重点加重 |
| 13 | 附录：来源与搜索记录 | Evidence Table + Search Queries + Source List（含 tier/标签）| — |

### 7.2 模板族 B · 8 段 PR-FAQ（适用 product-requirements，B2 八段模板）

1. **Press Release Frame**（PR-FAQ 风格，≤300 字）——先讲价值再讲功能。
2. **机会验证**：JTBD statement + ODI 前 5 desired outcomes（Imp/Sat/Opp）+ Kano 分级 + Opportunity Landscape。
3. **风险评估**（Cagan 四大风险）：每项给证据等级 + 来源。
4. **解空间**（Torres OST）：每个目标机会 ≥3 候选方案 + 最危险假设清单 + 既有 / 竞争方案。
5. **需求**：功能需求（outcome 语句，标 Kano）+ 非功能需求 + 非目标。
6. **成功度量**：主指标（leading）+ 次指标 + 护栏指标。
7. **证据与来源**：一手 / 二手（只用真实 URL）+ 每条声明置信度。
8. **未决问题与下一步**。

### 7.3 通用裁剪规则
- **Quick**：报告族第一段 + 核心判断 + 来源（含标签）。
- **Standard**：报告族骨干章 / 段（profile 指定）+ 简化机会矩阵 / 解空间。
- **Deep**：全模板，profile 指定的"不得删"章 / 段强制保留。

### 7.4 关键模板

**机会矩阵**（13 章模板 Ch 9 / 8 段模板第 2 段）：

| 机会点 | 对应 JTBD | 用户价值 | 商业价值 | 复杂度 | 证据强度 | 风险 | 优先级 | 验证方式 |
|---|---|---|---|---|---|---|---|---|

> **「复杂度」列**优先用竞品迭代节奏（§4.4 M-Changelog）做外部代理估算——对手把该能力打磨稳定用了多少版 / 多长时间 ≈ 我方 build-cost 下界——而非纯团队臆测；代理估算标 TM-4。

**定位图**（13 章 Ch 5 / innovation 白地图）：buyer-validated 双轴 value curve + 白地标注。

**指标定义**（13 章 Ch 11 / 8 段第 6 段）：指标 / 定义 / 计算方式 / 数据来源 / 成功标准（激活率、功能使用率、留存、付费转化、任务完成率、路径流失率为常用集）。

### 7.5 行文规范（expert prose — 给人读的写法，所有能力 floor）

报告维度对，不代表能用。"易读"是北极星三值之一，最易被做成机械堆砌——**维度是骨架，行文才是产品**。本节把真实优质产品分析（Stratechery / Built for Mars / Lenny's / McKinsey 行动标题 / Minto Pyramid / Amazon 六页备忘录）的写法立为硬约束。

**必须（DO）：**
1. **结论先行（BLUF / SCQA）**：每章、每节、全篇都先给判断，再给依据。
2. **标题即论点（action title）**：章节标题写成**带结论的整句**，不是话题标签。**通读测试**：只读所有标题应能串成完整论证。
3. **论点段先行**：每段一个论点，topic sentence 打头，后面才是支撑。
4. **表格是论点下的证据，不是论证本身**：先用散文说清 "so what"，表格只做并列对照 / 取证。**禁止**用一连串表格替代论证。原始数据下沉到附录。
5. **按主题综合，不要逐对象流水账**：归纳为"模式 / 张力 / 白地"，而不是"竞品 A / B / C 各怎样"。
6. **给中心命名**：核心洞察起一个可被引用的名字（如"群体智能护城河"），逼出清晰主张。
7. **吸收反论**：在散文里主动提最强反驳并就地化解。
8. **校准的不确定性**：把**可能性**与**信心**分开明示；避免"可能也许或许"式空心对冲。
9. **收尾给行动**：结尾是具体建议 + 下一步动作，不是发现的复述。

**禁止（AVOID）：**把建议埋在结尾 / 附录；用话题式标题（"市场概览""SWOT"）；无论点的表格 / 数据堆砌；用"可能潜在或许"逃避判断；逐对象走流水账。

> 这条规范同时是 [rubric](../evaluation/rubric.md) 的整体可读性 floor：机械堆砌即使各维分够也判不通过。所有 capability profile 的 golden / 参考产出必须示范本规范。

---

## 8. Aspect Report Schema（通用扩展字段）

继承 Lapis 原方面报告字段（`aspect`/`findings`/`evidence`/`assumptions`/`risks`/`open_questions`/`confidence`）；通用扩展（capability 共享）：

```json
{
  "aspect": "<aspect-id>",
  "capability": "competitive|product-capability|innovation-direction|product-requirements",
  "mece6_dimensions": ["M2","M3"],
  "method": ["M-JTBD","M-Teardown"],
  "persona": "product-experience-analyst|product-strategist",
  "decision_intent": "enter|differentiate|build|improve|grow|ai-upgrade",
  "target_subject": { "name": "", "positioning": "", "scope": "" },

  "findings": [],
  "evidence": [
    { "title": "", "url": "", "source_type": "official|documentation|news|blog|forum|repository|unknown",
      "tier": 1, "display_label": "High", "retrieved_at": "YYYY-MM-DD",
      "summary": "", "related_claim": "", "epistemic_status": "evidenced|expert|assumption|speculation" }
  ],

  "visual_evidence": [ /* §6.2 字段 */ ],

  "gap_status": {
    "source_count_pass": true, "source_diversity_pass": true, "contradiction_resolved": true,
    "factual_grounding": false, "recency_pass": true, "visual_evidence_pass": false
  },

  "search_iterations": 2,
  "confidence": "low|medium|high"
}
```

**Capability-specific 字段**（由各 profile 定义；如竞品的 `capability_matrix`/`kano_grades`/`opportunity_scores`/`positioning`，需求的 `pr_faq`/`solution_space`/`requirements`/`metrics_tree`）见各 profile 文档。

> **本 §8 是 Skill 后处理 / 装配的视图，不是 Lapis 引擎输出 schema**（同竞品 v2.0 验证结论）。v2.0 里 aspect agent 实际只能产出 Lapis 合法的 `source_type` 7 枚举；扩展值是 **Skill 映射结果** + [接口 §6](orchestration-interface.md#6-引擎边界第一版不动引擎schema-扩展作为需求提给上游heye-2026-05-29-确认) 提给 4o3F 的**上游需求**，不可作为 aspect 直接输出。

---

## 9. Gap 检测清单 + Quality Floor（通用部分）

### 9.1 Gap 检测（Layer 1 收齐 aspect 后跨维度执行 — 通用项）

| 检测项 | 不合格条件 | 处理 |
|---|---|---|
| 目标定位 | 无官方 / 高可信来源 | 标假设 + 补搜 |
| 用户证据 | 只有推测，无评论 / 反馈 | 降置信 + 补用户证据 |
| 视觉证据 | 无截图 / 视频 / 页面 URL | 标缺口，**不得给强结论** |
| 时效性 | 市场 / 竞品数据 > 12 月 | 加日期过滤重搜 |
| 来源多样性 | 单一来源 / 单一类型 | 补另类型来源 |
| 矛盾未化解 | 来源间冲突未点名 | TM-12 言行分离点名 + 升级 / 降级置信 |

**capability-specific gap 检测**（如 competitive 的"竞品覆盖 <3"、requirements 的"解空间 <3 候选"）由各 profile 定义。

**迭代规则**：Standard ≤1 轮、Deep ≤2 轮 Gap 补充；2 轮后仍不满足→在"限制"章 / 段明确标注，不继续搜。

### 9.2 Quality Floor（Deep 模式通用最低门槛）

| 质量项 | 最低要求 |
|---|---|
| 目标基础资料 | ≥3 来源，优先 Tier 1/2 |
| 视觉证据 | ≥5 条（涉及功能 / 体验 / AI 时强制）|
| 用户证据 | ≥20 条评论 / 社媒摘要（Low 标签）；无法获得须说明缺口 |
| 置信度 | 每个关键结论标 high/medium/low + epistemic_status |
| 开放问题 | 证据不足 / 冲突 / 待验证假设单列 |
| 行文规范 | §7.5 DO 9 条全部满足；AVOID 全部不触犯 |

**capability-specific floor**（如 competitive 的"≥3 竞品 + 每格证据"、requirements 的"≥3 候选解 + 主 / 次 / 护栏指标三套齐"）由各 profile 定义。

不达标→报告标置信度警告或对应结论弃权（§6.4）。**Quality floor 同时是 [rubric](../evaluation/rubric.md) 的硬下限。**

---

## 10. 优雅降级

| 条件 | 行为 |
|---|---|
| Lapis MCP 正常 + 全 Provider 可用 | 全功能：多 Agent 并行 + 浏览器取视觉证据 |
| 部分 Provider 不可用 | 用可用 Provider，方法论章标覆盖限制 |
| Lapis MCP 不可用 | 退化为 **Claude-only**：直接调搜索 MCP，仍用本规格全套方法论 + 报告模板族 + 证据纪律 |
| 某 aspect 超时 / 失败 | 返回已完成 aspect，失败者标 gap，由 Layer 1 决定重试 / 跳过 |

Claude-only 不是失败——方法论增强（MECE-6 / 方法库 / 装配 / 证据完整性 / 模板族）是纯 prompt 能力，不依赖 Rust MCP。具体降级流程见 [`claude-only-degradation.md`](../../prompts/layer1/pm-deep-research/claude-only-degradation.md)。

---

## 11. Capability Profile 装配契约（每 profile 必给的 6 字段）

每 capability profile 文档**必须**给出以下 6 字段，作为通用 frame 的装配单：

| 字段 | 类型 | 说明 |
|---|---|---|
| 1. `decision_intent_affinity` | 列表 | 该 capability 亲和的 decision_intent 子集（见 §1.2）|
| 2. `mece6_emphasis` | `{primary, supporting, contextual}` | 该 capability 强调 / 支撑 / 语境化的 MECE-6 维度分组 |
| 3. `skeleton` | 列表 | 从 §4 方法库**组合**出的分析骨架（如竞品的五维、需求的八段）|
| 4. `report_template` | `{family, weighting, do_not_drop}` | 报告模板族选择（A=13章 / B=8段）+ 加重章 / 段 + 不可删章 / 段 |
| 5. `persona_tm_weighting` | `{ea_tm, strategist_tm}` | EA / Strategist 各重点 TM 列表 + 权重 |
| 6. `capability_specific` | `{aspect_fields, gap_checks, floor_items}` | capability-specific 字段 / gap 检测 / floor 项 |

外加文档头部强制标注（避免未验证 profile 被过度承诺）：

> ⚠ 本 profile 方法组合骨架已定；端到端 golden 验证状态：**已验证（v2.0）** / **延后到 vX.Y**。

各 profile 见 [`capabilities/`](capabilities/)。

---

## 12. 与 Lapis 引擎边界

**绝不变动 Lapis MCP 边界**：仍 `aspect_research` + `deep_research`，`schema_version="0.1"`，capability-specific 产品字段由 Skill 承载（不改 Rust）。

详见 [`orchestration-interface.md`](orchestration-interface.md) §1-§5（接口分步）+ §6（上游需求归集 — 由实跑验证驱动，不主动扩）。

> Phase 2′ **不**新增引擎需求；4 能力跑通后若发现共需的新字段，按接口 §6 统一整理提给 4o3F。

---

## 13. 与 Phase 1 决策对照（可追溯）

| 主题 | Phase 1 决策 | 本通用规格落地 |
|---|---|---|
| 维度框架 | 竞品=五维骨架（B1）；MECE-6=顶层（ADR-0006）| MECE-6（§2）= 跨 4 能力顶层；五维 / 八段 / capability 骨架 = profile 装配产物（§11）|
| 研究人格 | 2 核心 + 跨人格质量门（B3）| §3 通用 2 人格 + 13 TM + TM-4/11 质量门；capability 差异仅 TM 权重 |
| 可信度 | 4-tier 逻辑 + 展示标签（A1）| §6.1（4 能力共享）|
| 机会优先级 | ODI 完整 `Imp+max(0,Imp−Sat)` + Kano（B2 + ADR-0006）| §4 M-ODI / M-Kano + §7.4 机会矩阵模板 |
| 证据完整性 | 一等支柱（R2）| §6（4 能力共享）|
| 产品专家味 | 13 条 TM 注入人格（B3）| §3.1/3.2 + §11.5 各 profile TM 权重 |
| 报告模板 | 五维→13 章映射 + 需求 8 段 | §7 模板族 A（13 章）+ 模板族 B（8 段 PR-FAQ）|
| 引擎边界 | Lapis 由上游 4o3F 维护（Phase 2 §7）| §12 不动引擎 |

---

## 附录 · 历史与归档

- v2.0 竞品研究的验证产物（[Strava 黄金 23/24](evaluation/golden/competitive-strava-coach-upgrade.md)，R4-c canonical 2026-06-01）→ 通用规格的方法论承载来源。
- v2.1/v2.2/v2.3 黄金（Runna 23/24、AI Coach Bets 24/24、Endurance Biometric Coach PR-FAQ 24/24）见 [`evaluation/golden/`](evaluation/golden/) 索引。
