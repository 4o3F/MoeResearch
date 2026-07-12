# Layer 1 Prompt: Agent Allocation (Competitive — PM DeepResearch)

> Canonical mapping reference consumed by [`task-decomposition.md`](task-decomposition.md). It defines, for competitive deep research: five-dim spine → aspect → persona prompt, the per-tier aspect subset, the Build-intent version-history aspect, and the persona/TM rationale.

## Two personas (each supplies one persona portion of `instructions`)

MoeResearch has no persona concept; a persona is realised as the selected inline prompt within `task.aspects[].instructions`. For every search-enabled aspect, Layer 1 assembles the selected persona prompt, then `prompts/layer1/common/model-search-tool-contract.md`, then a request-specific Run Binding. There are exactly two persona prompts, both carrying the cross-cutting quality gates TM-4 (epistemic tagging) + TM-11 (falsifiability):

| key | file | angle | owns dims | TM |
|---|---|---|---|---|
| `experience-analyst` | [`../../layer2/pm-deep-research/persona-experience-analyst.md`](../../layer2/pm-deep-research/persona-experience-analyst.md) | user / experience / evidence | 2, 3, experience paths, JTBD half | TM-1/2/6/10/12 |
| `strategist` | [`../../layer2/pm-deep-research/persona-strategist.md`](../../layer2/pm-deep-research/persona-strategist.md) | strategy / trade-off / foresight | 1 (framing), 4, 5, threat, build-cost | TM-3/5/7/8/9/13 (+ TM-12 borrowed for build-cost) |

> CI/Market absorption: there is no separate Competitive-Intelligence or Market analyst. CI work (competitive map, feature matrix) lands in the experience-analyst teardown + strategist positioning/threat; market context is a strategist input (Porter at industry layer only). Add a separate CI/Market persona only after validating the need.

## Five-dim spine → aspect → persona

| aspect_id | spine dim | persona | research_question (template) | evidence standard → `success_criteria` |
|---|---|---|---|---|
| `job-and-competitive-set` | 1 | **strategist** | What job do users hire this product for, and by that job who is the real competitive set (incl. non-obvious substitutes)? | explicit job statement (situation→motivation→outcome) + ≥1 non-obvious substitute with inclusion reason |
| `capability-and-importance` | 2 + 3 | experience-analyst | How do target vs competitors compare on buyer-relevant capabilities, and which are Must-be / Performance / Attractive (Kano)? | every matrix cell has inline evidence or is marked assumption; Kano grades rest on user evidence or are tagged practitioner interpretation (TM-4) |
| `opportunity-gaps` | 4 (ODI) | strategist | What is each desired outcome's Importance / Satisfaction, and the ODI opportunity ranking? | Importance/Satisfaction sourced first-party, else estimated + TM-4; `Opportunity = Importance + max(0, Importance − Satisfaction)` computed |
| `positioning-whitespace` | 5 (+ threat grading) | strategist | On buyer-validated axes, what is each player's value curve, where is the whitespace, and which threats are sustaining vs disruptive? | axes are buyer-validated purchase dimensions (not invented); whitespace has a "why unoccupied" reason; per-competitor sustaining/disruptive call |
| `experience-paths` | 2 deepened | experience-analyst | Where are the experience breakpoints on the core paths (entry / operation / feedback / retention / conversion), backed by visual evidence? | each conclusion backed by a visual-evidence item (screenshot/video/app-store url); gaps with no media url go to `open_questions` |
| `build-cost-version-history` | iteration velocity | strategist | How fast and on what do competitors actually ship (changelog/version history), and what does that imply about build-cost for the target capability? | traceable version timeline (App Store/Play version history, official release notes); inferred investment priority; build-cost estimate; evidence url = version-history page |

### W3 — dim-1 persona ownership (do not re-litigate)

One MoeResearch aspect carries exactly one `instructions` persona prompt, so the "Strategist frames + Experience does JTBD" split cannot be literally applied inside one aspect. **`job-and-competitive-set` is owned by the `strategist` persona, with the JTBD job-statement work folded into its `question` + `success_criteria`.** Only split a dedicated `jtbd-jobs` aspect (owned by `experience-analyst`) when a study genuinely needs a standalone JTBD teardown — otherwise keep the single strategist-owned aspect.

## Per-tier aspect subsets

| tier | aspects | rationale |
|---|---|---|
| `quick` | `job-and-competitive-set`, `capability-and-importance` | fastest defensible read: who's the real competitor + how do capabilities compare |
| `standard` | + `opportunity-gaps`, `positioning-whitespace` (4 total) | adds gap ranking + positioning |
| `deep` / `deep_evidence_pack` | + `experience-paths` (5 total; + per-competitor profile on demand) | adds experience-breakpoint + visual evidence |

**Build-intent overlay (any tier where `decision_intent = build`)**: append `build-cost-version-history` (strategist). Judging "should we build X" requires a build-cost estimate from competitor iteration cadence, not just user value. Treat the changelog as the competitor's *deeds* (TM-12 say-vs-do): what they ship, how many versions, how often, reveals true investment priority.

## Limits per aspect (hand off to `task-decomposition.md` Step 4)

Each aspect carries its own `limits { max_turns, max_tool_calls, max_search_calls, timeout_ms }`. The discriminating values per tier are: per-aspect `max_search_calls` = 3 (quick) / 8 (standard) / 8 (deep); per-aspect `timeout_ms` = **600000 always**. Top-level `limits` and `total_timeout_ms = ceil(max_agents / max_concurrent_agents) × 600000` are computed in `task-decomposition.md` Step 4.

## Provider selection per aspect

`model_provider` and `search_provider` come from the user's configured allowlists (`available_*_providers`). Guidance:
- **Entity-discovery-heavy** aspects (`job-and-competitive-set`, `positioning-whitespace`) favour a semantic-discovery search provider (e.g. `exa`) to surface non-obvious substitutes and real players.
- **Synthesis** aspects (`capability-and-importance`, `opportunity-gaps`, `experience-paths`, `build-cost-version-history`) default to the configured synthesis provider (e.g. `grok`).
- If only one search provider is configured, use it for every aspect. `allowed_providers` is an allowlist, not a fallback order.

## Invariants

1. Each search-enabled aspect → exactly one persona prompt, then `prompts/layer1/common/model-search-tool-contract.md`, then a request-specific Run Binding, passed inline (non-empty, < 64 KiB).
2. Aspects are MECE across the spine — no dimension covered twice.
3. `success_criteria` carries the dimension's evidence standard so the engine enforces our evidence bar.
4. `decision_intent` lives in `context.summary` (the aspect agents read it there).
5. Evidence source type and evidence-level confidence are host-owned after candidate selection. The 4-tier credibility labels are Skill post-processing, never model-output fields.

## Run Binding handoff

For every search-enabled aspect, persona selection is followed by the complete inline assembly order: selected persona Markdown, then `prompts/layer1/common/model-search-tool-contract.md`, then the request-specific Run Binding. The binding is derived from that aspect and `policy.search` according to `moe.run_binding.v1`; it carries only semantic `allowed_*` intent choices, safe defaults, literal aspect ID/name anchors, and evidence-closure hints. It must not expose provider routing, budgets, domains, raw policy tool fields, or credentials.

This three-part order is mandatory for every search-enabled aspect. The fixed-category rule is profile-neutral: fixed `academic` permits `general` or `academic`; an unset category permits the full source-focus vocabulary.
