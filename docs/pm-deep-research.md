# PM DeepResearch with MoeResearch

PM DeepResearch is a product-manager deep research workflow built on top of the MoeResearch MCP engine. MoeResearch runs the structured research tools; the PM DeepResearch skill provides product methodology, aspect decomposition, evidence checks, and report synthesis prompts.

Use this guide when you want to produce a product research report such as:

- competitive analysis;
- single-product capability diagnosis;
- innovation direction / future-bet research;
- product-requirements research using an 8-section PR-FAQ template.

## What MoeResearch Provides

MoeResearch provides the MCP tools:

| Tool | Use |
| --- | --- |
| `deep_research` | Multi-aspect research. Use this for complete PM DeepResearch reports. |
| `aspect_research` | One targeted aspect. Use this for retries, gap backfill, or a narrow dive. |

PM DeepResearch adds the skill and prompt assets:

| Asset | Public repository path |
| --- | --- |
| Skill entry | `skills/pm-deep-research.md` |
| Layer 1 orchestration prompts | `prompts/layer1/pm-deep-research/` |
| Layer 2 persona prompts | `prompts/layer2/pm-deep-research/` |

Rust/MoeResearch owns provider calls, search, budgets, agent loops, schema validation, and byte-equal evidence provenance. The skill layer owns product methodology, report assembly, host-side fact verification, and writing quality.

## Current Installation Model

The current public delivery path is a binary install plus a repository checkout for skill assets:

1. Install or build the `moeresearch` binary.
2. Keep a checkout of this repository available so your MCP client can load `skills/pm-deep-research.md` and the prompt assets under `prompts/`.
3. Register the MoeResearch MCP server with your MCP client.
4. Load the PM DeepResearch skill through your client's skill or instruction mechanism.

Current release packaging configuration declares binary installers and package-manager metadata, but it does not declare a PM DeepResearch skill/prompt asset installation or discovery path. If you install only the binary from a release, Homebrew, npm, or MSI package, keep a repository checkout for the skill and prompt files.

## 1. Install MoeResearch

Install from a release artifact, package manager, or source as described in the repository README. A source build looks like:

```bash
cargo build --release
./target/release/moeresearch --help
```

If you use a source build, replace `moeresearch` below with `./target/release/moeresearch`.

## 2. Configure Providers

Create a user-level config:

```bash
moeresearch init --config ~/.config/moeresearch/moeresearch.toml
```

Enable at least one model provider and at least one search provider. For example, a common setup is:

- model provider: OpenAI;
- search provider: Grok, Exa, or Tavily.

Export the provider keys referenced by your config:

```bash
export OPENAI_API_KEY="..."
export XAI_API_KEY="..."
export EXA_API_KEY="..."
export TAVILY_API_KEY="..."
```

Only enabled providers require environment variables. MoeResearch stores environment variable names in `moeresearch.toml`; it does not store raw provider keys.

Check local readiness:

```bash
moeresearch check --config ~/.config/moeresearch/moeresearch.toml
```

`moeresearch check --live` does not prove API key correctness in the current release; provider reachability checks are deferred.

## 3. Register the MCP Server

For Claude Code:

```bash
moeresearch mcp register --scope local --config ~/.config/moeresearch/moeresearch.toml
```

For other MCP clients, configure an equivalent stdio server:

```json
{
  "mcpServers": {
    "moeresearch": {
      "type": "stdio",
      "command": "moeresearch",
      "args": ["serve", "--config", "/absolute/path/to/moeresearch.toml"],
      "env": {
        "OPENAI_API_KEY": "<redacted>",
        "XAI_API_KEY": "<redacted>",
        "EXA_API_KEY": "<redacted>",
        "TAVILY_API_KEY": "<redacted>"
      }
    }
  }
}
```

Include only the environment variables for providers you enabled.

You can also start the server manually for debugging:

```bash
RUST_LOG=moe_research=debug moeresearch serve --config ~/.config/moeresearch/moeresearch.toml --log-format pretty
```

MCP protocol messages use stdin/stdout. Logs are written to stderr.

## 4. Load PM DeepResearch Skill Assets

Point your client or agent instructions at:

```text
skills/pm-deep-research.md
```

The skill expects the public prompt assets to be available at:

```text
prompts/layer1/pm-deep-research/
prompts/layer2/pm-deep-research/
```

If your client supports directory-style skills that require `SKILL.md`, create a local wrapper directory using this repository checkout as the source of truth:

```text
pm-deep-research/
  SKILL.md                  # copied from skills/pm-deep-research.md
  prompts/
    layer1/pm-deep-research/
    layer2/pm-deep-research/
```

Keep the prompt paths consistent with the instructions in the skill file. Do not copy only the skill entry without its prompt assets.

## 5. Run a Product Research Request

