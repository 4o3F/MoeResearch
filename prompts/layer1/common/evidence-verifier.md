# Layer 1 Common Module: Evidence Verifier

Claim-level verifier for rows already extracted into the Claim Ledger. It complements `evidence-postprocess.md`; it does not rewrite frozen provenance and does not require Rust/MoeResearch schema changes.

## Inputs

- Frozen MoeResearch `evidence_index` rows.
- Claim Ledger rows.
- Source-audit base rows from `evidence-postprocess.md`.
- Host verification rows (`HV-*`) from `host-verification-backfill.md`.

Host or manual rows can affect confidence and action, but they do not become MoeResearch `Evidence` rows and must not be inserted into `evidence_refs`. Keep `HV-*` in `host_verification_refs` only.

## Verification steps

Run these steps for every load-bearing claim in `deep` / `deep_evidence_pack`, and for selected key claims in `standard`.

1. Support check: cited evidence directly supports the claim.
2. Contradiction check: equal or stronger sources do not disagree, or conflict is disclosed. Use host WebSearch/WebFetch only through `host-verification-backfill.md`.
3. Freshness check: time-sensitive claims have date/version/context boundaries.
4. Independence check: independent evidence is separated from vendor-owned, practitioner self-report, community, or unknown evidence.
5. Domain validity check: academic, technical, security, benchmark, regulatory, or profile-specific criteria where relevant.
6. Action assignment: keep, narrow, downgrade, move to open question, or abstain.

## Host verification use

When `host-verification-backfill.md` returns `HV-*` rows:

- Link them to Claim Ledger rows using `claim_id`.
- Keep `host_verification_refs` and `source_origin` on the ledger row.
- Do not add `HV-*` to `evidence_refs`.
- Do not change frozen MoeResearch evidence fields.
- If host verification contradicts MoeResearch evidence, keep both origins visible and downgrade or move the claim unless the stronger source clearly resolves the conflict.
- If WebFetch was unavailable for a claim requiring original wording, mark the claim `partial` or `not_checked`; do not treat a WebSearch snippet as full support.

## Support status

| Status | Meaning | Required action |
|---|---|---|
| `supported` | Evidence directly supports the claim at the stated strength. | Keep if no unresolved conflict. |
| `partial` | Evidence is related but weaker, indirect, dated, or narrower. | Downgrade confidence or narrow claim. |
| `unsupported` | Evidence does not support the claim. | Remove from body; move to open question or abstain. |
| `not_checked` | Verification was not run due to budget or tool failure. | Only allowed for non-load-bearing claims; disclose limitation. |

## Source audit

For each cited source, assess:

| Dimension | Check |
|---|---|
| Authority | Official, regulator, academic, independent media, vendor, community, repository, unknown. |
| Independence | Whether the source has commercial or product-side interest. |
| Freshness | Whether the source is current enough for the claim. |
| Directness | Whether the source directly proves the claim or only provides background. |
| Specificity | Whether it matches target region, user segment, device, version, product, platform, or business model. |
| Replicability | Whether method, data, screenshot, sampling, statistics, or provenance can be checked. |
| Conflict | Whether equal or stronger sources disagree. |

## Academic audit

Academic audit is not "a paper exists." Check three layers:

1. The paper / guideline / consensus is real, current, and not retracted.
2. The study design can support the claim type.
3. The report wording does not exceed the evidence boundary.

Use PRISMA, GRADE, CONSORT, STROBE, AMSTAR 2, Cochrane RoB 2, ROBINS-I, CASP, or JBI as lightweight checklists when applicable.

Publication validity:

- Match title, author, venue, DOI / PMID / official journal page where possible.
- Mark publication status: peer-reviewed, accepted, preprint, under review, desk_rejected, unclear, retracted_or_concern.
- Check for retraction, expression of concern, or major correction when the claim is high-risk.
- Mark funding and conflicts of interest when visible.

Methodological vulnerabilities to flag when present:

- Population mismatch; intervention / exposure mismatch; proxy outcomes.
- Weak design for causal wording; small sample or no power rationale.
- Missing data / dropout risk; confounding not handled; selective reporting.
- Effect size not meaningful despite significance; harms not reported.
- Funding / COI risk; retraction / correction risk.

## Technical audit

Check version/platform/API compatibility, official documentation versus community workaround, repository and release health, license/compliance visibility, advisory status, supply-chain posture, benchmark reproducibility, and migration/integration cost evidence.

## Contradiction handling

- Do not select only supporting evidence.
- If conflict remains unresolved, downgrade confidence and keep the conflict in the report annex.
- If conflict affects a primary recommendation, decision gate, or safety boundary, require an explicit test, kill criterion, open question, or abstention.

## Output

```json
{
  "claim_ledger": [],
  "verifier_summary": {
    "load_bearing_claims": 0,
    "checked_load_bearing_claims": 0,
    "unsupported_load_bearing_claims": 0,
    "partial_load_bearing_claims": 0,
    "contradiction_unresolved": 0,
    "academic_audits": 0,
    "technical_audits": 0,
    "abstained_claims": 0,
    "host_verified_claims": 0,
    "host_contradicted_claims": 0,
    "host_not_checked_claims": 0,
    "notes": []
  }
}
```
