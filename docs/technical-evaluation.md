# Technical Evaluation

Technical Evaluation is a Skill/profile layer over the MoeResearch MCP core for evidence-backed engineering decisions.

## Use Cases

- Compare libraries, frameworks, SDKs, databases, queues, runtimes, or platforms.
- Evaluate architecture options and integration trade-offs.
- Assess dependency security, maintenance, license, and supply-chain risk.
- Plan migrations, upgrades, or replacements.
- Review benchmark claims and performance evidence.

## Capabilities

`library-framework-comparison`, `architecture-option-evaluation`, `dependency-risk-assessment`, `migration-upgrade-assessment`, `benchmark-performance-review`, and `technical-due-diligence`.

## Setup

```bash
moeresearch mcp register --scope local --config ~/.config/moeresearch/moeresearch.toml
moeresearch assets install research-skills --config ~/.config/moeresearch/moeresearch.toml
```

The research skills asset installs Technical Evaluation prompts under `prompts/layer1/technical-evaluation/` and `prompts/layer2/technical-evaluation/`.

## Model Retrieval Contract

Technical personas use the shared model-only `search` protocol: `query`, optional `max_results`, and a required semantic `intent` with `source_focus`, `timeliness`, `coverage`, and `detail`. Layer 1 assembles instructions as persona + common model-search contract + Run Binding for search-enabled aspects. Default technical category is open, but the same profile-neutral binding projection applies if a category or another semantic intent ceiling is fixed later. Rust resolves intent against exactly one selected provider and `policy.search`, then reports actual per-dimension `intent_resolution` as `enforced`, `best_effort`, or `unsupported`. The model final JSON selects candidate evidence IDs only and copies aspect identity literally; the host rehydrates provenance and evidence metadata.

## Request Example

```text
Use Technical Evaluation with MoeResearch.
Topic: Evaluate Axum vs Actix-web for a Rust MCP service backend.
Capability: library-framework-comparison
Depth: standard
Output language: Chinese
Special requirements:
- compare API stability, ecosystem, performance evidence, maintenance risk;
- include a minimal spike validation plan.
```

## Expected Report Shape

Technical reports are delivered by the unified `deep-research` Skill as a `typst-project-v1` source package: `report.typ`, `modules/report-style.typ`, `sections/body.typ`, `sections/annex.typ`, and `references.bib`. The selected capability determines the decision sections; all reports use Typst's built-in IEEE bibliography, retain evidence IDs/citekeys/applicability conditions/confidence/adoption gates/kill criteria/contradictions/self-verification/abstentions/tool provenance in Annex A.1–A.8, and keep native `@citekey` citations rather than manual numeric references. Reader-facing body prose uses citekeys rather than raw evidence IDs; `citation_map` and Annex A.1 preserve the audit mapping to frozen evidence IDs. Body tables use citekeys, Annex A.1 cross-references, and readable source-origin or source-class labels by default; a literal audit ID is reserved for material origin disambiguation.

The Layer 1 report guidance performs bounded evidence preparation, decision-section planning, evidence-led assembly, and self-verification. It uses labeled, accessible visual callouts for decisions, risks, limitations, and validation conditions; color is reinforcement rather than the only meaning. Long prose tables are split into readable panels or label–value cards instead of shrinking text or forcing many narrative columns onto an A4 page. It does not add provider routing, retries, or extra research rounds. A Typst project is only materialized in a caller-specified destination and is never compiled automatically; callers may explicitly run `typst compile --root <project-dir> <project-dir>/report.typ <project-dir>/report.pdf` after reviewing the generated source.

## Source Quality Rules

Prefer official docs, release notes, repositories, security advisories, standards, benchmark methodology pages, package registries, license files, and credible independent engineering writeups. Community posts are useful for signals but should not carry load-bearing decisions alone.

## Benchmark and Security Caveats

Benchmark claims require versions, environment, workload, concurrency, warmup, variance, and reproducibility context. Security claims must not infer safety from limited search; phrase as "no advisory was found in this evidence set" when appropriate.

## Limitations

Technical Evaluation supports decision preparation but does not replace a local spike, code review, SCA/SBOM tooling, production load testing, legal license review, or organization-specific architecture governance.
