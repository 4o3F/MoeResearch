# PM DeepResearch — 评测 Rubric（通用 + 能力 profile delta）

> Status: Phase 2 WS2 产出（2026-05-29 签收）；Phase 2′ G3 泛化（2026-05-30）：12 维顶层 + §6 capability profile delta。
> 目的：把「可信度远超普通 LLM」变成**可打分、可证伪**的标尺。通用规格 [`../specs/pm-deep-research-spec.md`](../specs/pm-deep-research-spec.md)（4 能力共享）+ 各 [`../capabilities/`](../capabilities/) profile 须通过本 rubric（自评 = Phase 2 退出标准之一）；黄金样例 [`golden/`](golden/) 是 rubric 的参考锚点。
> 借鉴：**DeepTRACE 8 维**（arXiv 2509.04499）、**ResearchRubrics**（2511.07685）、**FActScore**（2305.14251）、**CiteEval**；方法论严谨维（B 组）来自本项目 Track B。
> **回归承诺**：本次泛化对 A 组 / C 组评分判据**完全不变**；B 组判据从竞品-specific 词面**泛化为 profile-agnostic 词面**，且 v2.0 竞品报告在新 rubric 下仍判 22/24（B1/B3 实例化项 = competitive profile §1/§3，与原 rubric 词面 1:1 等价；见 §6.1）。

---

## 0. 怎么用

- 每个被评对象（任一能力的报告 / 规格自评）按下列 **3 组 × 共 12 维**打分。
- 每维 **0 / 1 / 2** 分（0=缺失或错误，1=部分达标，2=完全达标）。
- **硬门槛（floor）**：标 ⛔ 的维度**必须 ≥1**，任一为 0 即整体不通过（不论总分）。这些对应通用规格 §9.2 + 各 profile §3.2 quality floor。
- **行文 floor（expert prose）**：报告须满足通用规格 §7.5 行文规范（Phase 2′ 抽取前位于原 canonical §7.4）——论点先行、标题即论点、表格作证据而非论证本身、按主题综合。**机械堆砌表格 / 无论点流水账即使各维分够也判不通过**（"易读"是北极星三值之一，不是可选项）。
- 通过线：**floor 全过 且 总分 ≥ 18/24（75%）**；黄金样例参考产出应 ≥ 22/24。
- 每维同时给出**普通 LLM 基线**——这是"远超"要拉开的差距，也是可证伪点：若 PM DeepResearch 产出与基线无差，则证伪了价值主张。
- **能力 profile 评分**：B 组判据按 §6 的 profile-specific 实例化项打分（B1 = 该 profile 的 skeleton 覆盖度；B3 = 该 profile 装配契约的核心证据机制达标度）。其余 10 维（A 组 + B2/B4 + C 组）对所有能力**通用**。

---

## 1. A 组 · 可信度与证据（DeepTRACE 映射，5 维）

| # | 维度 | 0 / 1 / 2 判据 | floor | 普通 LLM 基线 |
|---|---|---|---|---|
| A1 | **引用充分性**（citation sufficiency）| 关键声明被引用比例：0=<50% / 1=50–85% / 2=>85% | ⛔ | 多数论断无引用 |
| A2 | **引用准确性/忠实性**（CiteEval）| 抽样核验"声明能否从被引源推出"：0=≤60% / 1=60–85% / 2=>85% | ⛔ | 引用存在但常"事后合理化"，链接打不开/不支持 |
| A3 | **无支撑率**（unsupported rate）| 无任何支撑的事实性声明占比：0=>20% / 1=5–20% / 2=<5% | ⛔ | 大量未支撑断言 |
| A4 | **来源质量与多样性**（source quality）| 每维 ≥3 来源且 tier 分布合理（非全 Low/Unknown）：0=不足 / 1=数量够但质量偏低 / 2=数量+tier 都达标 | ⛔ | 来源少、偏单一搜索引擎、tier 不标 |
| A5 | **置信度校准 + 认识论标注**（confidence, TM-4）| 关键结论是否标 high/med/low + epistemic_status（实证/专家/假设/推测）：0=不标 / 1=部分 / 2=全标且校准合理 | ⛔ | 把推测写成事实，置信度一律"确定" |

---

## 2. B 组 · 方法论严谨（4 维 — B1/B3 按 profile 实例化，见 §6）

