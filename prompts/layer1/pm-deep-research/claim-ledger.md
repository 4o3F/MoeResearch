# Layer 1 Module: Claim Ledger (PM DeepResearch)

> Skill-layer module. Do not require a Rust/Lapis schema change. Build this ledger from `DeepResearchResult`, `evidence_index`, draft report text, and the evidence post-processing output.

## Purpose

Create a compact audit trail for claims that affect product decisions. The ledger is not a longer bibliography. It answers:

- What claim is being made?
- Where does it appear?
- What evidence supports it?
- Has the support, conflict, freshness, source independence, and academic status been checked?
- Was any host-side WebSearch/WebFetch verification used, and did it change the claim?
- Should the claim stay in the body, be downgraded, move to open questions, or be abstained?

## Scope by Complexity

| Complexity | Claim Ledger Scope |
|---|---|
| `quick` | Not mandatory. Mark obvious low-confidence findings inline. |
| `standard` | Extract 5-10 key claims from the PR-FAQ, recommendations, risks, and metrics. |
| `deep` | Extract 100% of load-bearing claims. Sample ordinary claims. |
| `deep_evidence_pack` | Extract all claims that appear in body sections and Annex A. |

`load_bearing=true` when the claim would change the PR-FAQ recommendation, P0/P1 priority, risk rating, requirement scope, metrics, safety boundary, or next-step decision if wrong.

## Claim Types

- `fact`: directly checkable fact.
- `interpretation`: evidence-based explanation or pattern.
- `estimate`: proxy-based or non-primary estimate.
- `recommendation`: PM recommendation.
- `risk`: risk judgment.
- `academic_claim`: depends on paper, guideline, consensus, or scientific evidence.
- `regulatory_claim`: depends on regulator or legal/compliance guidance.

## Ledger Schema

```json
{
  "claim_id": "CL-001",
  "claim_text": "string",
  "claim_type": "fact|interpretation|estimate|recommendation|risk|academic_claim|regulatory_claim",
  "load_bearing": true,
  "appears_in": ["body:segment-1", "annex:a1"],
  "evidence_refs": ["E1"],
  "host_verification_refs": ["HV-001"],
  "source_origin": "lapis_only|host_verified|mixed|manual_host_verified",
  "source_tiers": ["Tier 1-2"],
  "support_status": "supported|partial|unsupported|not_checked",
  "contradiction_status": "no_conflict_found|conflict_resolved|conflict_unresolved|not_checked",
  "freshness_status": "current|stale|date_unknown|not_time_sensitive",
  "academic_status": "not_academic|peer_reviewed|preprint|accepted|under_review|desk_rejected|retracted_or_concern|unclear",
  "independence_status": "independent|vendor_owned|practitioner_self_report|community|unknown",
  "confidence": "high|medium|low",
  "confidence_reason": "string",
  "action": "keep|downgrade|move_to_open_question|abstain"
}
```

## Extraction Rules

1. Extract atomic claims. Split compound sentences into separate claims when different evidence supports each part.
2. Preserve exact meaning. Do not strengthen weak language during extraction.
3. Mark recommendations and risks as load-bearing by default.
4. Mark PR-FAQ value proposition claims as load-bearing.
5. Mark product-requirements P0/P1 functional requirements as load-bearing.
6. Mark sports, fitness, health, regulatory, safety, injury, recovery, nutrition, REDs, return-to-play, and medical-like claims as load-bearing unless clearly trivial.
7. Claims supported only by community evidence can stay as sentiment or hypotheses, not facts.
8. `evidence_refs` can only contain frozen Lapis evidence IDs. Host WebSearch/WebFetch rows use `host_verification_refs` (`HV-*`) and never replace Lapis evidence IDs.
9. If a host verification row changes confidence or wording, preserve the original claim meaning in `claim_text` and reflect the change through `confidence`, `confidence_reason`, and `action`.

## Output Placement

For product-requirements reports:

- Body: keep only claim IDs and confidence labels where they help decision-making.
- Annex A.1: add `claim_ids`, `independence_status`, `freshness_status`.
- Annex A.1: if host verification exists, include `host_verification_refs` and `source_origin`.
- Annex A.6: summarize coverage and unsupported / downgraded / abstained counts.
- If Annex A has fixed eight sections, put the full Claim Ledger table inside A.1 after the Evidence Index, not as a new top-level Annex section.
