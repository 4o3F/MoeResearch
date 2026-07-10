# Workflow Crate Responsibility Boundaries Design

**Date:** 2026-07-10  
**Status:** Draft for written-spec review  
**Scope:** `crates/moe-research-workflow`

## 1. Context

`moe-research-workflow` already distinguishes public/domain modules from private runtime modules in `src/lib.rs`, but most source files remain parallel at the crate root. The flat layout obscures several real ownership boundaries:

- `src/research.rs` combines public request DTOs, private effective plans, prompt projection, normalization, and provider/tool/name validation.
- `src/workflow.rs` combines both public entrypoints, deep-research scheduling, evidence namespacing, failure recording, and final aggregation.
- `src/agent_loop.rs` combines the turn state machine, replay state, model dispatch, search-tool execution, prompt/response construction, evidence conversion, output validation, partial synthesis, active deadlines, and local usage accounting.
- `src/runtime_budget.rs`, `src/tool_policy.rs`, `src/validator.rs`, and `src/log_safe.rs` are private implementation peers whose ownership is not apparent from their location.

The public files `limit.rs`, `budget.rs`, and `policy.rs` already have distinct responsibilities and must remain visible as existing public modules:

- `limit.rs` owns generic limit wire types and limit arithmetic.
- `budget.rs` owns public operator/request limit DTOs and static validation.
- `policy.rs` owns public model/search/tool/evidence/output policy DTOs and applies policy to provider-neutral downstream requests.

The goal is therefore not to place every file into a directory. It is to preserve clear public control-plane modules while grouping the mixed research, workflow, report-validation, and private runtime responsibilities into explicit ownership boundaries.

The current behavior is strongly constrained by integration tests. The refactor must preserve schema 0.2, public module-qualified paths, stricter config/request limit merging, explicit single-provider routing, prompt isolation, shared global budgets, evidence provenance, partial-result semantics, active cancellation, and public-safe errors.

## 2. Goals

1. Make every module answer one clear question: what does it own, how is it used, and what may it depend on?
2. Preserve the existing public API, serialized schemas, runtime behavior, and error semantics.
3. Express the existing public-domain/private-runtime split in the directory structure.
4. Separate scheduling and aggregation from single-aspect runtime mechanics.
5. Separate the agent state machine from model-turn and search-tool adapters.
6. Keep static limits distinct from live runtime accounting.
7. Keep workflow regression tests in `crates/moe-research-tests`.
8. Perform the migration incrementally so the workspace remains buildable after each stage.

## 3. Non-goals

This refactor will not:

- redesign schema 0.2 or rename public fields and types;
- remove or narrow existing public module-qualified paths;
- introduce a broad `contracts`, `common`, `shared`, or `utils` module;
- introduce new public traits or a generic workflow framework;
- move config-to-workflow mapping out of the CLI composition root;
- move provider-specific DTOs into the workflow crate;
- aggregate or fall back across search providers;
- change global limits into per-aspect limits;
- collapse public `budget.rs` and private runtime accounting solely because both involve limits;
- deduplicate defensive validation if doing so changes enforcement order or provider side effects;
- unify passive budget timeout checks and active Tokio deadline cancellation as part of a file move;
- remove currently public `ValidationStatus`, `ValidationIssue`, or `TokenUsage` re-exports;
- change `ModelPolicy.require_tool_call_support` behavior;
- change how `ResearchContext.prior_sources` participate in final evidence selection;
- rewrite the integration-test architecture;
- fix unrelated documentation drift such as older README schema-version text or PM tier numbers.

The underspecified public fields and evidence semantics above should be evaluated in separate behavior-change proposals, not hidden inside a structural refactor.

## 4. Chosen Architecture

Use stable public facades backed by responsibility-oriented submodules, while retaining already cohesive public control-plane files.

