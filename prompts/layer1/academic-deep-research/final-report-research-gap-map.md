# Layer 1 Prompt: Academic Typst Final Report — Research Gap Map

## Role

Convert validated MoeResearch results into a `typst-project-v1` research-gap map. Use `typst-report-contract.md`, Academic final-report guidance, and the Academic evidence overlay as binding prerequisites. A gap is valid only when existing coverage and its boundary are evidenced.

## Body assembly

Emit `sections/body.typ` as Typst source, not Markdown, using this section hierarchy:

```typst
= {Title}

== Abstract / Executive Answer
== Research Question and Scope
== Inclusion and Exclusion Criteria
== Search Strategy and Source Classes
== Current Frontier
== Evidence Coverage Map
== Methodological Gaps
== Theory, Construct, and Measurement Gaps
== Practical or Policy-Relevant Gaps
== Candidate Research Questions
== Future Study Design Options
== Prioritization and Feasibility
== Limitations
```

- Lead with the most decision-relevant gaps, why they matter, and the confidence that the gap is real rather than merely unsearched.
- For each gap, show what is already known, which evidence IDs/citekeys establish the coverage boundary, what remains untested or unresolved, and whether the gap is conceptual, empirical, methodological, measurement-related, population/setting-specific, replication-related, or translational.
- Do not label a missing search result, inaccessible source, or failed aspect as a field-wide evidence gap.
- Candidate questions and study options include a minimal design idea, target evidence, likely validity threat, feasibility caveat, and falsification signal.
- Prioritize by importance, tractability, expected information gain, evidence confidence, and ethical/operational caveats; do not fabricate numeric scores.

## Rendering rules

- Apply the `typst-report-contract.md` semantic highlighting vocabulary only to load-bearing gap claims, limitations, conflicts, and study-validation conditions; never use color without its visible label and boundary cue.
- Apply the common table-readability and degradation rules. Do not compress coverage/gap prose into four or more near-equal columns; split it into linked panels or label–value cards while retaining every audit field.

## Annex mapping

Use A.1–A.8 in `sections/annex.typ`. Put the gap-to-evidence matrix in A.2, conflict and coverage limitations in A.3, unresolved searches/identity checks in A.4, study validation routes in A.5, and gap-audit results in A.6.

## Capability gates

- Each proposed gap has evidence for both existing coverage and the remaining uncertainty.
- Future-study advice is conditional, evidence-backed, and not an ethics/IRB, power-analysis, or grant-protocol claim.
- Search gaps, implementation gaps, theory gaps, measurement gaps, and replication gaps remain distinct.
- Return the fixed project handoff with `format: "typst-project-v1"`; do not return a Markdown report or automatically compile a PDF.
