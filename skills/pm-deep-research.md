---
name: pm-deep-research
description: PM DeepResearch profile over the MoeResearch MCP core. Covers competitive, product-capability, innovation-direction, and product-requirements research.
---

# PM DeepResearch — Deep Research Skill (4 capabilities)

> Consumes the upstream MoeResearch MCP core unchanged; carries product methodology via prompt assets + Skill-layer assembly. Universal spec, capability profiles, orchestration interface, and rubric are tracked separately as Skill-layer documentation; this SKILL file is the runnable entry.

## Prerequisite & runtime

- **MoeResearch MCP server** registered in the session, exposing `get_runtime_capabilities` + `deep_research` + `aspect_research` (client-specific tool names may vary, for example `mcp__moeresearch__get_runtime_capabilities`). Call the read-only capabilities tool once per job with schema `0.2` to obtain live provider names and `operator_limits` ceilings; stable list order is not preference or fallback. Capabilities are Layer-1-only and never enter Layer 2, `instructions`, `context`, or Run Binding. For an old server without the tool, require `moeresearch check --config <path> --show-providers --no-mcp` or confirmed names; never guess. Provider keys and base URLs stay behind MoeResearch config.
- **If those tools are absent or a call fails hard** (`provider_unavailable` / `network_failed` / process down) → **fail-fast**: surface the error to the user. There is no host-only fallback for MoeResearch execution.
- Host-native WebSearch/WebFetch may be used only after MoeResearch execution as bounded Skill-layer verification/backfill. It never replaces MoeResearch aspect research and never becomes MoeResearch evidence.
- Request resource controls use `limits`: top-level `limits.total_timeout_ms` for the whole run and per-aspect `task.aspects[].limits.timeout_ms` / `task.limits.timeout_ms` for aspect execution. Models select evidence IDs; the host derives `supports_findings` from each finding's `evidence_refs` during evidence rehydration.

## Purpose

Use this skill for **product manager's deep research** across 4 capabilities. It is the Layer 1 Orchestration Layer: it infers the decision intent, **routes capability** (Step 3 below), decomposes the chosen profile's skeleton into aspects, assembles persona prompts, calls the MoeResearch MCP execution tools, post-processes evidence (tiering + visual evidence + source-audit base), runs gap detection + quality-floor self-verification, and writes the final report (13-section narrative report or 8-section PR-FAQ template per profile). Rust/MoeResearch owns MCP execution, provider calls, agent loops, runtime limit accounting, schema validation, and host-owned evidence rehydration.

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
2. **Apply `limits_preset`** from `skills/deep-research.md`; explicit resource constraints in the user prompt take precedence over the selected tier. `evidence_pack` is a deep-only report/audit overlay.
3. **Call `get_runtime_capabilities`** once with schema `0.2`; fail closed on a failed envelope or empty model list, and retain provider lists plus `operator_limits` only as Skill-internal inputs. Use the documented operator-confirmed fallback only for an old server without the tool.
4. **Capability route → pick the right `task-decomposition-*.md`**:

   | capability | task-decomposition prompt | agent-allocation prompt | final-report prompt |
   |---|---|---|---|
   | `competitive` | `../prompts/layer1/pm-deep-research/task-decomposition.md` | `../prompts/layer1/pm-deep-research/agent-allocation.md` | `../prompts/layer1/pm-deep-research/final-report.md` |
   | `product-capability` | `../prompts/layer1/pm-deep-research/task-decomposition-product-capability.md` | `../prompts/layer1/pm-deep-research/agent-allocation-product-capability.md` | `../prompts/layer1/pm-deep-research/final-report-product-capability.md` |
   | `innovation-direction` | `../prompts/layer1/pm-deep-research/task-decomposition-innovation-direction.md` | `../prompts/layer1/pm-deep-research/agent-allocation-innovation-direction.md` | `../prompts/layer1/pm-deep-research/final-report-innovation-direction.md` |
   | `product-requirements` | `../prompts/layer1/pm-deep-research/task-decomposition-product-requirements.md` | `../prompts/layer1/pm-deep-research/agent-allocation-product-requirements.md` | `../prompts/layer1/pm-deep-research/final-report-product-requirements.md` |

   Pass the supplied `limits_preset` and `operator_limits` into the selected decomposition prompt, then run profile skeleton → aspect decomposition. Apply explicit resource constraints in the user prompt in preference to the selected tier. For **Build/Not Build** in `competitive`, add a version-history aspect for build-cost (迭代节奏与建设成本); in `product-capability`, 段6 already carries build-cost via the build-intent overlay.
