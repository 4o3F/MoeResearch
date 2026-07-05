# MCP Usage Guide

This document is for clients that call MoeResearch only through MCP. It describes the MCP transport, lifecycle messages, tool names, request payloads, response envelopes, and error formats.

It documents only the MCP surface that clients send and receive.

## 1. Transport

MoeResearch exposes an MCP server over stdio.

Protocol rules:

- The client sends JSON-RPC messages to the server process stdin.
- The server writes JSON-RPC responses and notifications to stdout.
- The server writes logs to stderr.
- Each JSON-RPC message is one JSON object followed by `\n`.
- Current stdio transport does not use `Content-Length` framing.
- Do not send API keys, Authorization headers, or other secrets in MCP payloads.

## 1.1 Claude Code registration

Prefer the official Claude Code MCP CLI instead of editing Claude settings by hand:

```bash
moeresearch mcp register --scope local --config ~/.config/moeresearch/moeresearch.toml
```

The command invokes `claude mcp add` with the current `moeresearch` executable path by default:

```bash
claude mcp add --transport stdio --scope local moeresearch -e OPENAI_API_KEY=<redacted> -- /absolute/path/to/moeresearch serve --config /absolute/path/to/moeresearch.toml
```

Use `--moeresearch-bin` when Claude Code should launch a different binary. The MCP server name appears before `-e` because Claude Code treats `--env <env...>` as variadic; placing environment flags before the name can make the name parse as an environment value.

The registration command validates the config file and the environment variables for enabled providers before invoking `claude`. It copies current enabled-provider environment values into Claude Code registration with `-e`; use `--dry-run` to print the command and JSON example without calling `claude`, with values redacted:

```bash
moeresearch mcp register --scope local --config ~/.config/moeresearch/moeresearch.toml --dry-run
```

For project-scoped configs, the equivalent `mcpServers` shape is:

```json
{
  "mcpServers": {
    "moeresearch": {
      "type": "stdio",
      "command": "moeresearch",
      "args": ["serve", "--config", "/absolute/path/to/moeresearch.toml"],
      "env": {
        "OPENAI_API_KEY": "<redacted>"
      }
    }
  }
}
```

Do not put provider API keys in `moeresearch.toml`. MoeResearch config stores only environment variable names in `api_key_env`; `moeresearch mcp register` reads current values for enabled providers and copies them into Claude Code registration. Dry-run output redacts the values.

MCP registration only configures the stdio MoeResearch server. It does not install PM DeepResearch skills or prompt assets. For Claude Code, run:

```bash
moeresearch assets install pm-deep-research
```

The installer writes `SKILL.md` under the Claude Code skill discovery directory. For other clients, use:

```bash
moeresearch assets install pm-deep-research \
  --target ~/.config/moeresearch/assets \
  --layout repo
```

Then configure that client to load the installed skill and prompt files explicitly.

## 2. MCP lifecycle

### 2.1 Initialize

Client request:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-11-25",
    "capabilities": {},
    "clientInfo": {
      "name": "example-client",
      "version": "0.1.0"
    }
  }
}
```

The server responds with MCP capabilities.

### 2.2 Initialized notification

Client notification:

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized"
}
```

### 2.3 List tools

Client request:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": {}
}
```

Expected tool names:

```text
aspect_research
deep_research
```

## 3. Tools

| Tool | Purpose | Arguments | Structured output |
| --- | --- | --- | --- |
| `aspect_research` | Run one research aspect. | `AspectResearchRequest` | `ToolEnvelope<AspectResearchResult>` |
| `deep_research` | Run multiple research aspects and aggregate them. | `DeepResearchRequest` | `ToolEnvelope<DeepResearchResult>` |

Both tools require:

```json
"schema_version": "0.1"
```

Any other value returns `unsupported_schema_version`.

## 4. Invocation shapes

### 4.1 Claude Code direct MCP invocation

Claude Code usually exposes MCP tools as direct tool names such as
`mcp__moeresearch__deep_research` or `mcp__moeresearch__aspect_research`.
In that mode, pass only the tool-specific MoeResearch request object as the
MCP tool input. That object is the same value shown below as `params.arguments`.

Do not paste the outer JSON-RPC wrapper into a Claude Code direct MCP tool
call. Do not wrap the request under `params`, `arguments`, `request`, `input`,
or `tool_input`.

### 4.2 Raw MCP `tools/call` wrapper

Raw MCP clients that speak JSON-RPC over stdio use the standard `tools/call`
method with `params.name` and `params.arguments`.

Shape:

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "tools/call",
  "params": {
    "name": "aspect_research",
    "arguments": {}
  }
}
```

