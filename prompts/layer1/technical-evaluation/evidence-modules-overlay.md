# Layer 1 Overlay: Technical Evidence and Annex Placement

Apply this overlay after the shared common evidence modules. It adds technical decision and Annex rules; it does not replace `evidence-postprocess.md`, `claim-ledger.md`, `host-verification-backfill.md`, `evidence-verifier.md`, `report-annex.md`, or `typst-report-contract.md`.

## Technical evidence interpretation

- Separate official documentation, repository/release/changelog evidence, benchmark evidence, advisory/vulnerability evidence, license/governance evidence, independent engineering analysis, and community opinion.
- Keep option/version/runtime/platform, workload/dataset/configuration, benchmark environment, source date, and transferability limits adjacent to the claims they qualify.
- Do not equate popularity, issue activity, missing advisories, marketing claims, or a single benchmark with production readiness.
- Label cost, migration duration, staffing effort, expected performance, rollback feasibility, and operational capacity as assumptions unless directly evidenced.
- Use ISO/IEC 25010, OWASP ASVS/Top 10, OpenSSF Scorecard, SLSA, SPDX, SemVer, CNCF maturity, or reproducible-benchmarking lenses only when relevant and evidenced. They are evaluation lenses, not automatically retrieved evidence.

## Annex A placement

Keep the common fixed order A.1–A.8. A.1 must retain every canonical logical baseline field in `report-annex.md`; add only these Technical extension fields without changing source origin:

| Annex | Technical additions |
| --- | --- |
| A.1 Evidence Index | profile extensions: `source_class | option_or_version | applicability_conditions` |
| A.2 Artifact / Structured Evidence | official docs/release/repo observations, compatibility artifacts, environment records, or local-spike artifacts when actually available |
| A.3 Contradiction / Risk / Limitation Register | advisory scope, license uncertainty, benchmark transferability, runtime/compatibility gaps, operational risks, and mitigation status |
| A.4 Open Questions | unsupported target environments, missing workload evidence, unresolved versions, unverified rollback, and failed aspects |
| A.5 Falsification / Validation Matrix | recommendation, measurable test, success metric, guardrail, kill threshold, owner, and affected decision |
| A.6 Self-Verification Record | version identity, evidence closure, source separation, benchmark discipline, gate/kill/rollback coverage, and confidence downgrades |
| A.7 Abstain Log | unsupported security, compatibility, cost, timeline, availability, performance, or legal assertions |
| A.8 Tool Provenance | distinct MoeResearch, `HV-*`, and manual/local contributions plus unavailable verification tools |

## Body-to-Annex discipline

A body section may summarize a matrix or risk register, but must keep the decision consequence, confidence, native Typst citekey, and unresolved condition visible. Never move a safety, adoption, benchmark, license, or rollback warning only to the Annex. Keep full evidence IDs in Annex A.1 and `citation_map`; do not emit raw IDs or invented evidence labels in body prose. For body tables, use readable source-origin or source-class labels by default; follow `typst-report-contract.md` for the narrow literal-audit-ID exception.
