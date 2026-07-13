# Layer 1 Prompt: Technical Typst Final Report — Technical Due Diligence

## Role

Convert validated MoeResearch results into a `typst-project-v1` broad technical due-diligence report. Use `typst-report-contract.md`, Technical final-report guidance, and the Technical evidence overlay as binding prerequisites. Use this template when the request does not fit a narrower comparison, architecture, dependency, migration, or benchmark capability.

## Body assembly

Emit `sections/body.typ` as Typst source, not Markdown, using this section hierarchy:

```typst
= Technical Evaluation: {Topic}

== Decision Summary
== Evaluation Scope and Constraints
== Technical Context and Requirements
== Architecture and Integration Findings
== Operational Readiness
== Security, Reliability, and Compliance Risks
== Ecosystem and Governance Health
== Cost, Migration, and Reversibility
== Alternatives Comparison
== Adoption Gate
== Minimal Spike and Verification Plan
== Open Risks and Kill Criteria
== Rollback and Exit Options
```

- State what can be concluded now, what must be verified before commitment, and the confidence/reversal condition first.
- Separate official documentation, repository/release evidence, benchmark/performance evidence, advisory/license evidence, independent engineering analysis, and community opinion.
- Treat technical feasibility, operational readiness, security/reliability, ecosystem health, cost, migration, and reversibility as separate decision dimensions.
- The adoption gate names evidence or local-spike results required before production use. Kill criteria are falsifiable and tied to user constraints.
- Cost, migration duration, staffing, capacity, and rollback feasibility are assumptions unless evidence establishes them.
- License analysis is engineering due diligence, not legal advice.

## Rendering rules

- Apply the `typst-report-contract.md` semantic highlighting vocabulary only to decisions, material risks, adoption gates, and validation conditions; never use color without its visible label and boundary cue.
- Apply the common table-readability and degradation rules. Do not compress due-diligence prose into four or more near-equal columns; split it into linked panels or label–value cards while retaining every decision field.

## Annex mapping

Use A.1–A.8 in `sections/annex.typ`. Put official/repository/structured evidence in A.2, risk and applicability limits in A.3, unresolved decision facts in A.4, spike/falsification details in A.5, and due-diligence closure in A.6.

## Capability gates

- The broad scope does not permit broad claims: each decision conclusion has evidence IDs/citekeys, confidence, validation need, and residual risk.
- Missing critical evidence converts the outcome to trial/defer/reject or an explicit abstention.
- Return the fixed project handoff with `format: "typst-project-v1"`; do not return a Markdown report or automatically compile a PDF.