`params.name` is the MCP tool name. `params.arguments` is the tool-specific request object.

## 5. Common request objects

### 5.1 Limit values

Budget and timeout fields use this wire format:

| JSON value | Meaning |
| --- | --- |
| `-1` | Unlimited. |
| Positive integer | Finite cap. |
| `null` | Accepted for limit fields where the generated schema permits it; clients should prefer `-1` or an explicit positive integer. |

Zero is invalid for runnable budgets and max result counts.

### 5.2 `ResearchContext`

```json
{
  "summary": "Shared context for all aspects.",
  "known_facts": ["Already established fact."],
  "excluded_assumptions": ["Assumption the agent must not rely on."],
  "prior_sources": []
}
```

Fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `summary` | string | Yes | Short shared background. Use an empty string when there is no context. |
| `known_facts` | string[] | Yes | Facts the client wants the research agent to treat as already known. |
| `excluded_assumptions` | string[] | Yes | Assumptions the agent must not use. |
| `prior_sources` | `Evidence[]` | Yes | Previously known evidence. Use `[]` when absent. |

### 5.3 `AspectSpec`

```json
{
  "aspect_id": "market-map",
  "name": "Market map",
  "role": "market analyst",
  "research_question": "Which vendors and product categories define this market?",
  "scope": ["vendors", "segments", "adoption"],
  "boundaries": ["exclude unrelated adjacent markets"],
  "success_criteria": ["identify major vendor groups", "cite evidence"],
  "aspect_agent_prompt": "# Aspect Agent\n\nReturn JSON matching AspectResearchResult.",
  "allowed_tools": ["search"],
  "model_provider": "openai",
  "search_provider": "grok"
}
```

Fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `aspect_id` | string | Yes | Stable id for this aspect. Must be non-empty. Must be unique inside `deep_research.aspect_tasks`. |
| `name` | string | Yes | Human-readable aspect name. Must be non-empty. |
| `role` | string | Yes | Role hint for the aspect. |
| `research_question` | string | Yes | The concrete question for this aspect. Must be non-empty. |
| `scope` | string[] | Yes | In-scope topics. |
| `boundaries` | string[] | Yes | Out-of-scope boundaries. |
| `success_criteria` | string[] | Yes | Criteria the result should satisfy. |
| `aspect_agent_prompt` | string | Yes | Inline instruction content. Must be non-empty. |
| `allowed_tools` | string[] | Yes | Currently supports `"search"`. Use `[]` for no tool access. |
| `model_provider` | string | Yes | Provider name selected by the client. It must be allowed by `model_policy.allowed_providers`. |
| `search_provider` | string or null | Conditional | Required when `allowed_tools` includes `"search"`; otherwise may be null. It must be allowed by `search_policy.allowed_providers`. |

Provider names are opaque MCP request values from the caller's perspective. The MCP server validates whether selected names are available and allowed.

### 5.4 `AspectResearchTask`

```json
{
  "aspect": {},
  "budget": {
    "max_turns": 4,
    "max_tool_calls": 2,
    "max_search_calls": 2,
    "timeout_ms": 180000
  }
}
```

Fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `aspect` | `AspectSpec` | Yes | The aspect to run. |
| `budget.max_turns` | limit | Yes | Maximum model turns for this aspect. |
| `budget.max_tool_calls` | limit | Yes | Maximum tool calls for this aspect. |
| `budget.max_search_calls` | limit | Yes | Maximum search calls for this aspect. |
| `budget.timeout_ms` | limit | Yes | Aspect timeout in milliseconds. |

### 5.5 `ModelPolicy`

```json
{
  "allowed_providers": ["openai"],
  "temperature": 0.2,
  "max_tokens": 3000,
  "require_tool_call_support": true
}
```

Fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `allowed_providers` | string[] | Yes | Authorization allowlist. It is not fallback order. |
| `temperature` | number or null | Yes | Model temperature hint. |
| `max_tokens` | integer or null | Yes | Maximum output token hint. Must be greater than zero when present. |
| `require_tool_call_support` | boolean | Yes | Client-declared requirement for tool-capable model behavior. |

### 5.6 `SearchPolicy`

```json
{
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
}
```

Fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `allowed_providers` | string[] | Yes | Authorization allowlist. It is not fallback order. |
| `max_results_per_query` | integer | Yes | Must be greater than zero. |
| `freshness` | object or null | Yes | Optional publish-date constraint. Use null when absent. |
| `depth` | `low_latency`, `balanced`, `high_recall`, or null | Yes | Provider-neutral speed/coverage preference. Policy value is a default and ceiling. |
| `content_level` | `compact`, `standard`, `detailed`, or null | Yes | Provider-neutral result detail preference. Policy value is a default and ceiling. |
| `recency` | `default`, `live`, `fresh`, `recent`, `cached`, or null | Yes | Provider-neutral source freshness preference, distinct from publish-date `freshness`. |
| `category` | `organizations`, `people`, `academic`, `news`, `personal_sites`, `financial_filings`, `code`, or null | Yes | Provider-neutral source/content focus. Policy value is a default and constraint. |
| `language` | string or null | Yes | Optional language hint. |
| `region` | string or null | Yes | Optional region hint. |
| `include_domains` | string[] | Yes | Domain allow filter. |
| `exclude_domains` | string[] | Yes | Domain deny filter. A domain must not appear in both include and exclude lists. |

The model-facing internal `search` tool accepts the same provider-neutral call-time fields. It rejects provider-specific fields such as provider overrides, Exa request `type`, `contents`, `maxAgeHours`, Tavily request `search_depth`, `topic`, `time_range`, `include_answer`, `include_raw_content`, or deep search mode names.

### 5.7 `EvidencePolicy`

```json
{
  "require_evidence_for_findings": true,
  "min_evidence_per_finding": 1
}
```

Fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `require_evidence_for_findings` | boolean | Yes | Whether findings must cite evidence. |
| `min_evidence_per_finding` | integer | Yes | Minimum evidence refs per finding when evidence is required. |

### 5.8 `OutputPolicy`

```json
{
  "language": "en-US",
  "max_findings_per_aspect": 5
}
```

Fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `language` | string | Yes | Desired output language for the structured result content. |
| `max_findings_per_aspect` | integer or null | Yes | Maximum findings per aspect. Use null for no explicit cap. |

### 5.9 `ExecutionPolicy`

```json
{
  "allow_partial_results": true,
  "fail_fast": false,
  "timeout_ms": 300000
}
```

Fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `allow_partial_results` | boolean | Yes | Return usable partial output when evidence was collected before a terminal failure; `deep_research` may return partial data even when all aspects failed. |
| `fail_fast` | boolean | Yes | For `deep_research`, stop after the first aspect failure when possible. |
| `timeout_ms` | limit | Yes | Request timeout in milliseconds. Use `-1` for unlimited. |

`execution_policy.timeout_ms` is MoeResearch's request deadline, not the outer MCP/client deadline. Configure the MCP client timeout higher than the largest finite `execution_policy.timeout_ms`, `task.budget.timeout_ms`, or `budget.total_timeout_ms` you intend to send, plus provider retry/backoff slack. If you use `-1` for unlimited request budgets, choose the outer client timeout deliberately; otherwise the client may be the first component to abort and can miss the final `ok` or `partial` envelope.

## 6. `aspect_research`

Use `aspect_research` when the client already has one concrete aspect to run.

### 6.1 Arguments

