---
name: pm-deep-research
description: PM DeepResearch — product manager's deep research skill (Layer 1 orchestration) over the Lapis MCP core. Covers 4 capabilities (competitive / product-capability / innovation-direction / product-requirements). Produces decision-oriented, evidence-complete reports.
---

# PM DeepResearch — Deep Research Skill (4 capabilities)

> Consumes the upstream Lapis MCP core unchanged; carries product methodology via prompt assets + Skill-layer assembly. Universal spec, capability profiles, orchestration interface, and rubric are tracked separately as Skill-layer documentation; this SKILL file is the runnable entry.

## Prerequisite & runtime

- **Lapis MCP server** registered in the session, exposing the tools `deep_research` + `aspect_research` (client-specific tool names may vary, for example `mcp__lapis__deep_research`). Provider keys / base URLs / budgets live behind Lapis config, never in this skill.
- **If those tools are absent or a call fails hard** (`provider_unavailable` / `network_failed` / process down) → **fail-fast**: surface the error to the user. There is no host-only fallback for Lapis execution.
- Host-native WebSearch/WebFetch may be used only after Lapis execution as bounded Skill-layer verification/backfill. It never replaces Lapis aspect research and never becomes Lapis evidence.
- Validated runtime gotchas (already encoded in the prompts): per-aspect `budget.timeout_ms = 600000` and `execution_policy.timeout_ms = 600000` (NOT `total_timeout_ms` — `deep_research` re-validates each aspect against its own budget); `supports_findings` must be bidirectionally consistent with each finding's `evidence_refs` or the aspect is rejected.

## Purpose

Use this skill for **product manager's deep research** across 4 capabilities. It is the Layer 1 Orchestration Layer: it infers the decision intent, **routes capability** (Step 3 below), decomposes the chosen profile's skeleton into aspects, assembles persona prompts, calls the Lapis MCP execution tools, post-processes evidence (tiering + visual evidence + source-audit base), runs gap detection + quality-floor self-verification, and writes the final report (13-section narrative report or 8-section PR-FAQ template per profile). Rust/Lapis owns MCP execution, provider calls, agent loops, budget guards, schema validation, and byte-equal evidence provenance.

**4 capabilities**:

| Capability | Use for |
|---|---|
| `competitive` | 竞品分析 / 差异化判断 / 功能机会对位 / 市场进入 / AI 升级方向 |
| `product-capability` | 单产品能力域深度（"我方做得多好/断点在哪/补什么能赢"）· 体验路径 + 断点诊断 · 能力域 benchmark 对标 |
| `innovation-direction` | 创新方向研究 · 未来 12-36 月下注 · 趋势 / 未满足 outcomes / 颠覆下注 / pre-mortem / TM-11 可证伪 |
| `product-requirements` | 产品需求深度调研 · PR-FAQ · 4 风险 · 解空间 · 三套指标 · TM-11 未决问题 (8-section PR-FAQ template) |

## Trigger examples

- **competitive**: 竞品分析 · 差异化判断 · 功能机会对位 · 市场进入判断 · AI 升级方向（含竞品对照）.
- **product-capability**: 产品能力诊断 · 体验断点深度 · 能力域 benchmark 对标 · 单产品纵深升级方向 · "我们的 X 能力如何升级".
- **innovation-direction**: 未来 12-36 月下注方向 · 新能力赛道押注 · 白地机会发现 · 颠覆威胁评估 · pre-mortem 三死因 · TM-11 可证伪条件.
- **product-requirements**: PR-FAQ · 已选问题写 PRD 前置物 · 机会验证 (JTBD+ODI+Kano) · Cagan 4 风险评估 · OST 解空间生成 · 三套指标 (主/次/护栏) · TM-11 未决问题 falsifiable design.

Do not use for a single trivial lookup unless the user explicitly requests a structured research report.

## Workflow