```text
crates/moe-research-workflow/src/
├── lib.rs
├── limit.rs
├── budget.rs
├── policy.rs
├── log_safe.rs
├── research/
│   ├── mod.rs
│   ├── request.rs
│   ├── plan.rs
│   └── prompt.rs
├── report/
│   ├── mod.rs
│   └── validator.rs
├── workflow/
│   ├── mod.rs
│   ├── aspect.rs
│   ├── deep.rs
│   └── aggregation.rs
└── runtime/
    ├── mod.rs
    ├── agent.rs
    ├── model_turn.rs
    ├── search_tool.rs
    ├── budget.rs
    └── deadline.rs
```

`log_safe.rs` intentionally remains a small private root module because both workflow coordination and runtime execution need the same safe logging rules. It has a specific cross-cutting responsibility and does not justify a generic utilities directory.

### 4.1 Existing public control-plane modules

These modules remain at their current public paths:

- `limit.rs`
  - owns `Limit<T>` and count/duration/token aliases;
  - owns serialization, schema, ordering, exhaustion, and concurrency helpers.
- `budget.rs`
  - owns `BudgetConfig`, `ResearchLimits`, and `AgentLimits`;
  - owns static config/request/relational validation and post-run usage checks;
  - does not own live reservations or cancellation state.
- `policy.rs`
  - owns public tool/model/search/evidence/output/execution policy DTOs;
  - applies public model/search policy to provider-neutral downstream requests;
  - retains current authorization and exact-provider semantics.

Their internal implementations may receive small import adjustments, but they are not restructuring targets.

### 4.2 `research`: request and execution-plan domain

`research` owns the public request model and deterministic conversion from external controls into executable plans.

- `research/mod.rs`
  - remains the stable `research::*` public facade;
  - declares private submodules and re-exports all existing public names;
  - contains no orchestration or provider calls.
- `research/request.rs`
  - owns schema 0.2 request, context, and task DTOs;
  - owns `SUPPORTED_SCHEMA_VERSIONS`;
  - contains no live runtime state.
- `research/plan.rs`
  - owns private `EffectiveResearchPlan` and `EffectiveAspectPlan`;
  - owns request/config normalization and duplicate-aspect validation;
  - owns stricter per-dimension limit merging and relational checks;
  - owns provider/tool/name validation that must occur before dispatch;
  - treats provider allowlists as authorization, not fallback selection.
- `research/prompt.rs`
  - owns crate-private `AspectPromptInput` projection;
  - ensures inline instructions remain exact System content;
  - ensures controls, limits, and provider policy do not leak into User JSON.

The name `plan.rs` is deliberate: public limits and policies remain owned by `budget.rs` and `policy.rs`; this module owns only their normalization into executable plans.

### 4.3 `report`: output domain and semantic validation

`report` owns public result/evidence data and the semantic validator for model-produced aspect output.

- `report/mod.rs`
  - owns and re-exports the existing public finding, evidence, open-question, failure, usage, validation-status, coverage, confidence, aspect-result, and deep-result DTOs;
  - preserves the existing `TokenUsage` re-export;
  - contains no scheduling, provider invocation, retry, or live budget logic.
- `report/validator.rs`
  - parses final aspect JSON;
  - validates aspect identity, cardinality, references, selected-evidence membership, bidirectional finding support, and byte-equal provenance;
  - remains private while continuing to return the existing public validation DTOs where required.

This refactor does not decide whether validator DTOs should remain public; it preserves compatibility.

### 4.4 `workflow`: public entrypoints and multi-aspect coordination

`workflow` owns public research operations and coordination across aspects, but not the internal agent loop.

- `workflow/mod.rs`
  - remains the stable `workflow::*` public facade;
  - re-exports the existing standalone/deep entrypoints and public failure/output types;
  - contains no provider loop implementation.
- `workflow/aspect.rs`
  - owns the standalone `aspect_research` entrypoint;
  - owns its existing public standalone output/failure types;
  - normalizes one request, creates the required guards, calls runtime, and preserves current standalone partial semantics.