| # | 维度 | 0 / 1 / 2 判据（通用词面）| floor | 普通 LLM 基线 |
|---|---|---|---|---|
| B1 | **Skeleton 覆盖**（profile 主用骨架全段覆盖）| 该 profile 的 skeleton 段（§6 给定）全覆盖：0=≤⅓ 段 / 1=⅓–⅔ 段 / 2=全段覆盖且各段有方法落点 | ⛔ | 无方法骨架，泛泛信息罗列 |
| B2 | **JTBD 真实研究单元**（job-centered framing）| 按 job 而非品类/标签锁定研究对象（competitive=真实竞争集 / capability=能力域服务的 jobs / innovation=未满足 outcomes / requirements=PR-FAQ 客户 job）：0=无 job 框定 / 1=有但浅 / 2=有 job statement + 单元选择理由 | | "竞品=同类 App"/"需求=功能列表"，错失真实研究单元 |
| B3 | **核心证据机制达标**（profile 装配契约的主证据要求）| 该 profile 的核心证据机制（§6 给定，如 competitive=teardown 每格证据 / capability=断点 visual+多源 / innovation=未来能力 Tier1-2 依据 / requirements=4 风险 4 类全 + 解空间 ≥3 候选 + 三套指标）：0=机制缺 / 1=机制有但局部漏 / 2=机制达标且每项可追溯 | ⛔ | 给结论无机制证据，凭印象 |
| B4 | **机会量化严谨（ODI/Kano）**| ODI 用完整公式 `Imp+max(0,Imp−Sat)` + Kano 类型叠加 + 估算标 TM-4：0=无量化或公式错 / 1=有打分但未标估算/无 Kano / 2=公式正确+Kano+估算标注 | | 拍脑袋排优先级，或 RICE 式猜测 |

---

## 3. C 组 · 产品味与可落地（3 维）

| # | 维度 | 0 / 1 / 2 判据 | floor | 普通 LLM 基线 |
|---|---|---|---|---|
| C1 | **视觉证据**（visual_evidence）| 涉及功能/体验/对比的结论附截图/视频 URL：0=无 / 1=有但<5 条或多处缺口未标 / 2=≥5 条且缺口显式标注。**family B PR-FAQ 报告适配（R4-g）**：模板天然出 table 不出 chart，≥5 张语义 table（ODI/OST/FR/NFR/metrics/OQ/自验证）等价 chart 类型计入视觉证据，同按 0/1/2 判（详 §6.4）| ⛔ | 纯文字描述界面，无任何视觉证据 |
| C2 | **产品专家思维动作**（TM）| 体现 TM-1 Job→Feature→Gap、TM-11 可证伪（最强反论+证伪条件）、TM-5 显性权衡等：0=无 / 1=零星 / 2=系统体现关键 TM | | 罗列信息，无"功能必要性/权衡/反论"推理 |
| C3 | **可落地**（actionability）| 机会矩阵（价值/**复杂度【build-cost 优先用竞品迭代节奏佐证】**/证据/优先级）+ Roadmap + 验证实验与指标：0=只有描述 / 1=有机会但无验证/指标 / 2=机会矩阵+Roadmap+可验证指标齐全 | ⛔ | "建议做 AI 教练"式空泛结论，无优先级/验证 |

---

## 4. 总分与判定

| 项 | 要求 |
|---|---|
| Floor（⛔ 维度）| A1–A5、B1、B3、C1、C3 **全部 ≥1**（B1/B3 按 §6 该 profile 的实例化判据）|
| 行文 floor | 满足通用规格 §7.5 行文规范（expert prose，Phase 2′ 抽取前位于原 canonical §7.4）；机械堆砌表格 / 无论点流水账判不通过 |
| 总分 | ≥ 18 / 24 通过；黄金参考产出目标 ≥ 22 / 24 |
| 一票否决 | 出现**捏造来源/案例/统计**（先验真伪失败）→ 直接判 0，整体不通过 |

> **可证伪性声明**：把同一课题交给"普通 LLM 单次作答"，按本 rubric 打分应显著落在通过线下（预期 floor 多项为 0：无视觉证据、无 tier、无 ODI、捏造案例）。PM DeepResearch 产出若不能在 A 组（可信度）+ C1（视觉）+ B3（证据矩阵）上系统性拉开差距，则**价值主张被证伪**——这正是黄金样例要验证的。

