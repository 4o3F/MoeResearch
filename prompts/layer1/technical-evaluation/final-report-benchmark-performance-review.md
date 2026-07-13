# Layer 1 Prompt: Technical Typst Final Report — Benchmark / Performance Review

## Role

Convert validated MoeResearch results into a `typst-project-v1` benchmark and performance decision report. Use `typst-report-contract.md`, Technical final-report guidance, and the Technical evidence overlay as binding prerequisites. Benchmark results are decision evidence only when workload and environment fit the user's constraints.

## Body assembly

Emit `sections/body.typ` as Typst source, not Markdown, using this section hierarchy:

```typst
= Technical Evaluation: {Topic}

== Decision Summary
== Evaluation Scope and Workload Assumptions
== Candidate Options and Versions
== Benchmark Evidence Inventory
== Methodology Appraisal
== Workload and Environment Fit
== Latency, Throughput, and Scalability Findings
== Variance, Reproducibility, and Bias Risks
== Operational Cost and Tuning Implications
== Alternatives Comparison
== Adoption Gate
== Minimal Local Benchmark Plan
== Open Risks and Kill Criteria
== Rollback and Exit Options
```

- State the decision implication, confidence, and principal transferability limit first.
- Every benchmark observation retains workload, version, data, hardware/runtime, configuration, method, variance/reproducibility limit, and sponsor/author incentive when available.
- Do not compare raw numbers across incompatible workloads, environments, versions, configurations, or measurement methods.
- Treat absent environment details as a reason for a local benchmark, not a reason to generalize.
- The local plan defines workload, success metric, guardrail metric, data size, environment, repeat/variance method, success threshold, and kill threshold.
- Operational cost and tuning recommendations distinguish measured evidence from estimate or inference.

## Rendering rules

- Apply the `typst-report-contract.md` semantic highlighting vocabulary only to decisions, transferability limits, kill thresholds, and validation conditions; never use color without its visible label and boundary cue.
- Apply the common table-readability and degradation rules. Do not compress workload/environment prose into four or more near-equal columns; split it into linked panels or label–value cards while retaining every decision field.
- Use native Typst citekeys in reader-facing body prose. Keep full evidence IDs in Annex A.1 and `citation_map`; use readable source-origin or source-class labels by default in body tables, following `typst-report-contract.md` for the narrow literal-audit-ID exception.

## Annex mapping

Use A.1–A.8 in `sections/annex.typ`. Put benchmark environment/method artifacts in A.2, variance/bias/transferability risks in A.3, missing workload or configuration facts in A.4, the local benchmark protocol in A.5, and reproducibility closure in A.6.

## Capability gates

- Every recommendation identifies whether it is directly supported, conditionally supported, or unverified locally.
- A performance claim with no workload or environment context does not support an adoption recommendation.
- Unsupported numbers, costs, and scalability claims become A.4/A.7 items.
- Return the fixed project handoff with `format: "typst-project-v1"`; do not return a Markdown report or automatically compile a PDF.
