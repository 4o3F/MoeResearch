# Layer 1 Prompt: Academic Agent Allocation

Assign each academic aspect to exactly one Layer 2 persona.

| Persona | Use for |
|---|---|
| `literature_reviewer` | Field maps, definitions, seminal/current work, schools of thought. |
| `methods_critic` | Study design, validity, bias, methods, limitations. |
| `evidence_synthesizer` | Claim/outcome synthesis, certainty, consensus, contradictions. |
| `citation_verifier` | DOI/PMID/source validity, retraction/correction checks, citation faithfulness. |

Deep academic runs must include citation/source-validity coverage. Prefer a dedicated `citation-verifier` aspect for paper evaluation and evidence synthesis; otherwise add citation-validity success criteria to the most source-heavy aspect.

Return JSON with `id`, `persona`, and `reason`. The task-decomposition step assembles `AspectRequest.instructions` according to the selected tools: persona only; persona + search contract + Run Binding; persona + WebFetch contract; or persona + both contracts + Run Binding. Do not pass prompt paths to Rust/MCP.

## Run Binding handoff

For every search-only aspect, persona selection is followed by the complete inline assembly order: selected persona Markdown, then `prompts/layer1/common/model-search-tool-contract.md`, then the request-specific Run Binding. The binding is derived from that aspect and `policy.search` according to `moe.run_binding.v1`; it carries only semantic `allowed_*` intent choices, safe defaults, literal aspect ID/name anchors, and evidence-closure hints. It must not expose provider routing, budgets, domains, raw policy tool fields, or credentials.

For a search-only aspect, the mandatory order is selected persona Markdown, then the common search contract, then a request-specific Run Binding. For a dual-tool aspect, insert the WebFetch contract before the Run Binding. The fixed-category rule is profile-neutral: fixed `academic` permits `general` or `academic`; an unset category permits the full source-focus vocabulary.
