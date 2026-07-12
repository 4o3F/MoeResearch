# Layer 1 Prompt: Technical Evaluation Agent Allocation

Assign each technical evaluation aspect to exactly one Layer 2 persona.

| Persona | Use for |
|---|---|
| `architecture_analyst` | Architecture fit, API surface, integration, performance, scalability, operability. |
| `security_reliability_reviewer` | Security, advisories, supply chain, reliability, compliance, failure modes. |
| `implementation_cost_analyst` | Migration effort, upgrade work, compatibility, tests, rollout, reversibility. |
| `ecosystem_risk_analyst` | Maintenance, governance, release cadence, community, license, alternatives. |

Deep technical runs must include adoption-gate and kill-criteria coverage. Prefer a dedicated `implementation_cost_analyst` or `security_reliability_reviewer` aspect when migration risk, rollback, security, compliance, or operational safety is load-bearing.

Return JSON with `id`, `persona`, and `reason`. For every search-enabled aspect, set `AspectRequest.instructions` to the selected persona Markdown, then `prompts/layer1/common/model-search-tool-contract.md`, then a request-specific Run Binding before calling MoeResearch; do not pass prompt paths to Rust/MCP.

## Run Binding handoff

For every search-enabled aspect, persona selection is followed by the complete inline assembly order: selected persona Markdown, then `prompts/layer1/common/model-search-tool-contract.md`, then the request-specific Run Binding. The binding is derived from that aspect and `policy.search` according to `moe.run_binding.v1`; it carries only semantic `allowed_*` intent choices, safe defaults, literal aspect ID/name anchors, and evidence-closure hints. It must not expose provider routing, budgets, domains, raw policy tool fields, or credentials.

This three-part order is mandatory for every search-enabled aspect. The fixed-category rule is profile-neutral: fixed `academic` permits `general` or `academic`; an unset category permits the full source-focus vocabulary.
