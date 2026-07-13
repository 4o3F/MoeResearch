# Layer 1 Common Module: Claim Ledger

Domain-neutral audit trail for claims that affect conclusions, recommendations, risk posture, or confidence. Skill-layer module only — do not require a Rust/MoeResearch schema change. Build the ledger from `DeepResearchResult`, `evidence_index`, draft report text, and evidence post-processing output.

## Purpose

Create a compact audit trail for load-bearing claims. The ledger is not a longer bibliography. It answers:

- What claim is being made?
- Where does it appear?
- What evidence supports it?
- Has support, conflict, freshness, source independence, and domain validity been checked?
- Was any host-side WebSearch/WebFetch/browser/manual verification used, and did it change the claim?
- Should the claim stay in the body, be downgraded, move to open questions, or be abstained?

## Scope

| Complexity | Claim Ledger Scope |
|---|---|
| `quick` | Optional; mark obvious weak claims inline. |
| `standard` | Extract the most important 5-10 claims. |
| `deep` | Extract all load-bearing claims and sample ordinary claims. |
| `deep` + `evidence_pack` | Also extract all body and annex claims that depend on evidence. |

`load_bearing=true` when a claim would change a conclusion, recommendation, risk rating, design choice, research gap, technical decision, safety boundary, or next-step plan if wrong.

## Ledger schema

Use literal frozen evidence IDs from the result. An individual aspect result may use `ev-1-1`; a deep result may use `aspect-id:ev-1-1`. Never reconstruct, shorten, or remove an aspect namespace.

```json
{
  "claim_id": "CL-001",
  "claim_text": "string",
  "claim_type": "fact|interpretation|estimate|recommendation|risk|academic_claim|technical_claim|regulatory_claim|benchmark_claim|security_claim|methodological_claim",
  "load_bearing": true,
  "appears_in": ["body:section-name"],
  "evidence_refs": ["aspect-id:ev-1-1"],
  "host_verification_refs": ["HV-001"],
  "source_origin": "moe_research_only|host_verified|mixed|manual_host_verified",
  "source_tiers": ["High"],
  "support_status": "supported|partial|unsupported|not_checked",
  "contradiction_status": "no_conflict_found|conflict_resolved|conflict_unresolved|not_checked",
  "freshness_status": "current|stale|date_unknown|not_time_sensitive",
  "academic_status": "not_academic|peer_reviewed|preprint|accepted|under_review|desk_rejected|retracted_or_concern|unclear",
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
4. Claims supported only by community / Low / Unknown evidence can stay as sentiment or hypotheses, not facts.
5. `evidence_refs` may contain only frozen MoeResearch evidence IDs.
6. Host WebSearch/WebFetch/browser/manual rows use `host_verification_refs` with `HV-*`; they never replace MoeResearch evidence IDs.
7. If a host verification row changes confidence or wording, preserve the original claim meaning in `claim_text` and reflect the change through `confidence`, `confidence_reason`, and `action`.
8. Unsupported load-bearing claims cannot remain as facts in the body. Narrow, downgrade, move to open questions, or abstain.
