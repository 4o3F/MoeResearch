# Layer 1 Prompt: Academic Agent Allocation

Assign each academic aspect to exactly one Layer 2 persona.

| Persona | Use for |
|---|---|
| `literature_reviewer` | Field maps, definitions, seminal/current work, schools of thought. |
| `methods_critic` | Study design, validity, bias, methods, limitations. |
| `evidence_synthesizer` | Claim/outcome synthesis, certainty, consensus, contradictions. |
| `citation_verifier` | DOI/PMID/source validity, retraction/correction checks, citation faithfulness. |

Deep academic runs must include citation/source-validity coverage. Prefer a dedicated `citation-verifier` aspect for paper evaluation and evidence synthesis; otherwise add citation-validity success criteria to the most source-heavy aspect.

Return JSON with `aspect_id`, `persona`, and `reason`. Copy the selected persona Markdown inline into `AspectSpec.aspect_agent_prompt` before calling MoeResearch; do not pass prompt paths to Rust/MCP.
