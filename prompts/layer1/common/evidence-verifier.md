# Layer 1 Common Module: Evidence Verifier

Claim-level verifier for rows already extracted into the Claim Ledger. It complements `evidence-postprocess.md`; it does not rewrite frozen provenance and does not require Rust/MoeResearch schema changes.

## Verification steps

Run these steps for every load-bearing claim in `deep` / `deep_evidence_pack`, and for selected key claims in `standard`.

1. Support check: cited evidence directly supports the claim.
2. Contradiction check: equal or stronger sources do not disagree, or conflict is disclosed.
3. Freshness check: time-sensitive claims have date/version/context boundaries.
4. Independence check: independent evidence is separated from vendor-owned, practitioner self-report, community, or unknown evidence.
5. Domain validity check: academic, technical, security, benchmark, regulatory, or profile-specific criteria where relevant.
6. Action assignment: keep, narrow, downgrade, move to open question, or abstain.

## Host verification use

Host or manual rows can affect confidence and action, but they do not become MoeResearch `Evidence` rows and must not be inserted into `evidence_refs`. Keep `HV-*` in `host_verification_refs` only.

## Academic audit

Check whether study design can support the claim type and whether wording exceeds evidence boundaries. Use PRISMA, GRADE, CONSORT, STROBE, AMSTAR 2, Cochrane RoB 2, ROBINS-I, CASP, or JBI as lightweight checklists when applicable.

## Technical audit

Check version/platform/API compatibility, official documentation versus community workaround, repository and release health, license/compliance visibility, advisory status, supply-chain posture, benchmark reproducibility, and migration/integration cost evidence.

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
