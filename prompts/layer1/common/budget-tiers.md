# Layer 1 Common Module: Budget Tiers

Canonical Layer-1 budget tiers for MoeResearch request assembly. `skills/deep-research.md` selects `limits_preset`; profiles only apply it.

Operator config still applies stricter-merge at the Rust core: a finite operator ceiling wins over a more generous request; `-1` does not override a finite peer.

## Tiers

| Tier | Top-level `limits` (`deep_research`) | Per-aspect `task.aspects[].limits` / `task.limits` | `min_evidence_per_finding` |
| --- | --- | --- | ---: |
| `quick` | `max_agents` 2, `max_concurrent_agents` 1, `max_total_model_calls` 12, `max_total_search_calls` 8, `total_timeout_ms` 300000, `max_tokens` -1 | `max_turns` 4, `max_tool_calls` 4, `max_search_calls` 2, `timeout_ms` 180000 | 1 |
| `standard` | `max_agents` 4, `max_concurrent_agents` 2, `max_total_model_calls` 40, `max_total_search_calls` 28, `total_timeout_ms` 600000, `max_tokens` -1 | `max_turns` 10, `max_tool_calls` 12, `max_search_calls` 8, `timeout_ms` 600000 | 1 |
| `deep` | `max_agents` 6, `max_concurrent_agents` 3, `max_total_model_calls` 70, `max_total_search_calls` 56, `total_timeout_ms` 1260000, `max_tokens` -1 | `max_turns` 8, `max_tool_calls` 8, `max_search_calls` 4, `timeout_ms` 600000 | 2 |

PM `evidence_pack` is a deep-only report/evidence-audit overlay; it never changes limits.

## Rules

1. Copy the selected row's top-level limits, per-aspect limits, and evidence minimum together; always require evidence for findings.
2. Keep finite per-aspect `timeout_ms` ≤ `total_timeout_ms` and `max_concurrent_agents` ≤ `max_agents`.
3. Aspect-only retries use the selected per-aspect row. Do not put these values in the Run Binding.

## Paths after install

- Repo / skill-relative: `../prompts/layer1/common/budget-tiers.md`
- Claude Code install layout: `./prompts/layer1/common/budget-tiers.md`
