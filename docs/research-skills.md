# MoeResearch Research Skills

MoeResearch has one Rust MCP backend and one Markdown asset installer. The asset slug is `research-skills`; the installed Markdown assets include Generic DeepResearch roots, PM DeepResearch, Academic DeepResearch, Technical Evaluation, common evidence modules, and a common model-search tool contract.

## Backend vs Markdown Skill Assets

- `moeresearch mcp register` registers the stdio MCP backend only.
- `moeresearch assets install research-skills` installs client-side Skill and prompt Markdown assets.
- Rust/MCP schemas stay domain-neutral. Profiles live in Skill and prompt assets.

## Installed Skill Profiles

| Profile | Use for | Prompt roots |
|---|---|---|
| PM DeepResearch | Product, competitor, roadmap, PRD, market, innovation research. | `prompts/layer1/pm-deep-research/`, `prompts/layer2/pm-deep-research/` |
| Academic DeepResearch | Literature review, evidence synthesis, paper evaluation, research gaps. | `prompts/layer1/academic-deep-research/`, `prompts/layer2/academic-deep-research/` |
| Technical Evaluation | Library/framework comparison, architecture evaluation, dependency risk, migration assessment. | `prompts/layer1/technical-evaluation/`, `prompts/layer2/technical-evaluation/` |
| Generic DeepResearch | Multi-aspect research that does not fit PM/academic/technical. | `prompts/layer1/task-decomposition.md`, `prompts/layer1/final-report.md`, `prompts/layer2/aspect-agent.md` |
| Common modules | Evidence tiering, claim ledger, host verification, verifier, annex, Typst report contract, partial-status host contract, budget tiers, model-search tool contract. | `prompts/layer1/common/` |

Academic and Technical final-report routes also load their profile `evidence-modules-overlay.md` and `final-report-guidance.md`, then emit a `typst-project-v1` source project. PM and Generic retain Markdown delivery until separately migrated.

The Rust backend exposes `get_runtime_capabilities`, `aspect_research`, and `deep_research`. Before request assembly, Layer 1 calls the read-only capabilities tool once per job for live provider names and operator ceilings; this snapshot remains Layer-1-only and must never enter personas, `instructions`, free-text `context`, or Run Binding.

Optional Generic Layer-2 helpers (not required when `aspect-agent.md` is inlined): `prompts/layer2/search-planner.md`, `prompts/layer2/evidence-extractor.md`.

For every search-enabled aspect, Layer 1 assembles `AspectRequest.instructions` as selected Layer 2 persona, then `prompts/layer1/common/model-search-tool-contract.md`, then a request-specific Run Binding (`moe.run_binding.v1`). The contract defines the shared model-only retrieval protocol; the binding projects only safe semantic `allowed_*` intent values plus literal aspect identity and evidence-closure hints. Neither adds fields to public MCP request schema 0.2.

## Model Retrieval Intent Contract

All profiles use the same logical `search` call:

```json
{
  "query": "string",
  "max_results": 5,
  "intent": {
    "source_focus": "general | organizations | people | academic | news | personal_sites | financial_filings | code",
    "timeliness": "any | stable | recent | fresh | live",
    "coverage": "focused | balanced | broad",
    "detail": "compact | standard | detailed"
  }
}
```

`intent` is required for model tool calls only. It is not a `DeepResearchRequest`, `AspectResearchRequest`, or `policy.search` field. Rust resolves it against exactly one selected provider and host policy, then returns `intent_resolution` for each dimension as `enforced`, `best_effort`, or `unsupported`. Profiles must not replace this protocol with raw `category`, `depth`, `content_level`, `recency`, or provider-native arguments.

Candidate IDs are host-generated as `ev-<search_turn>-<global_candidate_index>`; the second component is global across prior successful results, not a per-turn result position. The model must copy literal `results[].id` values, then set `selected_evidence` to the unique union of all `finding.evidence_refs`; it must not reconstruct IDs or return `evidence` objects. The host rehydrates immutable provenance and derives `supports_findings`; Skill-side post-processing consumes the returned host evidence without mutating it.

## Installation

```bash
moeresearch assets install research-skills --config /path/to/moeresearch.toml
```

Remote asset installation uses the complete MoeResearch configuration, including the optional network proxy. Omit `--config` only when a valid `moeresearch.toml` is available in the current working directory. The content is the full MoeResearch research skill asset set. Re-run this command after upgrading MoeResearch so installed skills receive the matching common model-search tool contract and Run Binding guidance; stale assets may omit the binding.

## Claude Code Layout

Default install target:

```text
~/.claude/skills/deep-research/
  SKILL.md
  prompts/
    layer1/
      task-decomposition.md
      final-report.md
      common/
        model-search-tool-contract.md
      pm-deep-research/
      academic-deep-research/
      technical-evaluation/
    layer2/
      aspect-agent.md
      search-planner.md
      evidence-extractor.md
      pm-deep-research/
      academic-deep-research/
      technical-evaluation/
```

Claude Code receives one discoverable `deep-research` skill directory for the unified research entry and simplicity. Profile prompts (PM, academic, technical) and Generic root prompts are installed inside that directory and are selected by explicit profile instructions or routing guidance; they are not installed as separate sibling Skill directories in the first release. The unified routing reference in repo/manual layout is `skills/deep-research.md`.

## Repo Layout

For non-Claude clients or manual loading:

```bash
moeresearch assets install research-skills \
  --config /path/to/moeresearch.toml \
  --target ~/.config/moeresearch/assets \
  --layout repo
```

This preserves sibling `skills/` and `prompts/` directories.

## Evidence and Verification Model

- MoeResearch evidence IDs and host-rehydrated provenance are immutable after the runtime returns them.
- Host WebSearch/WebFetch verification uses separate `HV-*` rows.
- Manual/browser/local inspection stays separate from MoeResearch evidence.
- Unsupported load-bearing claims should be narrowed, downgraded, moved to open questions, or abstained.

## Choosing a Profile

- Product / competitor / PRD / roadmap / JTBD / market entry → PM DeepResearch.
- Paper / literature / citation / evidence synthesis / research gap / methodology → Academic DeepResearch.
- Library / framework / architecture / dependency / migration / benchmark / security evaluation → Technical Evaluation.
- Other multi-aspect research → generic deep-research.

After `moeresearch assets install research-skills`, Generic prompts (`task-decomposition.md`, `final-report.md`, `aspect-agent.md`) must be present under the skill prompts tree. If they are missing, reinstall assets for the matching `moeresearch` binary version.

## Troubleshooting

If MCP tools are missing, register the backend with `moeresearch mcp register`. If prompts are missing, install the Markdown assets with `moeresearch assets install research-skills` for the same `moeresearch` version.
