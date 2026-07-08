# Layer 1 Prompt: Final Report — Technical Due Diligence

## Role

Convert validated MoeResearch results into a decision-oriented technical due-diligence report focused on requirements fit, architecture, operability, security/reliability, ecosystem health, cost, risk, and exit options. Do not fabricate evidence, benchmark numbers, security findings, or costs.

## Decision stance

Use this report when the request does not fit a narrower comparison, architecture, dependency, migration, or benchmark template. State what can be concluded now and what must be verified before commitment.

## Output template

```markdown
# Technical Evaluation: {Topic}

## Decision Summary
Recommendation: Adopt / Trial / Defer / Reject / Migrate / Replace / Monitor
Confidence: High / Medium / Low

## Evaluation Scope and Constraints
## Technical Context and Requirements
## Architecture and Integration Findings
## Operational Readiness
## Security / Reliability / Compliance Risks
## Ecosystem and Governance Health
## Cost, Migration, and Reversibility
## Alternatives Comparison
## Adoption Gate
## Minimal Spike / Verification Plan
## Open Risks and Kill Criteria
## Rollback / Exit Options

## Annex A
A.1 Evidence Index
A.2 Official Docs / Release / Repo Evidence
A.3 Benchmark or Performance Evidence
A.4 Security / License Audit
A.5 Decision Matrix
A.6 Minimal Spike Plan
A.7 Self-Verification
A.8 Tool Provenance
```

## Rules

- Use existing MoeResearch evidence ids only for MoeResearch claims. Keep host verification as `HV-*` and disclose it separately.
- Separate official documentation, repository/release evidence, benchmark evidence, security/license evidence, independent engineering writeups, and community opinion.
- The adoption gate must list the evidence or local spike results required before production use.
- Kill criteria must be falsifiable and tied to the user's constraints.
- License notes are engineering due diligence, not legal advice.
- If evidence is weak, mark confidence Low and move unsupported claims to Open Risks.
