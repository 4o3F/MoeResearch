# Layer 1 Prompt: Technical Typst Final Report — Dependency Risk Assessment

## Role

Convert validated MoeResearch results into a `typst-project-v1` dependency-risk decision report. Use `typst-report-contract.md`, Technical final-report guidance, and the Technical evidence overlay as binding prerequisites. Make risk acceptance, mitigation, and exit conditions explicit and testable.

## Body assembly

Emit `sections/body.typ` as Typst source, not Markdown, using this section hierarchy:

```typst
= Technical Evaluation: {Topic}

== Decision Summary
== Evaluation Scope and Constraints
== Dependency Identity and Usage Context
== Risk Register
== Maintenance and Release Health
== Security Advisory and Supply-Chain Review
== License and Compliance Notes
== Ecosystem Maturity and Governance
== Alternatives Comparison
== Mitigation Plan
== Adoption or Continued-Use Gate
== Open Risks and Kill Criteria
== Rollback and Exit Options
```

- State whether to adopt, keep, monitor, replace, or reject first, with confidence and material residual risk.
- Separate known vulnerabilities/advisories from theoretical weaknesses, issue chatter, missing evidence, and synthesis inference.
- Record affected/supported versions, maintainer response posture, release cadence, security policy, governance, and dependency footprint only when evidenced.
- Never state that a dependency is safe because no advisory was found. State what sources and versions were checked, what remains unknown, and the mitigation boundary.
- Popularity is weak evidence unless combined with maintenance, governance, and production-readiness evidence.
- Kill criteria are falsifiable: unpatched critical advisory, unsupported runtime, blocker license, unmanaged single-maintainer risk, or no credible migration route.

## Rendering rules

- Apply the `typst-report-contract.md` semantic highlighting vocabulary only to decisions, material risks, kill criteria, and validation conditions; never use color without its visible label and boundary cue.
- Apply the common table-readability and degradation rules. Do not compress risk/advisory/governance prose into four or more near-equal columns; split it into linked panels or label–value cards while retaining every decision field.
- Use native Typst citekeys in reader-facing body prose. Keep full evidence IDs in Annex A.1 and `citation_map`; use readable source-origin or source-class labels by default in body tables, following `typst-report-contract.md` for the narrow literal-audit-ID exception.

## Annex mapping

Use A.1–A.8 in `sections/annex.typ`. Put dependency/release/advisory artifacts in A.2, risk and license uncertainty in A.3, unknown affected versions/governance facts in A.4, mitigation tests in A.5, and risk-closure checks in A.6.

## Capability gates

- Risk acceptance has a concrete mitigation owner/condition or is a defer/replace recommendation.
- Security and license claims retain their evidence scope and legal boundary.
- Unsupported compatibility, maintenance, or migration assertions enter A.4/A.7.
- Return the fixed project handoff with `format: "typst-project-v1"`; do not return a Markdown report or automatically compile a PDF.
