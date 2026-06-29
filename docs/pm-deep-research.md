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

## Release Asset Installation

The supported release delivery path is a binary install plus CLI-installed skill assets:

1. Install or build the `moeresearch` binary.
2. Run `moeresearch assets install pm-deep-research` for Claude Code, or use `--target <dir> --layout repo` for a generic asset root.
3. Register the MoeResearch MCP server with your MCP client.
4. Load the PM DeepResearch skill through your client's skill or instruction mechanism.

The installer downloads the PM DeepResearch release asset matching the current binary version. It does not default to GitHub `latest`, so skill prompts stay aligned with the installed CLI.

Source checkouts remain useful for development and as a fallback when release assets are unavailable.

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

For Claude Code, inspect the install plan first if desired, then install the user-level skill assets:

```bash
moeresearch assets install pm-deep-research --dry-run
moeresearch assets install pm-deep-research
```

This creates the Claude Code discovery layout:

```text
~/.claude/skills/pm-deep-research/
  SKILL.md
  prompts/layer1/pm-deep-research/
  prompts/layer2/pm-deep-research/
```

For a project-local Claude Code skill:

```bash
moeresearch assets install pm-deep-research \
  --client claude-code \
  --scope project
```

Claude Code discovers skills from `~/.claude/skills/<name>/SKILL.md` or `./.claude/skills/<name>/SKILL.md`.

For non-Claude clients, install a repository-style asset root:

```bash
moeresearch assets install pm-deep-research \
  --target ~/.config/moeresearch/assets \
  --layout repo
```

Generic layout preserves sibling `skills/` and `prompts/` directories:

```text
~/.config/moeresearch/assets/skills/pm-deep-research.md
~/.config/moeresearch/assets/prompts/layer1/pm-deep-research/
~/.config/moeresearch/assets/prompts/layer2/pm-deep-research/
```

Generic target installs are not Claude Code discovery installs. Existing differing files are preserved by default; pass `--force` only when you want to overwrite files owned by the release manifest. Installer status is written to stderr, and normal stdout stays empty. Repository checkout remains a source/development fallback only.

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

For Claude Code, prefer the installer so `SKILL.md` and prompts are placed together:

```bash
moeresearch assets install pm-deep-research
```

Expected user-level layout:

```text
~/.claude/skills/pm-deep-research/SKILL.md
~/.claude/skills/pm-deep-research/prompts/layer1/pm-deep-research/
~/.claude/skills/pm-deep-research/prompts/layer2/pm-deep-research/
```

For generic clients, ensure the installed repo-like asset root contains both `skills/pm-deep-research.md` and the sibling `prompts/` tree.

## What This Does Not Do

- It does not add native WebFetch to the MoeResearch Rust engine.
- It does not make host WebSearch/WebFetch part of MoeResearch provenance.
- It does not make MCP registration install client-side PM DeepResearch skill assets.
- It does not replace source verification for high-risk health, fitness, safety, regulatory, or academic claims.
