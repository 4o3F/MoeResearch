# PM DeepResearch: Evidence Modules Overlay

Task-specific instructions for PM DeepResearch only.
Load the matching shared module under `../common/` first, then apply the overlay section below.
Do not use this file from academic, technical, or generic profiles.

## After `../common/claim-ledger.md`

Additional load-bearing defaults and report placement for product-decision work:

1. Treat PR-FAQ value propositions, P0/P1 functional requirements, recommendation priority, metrics, and safety boundaries as load-bearing by default.
2. Treat sports, fitness, health, regulatory, safety, injury, recovery, nutrition, REDs, return-to-play, and medical-like claims as load-bearing unless clearly trivial.
3. For `standard` product-requirements runs, extract 5-10 key claims from the PR-FAQ, recommendations, risks, and metrics.
4. Product-requirements placement:
   - Body: keep only claim IDs and confidence labels where they help decision-making.
   - Annex A.1: add `claim_ids`, `independence_status`, `freshness_status`; if host verification exists, include `host_verification_refs` and `source_origin`.
   - Annex A.6: summarize coverage and unsupported / downgraded / abstained counts.
   - If Annex A has fixed eight sections, put the full Claim Ledger table inside A.1 after the Evidence Index, not as a new top-level Annex section.

## After `../common/evidence-postprocess.md`

Product-research specializations:

1. Prefer `source_type` + domain heuristics used by PM reports (official / docs / release notes / app-store → High; named media/engineering blogs → Medium; app-store reviews / social / forums → Low; undated social → Unknown).
2. Visual-evidence assembly feeds Ch 7 / Annex A.2. For Deep (with `evidence_pack` optional), if visual items `< 5`, trigger host browser capture once for missing product surfaces named in experience-path open questions. If still short, record `visual_gap` and forbid strong UI breakpoint conclusions.
3. Host browser capture may save files under the project working directory (for example `captures/`) and add rows with `media_type=screenshot` and a real page URL or local path. Do not fabricate a URL for an image you did not capture.
4. CiteEval should sample importance ∈ {critical, high} findings and, at minimum, load-bearing claims that drive Ch 1 / Ch 5 / Ch 9 / Ch 10 narrative sections or the 8-section PR-FAQ decision spine. Unsupported claims move to the active profile's open-questions / assumptions section (13-section narrative Ch 12, or PR-FAQ 段8).

## After `../common/evidence-verifier.md`

1. Prefer host verification for current product facts, pricing, policy, release state, health/safety claims, academic claims, and commercial recommendations that affect product decisions.
2. Practitioner frameworks are acceptable for PM method discussion, not proof of causality.
3. If unresolved conflict affects P0/P1 recommendations, the Action Pack must include a test, kill criterion, or decision gate.
4. Product-requirements reports may link `HV-*` rows from Annex A.1 Claim Ledger when a load-bearing claim was host-verified; put verifier coverage counts in A.6.

## After `../common/host-verification-backfill.md`

1. Prefer host checks for product facts, pricing, policy, release state, health/safety claims, academic claims, and commercial recommendations that affect product decisions.
2. Place the host verification table in Annex A.6 or A.8 depending on the report template:
   - A.6 self-verification should summarize counts and confidence/action changes.
   - A.8 tool provenance should disclose host-side tools and unavailable-tool limitations.
   - Product-requirements reports may also link `HV-*` rows from A.1 Claim Ledger when a load-bearing claim was host-verified.
3. If host tools are unavailable, move unresolved high-impact product claims into open questions / Action Pack rather than inventing certainty.

