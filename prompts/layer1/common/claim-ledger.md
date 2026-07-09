# Layer 1 Common Module: Claim Ledger

Domain-neutral audit trail for claims that affect conclusions, recommendations, risk posture, or confidence.

## Scope

| Complexity | Claim Ledger Scope |
|---|---|
| `quick` | Optional; mark obvious weak claims inline. |
| `standard` | Extract the most important 5-10 claims. |
| `deep` | Extract all load-bearing claims and sample ordinary claims. |
| `deep_evidence_pack` | Extract all body and annex claims that depend on evidence. |

`load_bearing=true` when a claim would change a conclusion, recommendation, risk rating, design choice, research gap, technical decision, safety boundary, or next-step plan if wrong.

## Ledger schema

```json
{
  "claim_id": "CL-001",
  "claim_text": "string",
  "claim_type": "fact|interpretation|estimate|recommendation|risk|academic_claim|technical_claim|regulatory_claim|benchmark_claim|security_claim|methodological_claim",
  "load_bearing": true,
  "appears_in": ["body:section-name"],
  "evidence_refs": ["E1"],
  "host_verification_refs": ["HV-001"],
  "source_origin": "moe_research_only|host_verified|mixed|manual_host_verified",
  "source_tiers": ["High"],
  "support_status": "supported|partial|unsupported|not_checked",
  "contradiction_status": "no_conflict_found|conflict_resolved|conflict_unresolved|not_checked",
  "freshness_status": "current|stale|date_unknown|not_time_sensitive",
  "academic_status": "not_academic|peer_reviewed|preprint|accepted|under_review|retracted_or_concern|unclear",
  "technical_status": "not_technical|official_docs_checked|repo_checked|release_checked|benchmark_checked|security_checked|license_checked|unclear",
  "independence_status": "independent|vendor_owned|practitioner_self_report|community|unknown",
  "confidence": "high|medium|low",
  "confidence_reason": "string",
  "action": "keep|downgrade|move_to_open_question|abstain"
}
```

## Rules

1. Extract atomic claims; split compound claims when evidence support differs.
2. Preserve exact meaning. Do not strengthen weak wording.
3. Mark recommendations, risk judgments, causal claims, quantitative claims, scientific claims, security claims, benchmark claims, and current-state claims as load-bearing by default.
4. `evidence_refs` may contain only frozen MoeResearch evidence IDs.
5. Host WebSearch/WebFetch/browser/manual rows use `host_verification_refs` with `HV-*`; they never replace MoeResearch evidence IDs.
6. Unsupported load-bearing claims cannot remain as facts in the body. Narrow, downgrade, move to open questions, or abstain.
