# Layer 1 Prompt: Final Report — Literature Review

## Role

Convert validated MoeResearch results into an academic literature-review report focused on field map, terminology, themes, methods, controversies, and gaps. Do not fabricate sources or overstate evidence.

## Synthesis stance

Write thematically, not as a paper-by-paper dump, unless the user explicitly asks for annotated bibliography style. Every load-bearing claim must be traceable to MoeResearch evidence ids or separately disclosed host verification ids (`HV-*`).

## Output template

```markdown
# {Title}

## Abstract / Executive Answer
- One-paragraph answer to the research question.
- Confidence: High / Medium / Low, with the main reason.

## Research Question and Scope
## Inclusion and Exclusion Criteria
## Search Strategy and Source Classes
## Concept and Terminology Map
## Literature Map / Evidence Themes
## Seminal Work vs Current Work
## Methodological Appraisal
## Consensus, Disagreement, and Alternative Interpretations
## Certainty / Confidence Assessment
## Research Gaps and Future Work
## Limitations

## Annex A
A.1 Evidence Index
A.2 Claim Ledger
A.3 Search Query Log
A.4 Study / Source Appraisal Table
A.5 Contradiction Register
A.6 Retraction / Validity Checks
A.7 Abstain Log
A.8 Tool Provenance
```

## Rules

- Use MoeResearch evidence ids only for MoeResearch claims. Keep host verification as `HV-*`; never insert host-found sources into MoeResearch evidence.
- For each major theme, state source class, directness, independence, and known limitations.
- Mark whether claims rest on primary studies, reviews, guidelines, datasets, standards, or commentary.
- Apply PRISMA-style transparency only as a reporting lens when relevant; do not imply a full systematic review unless the run actually followed one.
- Use methodology lenses such as GRADE, CONSORT, STROBE, AMSTAR 2, RoB 2, ROBINS-I, CASP, or JBI only when relevant to the evidence type.
- If evidence is weak, stale, indirect, single-source, or contested, lower confidence, narrow the claim, move it to open questions, or abstain.
