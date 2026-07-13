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

## Model Retrieval Contract

Academic personas use the shared model-only `search` protocol: `query`, optional `max_results`, and a required semantic `intent` with `source_focus`, `timeliness`, `coverage`, and `detail`. Layer 1 appends the common contract plus a Run Binding that permits only `general` and `academic` for `source_focus` when `policy.search.category = "academic"`; this is the same projection used by every profile with a fixed category. Rust resolves each intent against one selected provider and rejects incompatible focuses before dispatch. Read returned `intent_resolution` (`enforced`, `best_effort`, or `unsupported`) before judging source coverage. The model final JSON selects candidate evidence IDs only and copies aspect identity literally; the host rehydrates provenance and evidence metadata.

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

Academic reports are delivered by the unified `deep-research` Skill as a `typst-project-v1` source package: `report.typ`, `modules/report-style.typ`, `sections/body.typ`, `sections/annex.typ`, and `references.bib`. The selected capability determines the body structure; all reports use Typst's built-in IEEE bibliography, retain source IDs/citekeys/confidence/contradictions/open questions/self-verification/abstentions/tool provenance in Annex A.1–A.8, and keep native `@citekey` citations rather than manual numeric references. Reader-facing body prose uses citekeys rather than raw evidence IDs; `citation_map` and Annex A.1 preserve the audit mapping to frozen evidence IDs. Body tables use citekeys, Annex A.1 cross-references, and readable source-origin or source-class labels by default; a literal audit ID is reserved for material origin disambiguation.

The Layer 1 report guidance performs bounded evidence preparation, section planning, thematic assembly, and self-verification. It uses labeled, accessible visual callouts for conclusions, limitations, and validation conditions; color is reinforcement rather than the only meaning. Long prose tables are split into readable panels or label–value cards instead of shrinking text or forcing many narrative columns onto an A4 page. It does not rerun research outside the existing budget/partial-status contracts or claim a formal systematic review without the required evidence. A Typst project is only materialized in a caller-specified destination and is never compiled automatically; callers may explicitly run `typst compile --root <project-dir> <project-dir>/report.typ <project-dir>/report.pdf` after reviewing the generated source.

## Evidence Quality Rules

Use academic methods such as PRISMA, GRADE, CONSORT, STROBE, AMSTAR 2, RoB 2, ROBINS-I, CASP, and JBI as appraisal lenses when relevant. Do not overstate evidence beyond study design, population, measurement, or validity boundaries.

## Host Verification Triggers

Use bounded host verification for exact paper identity, DOI/PMID, retraction/correction risk, guideline wording, quantitative claims, contradictions, and weak single-source evidence.

## Limitations

MoeResearch is not a substitute for a formal systematic review protocol, librarian-assisted database search, legal/medical advice, or expert peer review. Treat confidence levels as research-assistant judgments, not definitive academic conclusions.
