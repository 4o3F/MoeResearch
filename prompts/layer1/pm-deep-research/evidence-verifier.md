# Layer 1 Module: Evidence Verifier (PM DeepResearch)

> Skill-layer verifier for claims already extracted into the Claim Ledger. It complements `evidence-postprocess.md`; it does not rewrite frozen provenance and does not require Rust/Lapis changes.

## Purpose

Verify whether the evidence actually supports the claim and whether the claim is safe to keep in a product decision report.

Inputs may include:

- Frozen Lapis `evidence_index` rows.
- Claim Ledger rows.
- Source-audit base rows from `evidence-postprocess.md`.
- Host verification rows (`HV-*`) from `host-verification-backfill.md`.

Host verification rows can affect confidence and action, but they do not become Lapis `Evidence` rows and must not be inserted into `evidence_refs`.

## Verification Steps

Run these steps for every load-bearing claim in `deep` / `deep_evidence_pack`, and for selected key claims in `standard`.

1. **Support check**: Decide whether cited evidence directly supports the claim.
2. **Contradiction check**: Search or inspect for plausible disconfirming evidence, or record why this could not be done. Use host WebSearch/WebFetch only through `host-verification-backfill.md`.
3. **Freshness check**: Time-sensitive claims need dates: product features, pricing, market data, regulation, guidelines, competitor state, library/tool behavior.
4. **Independence check**: Distinguish independent evidence from vendor-owned, practitioner self-report, community evidence, or unknown independence.
5. **Academic audit**: For academic / scientific / health claims, check publication validity and study validity.
6. **Action assignment**: keep, downgrade, move to open question, or abstain.

## Host Verification Use

When `host-verification-backfill.md` returns `HV-*` rows:

- Link them to Claim Ledger rows using `claim_id`.
- Add a `host_verification_refs` field if the active report template supports it.
- Do not add `HV-*` to `evidence_refs`.
- Do not change frozen Lapis evidence fields.
- If host verification contradicts Lapis evidence, keep both origins visible and downgrade or move the claim unless the stronger source clearly resolves the conflict.
- If WebFetch was unavailable for a claim requiring original wording, mark the claim `partial` or `not_checked`; do not treat a WebSearch snippet as full support.

Recommended Claim Ledger extension:

```json
{
  "host_verification_refs": ["HV-001"],
  "source_origin": "lapis_only|host_verified|mixed|manual_host_verified"
}
```

## Support Status

| Status | Meaning | Required Action |
|---|---|---|
| `supported` | Evidence directly supports the claim at the stated strength. | Keep if no unresolved conflict. |
| `partial` | Evidence is related but weaker, indirect, dated, or narrower. | Downgrade confidence or narrow claim. |
| `unsupported` | Evidence does not support the claim. | Remove from body; move to open question or abstain. |
| `not_checked` | Verification was not run due to budget or tool failure. | Only allowed for non-load-bearing claims; disclose limitation. |

## Source Audit

For each cited source, assess:

| Dimension | Check |
|---|---|
| Authority | Official, regulator, academic, independent media, vendor, community, KOL, unknown. |
| Independence | Whether the source has commercial or product-side interest. |
| Freshness | Whether the source is current enough for the claim. |
| Directness | Whether the source directly proves the claim or only provides background. |
| Specificity | Whether it matches target region, user segment, device, version, product, or business model. |
| Replicability | Whether method, data, screenshot, sampling, statistics, or provenance can be checked. |
| Conflict | Whether equal or stronger sources disagree. |

## Academic Audit

Academic audit is not "a paper exists." Check three layers:

1. The paper / guideline / consensus is real, current, and not retracted.
2. The study design can support the claim.
3. The report wording does not exceed the evidence boundary.

### Publication Validity

- Match title, author, venue, DOI / PMID / official journal page where possible.
- Mark publication status: peer-reviewed, accepted, preprint, under review, unclear, retracted_or_concern.
- Check for retraction, expression of concern, or major correction when the claim is high-risk.
- Mark funding and conflicts of interest when visible.

### Study Validity

Use the relevant reporting standard as a lightweight checklist:

- RCT / experiment: CONSORT-like checks: randomization, control, sample, predefined outcomes, dropout, harms.
- Observational study: STROBE-like checks: cohort, exposure, outcome, confounders, missing data, generalizability.
- Systematic review / meta-analysis: PRISMA-like checks: search strategy, inclusion/exclusion, bias assessment, synthesis method, certainty.
- Guideline / consensus: evidence protocol, expert group, conflicts, update date.
- Practitioner framework: acceptable for PM method, not proof of causality.

### Methodological Vulnerabilities

Flag these when present:

- Population mismatch.
- Intervention / exposure mismatch.
- Outcome is a proxy.
- Weak design for causal wording.
- Small sample or no power rationale.
- Missing data or dropout risk.
- Confounding not handled.
- Selective reporting or post-hoc subgroup emphasis.
- Effect size not meaningful despite significance.
- Harms not reported.
- Funding / COI risk.
- Retraction / correction risk.

## Contradiction Handling

- Do not select only supporting evidence.
- If conflict remains unresolved, downgrade confidence and keep the conflict in Annex A.
- If conflict affects P0/P1 recommendations, the Action Pack must include a test, kill criterion, or decision gate.

## Output

Return updated Claim Ledger rows plus a short verifier summary:

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
    "abstained_claims": 0,
    "host_verified_claims": 0,
    "host_contradicted_claims": 0,
    "host_not_checked_claims": 0,
    "notes": []
  }
}
```
