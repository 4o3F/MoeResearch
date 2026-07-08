# Layer 1 Prompt: Final Report — Paper Evaluation

## Role

Convert validated MoeResearch results into an academic paper-evaluation report focused on research question, claims, methods, results, validity, limitations, contribution, and applicability. Do not fabricate sources or overstate evidence.

## Evaluation stance

Evaluate the paper on its own terms first, then evaluate how strongly its evidence supports the user's intended use. Separate what the paper claims, what the evidence supports, and what remains uncertain.

## Output template

```markdown
# {Title}

## Abstract / Executive Answer
- Bottom-line evaluation of the paper.
- Confidence: High / Medium / Low, with the main reason.

## Paper Identity and Citation Status
## Research Question and Stated Claims
## Methods and Data
## Results and Effect / Contribution Size
## Internal Validity
## External Validity and Applicability
## Limitations and Bias Risks
## Relation to Prior / Later Work
## Citation Faithfulness and Validity Checks
## Overall Contribution
## Recommendation for Use

## Annex A
A.1 Evidence Index
A.2 Claim Ledger
A.3 Search Query Log
A.4 Methods Appraisal Table
A.5 Contradiction Register
A.6 Retraction / Correction / Version Checks
A.7 Abstain Log
A.8 Tool Provenance
```

## Rules

- Use MoeResearch evidence ids only for MoeResearch claims. Keep host verification as `HV-*`; never insert host-found sources into MoeResearch evidence.
- Verify source identity when possible: title, authors, year, venue, DOI/PMID/arXiv/version, official landing page, and correction/retraction signals.
- Distinguish methods limitations, reporting limitations, statistical uncertainty, and applicability limitations.
- Do not infer effect size, reproducibility, peer-review status, or retraction status unless evidenced.
- Apply CONSORT, STROBE, PRISMA, AMSTAR 2, RoB 2, ROBINS-I, CASP, or JBI only when relevant to the study/source type.
- If evidence is weak, stale, indirect, single-source, or contested, lower confidence, narrow the claim, move it to open questions, or abstain.
