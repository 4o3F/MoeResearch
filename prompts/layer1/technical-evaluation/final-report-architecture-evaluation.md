# Layer 1 Prompt: Technical Typst Final Report — Architecture Option Evaluation

## Role

Convert validated MoeResearch results into a `typst-project-v1` architecture decision report. Use `typst-report-contract.md`, Technical final-report guidance, and the Technical evidence overlay as binding prerequisites. The report is ADR input: context, options, decision, consequences, verification, and reversal path.

## Body assembly

Emit `sections/body.typ` as Typst source, not Markdown, using this section hierarchy:

```typst
= Technical Evaluation: {Topic}

== Decision Summary
== Evaluation Scope and Constraints
== Candidate Architecture Options
== Decision Criteria Matrix
== Requirements and Quality Attributes
== Architecture and Integration Analysis
== Operational Model
== Performance and Scalability Evidence
== Security, Compliance, and Reliability Risks
== Ecosystem and Platform Maturity
== Alternatives Comparison
== Recommended Option and Consequences
== Adoption Gate
== Minimal Spike and Verification Plan
== Open Risks and Kill Criteria
== Rollback and Exit Options
```

- State the recommended option, confidence, and the trade-off that most threatens it first.
- Evaluate relevant quality attributes explicitly: availability, scalability, latency, consistency, durability, operability, observability, security, compliance, and maintainability.
- Separate documented guarantees from examples, marketing claims, community anecdotes, and Layer 1 engineering judgement.
- Benchmark evidence includes workload, version, runtime/hardware, methodology, variance, and transferability; otherwise it only supports a validation hypothesis.
- Consequences, adoption gates, local spike, kill criteria, and rollback/exit path must be testable and tied to the chosen constraints.

## Rendering rules

- Apply the `typst-report-contract.md` semantic highlighting vocabulary only to decisions, material risks, adoption gates, and validation conditions; never use color without its visible label and boundary cue.
- Apply the common table-readability and degradation rules. Do not compress architecture/quality-attribute prose into four or more near-equal columns; split it into linked panels or label–value cards while retaining every decision field.
- Use native Typst citekeys in reader-facing body prose. Keep full evidence IDs in Annex A.1 and `citation_map`; use readable source-origin or source-class labels by default in body tables, following `typst-report-contract.md` for the narrow literal-audit-ID exception.

## Annex mapping

Use A.1–A.8 in `sections/annex.typ`. Put architecture/repository/release artifacts in A.2, quality-attribute risks and benchmark limits in A.3, unresolved integration/operational facts in A.4, verification metrics in A.5, and ADR decision closure in A.6.

## Capability gates

- Every architecture conclusion distinguishes a factual evidence statement from a synthesis judgement.
- Missing local environment or operational evidence changes the recommendation to a conditional trial/defer, not a false certainty.
- License notes are engineering due diligence, not legal advice.
- Return the fixed project handoff with `format: "typst-project-v1"`; do not return a Markdown report or automatically compile a PDF.
