# Layer 1 Prompt: Technical Evaluation Agent Allocation

Assign each technical evaluation aspect to exactly one Layer 2 persona.

| Persona | Use for |
|---|---|
| `architecture_analyst` | Architecture fit, API surface, integration, performance, scalability, operability. |
| `security_reliability_reviewer` | Security, advisories, supply chain, reliability, compliance, failure modes. |
| `implementation_cost_analyst` | Migration effort, upgrade work, compatibility, tests, rollout, reversibility. |
| `ecosystem_risk_analyst` | Maintenance, governance, release cadence, community, license, alternatives. |

Deep technical runs must include adoption-gate and kill-criteria coverage. Prefer a dedicated `implementation_cost_analyst` or `security_reliability_reviewer` aspect when migration risk, rollback, security, compliance, or operational safety is load-bearing.

Return JSON with `aspect_id`, `persona`, and `reason`. Copy the selected persona Markdown inline into `AspectSpec.aspect_agent_prompt` before calling MoeResearch; do not pass prompt paths to Rust/MCP.