```json
{
  "schema_version": "0.1",
  "request_id": "aspect-request-1",
  "task": {
    "aspect": {
      "aspect_id": "sse-terminal-event",
      "name": "SSE terminal event",
      "role": "technical researcher",
      "research_question": "Which event indicates a completed Responses API SSE stream?",
      "scope": ["Responses API", "SSE"],
      "boundaries": ["Do not inspect unrelated APIs"],
      "success_criteria": ["Use search", "Return evidence-backed findings"],
      "aspect_agent_prompt": "# Aspect Agent\n\nUse search once, then return JSON matching AspectResearchResult.",
      "allowed_tools": ["search"],
      "model_provider": "openai",
      "search_provider": "grok"
    },
    "budget": {
      "max_turns": 4,
      "max_tool_calls": 2,
      "max_search_calls": 2,
      "timeout_ms": 180000
    }
  },
  "shared_context": {
    "summary": "Provider behavior verification.",
    "known_facts": [],
    "excluded_assumptions": [],
    "prior_sources": []
  },
  "model_policy": {
    "allowed_providers": ["openai"],
    "temperature": 0.0,
    "max_tokens": 3000,
    "require_tool_call_support": true
  },
  "search_policy": {
    "allowed_providers": ["grok"],
    "max_results_per_query": 2,
    "freshness": null,
    "depth": null,
    "content_level": null,
    "recency": null,
    "category": null,
    "language": "en",
    "region": null,
    "include_domains": [],
    "exclude_domains": []
  },
  "evidence_policy": {
    "require_evidence_for_findings": true,
    "min_evidence_per_finding": 1
  },
  "output_policy": {
    "language": "en-US",
    "max_findings_per_aspect": 3
  },
  "execution_policy": {
    "allow_partial_results": false,
    "fail_fast": true,
    "timeout_ms": 180000
  }
}
```

