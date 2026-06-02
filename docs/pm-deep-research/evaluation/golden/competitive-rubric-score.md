# 评分 · competitive Strava golden — search-tuning（纯引擎 6/6）

> 对象：**纯引擎 6/6 golden** `r4c-merged.result.json`（bundle=`r4c-engine-bundle.md`）。
> 引擎=上游 `9db7464` verbatim；模型 gpt-5.5 / 搜索 grok。
> 配置：per-aspect `max_search_calls=4`（max_turns8/tool8）+ 全局 `search_policy.recency=fresh` + `max_results_per_query=5`（無 depth=high_recall、無 content_level=detailed、無 category）。

## 总分：23 / 24（C1 持平）✅ 纯引擎、无回归

| # | 维度 | 得分 | 依据 |
|---|---|:--:|---|
| A1 | 引用充分性 | **2** | 34 findings 全挂 evidence_refs；dangling refs = 0 |
| A2 | 引用准确性 | **2** | 抽样真实源（apps.apple.com 版本史、garmin.com、strava press、reddit/forum）；source_type 正确（official 36/blog 5/news 5/unknown 4/forum 1）|
| A3 | 无支撑率 | **2** | finding_type(fact/interpretation/recommendation/risk/assumption) + TM-x 前缀；ODI 全 `estimated:true`；裸断言极少 |
| A4 | 来源质量与多样性 | **2** | 51 evidence / 21 域 / 5 source_type；每 aspect ≥4 ev。**关键提升**：build-cost aspect 获得 4 条 dated official App-Store 版本史证据（Runna/Strava/Garmin/NRC，2026-04/05 fresh 日期）——`recency=fresh` 提升 changelog 命中率，来源多样性评分从 1→2 |
| A5 | 置信度校准+TM-4 | **2** | 全 aspect confidence=medium + 逐 finding epistemic + 逐 evidence confidence |
| B1 | 五维骨架覆盖 | **2** | 6 aspect 全 dedicated（job/能力矩阵/ODI/定位白地/体验路径/build-cost）|
| B2 | 真实竞争集 JTBD | **2** | job-and-competitive-set 6 findings 含非显性替代 |
| B3 | 能力矩阵带证据 | **2** | capability-and-importance 能力矩阵每 cell 内联 evidence_refs 或 assumption 标 |
| B4 | ODI/Kano 严谨 | **2** | opportunity-gaps ODI 排名表 + 公式 + 全 estimated 标 + 过服务降级项；Kano 在能力矩阵 |
| C1 | 视觉证据 | **1** | experience-paths 引 7 个 app-store/产品页 URL（含截图库）但未抽取/校验为显式视觉证据项，诚实记 4 open_questions → 结构限制（deep_research 无 Layer-2 抓图），非回归 |
| C2 | 专家思维动作 TM | **2** | TM-3/4/5/8(pre-mortem)/9/12(revealed strategy)/13 系统体现 |
| C3 | 可落地 | **2** | staged build 路径 + injury-aware adaptive 优先级 + 过服务降级 + roadmap |
| | **合计** | **23 / 24** | C1=1 诚实 abstain（无 Layer-2 抓图）|

## 结论：无回归，search-tuning 提升 A4

1. **A4 源质量提升**：build-cost aspect 经 `recency=fresh`+`max_results=5` 产 4 条 dated official 版本史证据（baseline 2 条）。`recency=fresh` 提升 changelog 命中率的机制在引擎层验证。
2. **安全配置 = recency=fresh + max_results=5 + cap=4**：
   - cap=8 + `depth=high_recall` + `content_level=detailed` 导致执行 abort（`agent_loop.rs:182` 搜索预算超限，无优雅合成回退；`depth=high_recall` prompt_hint 怂恿过搜 → 贪婪 aspect 搜 9 次撞 cap=8 死）。
   - cap=4 + recency=fresh + max_results=5：零 budget_exceeded，6 aspect 全收敛，51 evidence（baseline 28）。
   - **核心约束**：search-tuning 只能选"不增搜索次数"的字段（recency/max_results 安全；depth=high_recall 危险）。
3. **引擎无 per-aspect search 字段**（`workflow.rs:322` 全 aspect 共享同一全局 `search_policy`；`AspectSpec` 仅 `search_provider`）。`category` 是 exact-match、不能全局 → per-aspect 通过 prompt-guided SearchToolArgs 实现（future work）。

## 产物
- 纯引擎结果：`r4c-merged.result.json` / bundle `r4c-engine-bundle.md`。
- harness：`build_r4c_args.py`（transform）、`build_r4c_retry.py`（retry 抽取）。
