# Layer 1 Prompt: Technical Typst Final Report — Migration / Upgrade Assessment

## Role

Convert validated MoeResearch results into a `typst-project-v1` migration or upgrade decision report. Use `typst-report-contract.md`, Technical final-report guidance, and the Technical evidence overlay as binding prerequisites. Separate evidence-backed migration requirements from inferred effort and local unknowns.

## Body assembly

Emit `sections/body.typ` as Typst source, not Markdown, using this section hierarchy:

```typst
= Technical Evaluation: {Topic}

== Decision Summary
== Evaluation Scope and Constraints
== Current and Target State
== Compatibility and Breaking Changes
== Code, Data, and Runtime Change Surface
== Operational Risk
== Performance and Scalability Evidence
== Security, Compliance, and License Risks
== Testing Strategy
== Rollout Plan
== Rollback and Exit Options
== Adoption Gate
== Minimal Spike and Verification Plan
== Open Risks and Kill Criteria
```

- State whether to migrate now, trial first, defer, replace, or monitor first, with confidence and the highest-impact unknown.
- Prioritize migration guides, release notes, changelogs, compatibility matrices, deprecations, repositories, and issue trackers; label non-official sources accordingly.
- Separate change surface into code, data/schema, runtime/deployment, observability, CI/CD, tests, and team learning.
- Cost and timeline estimates are assumptions unless direct evidence supports them. Give a bounded spike/validation step rather than invented precision.
- The rollout includes measurable gate(s), guardrail(s), rollback trigger(s), data recovery/compatibility assumptions, and ownership where available.
- Kill criteria are falsifiable: unsupported version path, irrecoverable data risk, unacceptable measured regression, blocker advisory/license issue, or missing test coverage.

## Rendering rules

- Apply the `typst-report-contract.md` semantic highlighting vocabulary only to decisions, material risks, rollout gates, and validation conditions; never use color without its visible label and boundary cue.
- Apply the common table-readability and degradation rules. Do not compress compatibility/change-surface prose into four or more near-equal columns; split it into linked panels or label–value cards while retaining every decision field.

## Annex mapping

Use A.1–A.8 in `sections/annex.typ`. Put release/compatibility artifacts in A.2, operational/security/license limitations in A.3, unknown change-surface facts in A.4, spike/rollout/rollback tests in A.5, and migration closure in A.6.

## Capability gates

- Do not present a feasible path as proved until compatibility, data, runtime, and rollback evidence support it.
- Every inferred effort or schedule carries an assumption marker and validation route.
- License notes are engineering due diligence, not legal advice.
- Return the fixed project handoff with `format: "typst-project-v1"`; do not return a Markdown report or automatically compile a PDF.