### 6.2 Full MCP call

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "tools/call",
  "params": {
    "name": "aspect_research",
    "arguments": {
      "schema_version": "0.1",
      "request_id": "aspect-request-1",
      "task": {
        "aspect": {
          "aspect_id": "sse-terminal-event",
          "name": "SSE terminal event",
          "role": "technical researcher",
          "research_question": "Which event indicates a completed Responses API SSE stream?",
          "scope": ["Responses API", "SSE"],
          "boundaries": ["Do not inspect unrelated APIs"],
          "success_criteria": ["Use search", "Return evidence-backed findings"],
          "aspect_agent_prompt": "# Aspect Agent\n\nUse search once, then return JSON matching AspectResearchResult.",
          "allowed_tools": ["search"],
          "model_provider": "openai",
          "search_provider": "grok"
        },
        "budget": {
          "max_turns": 4,
          "max_tool_calls": 2,
          "max_search_calls": 2,
          "timeout_ms": 180000
        }
      },
      "shared_context": {
        "summary": "Provider behavior verification.",
        "known_facts": [],
        "excluded_assumptions": [],
        "prior_sources": []
      },
      "model_policy": {
        "allowed_providers": ["openai"],
        "temperature": 0.0,
        "max_tokens": 3000,
        "require_tool_call_support": true
      },
      "search_policy": {
        "allowed_providers": ["grok"],
        "max_results_per_query": 2,
        "freshness": null,
        "depth": null,
        "content_level": null,
        "recency": null,
        "category": null,
        "language": "en",
        "region": null,
        "include_domains": [],
        "exclude_domains": []
      },
      "evidence_policy": {
        "require_evidence_for_findings": true,
        "min_evidence_per_finding": 1
      },
      "output_policy": {
        "language": "en-US",
        "max_findings_per_aspect": 3
      },
      "execution_policy": {
        "allow_partial_results": false,
        "fail_fast": true,
        "timeout_ms": 180000
      }
    }
  }
}
```

### 6.3 Successful structured output

`aspect_research` returns `ToolEnvelope<AspectResearchResult>` in `result.structuredContent`.

```json
{
  "schema_version": "0.1",
  "request_id": "aspect-request-1",
  "run_id": null,
  "status": "ok",
  "data": {
    "aspect_report": {
      "aspect_id": "sse-terminal-event",
      "aspect_name": "SSE terminal event",
      "question": "Which event indicates a completed Responses API SSE stream?",
      "scope": ["Responses API", "SSE"],
      "findings": [
        {
          "id": "finding-1",
          "claim": "A concise evidence-backed claim.",
          "finding_type": "fact",
          "importance": "high",
          "confidence": "medium",
          "evidence_refs": ["ev-1-1"],
          "contradicted_by": []
        }
      ],
      "assumptions": [],
      "risks": [],
      "counterarguments": [],
      "open_questions": [],
      "confidence": "medium",
      "limitations": []
    },
    "evidence": [
      {
        "id": "ev-1-1",
        "source_title": "Source title",
        "url": "https://example.test/source",
        "provider": "grok",
        "query": "example query",
        "snippet": "Short snippet.",
        "summary": "Why this source matters.",
        "published_at": null,
        "retrieved_at": "2026-05-28T00:00:00Z",
        "supports_findings": ["finding-1"],
        "source_type": "documentation",
        "confidence": "medium"
      }
    ]
  },
  "error": null
}
```

## 7. `deep_research`

Use `deep_research` when the client wants the MCP server to run multiple aspect tasks and return one aggregated result.

### 7.1 Arguments

```json
{
  "schema_version": "0.1",
  "request_id": "deep-request-1",
  "user_question": "What are the leading Rust async runtimes and their tradeoffs?",
  "aspect_tasks": [
    {
      "aspect": {
        "aspect_id": "ecosystem-overview",
        "name": "Async runtime ecosystem",
        "role": "researcher",
        "research_question": "Which async runtimes dominate the Rust ecosystem and how do they differ?",
        "scope": ["tokio", "async-std", "smol", "embassy"],
        "boundaries": ["exclude std-only abstractions"],
        "success_criteria": ["list 3-5 runtimes with primary use cases"],
        "aspect_agent_prompt": "# Aspect Agent\n\nAnswer with evidence-backed findings.",
        "allowed_tools": ["search"],
        "model_provider": "openai",
        "search_provider": "grok"
      },
      "budget": {
        "max_turns": 8,
        "max_tool_calls": 12,
        "max_search_calls": 6,
        "timeout_ms": 300000
      }
    }
  ],
  "budget": {
    "max_agents": 5,
    "max_concurrent_agents": 2,
    "max_total_model_calls": 30,
    "max_total_search_calls": 20,
    "total_timeout_ms": 300000,
    "max_tokens": -1
  },
  "model_policy": {
    "allowed_providers": ["openai"],
    "temperature": 0.2,
    "max_tokens": 3000,
    "require_tool_call_support": true
  },
  "search_policy": {
    "allowed_providers": ["grok"],
    "max_results_per_query": 5,
    "freshness": null,
    "depth": null,
    "content_level": null,
    "recency": null,
    "category": null,
    "language": "en",
    "region": null,
    "include_domains": [],
    "exclude_domains": []
  },
  "evidence_policy": {
    "require_evidence_for_findings": true,
    "min_evidence_per_finding": 1
  },
  "output_policy": {
    "language": "en-US",
    "max_findings_per_aspect": null
  },
  "shared_context": {
    "summary": "Rust async runtime landscape",
    "known_facts": [],
    "excluded_assumptions": [],
    "prior_sources": []
  },
  "execution_policy": {
    "allow_partial_results": true,
    "fail_fast": false,
    "timeout_ms": 300000
  }
}
```

### 7.2 Full MCP call

```json
{
  "jsonrpc": "2.0",
  "id": 20,
  "method": "tools/call",
  "params": {
    "name": "deep_research",
    "arguments": {
      "schema_version": "0.1",
      "request_id": "deep-request-1",
      "user_question": "What are the leading Rust async runtimes and their tradeoffs?",
      "aspect_tasks": [
        {
          "aspect": {
            "aspect_id": "ecosystem-overview",
            "name": "Async runtime ecosystem",
            "role": "researcher",
            "research_question": "Which async runtimes dominate the Rust ecosystem and how do they differ?",
            "scope": ["tokio", "async-std", "smol", "embassy"],
            "boundaries": ["exclude std-only abstractions"],
            "success_criteria": ["list 3-5 runtimes with primary use cases"],
            "aspect_agent_prompt": "# Aspect Agent\n\nAnswer with evidence-backed findings.",
            "allowed_tools": ["search"],
            "model_provider": "openai",
            "search_provider": "grok"
          },
          "budget": {
            "max_turns": 8,
            "max_tool_calls": 12,
            "max_search_calls": 6,
            "timeout_ms": 300000
          }
        }
      ],
      "budget": {
        "max_agents": 5,
        "max_concurrent_agents": 2,
        "max_total_model_calls": 30,
        "max_total_search_calls": 20,
        "total_timeout_ms": 300000,
        "max_tokens": -1
      },
      "model_policy": {
        "allowed_providers": ["openai"],
        "temperature": 0.2,
        "max_tokens": 3000,
        "require_tool_call_support": true
      },
      "search_policy": {
        "allowed_providers": ["grok"],
        "max_results_per_query": 5,
        "freshness": null,
        "depth": null,
        "content_level": null,
        "recency": null,
        "category": null,
        "language": "en",
        "region": null,
        "include_domains": [],
        "exclude_domains": []
      },
      "evidence_policy": {
        "require_evidence_for_findings": true,
        "min_evidence_per_finding": 1
      },
      "output_policy": {
        "language": "en-US",
        "max_findings_per_aspect": null
      },
      "shared_context": {
        "summary": "Rust async runtime landscape",
        "known_facts": [],
        "excluded_assumptions": [],
        "prior_sources": []
      },
      "execution_policy": {
        "allow_partial_results": true,
        "fail_fast": false,
        "timeout_ms": 300000
      }
    }
  }
}
```

### 7.3 Successful or partial structured output

`deep_research` returns `ToolEnvelope<DeepResearchResult>` in `result.structuredContent`.

```json
{
  "schema_version": "0.1",
  "request_id": "deep-request-1",
  "run_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "ok",
  "data": {
    "run_id": "550e8400-e29b-41d4-a716-446655440000",
    "completed_aspects": ["ecosystem-overview"],
    "failed_aspects": [],
    "aspect_reports": [],
    "evidence_index": [],
    "open_questions": [],
    "coverage_summary": {
      "requested_aspects": 1,
      "completed_aspects": 1,
      "failed_aspects": 0,
      "evidence_count": 0
    },
    "confidence_summary": {
      "high": 0,
      "medium": 1,
      "low": 0
    },
    "budget_usage": {
      "agents_started": 1,
      "model_calls_used": 2,
      "search_calls_used": 1,
      "elapsed_ms": 1000,
      "token_usage": null
    }
  },
  "error": null
}
```

`status` may be `partial` when `execution_policy.allow_partial_results` is true and usable evidence was collected before a terminal failure. For `deep_research`, `error` is `null`; `data.failed_aspects` describes failed aspect-level work, including each aspect's `retryable` value, and `data.evidence_index` may include evidence from failed aspects even when `completed_aspects` is empty. For `aspect_research`, `data` contains collected evidence with no findings, while `error` preserves the original failure metadata and `error.retryable`.

A final `schema_validation_failed` can still return `status=partial` when `execution_policy.allow_partial_results=true` and the agent collected evidence before final output validation failed. For `mutated_evidence_provenance`, MoeResearch returns preserved runtime-collected evidence, not the model-mutated output. Preserve returned evidence as-is; in `deep_research`, evidence ids may be aspect-namespaced, while provenance fields (`source_title`, `url`, `provider`, `query`, `snippet`, `summary`, `published_at`, `retrieved_at`) remain byte-equal to search-tool output. Treat findings as absent. Because `retryable` means retrying the same request later may help, a validation failure may be `retryable: false`; if useful, submit a changed corrective request for the failed aspect, such as a smaller evidence set or stricter prompt.

## 8. MCP response envelope

The MCP response is a standard `CallToolResult`. The stable MoeResearch payload is in `result.structuredContent`.

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{...same envelope serialized as JSON text...}"
      }
    ],
    "structuredContent": {
      "schema_version": "0.1",
      "request_id": "request-1",
      "run_id": null,
      "status": "ok",
      "data": {},
      "error": null
    },
    "isError": false
  }
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

Status values:

| Status | Meaning |
| --- | --- |
| `ok` | Tool completed successfully. |
| `partial` | The tool produced usable partial output; `deep_research` may have only failed aspects with preserved evidence, and `aspect_research` may include both `data` and `error`. |
| `failed` | Tool failed and no result data is available. |

## 9. Result object schemas

### 9.1 `AspectResearchResult`

```text
AspectResearchResult
  aspect_report: AspectReport
  evidence: Evidence[]
