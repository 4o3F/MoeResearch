# Layer 1 Module: Decision Closure (PM DeepResearch)

> Skill-layer module for turning research findings into PM action. It is especially important for `product-requirements`, where the output may feed PRD, design, engineering, experiment planning, or stakeholder review.

## Purpose

Every P0/P1 recommendation must close the loop:

- What has to be true?
- What is the cheapest credible test?
- What evidence can be obtained this week?
- What would make us stop, downgrade, or pivot?
- Which metrics and guardrails judge success without creating hidden harm?

If a recommendation cannot answer those questions, it is not ready to be a P0/P1 recommendation.

## Required Action Pack Fields

For each P0/P1 recommendation:

```json
{
  "recommendation_id": "REC-001",
  "recommendation": "string",
  "linked_claim_ids": ["CL-001"],
  "linked_evidence_refs": ["E1"],
  "load_bearing_assumptions": [
    {
      "assumption": "string",
      "risk_type": "desirability|viability|feasibility|usability|ethical|regulatory|safety",
      "confidence": "high|medium|low",
      "why_it_matters": "string"
    }
  ],
  "cheapest_test": "string",
  "evidence_to_get_this_week": "string",
  "kill_criterion": "Fails if ...",
  "success_metric": "string",
  "guardrail_metrics": ["string"],
  "owner": "string|null",
  "timebox": "string"
}
```

## Assumption Types

Use these types unless the product context requires a narrower label:

- `desirability`: users want it, understand it, and choose it over alternatives.
- `viability`: the business model, pricing, economics, GTM, or operating model works.
- `feasibility`: engineering, data, legal, content, operations, or vendor constraints are solvable.
- `usability`: users can complete the task without unacceptable friction or misunderstanding.
- `ethical`: the product avoids manipulation, discrimination, unhealthy incentives, or privacy harm.
- `regulatory`: the claim, workflow, or data use stays inside allowed boundaries.
- `safety`: wrong recommendations do not create unacceptable physical, financial, or user-trust harm.

## Test Design Rules

- Prefer the cheapest test that can falsify the risky assumption.
- Do not default to "do more research." Name the artifact: prototype, fake door, concierge test, landing page, diary study, concept test, A/B test, data audit, expert review, compliance review, or validation study.
- A test must have a decision rule. If it has no decision rule, it is not a test.
- For high-risk health / fitness / sports claims, include a safety gate and a no-go condition.
- For metrics, separate leading, secondary, and guardrail metrics.

## Product-Requirements Placement

- Body segment 8 keeps only the top 3 decision-critical actions.
- Annex A.4 keeps the full open-question table.
- Annex A.5 keeps falsification and kill criteria.
- Annex A.6 records whether every P0/P1 recommendation has an Action Pack.