- `workflow/deep.rs`
  - owns the `deep_research` entrypoint;
  - creates one shared research budget guard;
  - owns bounded-concurrency scheduling, fail-fast behavior, post-run global usage reconciliation, and request-order failure ordering;
  - delegates result recording/finalization to `aggregation`.
- `workflow/aggregation.rs`
  - records completed and failed aspect outcomes;
  - preserves failed-aspect evidence;
  - namespaces evidence IDs;
  - builds partial/final results and confidence summaries;
  - performs no model/search calls, control parsing, or budget reservation.

### 4.5 `runtime`: private single-aspect execution

`runtime` is crate-private and owns the mechanics of executing one aspect.

- `runtime/mod.rs`
  - exposes the narrow crate-private `AgentRuntime` boundary and existing runtime outcome/failure representations;
  - hides state-machine and adapter details from `workflow`.
- `runtime/agent.rs`
  - owns continuation/replay state and the single-aspect turn state machine;
  - preserves whole-batch tool-call validation before any tool dispatch;
  - coordinates model turns and search-tool execution;
  - owns terminal failure shaping and partial aspect synthesis;
  - does not construct provider requests inline.
- `runtime/model_turn.rs`
  - owns model request construction, structured response format, System/User message construction, global/agent model reservations, provider-neutral dispatch, usage merging, and model diagnostics;
  - keeps exact inline instructions in System and serialized `AspectPromptInput` in User.
- `runtime/search_tool.rs`
  - owns the logical `search` tool contract and model tool schema advertisement;
  - owns strict provider-neutral argument parsing, duplicate-ID batch validation, authorization, agent/global search reservations, exact-provider dispatch, and search-result-to-evidence conversion;
  - performs exactly one provider call per accepted tool invocation.
- `runtime/budget.rs`
  - owns `AgentBudgetGuard` and `ResearchBudgetGuard` live accounting;
  - owns per-aspect counters, atomic cross-aspect reservations, global timeout/token gating, usage merging, and usage snapshots;
  - remains distinct from public `budget.rs`, which defines static limits.
- `runtime/deadline.rs`
  - owns active Tokio cancellation of in-flight model/search work;
  - preserves the current runtime deadline semantics;
  - does not replace passive pre-action elapsed checks in budget guards.

### 4.6 Safe diagnostics

`log_safe.rs` continues to own workflow-local sanitization for errors, model identifiers, and generated evidence identifiers. It may be imported by both `workflow` and `runtime`, but it must not grow into a generic logging framework.

## 5. Dependency Rules

The intended high-level dependencies are shown as `consumer -> dependency`:

```text
budget
  -> limit

policy
  -> provider-neutral model/search DTOs

research::plan
  -> research::request
  -> limit / budget / policy

workflow
  -> research / report / runtime
  -> budget / policy / log_safe

runtime
  -> research::prompt / research::plan
  -> report::validator
  -> budget / policy / model service / search service / log_safe
```

The following rules are mandatory:

1. `research` and `report` must not depend on `workflow` or `runtime`.
2. `runtime` must not depend on `workflow`.
3. `workflow` coordinates runtime calls but must not implement provider loops.
4. CLI remains the only layer mapping config DTOs into workflow `BudgetConfig`.
5. Concrete provider response DTOs must not enter public workflow schemas.
6. Cross-module data must use existing domain types or a narrowly scoped crate-private type; no all-purpose context object may be introduced.
7. Module extraction alone is sufficient abstraction. New traits are not justified solely to make files appear decoupled.
8. Repeated validation that protects ordering or side-effect boundaries remains in place unless a separate behavior change proves it redundant.

## 6. Data Flow

### 6.1 Validate and normalize

```text
DeepResearchRequest / AspectResearchRequest
  → request-domain validation
  → policy, provider, tool, and limit validation
  → stricter config/request limit normalization
  → EffectiveResearchPlan / EffectiveAspectPlan
```

