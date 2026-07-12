# Layer 1 Common Module: Budget Tiers

Canonical Layer-1 budget tiers for MoeResearch request assembly. Skills must pick a named tier and copy these numbers unless the user explicitly overrides them.

Operator config still applies stricter-merge at the Rust core: a finite operator ceiling wins over a more generous request; `-1` does not override a finite peer.

## Tiers

| Tier | Top-level `limits` (`deep_research`) | Per-aspect `task.aspects[].limits` / `task.limits` |
| --- | --- | --- |
| `quick` | `max_agents` 2, `max_concurrent_agents` 1, `max_total_model_calls` 12, `max_total_search_calls` 8, `total_timeout_ms` 300000, `max_tokens` -1 | `max_turns` 4, `max_tool_calls` 4, `max_search_calls` 2, `timeout_ms` 180000 |
| `standard` | `max_agents` 4, `max_concurrent_agents` 2, `max_total_model_calls` 40, `max_total_search_calls` 28, `total_timeout_ms` 600000, `max_tokens` -1 | `max_turns` 10, `max_tool_calls` 12, `max_search_calls` 8, `timeout_ms` 600000 |
| `deep` | `max_agents` 6, `max_concurrent_agents` 3, `max_total_model_calls` 70, `max_total_search_calls` 56, `total_timeout_ms` 1260000, `max_tokens` -1 | `max_turns` 8, `max_tool_calls` 8, `max_search_calls` 4, `timeout_ms` 600000 |

## Profile defaults

| Profile / skill | Default tier |
| --- | --- |
| Generic deep-research | `standard` |
| Academic DeepResearch | `standard` unless the user asks for deeper coverage |
| Technical Evaluation | `standard` unless the user asks for deeper coverage |
| PM DeepResearch | `deep` (use `standard` or `quick` only when the user wants a cheaper run) |

## Rules

1. Do not invent a fourth ad-hoc budget set in a profile skill.
2. When substituting tiers, change both top-level `limits` and per-aspect `limits` together.
3. Keep finite `task.*.limits.timeout_ms` ≤ finite `limits.total_timeout_ms`.
4. Keep finite `max_concurrent_agents` ≤ finite `max_agents`.
5. Aspect-only retries (`aspect_research`) use the same per-aspect row as the chosen tier.

## Paths after install

- Repo / skill-relative: `../prompts/layer1/common/budget-tiers.md`
- Claude Code install layout: `./prompts/layer1/common/budget-tiers.md`