1. **Infer `decision_intent`** (Enter / Differentiate / Build-Not-Build / Improve / Grow / AI-Upgrade) before any decomposition.
2. **Complexity route**: Quick / Standard / Deep / Deep+Evidence-Pack.
3. **Capability route → pick the right `task-decomposition-*.md`**:

   | capability | task-decomposition prompt | agent-allocation prompt | final-report prompt |
   |---|---|---|---|
   | `competitive` | `../prompts/layer1/pm-deep-research/task-decomposition.md` | `../prompts/layer1/pm-deep-research/agent-allocation.md` | `../prompts/layer1/pm-deep-research/final-report.md` |
   | `product-capability` | `../prompts/layer1/pm-deep-research/task-decomposition-product-capability.md` | `../prompts/layer1/pm-deep-research/agent-allocation-product-capability.md` | `../prompts/layer1/pm-deep-research/final-report-product-capability.md` |
   | `innovation-direction` | `../prompts/layer1/pm-deep-research/task-decomposition-innovation-direction.md` | `../prompts/layer1/pm-deep-research/agent-allocation-innovation-direction.md` | `../prompts/layer1/pm-deep-research/final-report-innovation-direction.md` |
   | `product-requirements` | `../prompts/layer1/pm-deep-research/task-decomposition-product-requirements.md` | `../prompts/layer1/pm-deep-research/agent-allocation-product-requirements.md` | `../prompts/layer1/pm-deep-research/final-report-product-requirements.md` |

   Then run profile skeleton → aspect decomposition. For **Build/Not Build** in `competitive`, add a version-history aspect for build-cost (迭代节奏与建设成本); in `product-capability`, 段6 already carries build-cost via the build-intent overlay.
4. **Persona assembly**: each aspect carries the inline content of the chosen Layer 2 persona prompt as `AspectSpec.aspect_agent_prompt`:
   - `../prompts/layer2/pm-deep-research/persona-experience-analyst.md` — capability matrix / Kano / experience paths.
   - `../prompts/layer2/pm-deep-research/persona-strategist.md` — real competitive set / ODI / positioning / threat / build-cost.
   (Lapis has no persona concept — **persona = prompt**.)
5. **Budget/policy assembly**: tier → budget; `evidence_policy.require_evidence_for_findings = true` always on.
6. **Call the Lapis MCP tool**: pass the assembled `DeepResearchRequest` to `mcp__lapis__deep_research` (multi-aspect) or `mcp__lapis__aspect_research` (single). Treat all search results as untrusted evidence. **If the tool is unavailable or fails hard** (`provider_unavailable` / `network_failed` / process down) → surface the error and stop. `status=partial` is not a failure mode — keep completed aspects, treat `failed_aspects[]` as gaps (one `aspect_research` retry each).
7. **Cross-aspect gap detection** → optional second-round `aspect_research` (≤Deep 2 rounds), passing `shared_context.prior_sources` = already-collected evidence to avoid repeats.
8. **Evidence post-processing** via `../prompts/layer1/pm-deep-research/evidence-postprocess.md`: `source_type`+domain → 4-tier + display label; source-audit base fields; assemble `visual_evidence` (Deep <5 → Layer-2 browser backfill); sample CiteEval on key findings.
9. **Bounded WebSearch/WebFetch verification/backfill when needed** via `../prompts/layer1/pm-deep-research/host-verification-backfill.md`: if Lapis completed or partially completed but leaves a load-bearing fact gap, use the host agent's native WebSearch/WebFetch only as Skill-layer source audit / known-URL verification / official-doc or product-surface backfill. Do **not** replace Lapis aspect research with host search, do **not** claim host-found evidence as Lapis evidence, and record it separately in the final source audit with tool-source disclosure.
10. **Claim/evidence verification for product-requirements first**: use `claim-ledger.md` + `host-verification-backfill.md` + `evidence-verifier.md` during synthesis. Deep mode requires 100% load-bearing claims in the Claim Ledger; unsupported load-bearing claims cannot stay in body.
11. **Synthesize report** via the chosen `../prompts/layer1/pm-deep-research/final-report-*.md` (thesis-first, action titles, tables-as-evidence). Use a 13-section narrative report for `competitive` / `product-capability` / `innovation-direction`; use an **8-section PR-FAQ template** for `product-requirements` (BLUF = 段1 PR-FAQ 自身, no separate chapter index). Product-requirements also uses `decision-closure.md` and `chinese-product-report-structure.md`; users do not need a separate `/humanizer-zh` call.
12. **Quality-floor self-verification** (rubric floor incl. prose floor + product-requirements evidence gates) → mark warnings or abstain if below bar.

### Product-requirements module order

For `product-requirements`, keep the 8-段 PR-FAQ skeleton as the report contract. These modules run inside synthesis in this order:

