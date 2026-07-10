# MCP Usage Guide

This document is for clients that call MoeResearch through MCP. It covers transport rules, tool names, request payloads, response envelopes, and public error formats.

## 1. Transport

MoeResearch exposes an MCP server over stdio.

Protocol rules:

- The client sends JSON-RPC messages to the server process stdin.
- The server writes JSON-RPC responses and notifications to stdout.
- The server writes logs to stderr.
- Each JSON-RPC message is one JSON object followed by `\n`.
- Current stdio transport does not use `Content-Length` framing.
- Do not send API keys, Authorization headers, cookies, JWTs, or other secrets in MCP payloads.

## 2. Claude Code registration

Prefer the CLI registration command instead of editing Claude settings by hand:

```bash
moeresearch mcp register --scope local --config ~/.config/moeresearch/moeresearch.toml
```

Use `--dry-run` to print the command and JSON example without invoking `claude`; provider environment values are redacted:

```bash
moeresearch mcp register --scope local --config ~/.config/moeresearch/moeresearch.toml --dry-run
```

MCP registration only configures the stdio server. It does not install research Skill or prompt assets. For Claude Code, run:

```bash
moeresearch assets install research-skills
```

## 3. MCP lifecycle

Expected tool names from `tools/list`:

```text
aspect_research
deep_research
```

Both tools use MoeResearch request schema version `0.2`:

```json
"schema_version": "0.2"
```

Any other value returns `unsupported_schema_version`.

## 4. Invocation shapes

### 4.1 Claude Code direct MCP invocation

Claude Code usually exposes tools as direct names such as `mcp__moeresearch__deep_research` or `mcp__moeresearch__aspect_research`. In that mode, pass only the tool-specific MoeResearch request object.

Do not paste the outer JSON-RPC wrapper into a direct Claude Code MCP tool call. Do not wrap the request under `params`, `arguments`, `request`, `input`, or `tool_input`.

### 4.2 Raw MCP `tools/call` wrapper

