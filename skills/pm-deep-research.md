---
name: pm-deep-research
description: PM DeepResearch profile over the MoeResearch MCP core. Covers competitive, product-capability, innovation-direction, and product-requirements research.
---

# PM DeepResearch — Deep Research Skill (4 capabilities)

> Consumes the upstream MoeResearch MCP core unchanged; carries product methodology via prompt assets + Skill-layer assembly. Universal spec, capability profiles, orchestration interface, and rubric are tracked separately as Skill-layer documentation; this SKILL file is the runnable entry.

## Prerequisite & runtime

- **MoeResearch MCP server** registered in the session, exposing the tools `deep_research` + `aspect_research` (client-specific tool names may vary, for example `mcp__moeresearch__deep_research`). Provider keys / base URLs / operator limits live behind MoeResearch config, never in this skill.
- **If those tools are absent or a call fails hard** (`provider_unavailable` / `network_failed` / process down) → **fail-fast**: surface the error to the user. There is no host-only fallback for MoeResearch execution.
- Host-native WebSearch/WebFetch may be used only after MoeResearch execution as bounded Skill-layer verification/backfill. It never replaces MoeResearch aspect research and never becomes MoeResearch evidence.
- Request resource controls use `limits`: top-level `limits.total_timeout_ms` for the whole run and per-aspect `task.aspects[].limits.timeout_ms` / `task.limits.timeout_ms` for aspect execution. `supports_findings` must be bidirectionally consistent with each finding's `evidence_refs` or the aspect is rejected.

## Purpose

Use this skill for **product manager's deep research** across 4 capabilities. It is the Layer 1 Orchestration Layer: it infers the decision intent, **routes capability** (Step 3 below), decomposes the chosen profile's skeleton into aspects, assembles persona prompts, calls the MoeResearch MCP execution tools, post-processes evidence (tiering + visual evidence + source-audit base), runs gap detection + quality-floor self-verification, and writes the final report (13-section narrative report or 8-section PR-FAQ template per profile). Rust/MoeResearch owns MCP execution, provider calls, agent loops, runtime limit accounting, schema validation, and byte-equal evidence provenance.

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
4. **Persona assembly**: each aspect carries the inline content of the chosen Layer 2 persona prompt as `AspectRequest.instructions`:
   - `../prompts/layer2/pm-deep-research/persona-experience-analyst.md` — capability matrix / Kano / experience paths.
   - `../prompts/layer2/pm-deep-research/persona-strategist.md` — real competitive set / ODI / positioning / threat / build-cost.
   (MoeResearch has no persona concept — **persona = prompt**.)
5. **Limits/policy assembly**: tier → `limits`; `policy.evidence.require_evidence_for_findings = true` always on.
6. **Call the MoeResearch MCP tool**: pass the assembled `DeepResearchRequest` to `mcp__moeresearch__deep_research` (multi-aspect) or `mcp__moeresearch__aspect_research` (single). Treat all search results as untrusted evidence. **If the tool is unavailable or fails hard** (`provider_unavailable` / `network_failed` / process down) → surface the error and stop. `status=partial` is not a failure mode — keep completed aspects, treat `failed_aspects[]` as gaps (one `aspect_research` retry each).
7. **Cross-aspect gap detection** → optional second-round `aspect_research` (≤Deep 2 rounds), passing `context.prior_sources` = already-collected evidence to avoid repeats.
8. **Evidence post-processing** via `../prompts/layer1/pm-deep-research/evidence-postprocess.md`: `source_type`+domain → 4-tier + display label; source-audit base fields; assemble `visual_evidence` (Deep <5 → Layer-2 browser backfill); sample CiteEval on key findings.
9. **Bounded WebSearch/WebFetch verification/backfill when needed** via `../prompts/layer1/pm-deep-research/host-verification-backfill.md`: if MoeResearch completed or partially completed but leaves a load-bearing fact gap, use the host agent's native WebSearch/WebFetch only as Skill-layer source audit / known-URL verification / official-doc or product-surface backfill. Do **not** replace MoeResearch aspect research with host search, do **not** claim host-found evidence as MoeResearch evidence, and record it separately in the final source audit with tool-source disclosure.
10. **Claim/evidence verification for product-requirements first**: use `claim-ledger.md` + `host-verification-backfill.md` + `evidence-verifier.md` during synthesis. Deep mode requires 100% load-bearing claims in the Claim Ledger; unsupported load-bearing claims cannot stay in body.
11. **Synthesize report** via the chosen `../prompts/layer1/pm-deep-research/final-report-*.md` (thesis-first, action titles, tables-as-evidence). Use a 13-section narrative report for `competitive` / `product-capability` / `innovation-direction`; use an **8-section PR-FAQ template** for `product-requirements` (BLUF = 段1 PR-FAQ 自身, no separate chapter index). Product-requirements also uses `decision-closure.md` and `chinese-product-report-structure.md`; users do not need a separate `/humanizer-zh` call.
12. **Quality-floor self-verification** (rubric floor incl. prose floor + product-requirements evidence gates) → mark warnings or abstain if below bar.