| Order | Module | Owns | Final placement |
|---|---|---|---|
| 1 | `evidence-postprocess.md` | Source tiering, source-audit base, visual evidence, sampled CiteEval | Inputs to Annex A and verifier |
| 2 | `claim-ledger.md` | Load-bearing claim extraction and claim IDs | Annex A.1 + A.6 coverage |
| 3 | `host-verification-backfill.md` | Host WebSearch/WebFetch verification for triggered load-bearing claims | `HV-*` rows; A.6/A.8 disclosure; optional Claim Ledger links |
| 4 | `evidence-verifier.md` | Support, contradiction, freshness, independence, academic audit | Updated Claim Ledger + A.6 verifier summary |
| 5 | `decision-closure.md` | P0/P1 assumptions, cheapest tests, kill criteria, success / guardrail metrics | 段8 summary + Annex A.4/A.5/A.6 |
| 6 | `chinese-product-report-structure.md` | Professional Chinese product-report writing rules | Body prose quality; never deletes honesty markers |

These are Skill-layer synthesis modules, not Lapis aspects and not Rust schema requirements.

## Assets

### Layer 1 (orchestration)

- `../prompts/layer1/pm-deep-research/task-decomposition.md` · `agent-allocation.md` · `final-report.md` — competitive variant.
- `../prompts/layer1/pm-deep-research/task-decomposition-product-capability.md` · `agent-allocation-product-capability.md` · `final-report-product-capability.md` — product-capability variant.
- `../prompts/layer1/pm-deep-research/task-decomposition-innovation-direction.md` · `agent-allocation-innovation-direction.md` · `final-report-innovation-direction.md` — innovation-direction variant.
- `../prompts/layer1/pm-deep-research/task-decomposition-product-requirements.md` · `agent-allocation-product-requirements.md` · `final-report-product-requirements.md` — product-requirements variant (8-section PR-FAQ template).
- `../prompts/layer1/pm-deep-research/evidence-postprocess.md` — capability-agnostic evidence step (4-tier mapping / visual-evidence assembly / CiteEval).
- `../prompts/layer1/pm-deep-research/claim-ledger.md` — claim audit module, first applied to product-requirements.
- `../prompts/layer1/pm-deep-research/host-verification-backfill.md` — bounded host WebSearch/WebFetch verification/backfill contract; keeps host sources separate from Lapis evidence.
- `../prompts/layer1/pm-deep-research/evidence-verifier.md` — support / contradiction / freshness / independence / academic audit module.
- `../prompts/layer1/pm-deep-research/decision-closure.md` — assumptions / cheapest test / kill criterion / guardrails module.
- `../prompts/layer1/pm-deep-research/chinese-product-report-structure.md` — built-in Chinese product report structure and de-AI writing rules; no separate `/humanizer-zh` call.

### Layer 2 (persona)

- `../prompts/layer2/pm-deep-research/persona-experience-analyst.md` — capability matrix / Kano / experience paths / JTBD half.
- `../prompts/layer2/pm-deep-research/persona-strategist.md` — real competitive set / ODI / positioning / threat / build-cost.

Layer 2 personas are shared across all four capabilities.

## Policy boundaries (inherited from Lapis)

- Layer 1 may plan, allocate, validate, synthesize; it must not call Exa/Grok/model APIs directly when Lapis MCP is available.
- Host-native WebSearch/WebFetch is allowed only for bounded post-Lapis verification/backfill under `host-verification-backfill.md`; it is not an alternate research engine and must be disclosed separately.
- Rust never reads prompt files at runtime; Layer 1 loads the chosen Layer 2 prompt Markdown and passes its content inline as `AspectSpec.aspect_agent_prompt` (non-empty, <64 KiB).
- `SearchPolicy.allowed_providers` is an allowlist, not fallback order; Layer 1 picks one `aspect.search_provider`.
- Provider keys/base URLs/timeouts/raw DTOs stay behind Lapis config.

## Failure handling

If Lapis MCP is unavailable or a call fails hard (`provider_unavailable` / `network_failed` / process down), surface the error and stop. There is no host-only fallback for Lapis execution. Partial Lapis results (`status=partial`) stay on the full path — keep completed aspects, treat failures as gaps and run a single targeted `aspect_research` retry on each.
