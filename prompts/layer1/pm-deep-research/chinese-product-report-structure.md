# Layer 1 Module: Chinese Product Research Report Structure

> Skill-layer writing module. Use this inside PM DeepResearch final synthesis. Do not require users to call `/humanizer-zh` separately.

## Positioning

The Chinese output is a professional product research report for PMs and product leaders. It is not a casual article, marketing copy, academic paper, or raw evidence dump.

The reader should understand within 3-5 minutes:

- What decision is being made.
- What the recommendation is.
- What not to do.
- Which evidence supports the recommendation.
- What is uncertain.
- What action can be taken this week.

## Three-Layer Structure

| Layer | Reader Job | Content Boundary |
|---|---|---|
| Decision Memo Body | Decide whether to accept the recommendation. | Conclusion, evidence summary, trade-offs, risks, action. No long method exposition. |
| Annex A Evidence Pack | Audit facts and evidence quality. | Lapis Evidence Index, Claim Ledger, host verification backfill, source audit, contradiction log, academic audit, self-verification. |
| Action Pack | Start validation or iteration. | Assumptions, cheapest tests, kill criteria, metrics, guardrails, owner, cadence. |

For `product-requirements`, keep the existing 8-segment PR-FAQ skeleton. This module changes writing quality and section emphasis; it does not replace the skeleton.

## Chinese Writing Rules

Integrate de-AI writing rules directly into PM DeepResearch. Do not ask the user to run a separate humanizing skill.

- Lead with the decision, then give evidence.
- Use action-title headings. A heading should state the section's point, not just name the topic.
- Avoid empty connectors and inflated phrases such as "此外", "值得注意的是", "至关重要", "复杂格局", "深度赋能".
- Avoid formulaic "这不仅是 X，更是 Y" structures.
- Avoid mechanical three-part lists unless the evidence really has three parts.
- Use bold sparingly. Bold only decisions, risks, and action gates.
- Keep uncertainty, but make it specific: "缺少独立 validation study" is better than "仍需进一步观察".
- Use PM verbs: verify, downgrade, pause, pivot, narrow scope, change claim wording, add guardrail.
- Do not remove honesty markers: confidence, estimates, abstain, source gaps, visual-evidence gaps, contradiction, and tool provenance.
- Do not merge source origins for smoother prose. Keep Lapis evidence, Skill-side WebSearch/WebFetch backfill, and manual/host verification visibly separate in Annex A.

## Product Report Tone

Good:

> 这个能力的价值不在于多给一个分数，而在于解释"今天为什么该练或该休息"。如果 HRV、睡眠和训练负荷的解释链说不清，就不应把它包装成恢复处方。

Bad:

> 该能力不仅将重塑用户训练体验，更将成为运动健康智能化的重要里程碑。

## Self-Check

Before emitting:

- The first segment can stand alone as the decision summary.
- Every P0/P1 recommendation links to claim IDs and evidence refs.
- Every high-risk claim keeps confidence and safety boundary.
- Every host-verified claim keeps its `HV-*` reference or a clear host-verification note in Annex A.
- The body is readable without Annex A.
- Annex A contains enough detail to audit the body.
- The report does not depend on post-hoc rewriting.