Start with a concrete PM decision. Example:

```text
Use PM DeepResearch with MoeResearch.

Topic: Zone 2 anti-aging theory in commercial fitness products. If we consider a smart hardware + software product, is there a real opportunity?

Capability: product-requirements
Depth: deep
Output language: Chinese
Special requirements:
- verify health, fitness, and academic claims carefully;
- separate MoeResearch evidence from host WebSearch/WebFetch verification;
- include safety boundaries, no-go claims, and metrics guardrails.
```

The skill should:

1. choose the `product-requirements` capability;
2. decompose the request into MoeResearch aspect tasks;
3. call `deep_research`;
4. retry failed aspects once when useful;
5. post-process evidence;
6. run Claim Ledger, Host Verification Backfill, Evidence Verifier, and Decision Closure;
7. produce an 8-section PR-FAQ report plus Annex A.

## Expected Report Shape

For `product-requirements`, expect:

1. PR-FAQ frame;
2. opportunity validation with JTBD / ODI / Kano;
3. Cagan four risks;
4. Torres opportunity-solution tree;
5. functional requirements, non-functional requirements, and non-goals;
6. leading / secondary / guardrail metrics;
7. evidence and source summary;
8. open questions and next steps;
9. Annex A with evidence index, Claim Ledger, visual evidence, host verification, falsification matrix, self-verification, abstain log, and tool provenance.

For competitive, product-capability, and innovation-direction reports, expect a 13-section narrative report with Annex A.

## Evidence and Fact Verification

MoeResearch evidence and host-side verification must stay separate.

| Source origin | Meaning |
| --- | --- |
| MoeResearch evidence | Frozen `DeepResearchResult.evidence_index` and aspect findings. |
| Skill-side WebSearch/WebFetch backfill | Host-native verification rows for load-bearing facts that need original-source checks. |
| Manual/host verification | Browser captures, local logs, screenshots, or direct artifact inspection. |

Host WebSearch/WebFetch is mandatory when triggered by:

- time-sensitive product, pricing, release, policy, API, or competitor-state claims;
- official-source claims where exact wording matters;
- academic, scientific, health, sports, fitness, injury, recovery, safety, or nutrition claims;
- quantitative claims that drive a recommendation;
- weak-source evidence used for P0/P1 decisions;
- contradictions between sources.

Host-found sources use `HV-*` references. They must not be inserted into MoeResearch `evidence_index` or represented as MoeResearch evidence.

If WebSearch/WebFetch is unavailable, the report must disclose the limitation, lower confidence, and move unresolved high-impact claims to open questions or the Action Pack.

## Partial Results and Failed Aspects

`status=partial` is still useful. Completed aspects should be kept. Failed aspects should be treated as gaps and, when worthwhile, retried once with `aspect_research` using prior sources.

The final report must disclose failed aspects in open questions or Annex A. It must not silently synthesize unsupported recommendations from missing aspects.

## Troubleshooting

### MCP handshake failure

Run:

```bash
moeresearch check --config ~/.config/moeresearch/moeresearch.toml
moeresearch serve --config ~/.config/moeresearch/moeresearch.toml --log-format pretty
```

Confirm that the MCP client launches the same `moeresearch` binary and passes the intended config path. Check stderr logs for provider, config, or panic messages.

### Missing provider environment variables

`moeresearch check` validates that enabled providers reference environment variables that are set. Export the variables in the environment where the MCP client launches MoeResearch, not only in your shell.

### HTTP 429 or provider rate limits

Reduce concurrency or depth. Use a smaller PM DeepResearch tier (`quick` or `standard`), or retry later. Do not remove evidence requirements to bypass rate limits.

### `schema_validation_failed: mutated_evidence_provenance`

The aspect agent changed a frozen evidence field. Evidence fields copied from MoeResearch search results must be byte-equal. Retry the failed aspect with a smaller evidence set and avoid copying all search results.

### WebSearch/WebFetch unavailable

Continue with MoeResearch results only when claims can be supported by MoeResearch evidence. For claims that require original-source checks, lower confidence, disclose the limitation in Annex A.8, and move high-impact unresolved claims to open questions or validation tests.

### Skill asset path mismatch

Make sure the skill entry and prompt directories are present together:

```text
skills/pm-deep-research.md
prompts/layer1/pm-deep-research/
prompts/layer2/pm-deep-research/
```

If your client copies the skill into another directory, copy the prompt assets too or adjust the paths in the local wrapper.

## What This Does Not Do

- It does not add native WebFetch to the MoeResearch Rust engine.
- It does not make host WebSearch/WebFetch part of MoeResearch provenance.
- It does not install PM DeepResearch skill assets through the current release, Homebrew, npm, or MSI packages.
- It does not replace source verification for high-risk health, fitness, safety, regulatory, or academic claims.