Raw MCP clients use the standard JSON-RPC `tools/call` method with `params.name` and `params.arguments`:

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "tools/call",
  "params": {
    "name": "aspect_research",
    "arguments": {
      "schema_version": "0.2"
    }
  }
}
```

`params.arguments` is the tool-specific request object.

## 5. Common request objects

### 5.1 Limit values

Limit and timeout fields use this wire format:

| JSON value | Meaning |
| --- | --- |
| `-1` | Unlimited. |
| Positive integer | Finite cap. |
| `null` | Accepted for generated schema compatibility; clients should prefer `-1` or an explicit positive integer. |

Zero is invalid for runnable limits and max result counts.

### 5.2 `AspectRequest`

`AspectRequest` is the single runnable research unit. It appears as the top-level `task` in `AspectResearchRequest` and inside `DeepResearchRequest.task.aspects[]`.

```json
{
  "id": "market-map",
  "name": "Market map",
  "role": "market analyst",
  "question": "Which vendors and product categories define this market?",
  "scope": ["vendors", "segments", "adoption"],
  "boundaries": ["exclude unrelated adjacent markets"],
  "success_criteria": ["identify major vendor groups", "cite evidence"],
  "instructions": "# Aspect Agent\n\nReturn JSON matching AspectResearchResult.",
  "tools": ["search"],
  "model_provider": "openai",
  "search_provider": "grok",
  "limits": {
    "max_turns": 4,
    "max_tool_calls": 2,
    "max_search_calls": 2,
    "timeout_ms": 600000
  }
}
```

Fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | Yes | Stable aspect id. Must be non-empty. Must be unique inside `DeepResearchRequest.task.aspects`. |
| `name` | string | Yes | Human-readable aspect name. |
| `role` | string | Yes | Role hint for the aspect agent. |
| `question` | string | Yes | Concrete question for this aspect. |
| `scope` | string[] | Yes | In-scope topics. |
| `boundaries` | string[] | Yes | Out-of-scope boundaries. |
| `success_criteria` | string[] | Yes | Criteria the result should satisfy. |
| `instructions` | string | Yes | Inline Layer 2 prompt content. Rust core never reads prompt files. Must be non-empty and below 64 KiB. |
| `tools` | string[] | Yes | Currently supports `"search"`. Use `[]` for no tool access. |
| `model_provider` | string | Yes | Explicit provider selected by the client. It must be allowed by `policy.model.allowed_providers`. |
| `search_provider` | string or null | Conditional | Required when `tools` includes `"search"`; otherwise may be null. It must be allowed by `policy.search.allowed_providers`. |
| `limits` | object | Yes | Per-aspect resource and timeout limits. |

### 5.3 `ResearchLimits` and `AgentLimits`

Top-level research limits (example = **standard** tier):

```json
{
  "max_agents": 4,
  "max_concurrent_agents": 2,
  "max_total_model_calls": 32,
  "max_total_search_calls": 20,
  "total_timeout_ms": 600000,
  "max_tokens": -1
}
```

Per-aspect limits (example = **standard** tier):

```json
{
  "max_turns": 8,
  "max_tool_calls": 12,
  "max_search_calls": 6,
  "timeout_ms": 600000
}
```

The server takes the stricter value between request limits and operator configuration limits. `-1` means that layer does not add a cap; it does not override a finite cap from another layer.

### Recommended Layer-1 budget tiers

Shipped skill source of truth: `prompts/layer1/common/budget-tiers.md` (installed with the research-skills pack). Skills load that module; this section mirrors it for MCP developers.

| Tier | Top-level `limits` (`deep_research`) | Per-aspect `task.aspects[].limits` / `task.limits` |
| --- | --- | --- |
| `quick` | agents 2, concurrent 1, model calls 12, search calls 8, total_timeout_ms 300000, max_tokens -1 | turns 4, tool_calls 4, search_calls 2, timeout_ms 180000 |
| `standard` | agents 4, concurrent 2, model calls 32, search calls 20, total_timeout_ms 600000, max_tokens -1 | turns 8, tool_calls 12, search_calls 6, timeout_ms 600000 |
| `deep` | agents 6, concurrent 3, model calls 70, search calls 56, total_timeout_ms 1260000, max_tokens -1 | turns 8, tool_calls 8, search_calls 4, timeout_ms 600000 |

Profile defaults: generic/academic/technical → `standard`; PM DeepResearch → `deep`.

### 5.4 `ResearchPolicy`

Policies shape provider usage, evidence requirements, output constraints, and failure behavior. They do not carry resource limits.

```json
{
  "model": {
    "allowed_providers": ["openai"],
    "temperature": 0.2,
    "max_tokens": 3000,
    "require_tool_call_support": true
  },
  "search": {
    "allowed_providers": ["grok"],
    "max_results_per_query": 5,
    "freshness": null,
    "depth": "balanced",
    "content_level": "standard",
    "recency": "default",
    "category": null,
    "language": "en",
    "region": null,
    "include_domains": [],
    "exclude_domains": []
  },
  "evidence": {
    "require_evidence_for_findings": true,
    "min_evidence_per_finding": 1
  },
  "output": {
    "language": "en-US",
    "max_findings_per_aspect": 5
  },
  "execution": {
    "allow_partial_results": true,
    "fail_fast": false
  }
}
```

Policy rules:

- `policy.model.allowed_providers` and `policy.search.allowed_providers` are allowlists, not fallback order.
- Every aspect chooses exactly one `model_provider` and, when search is enabled, exactly one `search_provider`.
- `policy.search` accepts provider-neutral search controls only. Do not include provider-native fields such as Exa `type`, `contents`, `maxAgeHours`, Tavily `search_depth`, `topic`, `time_range`, `include_answer`, or `include_raw_content`.
- `policy.execution` contains only `allow_partial_results` and `fail_fast`. Request deadlines belong in `limits.total_timeout_ms` and `task.limits.timeout_ms`.
- Unknown fields in request and policy objects are rejected.

### 5.5 `ResearchContext`

```json
{
  "summary": "Shared context for all aspects.",
  "known_facts": ["Already established fact."],
  "excluded_assumptions": ["Assumption the agent must not rely on."],
  "prior_sources": []
}
```

Use empty strings and empty arrays when no context is available.

## 6. `aspect_research`

Use `aspect_research` when the client already has one concrete aspect to run.

### 6.1 Arguments

```json
{
  "schema_version": "0.2",
  "request_id": "aspect-request-1",
  "task": {
    "id": "sse-terminal-event",
    "name": "SSE terminal event",
    "role": "technical researcher",
    "question": "Which event indicates a completed Responses API SSE stream?",
    "scope": ["Responses API", "SSE"],
    "boundaries": ["Do not inspect unrelated APIs"],
    "success_criteria": ["Use search", "Return evidence-backed findings"],
    "instructions": "# Aspect Agent\n\nUse search once, then return JSON matching AspectResearchResult.",
    "tools": ["search"],
    "model_provider": "openai",
    "search_provider": "grok",
    "limits": {
      "max_turns": 4,
      "max_tool_calls": 2,
      "max_search_calls": 2,
      "timeout_ms": 600000
    }
  },
  "policy": {
    "model": {"allowed_providers": ["openai"], "temperature": 0.0, "max_tokens": 3000, "require_tool_call_support": true},
    "search": {"allowed_providers": ["grok"], "max_results_per_query": 2, "freshness": null, "depth": null, "content_level": null, "recency": null, "category": null, "language": "en", "region": null, "include_domains": [], "exclude_domains": []},
    "evidence": {"require_evidence_for_findings": true, "min_evidence_per_finding": 1},
    "output": {"language": "en-US", "max_findings_per_aspect": 3},
    "execution": {"allow_partial_results": false, "fail_fast": true}
  },
  "context": {
    "summary": "Provider behavior verification.",
    "known_facts": [],
    "excluded_assumptions": [],
    "prior_sources": []
  }
}
```

## 7. `deep_research`

Use `deep_research` for a multi-aspect plan.

```json
{
  "schema_version": "0.2",
  "request_id": "deep-request-1",
  "task": {
    "question": "What are the leading Rust async runtimes and their tradeoffs?",
    "aspects": [
      {
        "id": "ecosystem-overview",
        "name": "Async runtime ecosystem",
        "role": "researcher",
        "question": "Which async runtimes dominate the Rust ecosystem and how do they differ?",
        "scope": ["tokio", "async-std", "smol", "embassy"],
        "boundaries": ["exclude std-only abstractions"],
        "success_criteria": ["list 3-5 runtimes with primary use cases"],
        "instructions": "# Aspect Agent\n\nAnswer with evidence-backed findings.",
        "tools": ["search"],
        "model_provider": "openai",
        "search_provider": "grok",
        "limits": {"max_turns": 8, "max_tool_calls": 12, "max_search_calls": 6, "timeout_ms": 600000}
      }
    ]
  },
  "limits": {
    "max_agents": 4,
    "max_concurrent_agents": 2,
    "max_total_model_calls": 32,
    "max_total_search_calls": 20,
    "total_timeout_ms": 600000,
    "max_tokens": -1
  },
  "policy": {
    "model": {"allowed_providers": ["openai"], "temperature": 0.2, "max_tokens": 3000, "require_tool_call_support": true},
    "search": {"allowed_providers": ["grok"], "max_results_per_query": 5, "freshness": null, "depth": null, "content_level": null, "recency": null, "category": null, "language": "en", "region": null, "include_domains": [], "exclude_domains": []},
    "evidence": {"require_evidence_for_findings": true, "min_evidence_per_finding": 1},
    "output": {"language": "en-US", "max_findings_per_aspect": null},
    "execution": {"allow_partial_results": true, "fail_fast": false}
  },
  "context": {
    "summary": "Rust async runtime landscape",
    "known_facts": [],
    "excluded_assumptions": [],
    "prior_sources": []
  }
}
```

Validation highlights:

- `task.question` must be non-empty.
- `task.aspects` must be non-empty.
- `task.aspects[].id` must be unique.
- `task.aspects[].limits.timeout_ms` must not exceed `limits.total_timeout_ms` when both are finite.
- The number of aspects must not exceed `limits.max_agents` after config/request limit merging.

## 8. Response envelope

The MCP response is a standard `CallToolResult`. The stable MoeResearch payload is in `result.structuredContent`.

```json
{
  "schema_version": "0.2",
  "request_id": "request-1",
  "run_id": null,
  "status": "ok",
  "data": {},
  "error": null
}
```

Envelope fields:

| Field | Type | Notes |
| --- | --- | --- |
| `schema_version` | string | Echoes the request schema version. |
| `request_id` | string | Echoes the client request id. |
| `run_id` | string or null | Present for successful or partial `deep_research`; null for `aspect_research`. |
| `status` | `ok`, `partial`, or `failed` | Tool-level outcome. |
| `data` | object or null | Tool result when status is `ok` or `partial`. |
| `error` | `ToolError` or null | Public-safe error when status is `failed`; also present for `aspect_research` partial failures. |

`status=partial` is possible when `policy.execution.allow_partial_results=true` and usable evidence was collected before a terminal failure. For `deep_research`, `data.failed_aspects` describes failed aspect-level work and `data.evidence_index` may include evidence from failed aspects. For `aspect_research`, `data` contains collected evidence with no findings while `error` preserves the original failure metadata.

### Partial-status host contract (frozen)

Schema 0.2 intentionally keeps deep vs aspect partial asymmetry. Do not “normalize” envelopes in the client by dropping fields.

**Shipped host source of truth:** Layer 1 assets install `prompts/layer1/common/partial-status-host-contract.md` with the research-skills pack. Skills must load that module after install; they must not depend on this `docs/` file (repo docs are not part of the asset install).

This section mirrors the shipped contract for developers reading the MCP guide:

| Case | `status` | `data` | `error` | Host must |
| --- | --- | --- | --- | --- |
| `deep_research` partial | `partial` | present (`completed_aspects`, `aspect_reports`, `failed_aspects`, `evidence_index`, …) | **null** | Keep completed aspects; treat `failed_aspects[]` as gaps; at most one `aspect_research` retry per failed aspect |
| `aspect_research` partial | `partial` | present (frozen evidence; findings usually empty) | **present** | Use `data`; do not treat as pure hard-fail discard; inspect `error.retryable` / code for one retry |
| Partials disabled (`allow_partial_results=false`) | `failed` | null / no partial payload | present | Stop; no partial report path |
| Hard transport/config failure | `failed` | null | present | Surface stable code; no host-only substitute for MoeResearch execution |

Claude Code direct tools: read `result.structuredContent`.
Raw MCP: unwrap `tools/call` result per §4.2, then the same envelope.

## 9. Result object schemas

```text
AspectResearchResult
  aspect_report: AspectReport
  evidence: Evidence[]