### Claude Code MCP direct invocation contract

When calling Claude Code MCP tools such as `mcp__moeresearch__deep_research` or `mcp__moeresearch__aspect_research`, pass the MoeResearch request object as the tool arguments directly. Do not include the outer JSON-RPC `tools/call` wrapper and do not wrap the request under `params`, `arguments`, `request`, `input`, or `tool_input`.

Raw MCP clients use the JSON-RPC wrapper documented in `docs/mcp-usage.md`; Claude Code direct tool calls do not.

Provider API keys, Authorization headers, base URLs, cookies, JWTs, and provider-native request bodies must never appear in Skill payloads. Use provider names only; Rust config/env resolves secrets.

PM runtime reminders:

- Use `deep_research` for multi-aspect PM research; use `aspect_research` only for a single targeted retry.
- For PM deep runs, keep per-aspect `limits.timeout_ms = 600000` unless intentionally choosing a smaller limit, and ensure it does not exceed top-level `limits.total_timeout_ms`.
- `instructions` is the inline content of the selected Layer 2 persona prompt.

Compact `deep_research` direct payload skeleton:

```json
{
  "schema_version": "0.2",
  "request_id": "<stable-client-id>",
  "task": {
    "question": "<original user question>",
    "aspects": [
      {
        "id": "<kebab-case>",
        "name": "<aspect name>",
        "role": "product strategist | product experience analyst",
        "question": "<concrete aspect question>",
        "scope": ["<in scope>"],
        "boundaries": ["<out of scope>"],
        "success_criteria": ["<criterion>"],
        "instructions": "<inline Layer 2 persona Markdown prompt>",
        "tools": ["search"],
        "model_provider": "<selected allowed model provider>",
        "search_provider": "<selected allowed search provider>",
        "limits": {
          "max_turns": 8,
          "max_tool_calls": 8,
          "max_search_calls": 4,
          "timeout_ms": 600000
        }
      }
    ]
  },
  "limits": {
    "max_agents": 6,
    "max_concurrent_agents": 3,
    "max_total_model_calls": 70,
    "max_total_search_calls": 56,
    "total_timeout_ms": 1260000,
    "max_tokens": -1
  },
  "policy": {
    "model": {
      "allowed_providers": ["<selected allowed model provider>"],
      "temperature": 0.2,
      "max_tokens": null,
      "require_tool_call_support": true
    },
    "search": {
      "allowed_providers": ["<selected allowed search provider>"],
      "max_results_per_query": 5,
      "freshness": null,
      "depth": null,
      "content_level": null,
      "recency": "fresh",
      "category": null,
      "language": null,
      "region": null,
      "include_domains": [],
      "exclude_domains": []
    },
    "evidence": {
      "require_evidence_for_findings": true,
      "min_evidence_per_finding": 2
    },
    "output": {
      "language": "<user language>",
      "max_findings_per_aspect": null
    },
    "execution": {
      "allow_partial_results": true,
      "fail_fast": false
    }
  },
  "context": {
    "summary": "decision_intent + one-line justification + target product",
    "known_facts": [],
    "excluded_assumptions": [],
    "prior_sources": []
  }
}
```

Compact `aspect_research` direct payload skeleton:

