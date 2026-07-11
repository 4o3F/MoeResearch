# Technical Evaluation

Technical Evaluation is a Skill/profile layer over the MoeResearch MCP core for evidence-backed engineering decisions.

## Use Cases

- Compare libraries, frameworks, SDKs, databases, queues, runtimes, or platforms.
- Evaluate architecture options and integration trade-offs.
- Assess dependency security, maintenance, license, and supply-chain risk.
- Plan migrations, upgrades, or replacements.
- Review benchmark claims and performance evidence.

## Capabilities

`library-framework-comparison`, `architecture-option-evaluation`, `dependency-risk-assessment`, `migration-upgrade-assessment`, `benchmark-performance-review`, and `technical-due-diligence`.

## Setup

```bash
moeresearch mcp register --scope local --config ~/.config/moeresearch/moeresearch.toml
moeresearch assets install research-skills --config ~/.config/moeresearch/moeresearch.toml
```

The research skills asset installs Technical Evaluation prompts under `prompts/layer1/technical-evaluation/` and `prompts/layer2/technical-evaluation/`.

## Request Example

```text
Use Technical Evaluation with MoeResearch.
Topic: Evaluate Axum vs Actix-web for a Rust MCP service backend.
Capability: library-framework-comparison
Depth: standard
Output language: Chinese
Special requirements:
- compare API stability, ecosystem, performance evidence, maintenance risk;
- include a minimal spike validation plan.
```

## Expected Report Shape

Technical reports include decision summary, scope and constraints, requirements fit, architecture and integration, API/developer experience, performance evidence, security/compliance/license risk, ecosystem maturity, operational cost, alternatives, verification plan, open risks, and kill criteria.

## Source Quality Rules

Prefer official docs, release notes, repositories, security advisories, standards, benchmark methodology pages, package registries, license files, and credible independent engineering writeups. Community posts are useful for signals but should not carry load-bearing decisions alone.

## Benchmark and Security Caveats

Benchmark claims require versions, environment, workload, concurrency, warmup, variance, and reproducibility context. Security claims must not infer safety from limited search; phrase as "no advisory was found in this evidence set" when appropriate.

## Limitations

Technical Evaluation supports decision preparation but does not replace a local spike, code review, SCA/SBOM tooling, production load testing, legal license review, or organization-specific architecture governance.