DeepResearchResult
  run_id: string
  completed_aspects: string[]
  failed_aspects: AspectFailure[]
  aspect_reports: AspectReport[]
  evidence_index: Evidence[]
  open_questions: OpenQuestion[]
  coverage_summary: CoverageSummary
  confidence_summary: ConfidenceSummary
  budget_usage: ResearchBudgetUsage
```

Runtime usage objects are result-only. They are not accepted in request schemas.

`Evidence.source_type` values are exactly:

```text
official | documentation | news | blog | forum | repository | unknown
```

## 10. Error format

When `status` is `failed`, or when single-aspect `aspect_research` returns partial evidence with original failure metadata, `error` has this shape:

```json
{
  "code": "schema_validation_failed",
  "message": "Public-safe diagnostic message.",
  "aspect_id": "market-map",
  "retryable": false,
  "failed_aspects": []
}
```

Error codes:

```text
invalid_input
unsupported_schema_version
config_invalid
provider_unavailable
network_failed
budget_exceeded
tool_policy_denied
schema_validation_failed
timeout
partial_result
internal
```

Public error messages must not contain secrets, Authorization headers, provider raw response bodies, provider raw request bodies, or host file paths.

## 11. Common validation failures

### `unsupported_schema_version`

Use:

```json
"schema_version": "0.2"
```

### `provider_unavailable`

Client-side checks:

- `task.model_provider` or `task.aspects[].model_provider` is included in `policy.model.allowed_providers`.
- If search is enabled, `task.search_provider` or `task.aspects[].search_provider` is included in `policy.search.allowed_providers`.
- Provider names match the MCP server environment you are calling.
- Inspect `retryable`: retry transient provider-side failures, but fix configuration, environment, or policy failures before retrying.

### `tool_policy_denied`

Client-side checks:

- Use `tools: ["search"]` only when the aspect may search.
- Use `tools: []` when the aspect must not search.
- If search is allowed, provide a non-null `search_provider`.

### `budget_exceeded`

Client-side checks:

- Use positive integers or `-1` for limit fields.
- Keep `max_concurrent_agents <= max_agents` when both are finite.
- Keep each aspect timeout within the parent research timeout when both are finite.

### `schema_validation_failed`

The final structured result failed validation. If evidence was already collected and partial results are allowed, `aspect_research` may return `status=partial` with frozen evidence and this error metadata; `deep_research` partials carry failed aspect metadata in `data.failed_aspects` and keep top-level `error: null`.

Client-side checks:

- `instructions` should explicitly ask for JSON matching the expected result schema.
- Findings should cite evidence ids in `evidence_refs` when evidence is required.
- `policy.output.max_findings_per_aspect` should be large enough for the requested output.

## 12. Minimal stdio smoke test

This example performs only the MCP handshake and `tools/list`. It does not call live research tools.

```python
import json
import subprocess

proc = subprocess.Popen(
    ["moeresearch", "serve", "--config", "/absolute/path/to/moeresearch.toml"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    bufsize=1,
)

def send(message):
    proc.stdin.write(json.dumps(message, separators=(",", ":")) + "\n")
    proc.stdin.flush()

def recv():
    return json.loads(proc.stdout.readline())

send({
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2025-11-25",
        "capabilities": {},
        "clientInfo": {"name": "smoke-test", "version": "0.1.0"},
    },
})
print(recv())

send({"jsonrpc": "2.0", "method": "notifications/initialized"})
send({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
print(recv())

proc.terminate()
```
