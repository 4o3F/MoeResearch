# Layer 1 Common Module: Partial-Status Host Contract

Frozen MoeResearch MCP envelope semantics for Layer 1 hosts. This module is the **shipped** single source of truth for skill/runtime hosts after `moeresearch assets install research-skills`.

Do not reinterpret fields or invent a second envelope shape. Schema 0.2 intentionally keeps deep vs aspect partial asymmetry.

## Read path

- Claude Code direct tools: read `result.structuredContent`.
- Raw MCP clients: unwrap the `tools/call` result payload, then apply the same envelope rules (request object is not nested under extra wrappers for Claude Code direct calls).

## Rules

1. **Hard fail** (`status=failed`, no usable partial payload; codes such as `provider_unavailable`, `network_failed`, process/tool missing): surface the stable error code, `retryable`, and the smallest safe next action. Stop. There is no host-only fallback for MoeResearch execution.
2. **`deep_research` partial** (`status=partial`, `data` present, **`error` is null**): this is not full success. Keep completed aspects from `data` (`completed_aspects` / `aspect_reports`) and treat each `data.failed_aspects` entry as a gap. Run at most one targeted `aspect_research` retry per failed aspect only after classifying the failure:
   - retry a transient failure when `retryable = true`;
   - for `budget_exceeded`, do not retry with the same exhausted limits—widen only the exhausted request dimension when explicit user constraints and operator ceilings allow it, or narrow the aspect scope;
   - for a Layer-1 schema/prompt defect such as `schema_validation_failed`, fix the request or instructions before retrying.
   If no repaired retry is feasible, preserve evidence and disclose the remaining gap.
3. **`aspect_research` partial** (`status=partial`, **`data` and `error` both set**): keep frozen evidence in `data`. Do not discard `data` because `error` is present. Retry at most once only after the same classification and repair rules from rule 2; never blind-retry a non-retryable failure with an unchanged request.
4. **`allow_partial_results=false`**: expect hard `failed` with no partial payload; do not invent a partial report.
5. **Insufficient evidence after success/partial**: do not invent conclusions; emit a gap list and use only a repaired, bounded follow-up allowed by the selected budget and explicit user constraints.

## Envelope table

| Case | `status` | `data` | `error` | Host must |
| --- | --- | --- | --- | --- |
| `deep_research` partial | `partial` | present (`completed_aspects`, `aspect_reports`, `failed_aspects`, `evidence_index`, …) | **null** | Keep completed aspects; classify each failed aspect; retry at most once only after transient/repaired-failure gating |
| `aspect_research` partial | `partial` | present (frozen evidence; findings usually empty) | **present** | Use `data`; inspect code/retryability; repair before one retry |
| Partials disabled (`allow_partial_results=false`) | `failed` | null / no partial payload | present | Stop; no partial report path |
| Hard transport/config failure | `failed` | null | present | Surface stable code; no host-only substitute for MoeResearch execution |

## Profile notes

- Default profiles (generic, academic, technical): follow rules 1–5 as written (`at most one` repaired retry).
- PM profile: same contract; one retry is required only when it is feasible after transient-failure gating or request repair. A same-cap `budget_exceeded` retry is never required.

## Paths after install

- Repo / skill-relative load: `../prompts/layer1/common/partial-status-host-contract.md` from a skill file next to `skills/`.
- Claude Code install layout: `./prompts/layer1/common/partial-status-host-contract.md` inside `~/.claude/skills/deep-research/`.