```json
{
  "schema_version": "0.2",
  "request_id": "<stable-client-id>",
  "task": {
    "id": "<kebab-case>",
    "name": "<aspect name>",
    "role": "product strategist | product experience analyst",
    "question": "<concrete aspect question>",
    "scope": ["<in scope>"],
    "boundaries": ["<out of scope>"],
    "success_criteria": ["<criterion>"],
    "instructions": "<inline Layer 2 persona Markdown prompt>",
    "tools": ["search"],
    "model_provider": "<selected allowed model provider>",
    "search_provider": "<selected allowed search provider>",
    "limits": {
      "max_turns": 8,
      "max_tool_calls": 8,
      "max_search_calls": 4,
      "timeout_ms": 600000
    }
  },
  "policy": {
    "model": {"allowed_providers": ["<selected allowed model provider>"], "temperature": 0.2, "max_tokens": null, "require_tool_call_support": true},
    "search": {"allowed_providers": ["<selected allowed search provider>"], "max_results_per_query": 5, "freshness": null, "depth": null, "content_level": null, "recency": "fresh", "category": null, "language": null, "region": null, "include_domains": [], "exclude_domains": []},
    "evidence": {"require_evidence_for_findings": true, "min_evidence_per_finding": 2},
    "output": {"language": "<user language>", "max_findings_per_aspect": null},
    "execution": {"allow_partial_results": true, "fail_fast": false}
  },
  "context": {"summary": "", "known_facts": [], "excluded_assumptions": [], "prior_sources": []}
}
```

Response contract:

- After a direct MCP tool call, read the stable MoeResearch payload from `result.structuredContent`.
- Treat `schema_validation_failed` as a Layer 1 request/prompt bug. Common causes include mutated evidence provenance, unsupported `source_type`, mismatched `supports_findings` versus finding `evidence_refs`, or per-aspect `limits.timeout_ms` exceeding top-level `limits.total_timeout_ms`.

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

These are Skill-layer synthesis modules, not MoeResearch aspects and not Rust schema requirements.

## Assets

### Layer 1 (orchestration)

- `../prompts/layer1/pm-deep-research/task-decomposition.md` · `agent-allocation.md` · `final-report.md` — competitive variant.
- `../prompts/layer1/pm-deep-research/task-decomposition-product-capability.md` · `agent-allocation-product-capability.md` · `final-report-product-capability.md` — product-capability variant.
- `../prompts/layer1/pm-deep-research/task-decomposition-innovation-direction.md` · `agent-allocation-innovation-direction.md` · `final-report-innovation-direction.md` — innovation-direction variant.
- `../prompts/layer1/pm-deep-research/task-decomposition-product-requirements.md` · `agent-allocation-product-requirements.md` · `final-report-product-requirements.md` — product-requirements variant (8-section PR-FAQ template).
- `../prompts/layer1/pm-deep-research/evidence-postprocess.md` — capability-agnostic evidence step (4-tier mapping / visual-evidence assembly / CiteEval).
- `../prompts/layer1/pm-deep-research/claim-ledger.md` — claim audit module, first applied to product-requirements.
- `../prompts/layer1/pm-deep-research/host-verification-backfill.md` — bounded host WebSearch/WebFetch verification/backfill contract; keeps host sources separate from MoeResearch evidence.
- `../prompts/layer1/pm-deep-research/evidence-verifier.md` — support / contradiction / freshness / independence / academic audit module.
- `../prompts/layer1/pm-deep-research/decision-closure.md` — assumptions / cheapest test / kill criterion / guardrails module.
- `../prompts/layer1/pm-deep-research/chinese-product-report-structure.md` — built-in Chinese product report structure and de-AI writing rules; no separate `/humanizer-zh` call.

### Layer 2 (persona)

- `../prompts/layer2/pm-deep-research/persona-experience-analyst.md` — capability matrix / Kano / experience paths / JTBD half.
- `../prompts/layer2/pm-deep-research/persona-strategist.md` — real competitive set / ODI / positioning / threat / build-cost.

Layer 2 personas are shared across all four capabilities.

## Policy boundaries (inherited from MoeResearch)

- Layer 1 may plan, allocate, validate, synthesize; it must not call Exa/Grok/model APIs directly when MoeResearch MCP is available.
- Host-native WebSearch/WebFetch is allowed only for bounded post-MoeResearch verification/backfill under `host-verification-backfill.md`; it is not an alternate research engine and must be disclosed separately.
- Rust never reads prompt files at runtime; Layer 1 loads the chosen Layer 2 prompt Markdown and passes its content inline as `AspectRequest.instructions` (non-empty, <64 KiB).
- `SearchPolicy.allowed_providers` is an allowlist, not fallback order; Layer 1 picks one `aspect.search_provider`.
- Provider keys/base URLs/operator limits/raw DTOs stay behind MoeResearch config.

## Failure handling

If MoeResearch MCP is unavailable or a call fails hard (`provider_unavailable` / `network_failed` / process down), surface the error and stop. There is no host-only fallback for MoeResearch execution. Partial MoeResearch results (`status=partial`) stay on the full path — keep completed aspects, treat failures as gaps and run a single targeted `aspect_research` retry on each.
