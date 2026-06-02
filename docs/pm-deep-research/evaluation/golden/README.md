# PM DeepResearch 黄金报告索引（4 能力）

> 4 个 capability 各 1 个端到端真引擎产出 + 1 个 rubric 自评。引擎：Lapis `9db7464`。

| 能力 | 课题 | 黄金报告 | Rubric 自评 | 分数 | 已验证配置 |
|---|---|---|---|---|---|
| **competitive** | Strava AI-coach 升级方向 | [competitive-strava-coach-upgrade.md](competitive-strava-coach-upgrade.md) | [competitive-rubric-score.md](competitive-rubric-score.md) | **23/24** | `recency=fresh` + `max_results=5` + per-aspect `max_search_calls=4` |
| **product-capability** | Runna 训练计划自适应能力深度 | [product-capability-runna-training-plan.md](product-capability-runna-training-plan.md) | [product-capability-rubric-score.md](product-capability-rubric-score.md) | **23/24** | `recency=fresh` + `max_results=5` + per-aspect `max_search_calls=3` |
| **innovation-direction** | 海外运动/健身 AI Coach 未来 12-36 月下注 | [innovation-direction-ai-coach-bets.md](innovation-direction-ai-coach-bets.md) | [innovation-direction-rubric-score.md](innovation-direction-rubric-score.md) | **24/24** | `recency=fresh` + `max_results=5` + per-aspect `max_search_calls=6`（recommended-bets aspect 在 recency=fresh 下 search appetite ~6；max_search_calls=5 持续触发执行中止） |
| **product-requirements** | Endurance-athlete Explainable Biometric Coach PR-FAQ | [product-requirements-prfaq.md](product-requirements-prfaq.md) | [product-requirements-rubric-score.md](product-requirements-rubric-score.md) | **24/24** | 段3 cagan 拆 4 single-class micro-aspect（每 max_search_calls=4）；`recency=fresh` **不推荐**（requirements-fn-nfn-nongoals aspect 在 recency=fresh 下结构性 synthesis-time fragile，多次 retry 均失败；等引擎支持 per-aspect search_policy override 后回头） |

---

## 命名约定

- `<capability>-<topic-slug>.md` — 黄金报告本体
- `<capability>-rubric-score.md` — 该报告的 rubric 自评（12 维 / 24 分；详 `../rubric.md`）

## 评分锚点解释

- `competitive` 23/24：build-cost aspect 经 `recency=fresh` 路径产 4 条 dated official 版本史证据（默认 2 条）；A4=2；C1=1（deep_research 无 Layer-2 抓图，结构限制）。
- `product-capability` 23/24：`recency=fresh` 无回归；53 evidence/15 domains/6 source_types/0 dangling；A4=2；C1=1。
- `innovation-direction` 24/24：C1 首次满分（7 战略图，视觉类型偏 trend/canvas/capability map，不依赖 Layer-2 in-app capture）；key finding：recommended-bets aspect 在 recency=fresh 下 search appetite ~6，`max_search_calls=5` 持续触发执行中止，`max_search_calls=6` 为强制门槛。
- `product-requirements` 24/24：段3 cagan 拆 4 single-class micro-aspect，解决 multi-class 任务 search-saturation 问题（B1=2 + B3=2）；C1=2（6 张语义 table ≥5）；8-section PR-FAQ template 首落地证明实测可承载。`recency=fresh` 不推荐（requirements-fn-nfn-nongoals synthesis-time fragile，多次 retry 均失败）。

## 引擎漂移闸门

4 黄金引擎（Lapis `9db7464`）全量复跑全 PASS、零回归（competitive 22 / product-capability 23 / innovation-direction 24 / product-requirements 24）。

## 参见

- 通用规格：[`../../pm-deep-research-spec.md`](../../pm-deep-research-spec.md)
- Rubric：[`../rubric.md`](../rubric.md)
- 4 能力 profile：[`../../capabilities/`](../../capabilities/)
- 引擎接口边界：[`../../orchestration-interface.md`](../../orchestration-interface.md)
- 搜索参数策略（上游 #7 已落地）：[`../../search-parameter-strategy.md`](../../search-parameter-strategy.md)