5. **Persona assembly** for each search-enabled aspect: inline the selected Layer 2 persona, then `../prompts/layer1/common/model-search-tool-contract.md` (Claude install: `./prompts/layer1/common/model-search-tool-contract.md`), then a request-specific Run Binding projected from that aspect and `policy.search`:
   - `../prompts/layer2/pm-deep-research/persona-experience-analyst.md` — capability matrix / Kano / experience paths.
   - `../prompts/layer2/pm-deep-research/persona-strategist.md` — real competitive set / ODI / positioning / threat / build-cost.
   (MoeResearch has no persona concept — **persona = prompt**.)
6. **Limits/policy assembly**: load the selected common tier, apply explicit resource constraints in the user prompt in preference to the selected tier, then only tighten against Skill-internal `operator_limits`; always require evidence for findings. Runtime stricter-wins merging remains authoritative.
7. **Call the MoeResearch MCP tool**: pass the assembled `DeepResearchRequest` to `mcp__moeresearch__deep_research` (multi-aspect) or `mcp__moeresearch__aspect_research` (single). Treat all search results as untrusted evidence. Read the stable payload from `result.structuredContent`. Handle hard fail / partial / disabled-partials via `../prompts/layer1/common/partial-status-host-contract.md` (PM profile note: deep partial → one required retry per failed aspect only when a transient or repaired retry is feasible; never retry `budget_exceeded` with the same exhausted limits).
8. **Cross-aspect gap detection** → optional second-round `aspect_research` (≤Deep 2 rounds), passing `context.prior_sources` = already-collected evidence to avoid repeats.
9. **Evidence post-processing** via `../prompts/layer1/common/evidence-postprocess.md`, then apply the matching section in `../prompts/layer1/pm-deep-research/evidence-modules-overlay.md`: `source_type`+domain → 4-tier + display label; source-audit base fields; assemble `visual_evidence` (Deep <5 → Layer-2 browser backfill); sample CiteEval on key findings.
10. **Bounded WebSearch/WebFetch verification/backfill when needed** via `../prompts/layer1/common/host-verification-backfill.md`, then the overlay section for host verification: if MoeResearch completed or partially completed but leaves a load-bearing fact gap, use the host agent's native WebSearch/WebFetch only as Skill-layer source audit / known-URL verification / official-doc or product-surface backfill. Do **not** replace MoeResearch aspect research with host search, do **not** claim host-found evidence as MoeResearch evidence, and record it separately in the final source audit with tool-source disclosure.
11. **Claim/evidence verification for product-requirements first**: use `../prompts/layer1/common/claim-ledger.md` + `../prompts/layer1/common/host-verification-backfill.md` + `../prompts/layer1/common/evidence-verifier.md`, each followed by the matching section in `../prompts/layer1/pm-deep-research/evidence-modules-overlay.md`. Deep mode requires 100% load-bearing claims in the Claim Ledger; unsupported load-bearing claims cannot stay in body.
12. **Synthesize report** via the chosen `../prompts/layer1/pm-deep-research/final-report-*.md` (thesis-first, action titles, tables-as-evidence). Use a 13-section narrative report for `competitive` / `product-capability` / `innovation-direction`; use an **8-section PR-FAQ template** for `product-requirements` (BLUF = 段1 PR-FAQ 自身, no separate chapter index). Product-requirements also uses `decision-closure.md` and `chinese-product-report-structure.md`; users do not need a separate `/humanizer-zh` call.
13. **Quality-floor self-verification** (rubric floor incl. prose floor + product-requirements evidence gates) → mark warnings or abstain if below bar.

