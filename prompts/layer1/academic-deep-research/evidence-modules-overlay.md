# Layer 1 Overlay: Academic Evidence and Annex Placement

Apply this overlay after the shared common evidence modules. It adds academic placement and appraisal rules; it does not replace `evidence-postprocess.md`, `claim-ledger.md`, `host-verification-backfill.md`, `evidence-verifier.md`, `report-annex.md`, or `typst-report-contract.md`.

## Academic evidence interpretation

- Preserve distinctions among primary studies, reviews, guidelines, datasets, standards, preprints, conference papers, journal articles, commentary, and secondary summaries when evidence supports the classification.
- Record directness, population/setting fit, study/source type, likely dependence on common data or review lineage, and material method limitations for major synthesis claims.
- Use PRISMA, GRADE, CONSORT, STROBE, AMSTAR 2, RoB 2, ROBINS-I, CASP, or JBI only as applicable reporting or appraisal lenses. Do not claim a formal score, systematic-review protocol, or risk-of-bias result without the evidence needed to support it.
- A paper title, author list, DOI/PMID/arXiv identifier, venue, peer-review state, correction, or retraction status is unknown until evidence establishes it.

## Annex A placement

Keep the common fixed order A.1–A.8. A.1 must retain every canonical logical baseline field in `report-annex.md`; add only these Academic extension fields without changing source origin:

| Annex | Academic additions |
| --- | --- |
| A.1 Evidence Index | profile extensions: `source_or_study_type | directness | independence_note` |
| A.2 Artifact / Structured Evidence | corpus/source-selection record, terminology map, dataset or instrument observations when actually available |
| A.3 Contradiction / Limitation Register | competing interpretations, outcome-definition differences, setting differences, method limitations, and shared-lineage cautions |
| A.4 Open Questions | unresolved identity/version checks, coverage gaps, missing populations/settings, and failed aspects |
| A.5 Falsification / Validation Matrix | finding, contrary evidence needed, replication/validation route, and affected section |
| A.6 Self-Verification Record | evidence closure, identity checks, independence audit, conflict coverage, certainty downgrade, and remaining gaps |
| A.7 Abstain Log | unsupported effect, identity, quality, causality, generalizability, or design assertions |
| A.8 Tool Provenance | distinct MoeResearch, `HV-*`, and manual/local contributions plus unavailable verification tools |

## Body-to-Annex discipline

A body section may summarize an appraisal or contradiction, but must retain the consequence for the conclusion and link to the detailed Annex row. Never hide an evidence downgrade by moving it only to the Annex. Keep confidence labels, native Typst citekeys, and abstention markers visible where the reader makes the decision. Keep full evidence IDs in Annex A.1 and `citation_map`; do not emit raw IDs or invented evidence labels in body prose. For body tables, use readable source-origin or source-class labels by default; follow `typst-report-contract.md` for the narrow literal-audit-ID exception.
