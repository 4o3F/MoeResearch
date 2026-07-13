# Layer 1 Prompt: Academic Typst Final Report — Literature Review

## Role

Convert validated MoeResearch results into a `typst-project-v1` literature review. Use `typst-report-contract.md`, Academic final-report guidance, and the Academic evidence overlay as binding prerequisites. Do not fabricate sources, metadata, formal appraisal, or certainty.

## Body assembly

Emit `sections/body.typ` as Typst source, not Markdown, using this section hierarchy:

```typst
= {Title}

== Abstract / Executive Answer
== Research Question and Scope
== Inclusion and Exclusion Criteria
== Search Strategy and Source Classes
== Concept and Terminology Map
== Literature Map and Evidence Themes
== Seminal Work and Current Work
== Methodological Appraisal
== Consensus, Disagreement, and Alternative Interpretations
== Certainty and Confidence Assessment
== Research Gaps and Future Work
== Limitations
```

- Open the abstract with the answer, confidence, and main downgrade reason.
- Build the literature map by theme, construct, method, setting, or outcome; do not serialize one paper after another unless the user explicitly asks for an annotated bibliography.
- For each major theme, state source/study class, directness, independence/shared lineage, native Typst citekeys, and material limitations. Keep the evidence-ID audit trace in Annex A.1 and `citation_map`.
- Separate seminal influence from current evidence. Citation count, popularity, or age alone is not proof of methodological quality.
- Explain disagreement rather than averaging it away. State whether differing populations, definitions, designs, settings, or evidence quality plausibly bound the conclusion.
- Use PRISMA-style transparency and appraisal lenses only as appropriate reporting lenses. Do not claim a systematic review or formal score unless the run supports it.

## Rendering rules

- Apply the `typst-report-contract.md` semantic highlighting vocabulary only to load-bearing synthesis, limitations, conflicts, and future-work validation conditions; never use color without its visible label and boundary cue.
- Apply the common table-readability and degradation rules. Do not compress literature/appraisal prose into four or more near-equal columns; split it into linked panels or label–value cards while retaining every audit field.
- Use native Typst citekeys in reader-facing body prose. Keep full evidence IDs in Annex A.1 and `citation_map`; use readable source-origin or source-class labels by default in body tables, following `typst-report-contract.md` for the narrow literal-audit-ID exception.

## Annex mapping

Use A.1–A.8 in `sections/annex.typ`. Academic additions include the terminology/source map in A.2, study/source appraisal in A.3, unresolved coverage and identity checks in A.4, validation routes in A.5, and certainty/independence checks in A.6.

## Capability gates

- Every major theme has at least one evidence cluster or is explicitly marked as a coverage gap.
- A claim about a paper's version, correction, retraction, peer-review state, DOI/PMID/arXiv record, or venue appears only when evidenced.
- Weak, stale, indirect, single-lineage, or contested evidence lowers confidence, narrows the conclusion, enters A.4/A.7, or is omitted from the body.
- Return the fixed project handoff with `format: "typst-project-v1"`; do not return a Markdown report or automatically compile a PDF.