```

```text
AspectReport
  aspect_id: string
  aspect_name: string
  question: string
  scope: string[]
  findings: Finding[]
  assumptions: string[]
  risks: string[]
  counterarguments: string[]
  open_questions: OpenQuestion[]
  confidence: low | medium | high
  limitations: string[]
```

```text
Finding
  id: string
  claim: string
  finding_type: fact | interpretation | recommendation | risk | assumption
  importance: low | medium | high | critical
  confidence: low | medium | high
  evidence_refs: string[]
  contradicted_by: string[]
```

```text
Evidence
  id: string
  source_title: string
  url: string | null
  provider: string
  query: string
  snippet: string
  summary: string
  published_at: string | null
  retrieved_at: string
  supports_findings: string[]
  source_type: official | documentation | news | blog | forum | repository | unknown
  confidence: low | medium | high
```

```text
OpenQuestion
  id: string
  question: string
  reason: string
  suggested_follow_up: string[]
```

### 9.2 `DeepResearchResult`

```text
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

```text
AspectFailure
  aspect_id: string
  error_code: string
  message: string
  retryable: boolean
```

```text
CoverageSummary
  requested_aspects: integer
  completed_aspects: integer
  failed_aspects: integer
  evidence_count: integer
```

```text
ConfidenceSummary
  high: integer
  medium: integer
  low: integer
```

