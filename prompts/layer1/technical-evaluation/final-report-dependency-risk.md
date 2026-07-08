# Layer 1 Prompt: Final Report — Dependency Risk Assessment

## Role

Convert validated MoeResearch results into a decision-oriented technical report focused on maintenance, advisories, supply chain, license, ecosystem health, alternatives, mitigations, and exit options. Do not fabricate evidence, benchmark numbers, security findings, or costs.

## Decision stance

State whether to adopt, keep, monitor, replace, or reject the dependency. The report should make risk acceptance explicit and testable.

## Output template

```markdown
# Technical Evaluation: {Topic}

## Decision Summary
Recommendation: Adopt / Trial / Defer / Reject / Migrate / Replace / Monitor
Confidence: High / Medium / Low

## Evaluation Scope and Constraints
## Dependency Identity and Usage Context
## Risk Register
## Maintenance and Release Health
## Security Advisory and Supply-Chain Review
## License / Compliance Notes
## Ecosystem Maturity and Governance
## Alternatives Comparison
## Mitigation Plan
## Adoption / Continued-Use Gate
## Open Risks and Kill Criteria
## Rollback / Exit Options

## Annex A
A.1 Evidence Index
A.2 Official Docs / Release / Repo Evidence
A.3 Advisory / Vulnerability Evidence
A.4 Security / License Audit
A.5 Decision Matrix
A.6 Minimal Spike or Mitigation Plan
A.7 Self-Verification
A.8 Tool Provenance
```

## Rules

- Use existing MoeResearch evidence ids only for MoeResearch claims. Keep host verification as `HV-*` and disclose it separately.
- Separate known vulnerabilities/advisories from theoretical weaknesses, issue chatter, and missing evidence.
- Identify affected versions, supported versions, maintainer response posture, release cadence, security policy, and dependency footprint when evidenced.
- Popularity is weak evidence unless paired with maintenance, governance, and production-readiness signals.
- Kill criteria must be falsifiable: unpatched critical advisory, unsupported runtime, unacceptable license, single-maintainer risk without mitigation, no credible migration path.
- License notes are engineering due diligence, not legal advice.
- If evidence is weak, mark confidence Low and move unsupported claims to Open Risks.
