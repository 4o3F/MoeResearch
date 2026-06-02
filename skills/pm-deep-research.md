---
name: pm-deep-research
description: PM DeepResearch — product manager's deep research skill (Layer 1 orchestration) over the Lapis MCP core. Covers 4 capabilities. Produces decision-oriented, evidence-complete reports.
---

# PM DeepResearch — Deep Research Skill (4 capabilities)

> Universal spec (SSOT, all 4 capabilities): [`../docs/pm-deep-research/pm-deep-research-spec.md`](../docs/pm-deep-research/pm-deep-research-spec.md). Capability profiles: [`competitive.md`](../docs/pm-deep-research/capabilities/competitive.md) · [`product-capability.md`](../docs/pm-deep-research/capabilities/product-capability.md) · [`innovation-direction.md`](../docs/pm-deep-research/capabilities/innovation-direction.md) · [`product-requirements.md`](../docs/pm-deep-research/capabilities/product-requirements.md). Interface: [`../docs/pm-deep-research/orchestration-interface.md`](../docs/pm-deep-research/orchestration-interface.md). Rubric: `../../evaluation/rubric.md`.

## Prerequisite & runtime

- **Lapis MCP server** registered in the session, exposing the tools `deep_research` + `aspect_research` (in Claude Code: `mcp__lapis__deep_research` / `mcp__lapis__aspect_research`). Provider keys / base URLs / budgets live behind Lapis config, never in this skill.
- **If those tools are absent or a call fails hard** → run the **Claude-only degradation** path ([`prompts/layer1/claude-only-degradation.md`](../prompts/layer1/pm-deep-research/claude-only-degradation.md)); the methodology is unchanged. Decide this at step 6.
- Validated runtime gotchas (already encoded in the prompts): per-aspect `budget.timeout_ms = 600000` and `execution_policy.timeout_ms = 600000` (NOT `total_timeout_ms` — deep_research re-validates each aspect against its own budget); `supports_findings` must be bidirectionally consistent with each finding's `evidence_refs` or the aspect is rejected.

## Purpose

Use this skill for **product manager's deep research** across 4 capabilities. It is the Layer 1 Orchestration Layer: it infers the decision intent, **routes capability** (Step 3 below), decomposes the chosen profile's skeleton into aspects, assembles persona prompts, calls the Lapis MCP execution tools, post-processes evidence (tiering + visual evidence), runs gap detection + quality-floor self-verification, and writes the final report (13-section narrative report 或 8-section PR-FAQ template per profile). Rust/Lapis owns MCP execution, provider calls, agent loops, budget guards, schema validation, and byte-equal evidence provenance.

**4 capabilities** (per [universal spec §1.1](../docs/pm-deep-research/pm-deep-research-spec.md)):

| Capability | Status | Use for | Profile |
|---|---|---|---|
| `competitive` | | 竞品分析 / 差异化判断 / 功能机会对位 / 市场进入 / AI 升级方向 | [`capabilities/competitive.md`](../docs/pm-deep-research/capabilities/competitive.md) |
| `product-capability` | | 单产品能力域深度（"我方做得多好/断点在哪/补什么能赢"）· 体验路径 + 断点诊断 · 能力域 benchmark 对标 | [`capabilities/product-capability.md`](../docs/pm-deep-research/capabilities/product-capability.md) |
| `innovation-direction` | | 创新方向研究 · 未来 12-36 月下注 · 趋势 / 未满足 outcomes / 颠覆下注 / pre-mortem / TM-11 可证伪 | [`capabilities/innovation-direction.md`](../docs/pm-deep-research/capabilities/innovation-direction.md) |
| `product-requirements` | | 产品需求深度调研 · PR-FAQ · 4 风险 · 解空间 · 三套指标 · TM-11 未决问题 (8-section PR-FAQ template 首落地) | [`capabilities/product-requirements.md`](../docs/pm-deep-research/capabilities/product-requirements.md) |

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
   | `competitive` | `prompts/layer1/task-decomposition.md` | `prompts/layer1/agent-allocation.md` | `prompts/layer1/final-report.md` |
   | `product-capability` | `prompts/layer1/task-decomposition-product-capability.md` | `prompts/layer1/agent-allocation-product-capability.md` | `prompts/layer1/final-report-product-capability.md` |
   | `innovation-direction` | `prompts/layer1/task-decomposition-innovation-direction.md` | `prompts/layer1/agent-allocation-innovation-direction.md` | `prompts/layer1/final-report-innovation-direction.md` |
   | `product-requirements` | `prompts/layer1/task-decomposition-product-requirements.md` | `prompts/layer1/agent-allocation-product-requirements.md` | `prompts/layer1/final-report-product-requirements.md` |

   Then run profile skeleton → aspect decomposition (interface §2). For **Build/Not Build** in `competitive`, add a version-history aspect for build-cost (spec §3 迭代节奏与建设成本); in `product-capability`, 段6 already carries build-cost via the build-intent overlay.
