# Academic DeepResearch

Academic DeepResearch is a Skill/profile layer over the MoeResearch MCP core for scholarly research workflows.

## Use Cases

- Literature reviews and field maps.
- Evidence synthesis across studies or guidelines.
- Paper evaluation and methodological critique.
- Research gap and future-work mapping.
- Study-design background research.

## Capabilities

`literature-review`, `evidence-synthesis`, `paper-evaluation`, `research-gap-analysis`, and `study-design-background`.

## Setup

```bash
moeresearch mcp register --scope local --config ~/.config/moeresearch/moeresearch.toml
moeresearch assets install research-skills --config ~/.config/moeresearch/moeresearch.toml
```

The research skills asset installs Academic DeepResearch prompts under `prompts/layer1/academic-deep-research/` and `prompts/layer2/academic-deep-research/`.

## Request Example

```text
Use Academic DeepResearch with MoeResearch.
Topic: Retrieval-augmented generation evaluation methods after 2023.
Capability: literature-review
Depth: standard
Output language: Chinese
Special requirements:
- separate peer-reviewed papers and preprints;
- identify methodological weaknesses;
- include open research gaps.
```

## Expected Report Shape

Academic reports include scope, inclusion/exclusion criteria, search strategy, literature map, evidence synthesis, methodological appraisal, certainty/confidence assessment, research gaps, limitations, and an evidence annex.

## Evidence Quality Rules

Use academic methods such as PRISMA, GRADE, CONSORT, STROBE, AMSTAR 2, RoB 2, ROBINS-I, CASP, and JBI as appraisal lenses when relevant. Do not overstate evidence beyond study design, population, measurement, or validity boundaries.

## Host Verification Triggers

Use bounded host verification for exact paper identity, DOI/PMID, retraction/correction risk, guideline wording, quantitative claims, contradictions, and weak single-source evidence.

## Limitations

MoeResearch is not a substitute for a formal systematic review protocol, librarian-assisted database search, legal/medical advice, or expert peer review. Treat confidence levels as research-assistant judgments, not definitive academic conclusions.