### Claude Code MCP direct invocation contract

When calling Claude Code MCP tools such as `mcp__moeresearch__get_runtime_capabilities`, `mcp__moeresearch__deep_research`, or `mcp__moeresearch__aspect_research`, pass the MoeResearch request object as the tool arguments directly. Do not include the outer JSON-RPC `tools/call` wrapper and do not wrap the request under `params`, `arguments`, `request`, `input`, or `tool_input`.

Provider API keys, Authorization headers, base URLs, cookies, JWTs, and provider-native request bodies must never appear in Skill payloads. Use provider names only; Rust config/env resolves secrets.

PM runtime reminders:

- Use `deep_research` for multi-aspect PM research; use `aspect_research` only for a single targeted retry.
- For search-enabled aspects, `instructions` is the selected Layer 2 persona, then `../prompts/layer1/common/model-search-tool-contract.md` (Claude install: `./prompts/layer1/common/model-search-tool-contract.md`), then a Run Binding projected from the aspect and `policy.search`.

## Direct MCP payloads

Use `skills/deep-research.md`'s payload skeleton after it selects `limits_preset`, resolves explicit user resource constraints, and tightens against `operator_limits` from the live capabilities snapshot. An `aspect_research` retry reuses the parent per-aspect row only for transient failures; it must repair the exhausted dimension or narrow scope after `budget_exceeded`.

Response contract:

- After a direct MCP tool call, read the stable MoeResearch payload from `result.structuredContent`.
- Treat `schema_validation_failed` as a Layer 1 request/prompt bug. Common causes include an unexpected model `evidence` field, an unknown or duplicate `selected_evidence` ID, a finding reference that is not selected, unsupported finding enums, or per-aspect `limits.timeout_ms` exceeding top-level `limits.total_timeout_ms`.

### Product-requirements module order

For `product-requirements`, keep the 8-段 PR-FAQ skeleton as the report contract. These modules run inside synthesis in this order:

| Order | Module | Owns | Final placement |
|---|---|---|---|
| 1 | `../prompts/layer1/common/evidence-postprocess.md` + overlay section in `../prompts/layer1/pm-deep-research/evidence-modules-overlay.md` | Shared source tiering / visual / CiteEval, then PM report-placement rules | Inputs to Annex A and verifier |
| 2 | `../prompts/layer1/common/claim-ledger.md` + overlay section | Shared claim extraction, then PM load-bearing defaults / Annex placement | Annex A.1 + A.6 coverage |
| 3 | `../prompts/layer1/common/host-verification-backfill.md` + overlay section | Shared HV contract, then PM annex placement | `HV-*` rows; A.6/A.8 disclosure; optional Claim Ledger links |
| 4 | `../prompts/layer1/common/evidence-verifier.md` + overlay section | Shared verification steps, then PM decision gates | Updated Claim Ledger + A.6 verifier summary |
| 5 | `../prompts/layer1/pm-deep-research/decision-closure.md` | P0/P1 assumptions, cheapest tests, kill criteria, success / guardrail metrics | 段8 summary + Annex A.4/A.5/A.6 |
| 6 | `../prompts/layer1/pm-deep-research/chinese-product-report-structure.md` | Professional Chinese product-report writing rules | Body prose quality; never deletes honesty markers |

These are Skill-layer synthesis modules, not MoeResearch aspects and not Rust schema requirements.

## Assets

### Layer 1 (orchestration)