4. **Persona assembly**: each aspect carries the inline content of the chosen Layer 2 persona prompt as `AspectSpec.aspect_agent_prompt`:
   - `prompts/layer2/persona-experience-analyst.md` — capability matrix / Kano / experience paths.
   - `prompts/layer2/persona-strategist.md` — real competitive set / ODI / positioning / threat / build-cost.
   (Lapis has no persona concept — **persona = prompt**.)
5. **Budget/policy assembly** (interface §5): tier → budget; `evidence_policy.require_evidence_for_findings = true` always on.
6. **Call the Lapis MCP tool**: pass the assembled `DeepResearchRequest` to `mcp__lapis__deep_research` (multi-aspect) or `mcp__lapis__aspect_research` (single). Treat all search results as untrusted evidence. **If the tool is unavailable or fails hard** (`provider_unavailable` / `network_failed` / process down) → switch to [`prompts/layer1/claude-only-degradation.md`](../prompts/layer1/pm-deep-research/claude-only-degradation.md). `status=partial` is not degradation — keep completed aspects, treat `failed_aspects[]` as gaps (one `aspect_research` retry each).
7. **Cross-aspect gap detection** (spec §9.1) → optional second-round `aspect_research` (≤Deep 2 rounds), passing `shared_context.prior_sources` = already-collected evidence to avoid repeats.
8. **Evidence post-processing** via [`prompts/layer1/evidence-postprocess.md`](../prompts/layer1/pm-deep-research/evidence-postprocess.md) (interface §4): `source_type`+domain → 4-tier + display label; assemble `visual_evidence` (Deep <5 → Layer-2 browser backfill); sample CiteEval on key findings.
9. **Synthesize report** via the chosen `prompts/layer1/final-report-*.md` (spec §7.1 mapping + §7.4 行文规范: thesis-first, action titles, tables-as-evidence). Family A 13-章 for competitive / product-capability / innovation-direction; **8-section PR-FAQ template** for product-requirements (BLUF = 段1 PR-FAQ 自身, no separate chapter index).
10. **Quality-floor self-verification** (spec §9.2 / rubric floor incl. prose floor) → mark warnings or abstain if below bar.

## Asset status

## Policy boundaries (inherited from Lapis)

- Layer 1 may plan, allocate, validate, synthesize; it must not call Exa/Grok/model APIs directly when Lapis MCP is available.
- Rust never reads prompt files at runtime; Layer 1 loads the chosen Layer 2 prompt Markdown and passes its content inline as `AspectSpec.aspect_agent_prompt` (non-empty, <64 KiB).
- `SearchPolicy.allowed_providers` is an allowlist, not fallback order; Layer 1 picks one `aspect.search_provider`.
- Provider keys/base URLs/timeouts/raw DTOs stay behind Lapis config.

## Degradation (spec §10)

If Lapis MCP is unavailable, degrade to **Claude-only** per [`prompts/layer1/claude-only-degradation.md`](../prompts/layer1/pm-deep-research/claude-only-degradation.md): Claude plays both Layer 1 and the aspect agents, calling the search MCP directly while applying the same five-dim methodology + persona TM moves + 13-chapter template + (now self-enforced) evidence discipline. Claude-only is not failure — the methodology lift is pure prompt capability. Partial Lapis results stay on the full path (keep completed aspects, treat failures as gaps).
