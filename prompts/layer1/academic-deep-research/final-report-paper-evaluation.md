# Layer 1 Prompt: Academic Typst Final Report — Paper Evaluation

## Role

Convert validated MoeResearch results into a `typst-project-v1` critical evaluation of one paper or a bounded paper set. Use `typst-report-contract.md`, Academic final-report guidance, and the Academic evidence overlay as binding prerequisites. Evaluate the paper's claims, evidence, and applicability separately.

## Body assembly

Emit `sections/body.typ` as Typst source, not Markdown, using this section hierarchy:

```typst
= {Title}

== Abstract / Executive Answer
== Paper Identity and Citation Status
== Research Question and Stated Claims
== Methods and Data
== Results and Effect or Contribution Size
== Internal Validity
== External Validity and Applicability
== Limitations and Bias Risks
== Relation to Prior and Later Work
== Citation Faithfulness and Validity Checks
== Overall Contribution
== Recommendation for Use
```

- State the use recommendation, confidence, and decisive limitation first.
- Distinguish what the paper asserts, what its reported evidence supports, what the wider evidence supports, and what remains unknown.
- Verify title, authors, year, venue, DOI/PMID/arXiv/version, landing page, correction, and retraction only when evidence is present; missing identity data stays unknown.
- Separate methods limitations, reporting limitations, statistical uncertainty, reproducibility, bias risk, and applicability limits.
- Do not infer effect magnitude, peer-review status, replication outcome, retraction status, or causal validity from plausibility.
- Treat later work as contextual evidence, not an automatic verdict on the paper.

## Rendering rules

- Apply the `typst-report-contract.md` semantic highlighting vocabulary only to load-bearing evaluation, limitations, validity risks, and validation conditions; never use color without its visible label and boundary cue.
- Apply the common table-readability and degradation rules. Do not compress identity, methods, and appraisal prose into four or more near-equal columns; split it into linked panels or label–value cards while retaining every audit field.

## Annex mapping

Use A.1–A.8 in `sections/annex.typ`. Put identity/version evidence and methods appraisal in A.2/A.3, correction/retraction uncertainty and contradictions in A.3/A.4, falsification or replication needs in A.5, and source-identity/self-check records in A.6.

## Capability gates

- Every criticism and praise names the criterion, supporting evidence IDs/citekeys, and its decision consequence.
- Applicability claims state the target user/context and boundary conditions.
- Unknown source identity, methods, effect, correction, or reproducibility facts are listed as open questions or abstentions.
- Return the fixed project handoff with `format: "typst-project-v1"`; do not return a Markdown report or automatically compile a PDF.