- `../prompts/layer1/pm-deep-research/task-decomposition.md` · `agent-allocation.md` · `final-report.md` — competitive variant.
- `../prompts/layer1/pm-deep-research/task-decomposition-product-capability.md` · `agent-allocation-product-capability.md` · `final-report-product-capability.md` — product-capability variant.
- `../prompts/layer1/pm-deep-research/task-decomposition-innovation-direction.md` · `agent-allocation-innovation-direction.md` · `final-report-innovation-direction.md` — innovation-direction variant.
- `../prompts/layer1/pm-deep-research/task-decomposition-product-requirements.md` · `agent-allocation-product-requirements.md` · `final-report-product-requirements.md` — product-requirements variant (8-section PR-FAQ template).
- `../prompts/layer1/common/model-search-tool-contract.md` — appended after every selected Layer 2 persona and before that aspect's request-specific Run Binding; defines the model-only `query` + optional `max_results` + required semantic `intent` search protocol and host-owned evidence selection rules.
- `../prompts/layer1/common/evidence-postprocess.md` — shared evidence step (4-tier mapping / visual-evidence assembly / CiteEval). Domain-neutral only.
- `../prompts/layer1/common/claim-ledger.md` — shared claim audit module. Domain-neutral only.
- `../prompts/layer1/common/host-verification-backfill.md` — shared bounded host WebSearch/WebFetch verification/backfill contract; keeps host sources separate from MoeResearch evidence.
- `../prompts/layer1/common/evidence-verifier.md` — shared support / contradiction / freshness / independence / academic audit module.
- `../prompts/layer1/pm-deep-research/evidence-modules-overlay.md` — PM-only task-specific rules applied after each matching common evidence module (PR-FAQ / Annex placement / P0-P1 gates). Not used by academic, technical, or generic profiles.
- `../prompts/layer1/pm-deep-research/decision-closure.md` — assumptions / cheapest test / kill criterion / guardrails module.
- `../prompts/layer1/pm-deep-research/chinese-product-report-structure.md` — built-in Chinese product report structure and de-AI writing rules; no separate `/humanizer-zh` call.

### Layer 2 (persona)

- `../prompts/layer2/pm-deep-research/persona-experience-analyst.md` — capability matrix / Kano / experience paths / JTBD half.
- `../prompts/layer2/pm-deep-research/persona-strategist.md` — real competitive set / ODI / positioning / threat / build-cost.

Layer 2 personas are shared across all four capabilities.

## Policy boundaries (inherited from MoeResearch)

- Layer 1 may plan, allocate, validate, synthesize; it must not call Exa/Grok/model APIs directly when MoeResearch MCP is available.
- Host-native WebSearch/WebFetch is allowed only for bounded post-MoeResearch verification/backfill under `../prompts/layer1/common/host-verification-backfill.md`; it is not an alternate research engine and must be disclosed separately.
- Rust never reads prompt files at runtime; Layer 1 loads the chosen Layer 2 prompt Markdown, appends the common search-tool contract, then appends a request-specific `moe.run_binding.v1` Run Binding for each search-enabled aspect and passes the combined content inline as `AspectRequest.instructions` (non-empty, <64 KiB).
- Default PM `policy.search.category` is null. If a study fixes a category or another intent ceiling, use the same profile-neutral Run Binding projection; do not create a PM-specific exception.
- `SearchPolicy.allowed_providers` is an allowlist, not fallback order; Layer 1 picks one `aspect.search_provider`.
- Provider keys/base URLs/operator limits/raw DTOs stay behind MoeResearch config.

## Failure handling

Apply `../prompts/layer1/common/partial-status-host-contract.md` (Claude install: `./prompts/layer1/common/partial-status-host-contract.md`).

PM profile note: for `deep_research` partial, run **one** targeted `aspect_research` retry per failed aspect only when the partial-status contract classifies a transient or repaired retry as feasible. A `budget_exceeded` retry must widen the exhausted dimension within explicit user constraints and operator ceilings, or narrow scope; it must never repeat the same exhausted limits. Do not copy the full five-rule table into this skill.
