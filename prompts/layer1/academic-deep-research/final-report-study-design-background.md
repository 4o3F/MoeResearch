# Layer 1 Prompt: Academic Typst Final Report — Study Design Background

## Role

Convert validated MoeResearch results into a `typst-project-v1` study-design background report. Use `typst-report-contract.md`, Academic final-report guidance, and the Academic evidence overlay as binding prerequisites. This supports planning; it is not an IRB protocol, statistical analysis plan, grant application, or ethics approval.

## Body assembly

Emit `sections/body.typ` as Typst source, not Markdown, using this section hierarchy:

```typst
= {Title}

== Abstract / Executive Answer
== Research Question and Scope
== Prior Work and Rationale
== Constructs and Definitions
== Candidate Measures and Data Sources
== Candidate Study Designs
== Validity Threats
== Feasibility and Ethics Considerations
== Evidence Gaps Affecting Design
== Recommended Minimal Study Option
== Alternative Designs
== Limitations
```

- State the best-fit direction, confidence, and largest design uncertainty first.
- Separate conceptual fit, measurement validity, sampling feasibility, causal identification, operational feasibility, and ethics/legal constraints.
- For each recommended design, identify the target evidence, native Typst citekeys, main validity threat, feasibility caveat, and observation that would falsify the rationale. Keep the evidence-ID audit trace in Annex A.1 and `citation_map`.
- Describe measures, datasets, interventions, populations, and methods only to the degree the evidence supports; do not invent availability, power, safety, compliance, or ethical approval.
- Present alternatives when different validity, feasibility, or construct-coverage trade-offs remain credible.

## Rendering rules

- Apply the `typst-report-contract.md` semantic highlighting vocabulary only to load-bearing design implications, limitations, validity threats, and validation conditions; never use color without its visible label and boundary cue.
- Apply the common table-readability and degradation rules. Do not compress measure/design appraisal prose into four or more near-equal columns; split it into linked panels or label–value cards while retaining every audit field.
- Use native Typst citekeys in reader-facing body prose. Keep full evidence IDs in Annex A.1 and `citation_map`; use readable source-origin or source-class labels by default in body tables, following `typst-report-contract.md` for the narrow literal-audit-ID exception.

## Annex mapping

Use A.1–A.8 in `sections/annex.typ`. Put measure/method appraisal and available artifacts in A.2, validity and feasibility risks in A.3, missing design evidence in A.4, validation/falsification routes in A.5, and design-evidence closure in A.6.

## Capability gates

- Each recommendation separates evidence-backed implication from a local assumption requiring expert review.
- Feasibility and ethics statements identify their evidence boundary and never imply approval.
- Weak, stale, indirect, single-source, or contested design implications are downgraded, conditional, open, or abstained.
- Return the fixed project handoff with `format: "typst-project-v1"`; do not return a Markdown report or automatically compile a PDF.