```text
ResearchBudgetUsage
  agents_started: integer
  model_calls_used: integer
  search_calls_used: integer
  elapsed_ms: integer
  token_usage: TokenUsage | null
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

Fields:

| Field | Type | Notes |
| --- | --- | --- |
| `code` | string | Stable error code. |
| `message` | string | Public-safe message. |
| `aspect_id` | string or null | Aspect id when the failure belongs to one aspect. |
| `retryable` | boolean | Whether retrying the same request later may be useful. |
| `failed_aspects` | `AspectFailure[]` | Aspect-level failures for aggregated `deep_research` failures. |

`provider_unavailable` has two retryability classes. Provider-side terminal/incomplete/error SSE events that the server classifies as transient return `retryable: true`. Configuration, policy, missing environment variable, or provider-not-configured failures return `retryable: false`; clients should use `retryable` as the source of truth.

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
"schema_version": "0.1"
```

### `provider_unavailable`

The request selected a provider name that is unavailable or not allowed by policy, or a provider-side SSE stream ended with a transient terminal/error event.

Client-side checks:

- `aspect.model_provider` is included in `model_policy.allowed_providers`.
- If search is enabled, `aspect.search_provider` is included in `search_policy.allowed_providers`.
- Provider names match the MCP server environment you are calling.
- Inspect `retryable`: retry transient provider-side failures, but fix configuration, environment, or policy failures before retrying.

### `tool_policy_denied`

The aspect or model-facing tool arguments violated tool policy.

Client-side checks:

- Use `allowed_tools: ["search"]` only when the aspect may search.
- Use `allowed_tools: []` when the aspect must not search.
- If search is allowed, provide a non-null `search_provider`.

### `budget_exceeded`

The request budget exceeds server limits or runtime usage exhausted the declared budget.

Client-side checks:

- Use positive integers or `-1` for limit fields.
- Keep `max_concurrent_agents <= max_agents` when both are finite.
- Keep request timeout within the relevant parent budget when both are finite.

### `schema_validation_failed`

The final structured result failed validation. If evidence was already collected and partial results are allowed, `aspect_research` may return `status=partial` with frozen evidence and this error metadata; `deep_research` partials carry failed aspect metadata in `data.failed_aspects` and keep top-level `error: null`.

Client-side checks:

- `aspect_agent_prompt` should explicitly ask for JSON matching the expected result schema.
- Findings should cite evidence ids in `evidence_refs` when evidence is required.
- `max_findings_per_aspect` should be large enough for the requested output.

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
