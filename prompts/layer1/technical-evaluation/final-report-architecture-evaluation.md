# Layer 1 Prompt: Final Report — Architecture Option Evaluation

## Role

Convert validated MoeResearch results into a decision-oriented technical report focused on quality attributes, integration, operational model, trade-offs, security/reliability boundaries, migration path, and exit options. Do not fabricate evidence, benchmark numbers, security findings, or costs.

## Decision stance

State the recommendation first, then make trade-offs explicit. The report should be usable as an ADR input: context, options, decision, consequences, verification, and reversal path.

## Output template

```markdown
# Technical Evaluation: {Topic}

## Decision Summary
Recommendation: Adopt / Trial / Defer / Reject / Migrate / Replace / Monitor
Confidence: High / Medium / Low

## Evaluation Scope and Constraints
## Candidate Architecture Options
## Decision Criteria Matrix
## Requirements and Quality Attributes
## Architecture and Integration Analysis
## Operational Model
## Performance and Scalability Evidence
## Security / Compliance / Reliability Risks
## Ecosystem and Platform Maturity
## Alternatives Comparison
## Recommended Option and Consequences
## Adoption Gate
## Minimal Spike / Verification Plan
## Open Risks and Kill Criteria
## Rollback / Exit Options

## Annex A
A.1 Evidence Index
A.2 Official Docs / Release / Repo Evidence
A.3 Benchmark Evidence
A.4 Security / License Audit
A.5 Decision Matrix
A.6 Minimal Spike Plan
A.7 Self-Verification
A.8 Tool Provenance
```

## Rules

- Use existing MoeResearch evidence ids only for MoeResearch claims. Keep host verification as `HV-*` and disclose it separately.
- Evaluate quality attributes explicitly: availability, scalability, latency, consistency, durability, operability, observability, security, compliance, and maintainability when relevant.
- Separate documented guarantees from examples, marketing claims, community anecdotes, and inferred engineering judgment.
- Benchmark claims must include workload, version, hardware/runtime, methodology, variance/reproducibility limits, and applicability to the user's context when available.
- The adoption gate and kill criteria must be falsifiable and tied to local verification where evidence is missing.
- License notes are engineering due diligence, not legal advice.
- If evidence is weak, mark confidence Low and move unsupported claims to Open Risks.
