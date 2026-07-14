# Layer 1 Common Module: Budget Tiers

Canonical Layer-1 budget baselines for MoeResearch request assembly. Layer 1 selects `limits_preset` and applies explicit resource constraints in the user prompt when assembling request limits.

Budget precedence is:

```text
operator ceiling > explicit user override > selected preset
```

The selected preset is a baseline, not permission to ignore the user. An explicit user constraint may narrow or widen the relevant request dimensions. Layer 1 then only tightens the resolved request against `operator_limits` from `get_runtime_capabilities`: an unlimited operator value leaves the request unchanged; a finite operator value takes the stricter value. Runtime stricter-wins merging remains authoritative.

`-1` means that the request layer adds no cap. It never disables a finite operator ceiling. Re-check finite `max_concurrent_agents <= max_agents` and aspect `timeout_ms <= total_timeout_ms`. If operator ceilings make explicit success criteria impossible, disclose the conflict and reduce scope with the user or stop; never silently drop criteria or silently replace the user's resource request.

## Tiers

| Tier | Top-level `limits` (`deep_research`) | Per-aspect `task.aspects[].limits` / `task.limits` |
| --- | --- | --- |
| `quick` | `max_agents` 2, `max_concurrent_agents` 1, `max_total_model_calls` 12, `max_total_search_calls` 8, `total_timeout_ms` 300000, `max_tokens` -1 | `max_turns` 4, `max_tool_calls` 4, `max_search_calls` 2, `timeout_ms` 180000 |
| `standard` | `max_agents` 4, `max_concurrent_agents` 2, `max_total_model_calls` 72, `max_total_search_calls` 28, `total_timeout_ms` 600000, `max_tokens` -1 | `max_turns` 10, `max_tool_calls` 16, `max_search_calls` 8, `timeout_ms` 600000 |
| `deep` | `max_agents` 6, `max_concurrent_agents` 3, `max_total_model_calls` 180, `max_total_search_calls` 144, `total_timeout_ms` 3600000, `max_tokens` -1 | `max_turns` 16, `max_tool_calls` 24, `max_search_calls` 12, `timeout_ms` 1200000 |

Dual-tool baselines reserve one generic tool-call slot for verification per search-call slot: Quick allows `2 search + 2 web_fetch`, Standard allows `8 + 8`, and Deep allows `12 + 12`. WebFetch prompt processing also consumes a research model call. The Standard model-call cap therefore allows four aspects using all `10` agent turns plus up to `28` verification calls, with four calls of headroom. The Deep cap covers six aspects using all `16` turns plus one verification call for each of their `12` searches (`168` calls total). Quick covers two aspects using all `4` turns plus two verification calls each (`12` calls total). The deep global search cap still leaves headroom above six aspects using their full per-aspect search allowance: `6 × 12 = 72`, below `144`. Two full concurrency waves at the per-aspect timeout consume `2400000 ms`, below the `3600000 ms` research deadline. Operator configuration remains the deployment hard ceiling.

PM `evidence_pack` is a deep-only report/evidence-audit overlay; it never changes limits.

## Explicit user prompt constraints

Layer 1 maps explicit resource wording in the user prompt directly to the normal `limits` fields when assembling the request. It does not add a request field, put resource controls in `context`, or expose them to Layer 2 instructions or Run Binding.

Use these normalization rules:

1. Explicit numeric limits such as “at most 10 searches per aspect” set the corresponding request dimension exactly before operator tightening.
2. Explicit “do not limit search rounds”, “unlimited searches”, or equivalent wording sets `max_total_search_calls = -1` and, for every search-enabled aspect, `max_search_calls = -1` plus `max_tool_calls = -1`. Keep model-call and timeout dimensions at the selected preset unless the user explicitly relaxes those too.
3. Explicit cost, time, concurrency, agent-count, token, model-call, or tool-call constraints map only to their existing request limit dimensions. Do not infer unrelated widening.
4. If wording is genuinely ambiguous and choosing incorrectly materially changes cost or duration, ask one focused clarification. Otherwise preserve the narrowest literal interpretation and state any remaining finite limits.
5. Never convert “deep”, “thorough”, or “comprehensive” alone into unlimited execution; those signals select the deep baseline. Unlimited request dimensions require explicit no-cap language.

## Rules

1. Load the selected row's top-level and per-aspect limits.
2. Apply explicit user prompt constraints to the relevant dimensions; do not let a preset silently overwrite them.
3. Only tighten the resolved request against `operator_limits`; always require evidence for findings.
4. Keep finite per-aspect `timeout_ms` ≤ `total_timeout_ms` and finite `max_concurrent_agents` ≤ `max_agents`.
5. For `budget_exceeded`, do not retry with the same exhausted limits. Widen the exhausted request dimension only when allowed by explicit user constraints and operator ceilings, or narrow the aspect scope; otherwise disclose the gap.
6. Do not put presets, overrides, effective limits, or operator capabilities in Run Binding.

## Paths after install

- Repo / skill-relative: `../prompts/layer1/common/budget-tiers.md`
- Claude Code install layout: `./prompts/layer1/common/budget-tiers.md`
