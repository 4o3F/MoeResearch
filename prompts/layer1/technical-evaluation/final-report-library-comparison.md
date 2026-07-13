# Layer 1 Prompt: Technical Typst Final Report — Library / Framework Comparison

## Role

Convert validated MoeResearch results into a `typst-project-v1` library/framework decision report. Use `typst-report-contract.md`, Technical final-report guidance, and the Technical evidence overlay as binding prerequisites. Do not fabricate benchmarks, security findings, license terms, costs, or migration facts.

## Body assembly

Emit `sections/body.typ` as Typst source, not Markdown, using this section hierarchy:

```typst
= Technical Evaluation: {Topic}

== Decision Summary
== Evaluation Scope and Constraints
== Candidate Options
== Decision Criteria Matrix
== Requirements Fit Matrix
== Architecture and Integration Analysis
== API and Developer Experience
== Performance and Scalability Evidence
== Security, Compliance, and License Risks
== Ecosystem Maturity
== Maintenance and Operational Cost
== Alternatives Comparison
== Adoption Gate
== Minimal Spike and Verification Plan
== Open Risks and Kill Criteria
== Rollback and Exit Options
```

- State `Adopt`, `Trial`, `Defer`, `Reject`, `Migrate`, `Replace`, or `Monitor` first, with confidence and constraints that could reverse the decision.
- The criteria and requirements matrices use Typst tables and retain native Typst citekeys, applicability conditions, confidence, and assumptions. Keep the evidence-ID audit trace in Annex A.1 and `citation_map`.
- Separate documented APIs/guarantees, repository/release facts, benchmark observations, advisory/license evidence, independent engineering analysis, and community opinion.
- Benchmark findings include workload, version, hardware/runtime, method, variance/reproducibility, and user-context fit when available. Do not compare incompatible raw numbers.
- Adoption gates name required evidence or local-spike results. Kill criteria are falsifiable, such as runtime incompatibility, unacceptable measured latency, blocker license, critical unpatched advisory, or absent rollback.

## Rendering rules

- Apply the `typst-report-contract.md` semantic highlighting vocabulary only to decisions, material risks, adoption gates, and validation conditions; never use color without its visible label and boundary cue.
- Apply the common table-readability and degradation rules. Do not compress requirements/comparison prose into four or more near-equal columns; split it into linked panels or label–value cards while retaining every decision field.
- Use native Typst citekeys in reader-facing body prose. Keep full evidence IDs in Annex A.1 and `citation_map`; use readable source-origin or source-class labels by default in body tables, following `typst-report-contract.md` for the narrow literal-audit-ID exception.

## Annex mapping

Use A.1–A.8 in `sections/annex.typ`. Put official/release/repository and structured comparison artifacts in A.2, benchmark and security/license limits in A.3, unknown integration or compatibility facts in A.4, spike/falsification metrics in A.5, and decision-gate coverage in A.6.

## Capability gates

- Every recommendation traces to requirements, native Typst citekeys, expected impact, validation step, and residual risk. Keep the evidence-ID audit trace in Annex A.1 and `citation_map`.
- Cost, effort, and migration duration are assumptions unless directly evidenced.
- License notes are engineering due diligence, not legal advice.
- Return the fixed project handoff with `format: "typst-project-v1"`; do not return a Markdown report or automatically compile a PDF.
