---
name: pm-deep-research
description: PM DeepResearch — product manager's deep research skill (Layer 1 orchestration) over the Lapis MCP core. Covers 4 capabilities (v2.0 competitive ✅ 23/24 + v2.1 product-capability ✅ 23/24 + v2.2 innovation-direction ✅ 24/24 + v2.3 product-requirements ✅ 24/24). Produces decision-oriented, evidence-complete reports.
version: 0.4.0
---

# PM DeepResearch — Deep Research Skill (4 capabilities, all ✅ end-to-end validated)

> Status: **v2.0 competitive RUNNABLE 23/24** (R4-c canonical) · **v2.1 product-capability RUNNABLE 23/24** (R4-c canonical) · **v2.2 innovation-direction RUNNABLE 24/24** (R4-c canonical, cap=6 必须) · **v2.3 product-requirements RUNNABLE 24/24** (R4-e + #9 rerun9; family B 8-段 PR-FAQ 首落地). Consumes the upstream Lapis MCP core unchanged (interface §6); carries product methodology via prompt assets + Skill-layer assembly.
> ✅ **v2.0 competitive**: 6-aspect Deep run on Strava AI-coaching upgrade golden, [report](../docs/pm-deep-research/evaluation/golden/competitive-strava-coach-upgrade.md) / [score 23/24](../docs/pm-deep-research/evaluation/golden/competitive-rubric-score.md)（R4-c：recency=fresh + max_results=5 + per-aspect cap=4；A4 1→2）.
> ✅ **v2.1 product-capability**: 6-aspect Deep run on Runna 训练计划自适应能力深度 golden, [report](../docs/pm-deep-research/evaluation/golden/product-capability-runna-training-plan.md) / [score 23/24](../docs/pm-deep-research/evaluation/golden/product-capability-rubric-score.md)（R4-c cap=3，锚点持平 + enrichment）.
> ✅ **v2.2 innovation-direction**: 8-aspect Deep run on 海外运动/健身 AI Coach 赛道 12-36 月下注 golden, [report](../docs/pm-deep-research/evaluation/golden/innovation-direction-ai-coach-bets.md) / [score 24/24](../docs/pm-deep-research/evaluation/golden/innovation-direction-rubric-score.md)（R4-c cap=6 强制；recommended-bets aspect 在 fresh prompt-hint 下 search appetite ~6，cap=5 持续 hard-kill）.
> ✅ **v2.3 product-requirements**: 8-aspect Deep run on Endurance-athlete Explainable Biometric Coach PR-FAQ golden, [report](../docs/pm-deep-research/evaluation/golden/product-requirements-prfaq.md) / [score 24/24](../docs/pm-deep-research/evaluation/golden/product-requirements-rubric-score.md)（family B 8-段 PR-FAQ 首落地证明；R4-e 段3 cagan 拆 4 micro-aspect + R4-g rubric §6.4 family B C1 适配；R4-c NOT canonical, 详 [task-decomposition-product-requirements.md](../prompts/layer1/pm-deep-research/task-decomposition-product-requirements.md) 注释）.
> Universal spec (SSOT, all 4 capabilities): [`pm-deep-research-spec.md`](../docs/pm-deep-research/pm-deep-research-spec.md). Capability profiles: [`competitive.md`](../docs/pm-deep-research/capabilities/competitive.md) · [`product-capability.md`](../docs/pm-deep-research/capabilities/product-capability.md) · [`innovation-direction.md`](../docs/pm-deep-research/capabilities/innovation-direction.md) · [`product-requirements.md`](../docs/pm-deep-research/capabilities/product-requirements.md). Interface: [`orchestration-interface.md`](../docs/pm-deep-research/orchestration-interface.md). Rubric: [`rubric.md`](../docs/pm-deep-research/evaluation/rubric.md). Goldens 索引: [`evaluation/golden/README.md`](../docs/pm-deep-research/evaluation/golden/README.md).

## Prerequisite & runtime

- **Lapis MCP server** registered in the session, exposing the tools `deep_research` + `aspect_research` (in Claude Code: `mcp__lapis__deep_research` / `mcp__lapis__aspect_research`). Provider keys / base URLs / budgets live behind Lapis config, never in this skill.
- **If those tools are absent or a call fails hard** → run the **Claude-only degradation** path ([`../prompts/layer1/pm-deep-research/claude-only-degradation.md`](../prompts/layer1/pm-deep-research/claude-only-degradation.md)); the methodology is unchanged. Decide this at step 6.
- Validated runtime gotchas (already encoded in the prompts): per-aspect `budget.timeout_ms = 600000` and `execution_policy.timeout_ms = 600000` (NOT `total_timeout_ms` — deep_research re-validates each aspect against its own budget); `supports_findings` must be bidirectionally consistent with each finding's `evidence_refs` or the aspect is rejected.

## Purpose

Use this skill for **product manager's deep research** across 4 capabilities. It is the Layer 1 Orchestration Layer: it infers the decision intent, **routes capability** (Step 3 below), decomposes the chosen profile's skeleton into aspects, assembles persona prompts, calls the Lapis MCP execution tools, post-processes evidence (tiering + visual evidence), runs gap detection + quality-floor self-verification, and writes the final report (13-章 family A 或 8-段 family B PR-FAQ per profile). Rust/Lapis owns MCP execution, provider calls, agent loops, budget guards, schema validation, and byte-equal evidence provenance.

**4 capabilities** (per [universal spec §1.1](../docs/pm-deep-research/pm-deep-research-spec.md)):

| Capability | Status | Use for | Profile |
|---|---|---|---|
| `competitive` | ✅ v2.0 RUNNABLE 23/24 (R4-c) | 竞品分析 / 差异化判断 / 功能机会对位 / 市场进入 / AI 升级方向 | [`capabilities/competitive.md`](../docs/pm-deep-research/capabilities/competitive.md) |
| `product-capability` | ✅ v2.1 RUNNABLE 23/24 (R4-c) | 单产品能力域深度（"我方做得多好/断点在哪/补什么能赢"）· 体验路径 + 断点诊断 · 能力域 benchmark 对标 | [`capabilities/product-capability.md`](../docs/pm-deep-research/capabilities/product-capability.md) |
| `innovation-direction` | ✅ v2.2 RUNNABLE 24/24 (R4-c, cap=6) | 创新方向研究 · 未来 12-36 月下注 · 趋势 / 未满足 outcomes / 颠覆下注 / pre-mortem / TM-11 可证伪 | [`capabilities/innovation-direction.md`](../docs/pm-deep-research/capabilities/innovation-direction.md) |
| `product-requirements` | ✅ v2.3 RUNNABLE 24/24 (R4-e/g; family B 首落地) | 产品需求深度调研 · PR-FAQ · 4 风险 · 解空间 · 三套指标 · TM-11 未决问题 (family B 8-段 PR-FAQ 首落地) | [`capabilities/product-requirements.md`](../docs/pm-deep-research/capabilities/product-requirements.md) |

## Trigger examples

- **competitive**: 竞品分析 · 差异化判断 · 功能机会对位 · 市场进入判断 · AI 升级方向（含竞品对照）.
- **product-capability**: 产品能力诊断 · 体验断点深度 · 能力域 benchmark 对标 · 单产品纵深升级方向 · "我们的 X 能力如何升级".
- **innovation-direction**: 未来 12-36 月下注方向 · 新能力赛道押注 · 白地机会发现 · 颠覆威胁评估 · pre-mortem 三死因 · TM-11 可证伪条件.
- **product-requirements**: PR-FAQ · 已选问题写 PRD 前置物 · 机会验证 (JTBD+ODI+Kano) · Cagan 4 风险评估 · OST 解空间生成 · 三套指标 (主/次/护栏) · TM-11 未决问题 falsifiable design.

Do not use for a single trivial lookup unless the user explicitly requests a structured research report.

## Workflow (per spec + interface)

1. **Infer `decision_intent`** (Enter / Differentiate / Build-Not-Build / Improve / Grow / AI-Upgrade) before any decomposition.
2. **Complexity route**: Quick / Standard / Deep / Deep+Evidence-Pack (spec §1.3).
3. **Capability route → pick the right `task-decomposition-*.md`** (interface §1.5 Step 1.5):

   | capability | task-decomposition prompt | agent-allocation prompt | final-report prompt |
   |---|---|---|---|
   | `competitive` | `../prompts/layer1/pm-deep-research/task-decomposition.md` | `../prompts/layer1/pm-deep-research/agent-allocation.md` | `../prompts/layer1/pm-deep-research/final-report.md` |
   | `product-capability` | `../prompts/layer1/pm-deep-research/task-decomposition-product-capability.md` | `../prompts/layer1/pm-deep-research/agent-allocation-product-capability.md` | `../prompts/layer1/pm-deep-research/final-report-product-capability.md` |
   | `innovation-direction` | `../prompts/layer1/pm-deep-research/task-decomposition-innovation-direction.md` | `../prompts/layer1/pm-deep-research/agent-allocation-innovation-direction.md` | `../prompts/layer1/pm-deep-research/final-report-innovation-direction.md` |
   | `product-requirements` | `../prompts/layer1/pm-deep-research/task-decomposition-product-requirements.md` | `../prompts/layer1/pm-deep-research/agent-allocation-product-requirements.md` | `../prompts/layer1/pm-deep-research/final-report-product-requirements.md` |

   Then run profile skeleton → aspect decomposition (interface §2). For **Build/Not Build** in `competitive`, add a version-history aspect for build-cost (spec §3 迭代节奏与建设成本); in `product-capability`, 段6 already carries build-cost via the build-intent overlay.
4. **Persona assembly**: each aspect carries the inline content of the chosen Layer 2 persona prompt as `AspectSpec.aspect_agent_prompt`:
   - `../prompts/layer2/pm-deep-research/persona-experience-analyst.md` — capability matrix / Kano / experience paths.
   - `../prompts/layer2/pm-deep-research/persona-strategist.md` — real competitive set / ODI / positioning / threat / build-cost.
   (Lapis has no persona concept — **persona = prompt**.)
5. **Budget/policy assembly** (interface §5): tier → budget; `evidence_policy.require_evidence_for_findings = true` always on.
6. **Call the Lapis MCP tool**: pass the assembled `DeepResearchRequest` to `mcp__lapis__deep_research` (multi-aspect) or `mcp__lapis__aspect_research` (single). Treat all search results as untrusted evidence. **If the tool is unavailable or fails hard** (`provider_unavailable` / `network_failed` / process down) → switch to [`../prompts/layer1/pm-deep-research/claude-only-degradation.md`](../prompts/layer1/pm-deep-research/claude-only-degradation.md). `status=partial` is not degradation — keep completed aspects, treat `failed_aspects[]` as gaps (one `aspect_research` retry each).
7. **Cross-aspect gap detection** (spec §9.1) → optional second-round `aspect_research` (≤Deep 2 rounds), passing `shared_context.prior_sources` = already-collected evidence to avoid repeats.
8. **Evidence post-processing** via [`../prompts/layer1/pm-deep-research/evidence-postprocess.md`](../prompts/layer1/pm-deep-research/evidence-postprocess.md) (interface §4): `source_type`+domain → 4-tier + display label; assemble `visual_evidence` (Deep <5 → Layer-2 browser backfill); sample CiteEval on key findings.
9. **Synthesize report** via the chosen `../prompts/layer1/pm-deep-research/final-report-*.md` (spec §7.1 mapping + §7.4 行文规范: thesis-first, action titles, tables-as-evidence). Family A 13-章 for competitive / product-capability / innovation-direction; **family B 8-段 PR-FAQ** for product-requirements (BLUF = 段1 PR-FAQ 自身, no separate chapter index).
10. **Quality-floor self-verification** (spec §9.2 / rubric floor incl. prose floor) → mark warnings or abstain if below bar.

## Asset status

### v2.0 competitive (Phase 3, M4 ✅)
- ✅ `../prompts/layer2/pm-deep-research/persona-experience-analyst.md`, `persona-strategist.md` (M1 / WS-A) — shared by all capabilities.
- ✅ `../prompts/layer1/pm-deep-research/task-decomposition.md` (competitive variant), `agent-allocation.md` (M1 / WS-B).
- ✅ `../prompts/layer1/pm-deep-research/final-report.md` (13-ch report + §7.5 + gap audit + quality-floor self-verification) (M2 / WS-C).
- ✅ `../prompts/layer1/pm-deep-research/evidence-postprocess.md` (capability-agnostic; 4-tier mapping / visual-evidence assembly + Layer-2 backfill / CiteEval) — standalone step-7 procedure (WS-E).
- ✅ `../prompts/layer1/pm-deep-research/claude-only-degradation.md` (capability-agnostic) + this SKILL entry wired runnable (WS-D).
- ✅ End-to-end validated on golden topic (M4): 6/6 aspects, 13-ch report, rubric **22/24** baseline → **R4-c canonical 23/24** (A4 1→2 via `recency=fresh` build-cost dated changelog hits). See [`competitive-rubric-score.md`](../docs/pm-deep-research/evaluation/golden/competitive-rubric-score.md).

### v2.1 product-capability (✅ M5 23/24)
- ✅ `../prompts/layer1/pm-deep-research/task-decomposition-product-capability.md` (6-段 skeleton, EA-heavy).
- ✅ `../prompts/layer1/pm-deep-research/agent-allocation-product-capability.md` (段→aspect→persona mapping; tier subset; 段5 ownership note).
- ✅ `../prompts/layer1/pm-deep-research/final-report-product-capability.md` (13-ch variant: Ch 6/7/4 加重, Ch 5 裁为 benchmark; profile §3.1/§3.2 gap+floor).
- ✅ Layer-2: reuses `persona-experience-analyst.md` + `persona-strategist.md` (no EA-deep variant needed — M5 validated).
- ✅ **M5 end-to-end validated**: 6/6 aspects, 13-ch report, rubric **23/24** (超 v2.0 M4 锚点 +1; A4 来源质量 1→2). **R4-c canonical (cap=3)** 持平 23/24 + enrichment (53 ev/15 domain/6 source_type vs anchor 48/13/4). Golden topic = Runna 训练计划自适应能力深度. See [report](../docs/pm-deep-research/evaluation/golden/product-capability-runna-training-plan.md) + [R4-c score](../docs/pm-deep-research/evaluation/golden/product-capability-rubric-score.md).

### v2.2 innovation-direction (✅ M6 24/24)
- ✅ `../prompts/layer1/pm-deep-research/task-decomposition-innovation-direction.md` (8-段 skeleton, Strategist-heavy / EA-light; 段8 TM-11 hard gate; 段6 pre-mortem 三死因强制).
- ✅ `../prompts/layer1/pm-deep-research/agent-allocation-innovation-direction.md` (段→aspect→persona mapping; tier subset quick=2/standard=5/deep=8; 段2 sole-EA note).
- ✅ `../prompts/layer1/pm-deep-research/final-report-innovation-direction.md` (13-ch variant: Ch 8/9/10/12 加重, Ch 5 裁为白地图, Ch 6 裁为承载力评估, Ch 11 加重 TM-11 验证实验; profile §3.1/§3.2 gap+floor).
- ✅ Layer-2: reuses `persona-experience-analyst.md` + `persona-strategist.md` (no Strategist-futurist variant — M6 validated; 通用 strategist 在 8 段全段下足够).
- ✅ **M6 end-to-end validated**: 8/8 aspects (4 deep + 4 sequential retry with prior_sources baseline), 13-ch variant report, rubric **24/24 (满分)** — 超 v2.1 锚点 +1, C1 视觉 1→2 (类型偏战略图天然达标), TM-11 hard gate 100% (3/3 推荐下注全 falsifiable). **R4-c canonical (cap=6 强制)** 持平 24/24 + 74 ev/23 domain；recommended-bets aspect 在 `recency=fresh` prompt-hint 下 search appetite ~6，cap=5 持续 hard-kill。Golden topic = 海外运动/健身 app AI Coach 赛道未来 12-36 月下注 (decision_intent = `ai-upgrade` + `enter`). See [report](../docs/pm-deep-research/evaluation/golden/innovation-direction-ai-coach-bets.md) + [R4-c score](../docs/pm-deep-research/evaluation/golden/innovation-direction-rubric-score.md).

### v2.3 product-requirements (✅ R4-e + #9 rerun9 24/24; family B 首落地)
- ✅ `../prompts/layer1/pm-deep-research/task-decomposition-product-requirements.md` (8-段 PR-FAQ skeleton, EA + Strategist 平衡; **5 hard gates**: 段3 4-risks 全 / 段4 ≥3 候选 / 段5 非目标显式 / 段6 三套指标 / 段8 TM-11 falsification 100%; 段3 cagan 拆 4 micro-aspect via R4-e; 段7 evidence-table 标 OPTIONAL via R4-f).
- ✅ `../prompts/layer1/pm-deep-research/agent-allocation-product-requirements.md` (段→aspect→persona mapping; tier subset quick=2/standard=4/deep=10 mandatory aspect; EA+Strategist 平衡 ownership note; family B working backwards 回填校验 note).
- ✅ `../prompts/layer1/pm-deep-research/final-report-product-requirements.md` (**family B 8-段 PR-FAQ template 首落地** — 不出现章节索引, BLUF = 段1 PR-FAQ 自身; 段间 narrative = Amazon working backwards; profile §3.1/§3.2 gap+floor + 5 hard gates 全 enforce).
- ✅ Layer-2: reuses `persona-experience-analyst.md` + `persona-strategist.md` (no EA-deep / Strategist-futurist variant needed — M5/M6 已分别验证; 平衡 profile 下二者均不需新变体).
- ✅ **R4-e + #9 rerun9 end-to-end validated, 24/24**: 段3 cagan 拆 4 micro-aspect (value/usability/feasibility/business，每 max_search=4 + 1 class evidence) 消除 search-saturation 病理 → 段3 升为 dedicated 4-risks (M7.5 预演 4/4 + #9 复跑 cagan 4/4 双坐实)；B1 1→2 + B3 1→2 (叠加 R4-g rubric §6.4 family B C1 适配 1→2) = **21→24**。**TM-11 hard gate 7/7=100% ✅**；family B 8 段 PR-FAQ 模板首落地证明 ✅；4 黄金 #9 全 PASS 零回归 (v2.0 22/v2.1 23/v2.2 24/v2.3 24)。**R4-c NOT canonical** for this capability — requirements-fn-nfn-nongoals aspect 在 `recency=fresh` prompt-hint 下结构性 synthesis-time fragile (4 retry 模式齐失败：cap=8 search hard-kill / cap=9 runtime / cap=9+aspect_timeout=900s 撞 execution_policy=600s 复校 / cap=9+双侧 900s 撞 CPA SSE flake)；rerun9 24/24 anchor 沿用。Golden topic = Endurance-athlete Explainable Biometric Coach PR-FAQ. See [report](../docs/pm-deep-research/evaluation/golden/product-requirements-prfaq.md) + [rerun9 score](../docs/pm-deep-research/evaluation/golden/product-requirements-rubric-score.md).

## Policy boundaries (inherited from Lapis)

- Layer 1 may plan, allocate, validate, synthesize; it must not call Exa/Grok/model APIs directly when Lapis MCP is available.
- Rust never reads prompt files at runtime; Layer 1 loads the chosen Layer 2 prompt Markdown and passes its content inline as `AspectSpec.aspect_agent_prompt` (non-empty, <64 KiB).
- `SearchPolicy.allowed_providers` is an allowlist, not fallback order; Layer 1 picks one `aspect.search_provider`.
- Provider keys/base URLs/timeouts/raw DTOs stay behind Lapis config.

## Degradation (spec §10)

If Lapis MCP is unavailable, degrade to **Claude-only** per [`../prompts/layer1/pm-deep-research/claude-only-degradation.md`](../prompts/layer1/pm-deep-research/claude-only-degradation.md): Claude plays both Layer 1 and the aspect agents, calling the search MCP directly while applying the same five-dim methodology + persona TM moves + 13-chapter template + (now self-enforced) evidence discipline. Claude-only is not failure — the methodology lift is pure prompt capability. Partial Lapis results stay on the full path (keep completed aspects, treat failures as gaps).