## 5. 规格自评（Phase 2 / Phase 2′ 退出用）
规格本身不是一份报告，故按"规格是否**强制**了上述每一维"自评：每维问"规格有没有把它写成硬要求 + 给了落地机制？"全部为"是"即规格通过自评。WS3 黄金样例则用完整 12 维实测一次。

---

## 6. Capability Profile Delta（B1 / B3 实例化 + capability-specific floor 追加项）

B 组的 B1（skeleton 覆盖）+ B3（核心证据机制）按各 profile 实例化；其余 10 维（A 组 + B2/B4 + C 组）对所有 capability 通用。

### 6.1 competitive（v2.0 已验证）

| 维 | 实例化判据 |
|---|---|
| **B1 skeleton 覆盖**（**5 段**）| Job&真实竞争集 / 能力对位矩阵 / Kano / ODI 缺口 / 定位白地（[profile §1](../capabilities/competitive.md#1-b1-五维骨架报告主干)）：0=≤2 维 / 1=3–4 维 / 2=全 5 维且各有方法落点 |
| **B3 核心证据机制** | **能力对位矩阵带证据**（teardown 每格）：0=无矩阵或无证据 / 1=有矩阵但部分格无证据 / 2=每格有证据或标假设 |
| **capability-specific floor 追加** | competitive profile [§3.2](../capabilities/competitive.md#32-competitive-quality-floordeep-模式追加项)：竞品 ≥3（直接 / 间接 / 替代）；能力对位矩阵每格证据；机会矩阵 ≥5 |

> **回归承诺**：competitive 的 B1/B3 实例化判据**字面等价于原 rubric §2 的 B1/B3**——只是把 hardcoded 词面外移到 profile 文档。同一 [黄金样例](golden/competitive-strava-coach-upgrade.md)（R4-c canonical 23/24）在新 rubric 下判分等价。

### 6.2 product-capability（v2.1，✅ M5 端到端 23/24）

| 维 | 实例化判据 |
|---|---|
| **B1 skeleton 覆盖**（**6 段**）| 能力域 + JTBD / 单域 teardown 深度 / 体验路径 + 断点 / Kano 域内 / ODI 域内 / benchmark + build-cost（[profile §2](../capabilities/product-capability.md#2-6-段骨架报告主干)）：0=≤2 段 / 1=3–4 段 / 2=全 6 段 |
| **B3 核心证据机制** | **体验路径 + 断点 visual_evidence + 用户多源**（每断点 ≥1 截图 + ≥3 条同模式 Tier-3 评论或标假设）：0=路径缺 / 1=路径有但断点 visual 漏 / 2=路径 + 断点 visual + 多源齐 |
| **capability-specific floor 追加** | profile [§3.2](../capabilities/product-capability.md#32-quality-floordeep-模式追加项)：体验路径图 ≥1；断点 ≥3 + 每断点 visual + ≥3 条用户证据；benchmark 2-3 best-in-class；能力域 outcome ≥3 跑过 ODI |

### 6.3 innovation-direction（v2.2，✅ M6 端到端 24/24 满分）

| 维 | 实例化判据 |
|---|---|
| **B1 skeleton 覆盖**（**8 段**）| 趋势扫 / 未满足 outcomes / 白地 canvas / 未来能力映射 / 颠覆威胁 + 可防御性 / pre-mortem / build-cost / 推荐下注 + 验证条件（[profile §2](../capabilities/innovation-direction.md#2-8-段骨架报告主干)）：0=≤3 段 / 1=4–6 段 / 2=全 8 段 |
| **B3 核心证据机制** | **未来能力依据（Tier 1/2）+ 颠覆判定逻辑 + pre-mortem 机制 + TM-11 falsifiability hard gate**：0=纯臆测 / 1=部分有依据 / 2=未来能力 Tier 1/2 全 + 颠覆 Christensen 判定全 + pre-mortem 三死因带机制 + 推荐下注 TM-11 覆盖率 100% |
| **capability-specific floor 追加** | profile [§3.2](../capabilities/innovation-direction.md#32-quality-floordeep-模式追加项)：趋势 ≥3 条带时间窗；underserved outcome ≥3；未来能力候选 ≥2；pre-mortem 三死因（机制 + 触发条件 + 早期信号 + 止损动作）；推荐下注 1-3 个 + 每个附 4 风险 + 验证条件 + 显性权衡；**每下注方向强制附"什么条件下错"（TM-11 hard gate；覆盖率 = 100%，非 ≥80%）** |
| **M6 实测锚点** | [v2.2 M6 实跑 24/24](golden/innovation-direction-rubric-score.md)（海外运动/健身 AI Coach 赛道未来 12-36 月下注）：B1=2（全 8 段）/ B3=2（5 未来能力候选 + 5 colliding 颠覆威胁 + 3 死因 Tiger/Elephant/Paper Tiger + 3/3 推荐下注 TM-11 100%）/ **C1=2 首次满分**（7 战略图 ≥5，类型偏 trend/canvas/capability map/press/docs 非 in-app screenshot，**innovation-direction 报告类型自身天然不依赖 Layer-2 capture**）|

### 6.4 product-requirements（v2.3，✅ R4-e 后 24/24：M7 21 → R4-g 22 → R4-e + #9 引擎复跑 24 / family B 8 段 PR-FAQ 模板首落地）

| 维 | 实例化判据 |
|---|---|
| **B1 skeleton 覆盖**（**B2 八段模板**）| PR-FAQ / 机会验证 / 四风险 / OST 解空间 / 需求 / 成功度量 / 证据来源 / 未决问题（[profile §2](../capabilities/product-requirements.md#2-8-段骨架b2-模板报告主干)）：0=≤3 段 / 1=4–6 段 / 2=全 8 段。**严格判定**：每段需有 dedicated aspect output 才算"方法落点"；跨段代偿（profile §3.1 fallback）仅算结构覆盖不算 method-level 落点 |
| **B3 核心证据机制** | **四风险 4 类全 + 解空间 ≥3 候选 + 三套指标（主 / 次 / 护栏）全 + 每未决问题 TM-11 falsification**：0=四机制缺多 / 1=四机制有但局部漏或 1 个跨段代偿 / 2=四机制齐全且每项 dedicated aspect 输出 |
| **capability-specific floor 追加** | profile [§3.2](../capabilities/product-requirements.md#32-quality-floordeep-模式追加项)：ODI outcomes ≥5；四风险 4 类全；每 outcome ≥3 候选 + 最危险假设 ≥1；显式非目标；三套指标 5 字段全；**每未决问题强制附"靠什么会决"（TM-11 hard gate；覆盖率 = 100%，非 ≥80%）** |
| **报告模板说明** | 本 profile 用**模板族 B 8 段 PR-FAQ**（非 13 章）；C3 actionability 的"机会矩阵 + Roadmap + 验证指标"在 8 段模板里分别对应"机会验证 + 需求段 + 未决问题与下一步"段——评分时按本 profile 的段对应判 |
| **C1 视觉 family B 适配** *（R4-g 补丁已应用 2026-05-30）* | family B PR-FAQ 模板天然出 table 不出 chart。**正式判据**：≥5 张语义 table（ODI/OST/FR/NFR/metrics/OQ/自验证）等价 chart 类型，计入 C1 视觉证据，按是否 ≥5 判 0/1/2（与 §3 C1 通用判据对齐）。M7 据此 C1 1→2 |
| **M7 实测锚点** | v2.3 M7 实跑 21/24 原始 → R4-g 补丁后 22/24 paper（Endurance-athlete Explainable Biometric Coach PR-FAQ；新创 build intent）：B1=1（6/8 dedicated aspect + 2 跨段代偿 段3+段7）/ B3=1（OST + 三套指标 + TM-11 全过；段3 4-risks 跨段代偿 confidence medium）/ **C1=2（R4-g 补丁后：6 张语义 table ≥5；补丁前字面 chart-centric 判 1）** / **TM-11 7/7=100% ✅** / 5 hard gates 中 4 pass + 1 partial。**B1/B3 R4-e 已坐实（2026-05-31）**：段3 cagan 拆 4 micro-aspect 后各 +1 → M7.5 预演 + #9 引擎全量复跑 **24/24**（[打分](golden/product-requirements-rubric-score.md)）。**段3 cagan-4risks aspect search-saturation pathology**（历史）= strategist persona 在 multi-class 4-risks 任务上无法在 18-call budget 内收敛（5 retries 全失败）→ **R4-e 已修复**: 拆为 4 single-class micro-aspect（每 max_search=4），#9 复跑 cagan 4/4 dedicated。**family B 8 段 PR-FAQ 模板首落地证明 ✅**：与 v2.0/v2.1/v2.2 family A 13 章并列，通用规格 §7.1 模板族 B 实测可承载 |

## 锚点与待办
- ✅ WS3 黄金样例（[`competitive-strava-coach-upgrade.md`](golden/competitive-strava-coach-upgrade.md)，Strava / AI 升级方向）实测自评 **22/24**（floor 全过；行文 floor 过——已按 §7.4 改写为论点先行的专家叙事；ODI 已数值化）。**交叉审计（Codex+subagent）后诚实降分**：B3=1（能力矩阵符号化、无 per-cell 证据 id）、C1=1（视觉 URL 指针非实图）。此即"专家级参考产出"的评分锚点。
- ✅ **Phase 3 M4 引擎实跑验证（2026-05-30）**：同一课题用 Lapis deep_research 真引擎 + 人格 prompt 端到端产出（[`competitive-strava-coach-upgrade.md`](golden/competitive-strava-coach-upgrade.md) 报告，[`competitive-rubric-score.md`](golden/competitive-rubric-score.md) 打分）= **22/24**，与手写样例同分但 **B3 1→2（提升）**——能力矩阵每格带 `evidence_refs`/`assumption`/`falsifiable_test`，证明 per-cell 证据可由 prompt 强制承载、不必人手补。扣分仅 A4=1（build-cost aspect 仅 2 证据，`max_search_calls=2` 预算所致）+ C1=1（视觉 3<5）；均非方法缺陷。**价值主张未被证伪：引擎产出在 A 组/B3 上系统性远超普通 LLM 基线。** **→ A4=1 已由 R4-c 修复（见下 ✅ R4-c 锚点；v2.0 锚点 22→23）。**
- [ ] Phase 3 把 floor（含行文 floor）做成 skill 内的自动 quality-gate（verification chain）。
- 改进点进度：B3 ✅（引擎已 per-cell 证据）；竞品版本时间线 ✅；**A4 ✅ M5 已解**（v2.1 product-capability 实跑 A4=2）**+ v2.0 ✅ R4-c 已解**（2026-06-01，`recency=fresh`+`max_results=5` → build-cost 2→4 ev，v2.0 22→23）；**C1 ✅ M6 首次满分**（v2.2 innovation-direction 报告视觉类型偏 trend/canvas/capability map/press/docs，Tier 1 official 来源天然 ≥7 张达 ≥5 阈值，**不依赖 Layer-2 in-app capture**——证明 C1 历史 1 不是引擎/profile 装配缺陷，而是 v2.0/v2.1 capability 类型偏 in-app screenshot 的特性使然。v2.0/v2.1 的 C1 仍待 Layer-2 浏览器抓 in-app 录屏才能升 C1=2，但本待办不延伸到 v2.2/v2.3）；**C1 v2.3 family B ✅ R4-g rubric 补丁已应用（2026-05-30）**（family B PR-FAQ ≥5 张语义 table = chart 等价 → M7 C1 1→2，paper 22/24）。
- ✅ **Phase 3 M7 引擎实跑验证（2026-05-30，v2.3 product-requirements）**：Endurance-athlete Explainable Biometric Coach PR-FAQ 课题，**21/24 诚实自评**（baseline，后由 R4-e/g 升至 24/24，[最新打分](golden/product-requirements-rubric-score.md)）。**family B 8 段 PR-FAQ 模板首落地 ✅**（通用规格 §7.1 模板族 B 实测可承载证明，与 family A 13 章并列）；TM-11 hard gate 7/7=100% ✅（含工程过程 OQ-NEW 自报）；5 hard gates 中 4 pass + 1 partial（段3 cagan-4risks aspect 5 次 backfill 全失败 → 跨段代偿 confidence medium）。**关键 v2.3 finding (3 处诚实失分根因 + 修复路径)**：(a) **B1 -1 / B3 -1**: 段3 cagan-4risks aspect search-saturation pathology — strategist persona 在 multi-class 4-risks 任务上无法在 18-call budget 内收敛 → **Phase 4 R4-e ✅（2026-05-31，已验证）**: 拆为 4 micro-aspect (1 risk class / aspect) → M7.5 预演 + #9 引擎全量复跑 **21→24**（B1/B3 各 +1，叠加 R4-g C1）；(b) 段7 evidence-table aspect schema_validation_failed → **Phase 4 R4-f**: 段7 in task-decomposition 标 OPTIONAL，默认 fallback to final-report Phase B 跨段聚合；(c) **C1 -1 → ✅ R4-g 已修（2026-05-30）**: rubric C1 chart-centric vs family B 模板 table-centric 错位 → rubric §6.4 + §3 (C1 描述) 已加 family B 适配段；M7 C1 1→2，paper 21→22。**4 profile 全 ✅ + evaluation 框架闭环 = Phase 3 完成最后一块**。
- ✅ **R4-c v2.0 search-tuning 验证（2026-06-01，v2.0 锚点 22→23，A4 1→2）**：纯引擎 6-aspect golden（[score](golden/competitive-rubric-score.md)），引擎 `9db7464` + canonical 配置（`recency=fresh` + `max_results_per_query=5` + per-aspect `max_search_calls=4`）。**A4 1→2 坐实**：build-cost aspect 经引擎 `recency=fresh` 路径产 **4 条 dated official App-Store 版本史证据**（锚点仅 2）；全 golden 51 evidence / 21 域。C1 持平 1（deep_research 无 Layer-2 抓图，结构限制）。**关键工程教训**：引擎对 search-budget 超限是 hard-kill（`agent_loop.rs`，无优雅合成回退）→ search-tuning 只挑「不增搜索次数」字段（`recency`/`max_results` 安全；`depth=high_recall` 怂恿过搜被秒死，rerun1 cap=8 验证）。引擎无 per-aspect search 字段（详 [interface §5.1](../orchestration-interface.md#51-上游-search-tuning-字段e04398d5--7r4-d-回补)）。
- ✅ **R4-e + #9 引擎全量复跑验证（2026-05-31，v2.3 24/24）**：#9 引擎（`9db7464`）全量复跑 Endurance-athlete Coach PR-FAQ，段3 cagan 拆 4 micro-aspect（value/usability/feasibility/business）→ **dedicated 4-risks**（M7.5 预演 4/4 + #9 复跑 cagan 4/4 双坐实），B1 1→2 + B3 1→2（叠加 R4-g C1）= **21→24**（[最终报告](golden/product-requirements-prfaq.md) / [打分](golden/product-requirements-rubric-score.md)）。**4 黄金 #9 复跑全 PASS 零回归**（v2.0 22 / v2.1 23 / v2.2 24 / v2.3 24，R1 引擎漂移闸门通过）；honest low markers（value WTP / business econ）保留不注水。
- ✅ **Phase 3 M6 引擎实跑验证（2026-05-30，v2.2 innovation-direction）**：海外运动/健身 AI Coach 赛道未来 12-36 月下注课题，**24/24 满分**（[score](golden/innovation-direction-rubric-score.md)）。TM-11 hard gate 3/3 推荐下注全过（100% 覆盖率，profile §3.2 核心 floor 首次达成）；C1 视觉首次满分（7 战略图，类型偏 trend/canvas/capability map）；段6 pre-mortem 三死因实测 Tiger/Elephant/Paper Tiger Christensen 类型 + 机制 + 触发条件 + 止损动作完整。**v2.2 innovation-direction profile 装配契约（8 段 Strategist-heavy + 段2 sole-EA + family A 13 章 Ch 8/9/10/12 加重 + Ch 5/6 裁切）端到端验证成功**。
- **v2.1 M5 锚点（2026-05-30）**：[`product-capability-runna-training-plan.md`](golden/product-capability-runna-training-plan.md) = **23/24**（10 维持平 M4 + A4 1→2 + C1 持平）；6 aspect 真引擎产出 + 段3 体验路径 + 3 断点 + 4 来源同模式用户证据/BP；段5 strategist persona-ownership note + EA 数据通过 `prior_sources` 注入实测可行。**承载性证明**：v2.1 product-capability profile 装配契约（6 段 EA-heavy + family A 13 章 Ch6/7/4 加重 + Ch5 benchmark 段 + §3.2 floor）可承载，**不需 EA-deep 变体**。
