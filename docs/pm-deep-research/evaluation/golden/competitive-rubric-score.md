# R4-c 评分 · v2.0 Strava competitive — search-tuning + budget fix（纯引擎 golden）

> 对象：**纯引擎 6-aspect golden** `r4c-merged.result.json`（bundle=`r4c-engine-bundle.md`）。
> 引擎=上游 `9db7464` verbatim（binary current）；模型 gpt-5.5（CPA）/ 搜索 grok。
> 配置（重组 R4-c）：per-aspect `max_search_calls=4`（max_turns8/tool8）+ 全局 `search_policy.recency=fresh` + `max_results_per_query=5`（**无** depth=high_recall、**无** content_level=detailed、**无** category）。
> 锚点：rerun9 = **22/24**（A4=1 build-cost 仅 2 ev；C1=1 无 Layer-2 抓图）。
> 闸门：R1 引擎漂移——任一维跌破锚点即暂停。

## 总分：23 / 24（A4 1→2，C1 持平）✅ 纯引擎、R1 PASS、无回归

| # | 维度 | 锚点 | R4-c | 依据 |
|---|---|:--:|:--:|---|
| A1 | 引用充分性 | 2 | **2** | 34 findings 全挂 evidence_refs；**dangling refs = 0** |
| A2 | 引用准确性 | 2 | **2** | 抽样真实源（apps.apple.com 版本史、garmin.com、strava press、reddit/forum）；source_type 正确（official 36/blog 5/news 5/unknown 4/forum 1）|
| A3 | 无支撑率 | 2 | **2** | finding_type(fact/interpretation/recommendation/risk/assumption) + 【TM-x/估算/置信】前缀；ODI 全 `estimated:true`；裸断言极少 |
| A4 | **来源质量与多样性** | **1** | **2 ✅** | **关键提升、纯引擎路径验证**。51 evidence / 21 域 / 5 source_type；每 aspect ≥4 ev。**build-cost（锚点 A4=1 扣分点，当时 2 ev）现 4 ev、全 official App-Store 版本史页（Runna/Strava/Garmin/NRC，2026-04/05 fresh 日期）**——`recency=fresh` 在真引擎上拉到 dated changelog 命中，正是 §6.1 预测的机制 |
| A5 | 置信度校准+TM-4 | 2 | **2** | 全 aspect confidence=medium + 逐 finding epistemic + 逐 evidence confidence |
| B1 | 五维骨架覆盖 | 2 | **2** | 6 aspect 全 dedicated（job/能力矩阵/ODI/定位白地/体验路径/build-cost）|
| B2 | 真实竞争集 JTBD | 2 | **2** | job-and-competitive-set 6 findings 含非显性替代 |
| B3 | 能力矩阵带证据 | 2 | **2** | capability-and-importance 能力矩阵每 cell 内联 evidence_refs 或 assumption 标 |
| B4 | ODI/Kano 严谨 | 2 | **2** | opportunity-gaps ODI 排名表 + 公式 + 全 estimated 标 + 过服务降级项；Kano 在能力矩阵 |
| C1 | 视觉证据 | 1 | **1 =** | experience-paths 引 7 个 app-store/产品页 URL（含截图库）但**未抽取/校验为显式视觉证据项**，诚实记 4 open_questions → 结构限制持平锚点（deep_research 无 Layer-2 抓图），**非回归** |
| C2 | 专家思维动作 TM | 2 | **2** | TM-3/4/5/8(pre-mortem)/9/12(revealed strategy)/13 系统体现 |
| C3 | 可落地 | 2 | **2** | staged build 路径 + injury-aware adaptive 优先级 + 过服务降级 + roadmap |
| | **合计** | **22** | **23** | A4 1→2 |

## R1 闸门判定：**PASS**（无任一维跌破锚点；A4 1→2 提升）

## R4-c 结论（已坐实）

1. **A4 1→2 = 纯引擎+search-tuning 路径验证**（不再有混合执行缺口）：build-cost 经引擎 `recency=fresh`+`max_results=5` 路径产 **4 条 dated official 版本史证据**（锚点 2 ev）。证实 §6.1 的"recency=fresh 提 changelog 命中率→build-cost 回补来源数"假设。
2. **安全配置 = recency=fresh + max_results=5 + 稳 cap（4）**：
   - rerun1（cap=8 + depth=high_recall + content_level=detailed）被引擎 **hard-kill**（`agent_loop.rs:182` 搜索预算超限秒死、无优雅合成回退；`depth=high_recall` 的 `prompt_hint` 怂恿过搜 → 贪婪 aspect 搜 9 次撞 cap=8 死）。
   - rerun2/3（重组 cap=4 + recency=fresh + max_results=5）**零 budget_exceeded**，6 aspect 全收敛、51 ev（锚点 28）。
   - **教训**：在 hard-kill 引擎上 search-tuning **只能挑"不增搜索次数"的字段**（recency/max_results 安全；depth=high_recall 危险）。
3. **引擎无 per-aspect search 字段**（`workflow.rs:322` 全 aspect 共享同一全局 `search_policy`；`AspectSpec` 仅 `search_provider`）→ §5.1 文档"挂到单 aspect 的 `aspect.search_*`"**在 9db7464 不存在**（待修正）。`category` 是 exact-match、不能全局 → per-aspect 红利延后 WS-U（prompt-guided SearchToolArgs）。

## 过程产物
- 纯引擎结果：`r4c-merged.result.json` / bundle `r4c-engine-bundle.md`。
- 中间态：`r4c-run1-failed.*`（rerun1 过搜死）、`r4c-claude-aspects.md`（CPA 中断期 Claude stand-in，已被纯引擎取代）、`r4c-deep-golden.args.json`（重组配置）。
- harness：`build_r4c_args.py`（transform，零漂移）、`build_r4c_retry.py`（retry 抽取）。