All schema, duplicate-ID, provider-authorization, tool-support, name, and relational-limit errors are rejected before dispatch. Runtime code receives effective plans and does not read or reinterpret the original deep request.

Defense-in-depth remains explicit:

- `research::plan` validates request-domain intent before dispatch;
- public policy application validates downstream provider-neutral requests;
- model/search services validate provider existence/configuration;
- `runtime::search_tool` validates each logical tool invocation before side effects.

The refactor clarifies these scopes but does not remove checks merely because similar validation exists elsewhere.

### 6.2 Establish shared execution state

The deep orchestrator creates one shared `ResearchBudgetGuard` before launching aspects. All aspect executions observe and update that same state. The standalone entrypoint creates the equivalent single-run guard required by current behavior.

Each aspect receives only the effective plan, prompt input, shared guard, and existing model/search service dependencies it needs. The complete deep request is not passed into the single-aspect runtime.

### 6.3 Execute one aspect

```text
AgentRuntime
  → agent state machine
  → model_turn dispatch
  → search_tool dispatch when requested
  → shared runtime budget accounting
  → active deadline cancellation
  → report validation
  → structured runtime outcome
```

System and User content remain separated:

```text
System = exact inline instructions
User   = serialized AspectPromptInput domain data
```

Whole-batch duplicate tool-call validation occurs before the first tool side effect. Budget/tool failures continue to occur before provider dispatch where current tests require that ordering.

The initial migration preserves the current narrow runtime boundary rather than introducing a replacement context object:

```rust
AgentRuntime::new(
    model_service,
    search_service,
    request: &EffectiveAspectPlan,
    research_budget: Arc<ResearchBudgetGuard>,
)

AgentRuntime::run()
    -> Result<AgentRuntimeOutput, AgentRuntimeFailure>
```

`AgentRuntimeOutput` continues to carry the result and usage data. `AgentRuntimeFailure` continues to carry the typed error plus optional partial output.

### 6.4 Aggregate outcomes

The runtime-to-workflow boundary must preserve the existing partial semantics. A simple `Result<Report, Error>` is insufficient because it would discard valid evidence accumulated before failure.

`workflow::aggregation` owns the existing `DeepResearchRun` accumulator and the equivalent of `record_aspect_result`. It consumes `Result<AgentRuntimeOutput, AgentRuntimeFailure>`, preserves failed-aspect evidence when partials are allowed, and retains request-order failure information for finalization.

Aggregation is a pure coordination step: it does not call providers, parse controls, update budgets, or re-run report validation.

## 7. Public Compatibility

The refactor must preserve:

- existing `limit::*`, `budget::*`, `policy::*`, `research::*`, `report::*`, and `workflow::*` module-qualified paths;
- existing crate-root re-exports;
- existing public functions, types, methods, and fields;
- serde field names, defaults, omission rules, and schema shape;
- schema version acceptance rules;
- public validation DTOs and `TokenUsage` re-export;
- public error codes, retryability, and safe messages;
- MCP-layer assumptions about success, partial, and failure results.

Compatibility is maintained with facade modules and explicit `pub use` statements. Internal source locations may change without requiring caller changes.

## 8. Error and Timeout Semantics

### 8.1 Pre-dispatch errors

Invalid schema versions, duplicate identifiers, invalid controls, unsupported tool combinations, unauthorized provider selections, and invalid relational limits fail before any aspect or provider call starts. They do not produce partial results.

### 8.2 Runtime errors

Model failures, search failures, policy violations, active cancellation, budget exhaustion, and output-validation failures retain their existing typed meaning. The refactor must not collapse distinguishable states into a generic internal error.

Internal functions retain typed errors and map them to the existing public-safe representation at the current boundary. New module boundaries must not introduce repeated wrapping or destroy retryability information.

