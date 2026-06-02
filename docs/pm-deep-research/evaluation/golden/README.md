# PM DeepResearch 黄金报告索引（4 能力）

> 4 个 capability 各 1 个端到端真引擎产出 + 1 个 rubric 自评。引擎：Lapis `9db7464`（除 v2.1 M5 baseline `02ec7d9` + 本地 SSE 补丁）。

| 能力 | 课题 | 黄金报告 | Rubric 自评 | 分数 | canonical 配置 |
|---|---|---|---|---|---|
| **competitive** (v2.0) | Strava AI-coach 升级方向 | [competitive-strava-coach-upgrade.md](competitive-strava-coach-upgrade.md) | [competitive-rubric-score.md](competitive-rubric-score.md) | **23/24** | R4-c canonical：`recency=fresh` + `max_results=5` + per-aspect cap=4；A4 1→2 |
| **product-capability** (v2.1) | Runna 训练计划自适应能力深度 | [product-capability-runna-training-plan.md](product-capability-runna-training-plan.md) | [product-capability-rubric-score.md](product-capability-rubric-score.md) | **23/24** | R4-c canonical：`recency=fresh` + `max_results=5` + per-aspect cap=3；锚点 23 持平 + enrichment |
| **innovation-direction** (v2.2) | 海外运动/健身 AI Coach 未来 12-36 月下注 | [innovation-direction-ai-coach-bets.md](innovation-direction-ai-coach-bets.md) | [innovation-direction-rubric-score.md](innovation-direction-rubric-score.md) | **24/24** | R4-c canonical（cap=6 必须）：`recency=fresh` + `max_results=5` + per-aspect cap=6（recommended-bets 综合 aspect 在 recency=fresh 下 search appetite ~6） |
| **product-requirements** (v2.3) | Endurance-athlete Explainable Biometric Coach PR-FAQ | [product-requirements-prfaq.md](product-requirements-prfaq.md) | [product-requirements-rubric-score.md](product-requirements-rubric-score.md) | **24/24** | R4-e + #9 rerun9 锚点；R4-c **NOT canonical**（requirements-fn-nfn-nongoals 在 recency=fresh 下结构性 synthesis-time fragile，4 retry 模式齐失败；等引擎支持 per-aspect search_policy override 或 per-aspect execution_policy.timeout_ms 后回头） |

---

## 命名约定

- `<capability>-<topic-slug>.md` — 黄金报告本体
- `<capability>-rubric-score.md` — 该报告的 rubric 自评（12 维 / 24 分；详 `../rubric.md`）

## 评分锚点解释

- v2.0 `competitive` 22/24（手写黄金）→ M4 引擎实跑 22/24 → **R4-c canonical 23/24**（A4 1→2，build-cost 经 `recency=fresh` 路径产 4 条 dated 版本史证据；C1 持平 1，deep_research 无 Layer-2 抓图结构限制）。
- v2.1 `product-capability` M5 23/24 → **R4-c canonical 23/24 持平**（无回归 + 53 evidence/15 domains/6 source_types/0 dangling，enrichment +5 ev/+2 domain）。
- v2.2 `innovation-direction` M6 24/24（首次 C1 满分；7 战略图 trend/canvas/capability map）→ **R4-c canonical (cap=6) 24/24 持平**（74 evidence/23 domains/5 source_types）；**key finding**：recommended-bets aspect 在 recency=fresh prompt-hint 下 search appetite ~6，cap=5 持续 hard-kill，cap=6 为强制门槛。
- v2.3 `product-requirements` M7 21/24（baseline）→ R4-g family B 视觉 C1 1→2 = 22 → **R4-e 段3 cagan 拆 4 micro-aspect + #9 引擎复跑 24/24**（B1 1→2 + B3 1→2）。R4-c **NOT adopted**：family B PR-FAQ 合成强度比 v2.2 recommended-bets 高一档，requirements-fn-nfn-nongoals aspect 在 recency=fresh 下结构性 synthesis-time fragile（4 retry 模式齐：cap=8 search hard-kill / cap=9 runtime / cap=9 + budget.timeout_ms=900s 撞 execution_policy=600s 复校 / cap=9 + 双侧 900s 撞 CPA SSE flake）。

## R1 引擎漂移闸门

4 黄金 #9 引擎全量复跑全 PASS、零回归（v2.0 22 / v2.1 23 / v2.2 24 / v2.3 24）。

## 参见

- 通用规格：[`../../pm-deep-research-spec.md`](../../pm-deep-research-spec.md)
- Rubric：[`../rubric.md`](../rubric.md)
- 4 能力 profile：[`../../capabilities/`](../../capabilities/)
- 引擎接口边界：[`../../orchestration-interface.md`](../../orchestration-interface.md)
- 搜索参数策略（上游 #7 已落地）：[`../../search-parameter-strategy.md`](../../search-parameter-strategy.md)
