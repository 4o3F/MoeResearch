# Layer 1 Common Module: Partial-Status Host Contract

Frozen MoeResearch MCP envelope semantics for Layer 1 hosts. This module is the **shipped** single source of truth for skill/runtime hosts after `moeresearch assets install research-skills`.

Do not reinterpret fields or invent a second envelope shape. Schema 0.2 intentionally keeps deep vs aspect partial asymmetry.

## Read path

- Claude Code direct tools: read `result.structuredContent`.
- Raw MCP clients: unwrap the `tools/call` result payload, then apply the same envelope rules (request object is not nested under extra wrappers for Claude Code direct calls).

## Rules

1. **Hard fail** (`status=failed`, no usable partial payload; codes such as `provider_unavailable`, `network_failed`, process/tool missing): surface the stable error code, `retryable`, and the smallest safe next action. Stop. There is no host-only fallback for MoeResearch execution.
2. **`deep_research` partial** (`status=partial`, `data` present, **`error` is null**): this is not full success. Keep completed aspects from `data` (`completed_aspects` / `aspect_reports`). Treat each entry in `data.failed_aspects` as a gap. For each failed aspect, run **at most one** targeted `aspect_research` retry with the same aspect plan and inline instructions. Then write a partial report that lists remaining gaps.
3. **`aspect_research` partial** (`status=partial`, **`data` and `error` both set**): keep frozen evidence in `data`. Do not discard `data` because `error` is present. Retry at most once if `error.retryable` is true and the failure is not a Layer-1 schema/prompt bug (`schema_validation_failed` → fix request/prompt, do not blind-retry).
4. **`allow_partial_results=false`**: expect hard `failed` with no partial payload; do not invent a partial report.
5. **Insufficient evidence after success/partial**: do not invent conclusions; emit a gap list and follow-up searches.

## Envelope table

| Case | `status` | `data` | `error` | Host must |
| --- | --- | --- | --- | --- |
| `deep_research` partial | `partial` | present (`completed_aspects`, `aspect_reports`, `failed_aspects`, `evidence_index`, …) | **null** | Keep completed aspects; treat `failed_aspects[]` as gaps; at most one `aspect_research` retry per failed aspect |
| `aspect_research` partial | `partial` | present (frozen evidence; findings usually empty) | **present** | Use `data`; do not treat as pure hard-fail discard; inspect `error.retryable` / code for one retry |
| Partials disabled (`allow_partial_results=false`) | `failed` | null / no partial payload | present | Stop; no partial report path |
| Hard transport/config failure | `failed` | null | present | Surface stable code; no host-only substitute for MoeResearch execution |

## Profile notes

- Default profiles (generic, academic, technical): follow rules 1–5 as written (`at most one` retry).
- PM profile: same contract; deep partial retries are **required once** per failed aspect when a retry is feasible (same plan + instructions), not optional.

## Paths after install

- Repo / skill-relative load: `../prompts/layer1/common/partial-status-host-contract.md` from a skill file next to `skills/`.
- Claude Code install layout: `./prompts/layer1/common/partial-status-host-contract.md` inside `~/.claude/skills/deep-research/`.