No provider raw response, credential, Authorization header, cookie, JWT, or API key may enter normal output or error text.

### 8.3 Timeout scopes

The existing scopes remain distinct:

- `AgentBudgetGuard` performs passive per-agent elapsed checks before actions;
- `ResearchBudgetGuard` applies shared research-level timeout and global reservations;
- `RuntimeDeadline` actively cancels in-flight model/search futures;
- network inactivity timeouts remain owned by the network/provider layers.

Moving these implementations into clearer modules must not merge their clocks or change enforcement timing.

### 8.4 Partial failures

A failed aspect must retain any evidence that current behavior considers valid. Aggregation continues to distinguish request rejection, total failure, partial completion, and full success according to existing semantics. Fail-fast behavior and `allow_partial_results` remain unchanged.

## 9. Migration Strategy

The migration is staged rather than performed as one large move.

### Stage 0: Baseline

Run and record:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Stage 1: Extract `research`

- create the `research/` facade and submodules;
- move public request DTOs, private effective plans/normalization, and prompt projection by responsibility;
- preserve all module and root re-exports;
- run schema, config, MCP-schema, and orchestrator tests.

### Stage 2: Extract `report`

- move public report DTOs into `report/mod.rs` without changing public exports;
- move parsing and semantic validation into `report/validator.rs` without changing invocation timing;
- run validator, schema, and deep-research tests.

### Stage 3: Establish runtime support modules

- move live guards into `runtime/budget.rs`;
- move active cancellation into `runtime/deadline.rs`;
- move the logical search tool contract, dispatch, and evidence adaptation into `runtime/search_tool.rs`;
- move model request/dispatch work into `runtime/model_turn.rs`;
- preserve reservation and side-effect ordering;
- run model, search, network-policy, timeout, and orchestrator tests.

### Stage 4: Extract the agent state machine

- move continuation/replay and turn control into `runtime/agent.rs`;
- keep the crate-private runtime entrypoint in `runtime/mod.rs`;
- preserve partial synthesis, diagnostics, and token accounting;
- run the complete orchestrator and policy-validator suites.

### Stage 5: Extract public workflow operations

- create the `workflow/` facade;
- move standalone execution into `workflow/aspect.rs`;
- move deep scheduling into `workflow/deep.rs`;
- move evidence recording, namespacing, and finalization into `workflow/aggregation.rs`;
- run MCP, deep-research, and orchestrator tests.

### Stage 6: Full verification and documentation

- run formatting, workspace tests, and clippy;
- update directly affected module indexes and file-location references;
- run the required change, quality, and security verification gates for the completed refactor.

Every stage must leave the workspace compilable. A stage is not complete while its targeted tests fail.

## 10. Test Strategy

Existing integration coverage is the primary behavior contract. Direct workflow coverage includes schema, MCP, deep-research, orchestrator, and validator suites; model, search, config, CLI, network-policy, and wire-trace suites provide supporting boundary coverage.

The refactor must preserve tests for:

1. schema 0.2 and serialized shape;
2. all existing module-qualified and crate-root public paths;
3. prompt isolation and exact System content;
4. duplicate-ID and other pre-dispatch rejection;
5. whole-batch duplicate tool-ID rejection before dispatch;
6. explicit single-provider routing and logical `search` tool shape;
7. config clamping and shared global model/search/token limits;
8. active cancellation and timeout scope distinctions;
9. provider side-effect ordering on failures;
10. evidence provenance validation;
11. failed-aspect evidence preservation and namespacing;
12. fail-fast and `allow_partial_results` behavior;
13. public-safe errors, retryability, and MCP envelope semantics.

All new regression tests belong in `crates/moe-research-tests`. A minimal public-path compatibility test is added only if existing tests do not already compile against every path that must remain stable. Tests must assert observable behavior rather than private file locations.

## 11. Documentation Impact

Update only documentation directly affected by the source layout:

