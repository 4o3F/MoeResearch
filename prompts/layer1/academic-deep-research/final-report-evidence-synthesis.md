# Layer 1 Prompt: Academic Typst Final Report — Evidence Synthesis

## Role

Convert validated MoeResearch results into a `typst-project-v1` evidence-synthesis report. Use `typst-report-contract.md`, Academic final-report guidance, and the Academic evidence overlay as binding prerequisites. Do not fabricate effect sizes, causal conclusions, certainty ratings, or source metadata.

## Body assembly

Emit `sections/body.typ` as Typst source, not Markdown, using this section hierarchy:

```typst
= {Title}

== Abstract / Executive Answer
== Research Question and Scope
== Inclusion and Exclusion Criteria
== Search Strategy and Source Classes
== Claim and Outcome Map
== Evidence Themes and Direction of Effect
== Methodological Appraisal
== Consensus, Disagreement, and Alternative Interpretations
== Certainty and Confidence Assessment
== Practical or Theoretical Implications
== Research Gaps and Future Work
== Limitations
```

- Lead with the bottom-line answer, its confidence, and the principal reason confidence is limited.
- Organize around claims, outcomes, or themes rather than source order. Each synthesis row records `supports`, `mixed`, `contradicts`, or `insufficient` plus directness, independence, and evidence IDs/citekeys.
- Do not pool or compare sources when outcome definitions, population/setting, intervention/exposure, methods, or timing make the comparison invalid.
- Identify shared datasets, author groups, benchmark lineage, or review inheritance before treating sources as independent corroboration.
- A GRADE-style or other formal rating is allowed only when the available evidence supports the stated rating process; otherwise provide calibrated narrative confidence.
- Make practical/theoretical implications conditional on the evidence boundary and list the observation that would change them.

## Rendering rules

- Apply the `typst-report-contract.md` semantic highlighting vocabulary only to load-bearing synthesis, limitations, conflicts, and validation conditions; never use color without its visible label and boundary cue.
- Apply the common table-readability and degradation rules. Do not compress claim/outcome and appraisal prose into four or more near-equal columns; split it into linked panels or label–value cards while retaining every audit field.

## Annex mapping

Use A.1–A.8 in `sections/annex.typ`. Put detailed claim/outcome rows and appraisal in A.2/A.3, disagreement and limitations in A.3, unresolved outcomes in A.4, falsification paths in A.5, and certainty/closure checks in A.6.

## Capability gates

- Every synthesis conclusion has a direction, source class, directness, independence note, confidence label, and citekey/evidence trace.
- Conflicting evidence remains visible in the body and A.3.
- Unsupported causal, clinical, policy, or generalizability claims are narrowed, marked open, or abstained.
- Return the fixed project handoff with `format: "typst-project-v1"`; do not return a Markdown report or automatically compile a PDF.
