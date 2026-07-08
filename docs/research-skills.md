# MoeResearch Research Skills

MoeResearch has one Rust MCP backend and one Markdown asset installer. The asset slug is `research-skills`; the installed Markdown assets include PM DeepResearch, Academic DeepResearch, Technical Evaluation, and common evidence modules.

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
| Common evidence modules | Evidence tiering, claim ledger, host verification, verifier, annex. | `prompts/layer1/common/` |

## Installation

```bash
moeresearch assets install research-skills
```

The content is the full MoeResearch research skill asset set.

## Claude Code Layout

Default install target:

```text
~/.claude/skills/deep-research/
  SKILL.md
  prompts/
    layer1/
      common/
      pm-deep-research/
      academic-deep-research/
      technical-evaluation/
    layer2/
      pm-deep-research/
      academic-deep-research/
      technical-evaluation/
```

Claude Code receives one discoverable `deep-research` skill directory for the unified research entry and simplicity. Academic and technical profiles are installed as prompts inside that directory and are selected by explicit profile instructions or routing guidance; they are not installed as separate sibling Skill directories in the first release. The unified routing reference in repo/manual layout is `skills/deep-research.md`.

## Repo Layout

For non-Claude clients or manual loading:

```bash
moeresearch assets install research-skills \
  --target ~/.config/moeresearch/assets \
  --layout repo
```

This preserves sibling `skills/` and `prompts/` directories.

## Evidence and Verification Model

- MoeResearch evidence IDs and provenance are immutable.
- Host WebSearch/WebFetch verification uses separate `HV-*` rows.
- Manual/browser/local inspection stays separate from MoeResearch evidence.
- Unsupported load-bearing claims should be narrowed, downgraded, moved to open questions, or abstained.

## Choosing a Profile

- Product / competitor / PRD / roadmap / JTBD / market entry → PM DeepResearch.
- Paper / literature / citation / evidence synthesis / research gap / methodology → Academic DeepResearch.
- Library / framework / architecture / dependency / migration / benchmark / security evaluation → Technical Evaluation.
- Other multi-aspect research → generic deep-research.

## Troubleshooting

If MCP tools are missing, register the backend with `moeresearch mcp register`. If prompts are missing, install the Markdown assets with `moeresearch assets install research-skills` for the same `moeresearch` version.