- `crates/moe-research-workflow/CLAUDE.md`;
- the root `CLAUDE.md` module index;
- `docs/development.md`;
- architecture text that names old source locations, including the `SUPPORTED_SCHEMA_VERSIONS` reference in `docs/research-agent-product.md`.

The discrepancy between `docs/configuration.md` language and the tested stricter-value clamping behavior should be recorded separately unless a moved-file reference requires touching the same section. Unrelated README schema-version and PM tier drift should not be bundled into this refactor.

## 12. Alternatives Considered

### Pipeline-first modules

Organizing by `input → planning → execution → aggregation → validation` makes the end-to-end flow visible, but shared budget, provider policy, and error semantics would span several stages. It also weakens ownership of public research and report types. This option was rejected.

### Hotspot-only extraction

Splitting only `research.rs` and `workflow.rs` minimizes immediate churn, but leaves the flat private-runtime files and overloaded agent loop without a clear owner. It reduces current file size without establishing a durable boundary. This option was rejected.

### Move every public file into one domain directory

Placing `limit`, `budget`, `policy`, `research`, and `report` under a single public API or contracts directory would simplify the tree superficially but erase domain ownership and break existing module-qualified paths. This option was rejected.

### Public API cleanup during the refactor

Removing apparently unused public validation DTOs, changing `TokenUsage` ownership, or resolving underspecified policy/context fields could reduce surface area, but each is a behavior or compatibility decision. Combining them with source movement would make regressions harder to attribute. This option was rejected.

## 13. Risks and Mitigations

| Risk | Mitigation |
| --- | --- |
| Re-export misses break module-qualified or root imports | Compile existing integration tests after each facade extraction; add one focused compatibility test only if needed. |
| Moving normalization changes precedence | Move logic without rewriting it; retain stricter config/request merge tests. |
| Runtime split changes defensive validation order | Preserve request validation, batch tool validation, policy application, and provider checks at their current side-effect boundaries. |
| Runtime split accidentally creates per-aspect global budgets | Construct one shared `ResearchBudgetGuard` in deep orchestration and pass the same handle to every runtime call. |
| Deadline extraction changes timeout timing | Move active cancellation without merging it with passive guard clocks; retain active-timeout regressions. |
| Search-tool extraction introduces fallback behavior | Keep explicit provider selection and authorization tests at the runtime boundary. |
| Failure paths lose evidence or retryability | Preserve existing structured outcomes and typed errors before separating aggregation. |
| Public DTO cleanup leaks into structural work | Preserve all current exports and record cleanup candidates separately. |
| Excessive file fragmentation | Require every file to have one unique responsibility and meaningful behavior; merge trivial fragments. |
| Documentation work expands scope | Update only references invalidated by the source move; track unrelated drift separately. |

## 14. Acceptance Criteria

The refactor is complete when all of the following are true:

1. Existing public control-plane modules remain at stable paths with unchanged public APIs.
2. Research requests, effective plans, and prompt projection have distinct ownership under `research/`.
3. Standalone execution, deep scheduling, and result aggregation have distinct ownership under `workflow/`.
4. The agent state machine, model-turn adapter, search-tool adapter, live budget guards, and active deadline have distinct ownership under private `runtime/`.
5. Existing module-qualified and crate-root public imports compile without caller changes.
6. Public schemas, serialization, validation DTOs, error codes, retryability, and MCP success/partial/failure semantics are unchanged.
7. Config/request limits still merge to the stricter effective values.
8. Every search call still uses exactly one explicitly selected provider with no fallback aggregation.
9. All deep aspects share the same global runtime budget and cancellation state.
10. Defensive validation and provider side-effect ordering remain unchanged.
11. Inline instructions, User JSON, evidence provenance, and public-safe logging constraints remain intact.
12. Workflow regressions remain in `crates/moe-research-tests`.
13. `cargo fmt --all -- --check`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
14. Directly affected module documentation and file references are updated.
