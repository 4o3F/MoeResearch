# Phase 3 — Workflow Crate Responsibility Boundaries Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Behavior-preserving internal restructure of `moe-research-workflow` only, closing audit findings B1, B4, and B9 by establishing the ownership layout in the workflow-boundary design doc while keeping schema 0.2, public module-qualified paths, and runtime semantics unchanged.

**Architecture:** Stable public facades (`research::*`, `report::*`, `workflow::*`) plus private `runtime::*`, with existing control-plane modules (`limit`, `budget`, `policy`) and `log_safe` kept at crate-root paths. Prefer move + re-export first, then split functions. Workspace must build and relevant tests stay green after every stage.

**Tech Stack:** Rust 2024 workspace (`moe-research-workflow`, `moe-research-tests`), `cargo test` / `cargo clippy`, existing integration suites as the behavior contract.

## Global Constraints

- Authoritative structure: `docs/superpowers/specs/2026-07-10-workflow-crate-responsibility-boundaries-design.md`
- Findings closed by this phase: **B1** (`agent_loop` density), **B4** (`research.rs` mixed roles), **B9** (`workflow.rs` density) from `docs/superpowers/specs/2026-07-10-architecture-audit-report.md`
- Roadmap parent: `docs/superpowers/plans/2026-07-10-architecture-remediation-roadmap.md` Phase 3
- **REQUIRED:** work on a **new git branch** (e.g. `refactor/workflow-boundaries`), never directly on `main`
- Scope crate: **`crates/moe-research-workflow` only** (docs that name old paths may update; no MCP/CLI/config behavior changes)
- Preserve public API paths: `limit::*`, `budget::*`, `policy::*`, `research::*`, `report::*`, `workflow::*` and existing crate-root re-exports in `src/lib.rs`
- Preserve schema **0.2**, serde field names/defaults/`deny_unknown_fields` on request/policy DTOs, and `SUPPORTED_SCHEMA_VERSIONS = ["0.2"]`
- Preserve runtime semantics: stricter config∩request limit merge, single explicit search provider per call (no fallback), shared `ResearchBudgetGuard` across deep aspects, whole-batch duplicate tool-call ID rejection before side effects, byte-equal evidence provenance, partial/fail-fast/allow_partial_results behavior, active `RuntimeDeadline` vs passive budget clocks kept distinct, public-safe errors/retryability
- No `contracts` / `common` / `utils` crate; no new public traits or workflow framework
- No config→workflow dependency; CLI remains sole composition root
- Workflow regressions stay in `crates/moe-research-tests` (do not scatter unit tests into production modules unless a pure helper has zero coverage and a characterization test is required)
- TDD mode for this refactor: **behavior-preserving** — existing tests must stay green; write characterization/public-path tests only if a seam lacks coverage
- Incremental stages: workspace must compile after each stage; commit after each stage
- Prefer **move + re-export first**, then function split inside the new module tree
- Do not change `ModelPolicy.require_tool_call_support`, `ResearchContext.prior_sources` participation, dual limit primitives (Phase 2), partial envelope asymmetry, packaging, or A7 `deny_unknown_fields` on result DTOs

### Out of scope (explicit)

- Dual limit primitives / CLI compose mapping (Phase 2: B3/B5/B6/B7)
- Packaging allowlists, skill contracts, partial envelope unify (Phase 1 / product freeze)
- A7 result DTO `deny_unknown_fields`, A8 provider discovery, A9 effective-limits projection (Phase 4)
- Grok adapter weight (B10), test-suite rewrite (B8), contracts crate, multi-provider search fallback

### Target layout (end state)

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

### Current → target ownership map

| Current file | Approx LOC | Target |
| --- | --- | --- |
| `src/research.rs` | 489 | `research/{mod,request,plan,prompt}.rs` (B4) |
| `src/report.rs` | 248 | `report/mod.rs` |
| `src/validator.rs` | 404 | `report/validator.rs` |
| `src/workflow.rs` | 533 | `workflow/{mod,aspect,deep,aggregation}.rs` (B9) |
| `src/agent_loop.rs` | 817 | `runtime/{mod,agent,model_turn,search_tool,deadline}.rs` (B1) |
| `src/runtime_budget.rs` | 307 | `runtime/budget.rs` |
| `src/tool_policy.rs` | 102 | fold into `runtime/search_tool.rs` (logical search tool contract) |
| `src/limit.rs`, `budget.rs`, `policy.rs`, `log_safe.rs` | — | stay root; import-only if needed |

### Public exports that must remain (crate root + modules)

From current `src/lib.rs` — do not drop or rename:

- Modules: `budget`, `limit`, `policy`, `report`, `research`, `workflow`
- Root re-exports: `AgentLimits`, `BudgetConfig`, `ResearchLimits`, limit aliases, policy DTOs, report DTOs including `TokenUsage` / `ValidationStatus` / `ValidationIssue`, research request DTOs, `aspect_research`, `deep_research`, `AspectResearchOutput`, `AspectResearchFailure`, `DeepResearchFailure`
- Crate-private boundary preserved for callers inside crate: `AgentRuntime::new(...).run()`, `AgentRuntimeOutput`, `AgentRuntimeFailure`, `ResearchBudgetGuard`, `AgentBudgetGuard`, effective plans, `AspectPromptInput`, `WorkflowValidationContext`, `effective_research_limits`, `SEARCH_TOOL_NAME` / tool schema helpers (visibility may stay `pub(crate)`; external crates must not need new imports)

### Required verification commands (use after stages)

```bash
cargo test -p moe-research-tests orchestrator
cargo test -p moe-research-tests deep_research
cargo test -p moe-research-tests policy_validator
cargo test -p moe-research-tests schema
cargo test -p moe-research-tests mcp
cargo clippy -p moe-research-workflow --all-targets -- -D warnings
```

Full gate (end of phase):

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

---

### Task 0: Branch, baseline, and public-path inventory

**Files:**
- Create branch only (no production code changes yet)
- Optional create only if missing: `crates/moe-research-tests/tests/schema_tests.rs` already covers schema; do **not** add a new test file unless Task 0 discovers a public path with zero compile coverage
- Read: `crates/moe-research-workflow/src/lib.rs`
- Read: `docs/superpowers/specs/2026-07-10-workflow-crate-responsibility-boundaries-design.md` §§4–9, 14

**Interfaces:**
- Consumes: clean `main` (or integration branch after Phase 1/2) with green workspace
- Produces: branch `refactor/workflow-boundaries` and recorded baseline test status

- [ ] **Step 1: Create the required feature branch**

```bash
cd /home/f4o3/EngineerProjects/Lapis
git status
git checkout main
git pull --ff-only origin main 2>/dev/null || true
git checkout -b refactor/workflow-boundaries
```

Expected: current branch is `refactor/workflow-boundaries`.

- [ ] **Step 2: Record baseline (must be green before moves)**

```bash
cargo test -p moe-research-tests orchestrator
cargo test -p moe-research-tests deep_research
cargo test -p moe-research-tests policy_validator
cargo test -p moe-research-tests schema
cargo test -p moe-research-tests mcp
cargo clippy -p moe-research-workflow --all-targets -- -D warnings
```

Expected: all pass. If any fail, stop and fix on a separate fix branch; do not begin structural moves on a red baseline.

- [ ] **Step 3: Inventory public import paths used by callers**

```bash
rg -n "moe_research_workflow::" crates --glob '*.rs'
rg -n "use moe_research_workflow" crates --glob '*.rs'
```

Confirm external crates use **crate-root** re-exports and/or `moe_research_workflow::{research,report,workflow,budget,policy,limit}` module paths — not private files. Note any `path =` attributes or include! hacks (there should be none).

- [ ] **Step 4: Capture current module graph for re-export checklists**

Document (in commit message / this plan checkboxes only; do not create extra report files) the following crate-private names that **internal** modules currently import:

| Symbol | Current module | Future home |
| --- | --- | --- |
| `EffectiveResearchPlan`, `EffectiveAspectPlan` | `research` | `research::plan` (re-export via `research`) |
| `AspectPromptInput` | `research` | `research::prompt` |
| `WorkflowValidationContext`, `effective_research_limits` | `research` | `research::plan` |
| `SUPPORTED_SCHEMA_VERSIONS`, `ASPECT_PROMPT_MAX_BYTES` | `research` | `request` / `prompt` as appropriate; re-export via `research` |
| `AgentRuntime`, `AgentRuntimeOutput`, `AgentRuntimeFailure` | `agent_loop` | `runtime` |
| `AgentBudgetGuard`, `ResearchBudgetGuard` | `runtime_budget` | `runtime::budget` |
| `OutputValidator` | `validator` | `report::validator` (crate-private) |
| `SEARCH_TOOL_NAME`, `ToolPolicyGuard`, `SearchToolArgs`, `search_model_tool` | `tool_policy` | `runtime::search_tool` |
| `RuntimeDeadline` | private in `agent_loop` | `runtime::deadline` |

- [ ] **Step 5: Commit branch marker (docs-only if desired) or empty commit is unnecessary**

If the branch has no commits yet beyond main, proceed without a no-op commit. Optionally commit this plan file if not already tracked:

```bash
git add docs/superpowers/plans/2026-07-10-phase3-workflow-boundaries.md
git status
git commit -m "$(cat <<'EOF'
docs(plan): add Phase 3 workflow boundaries implementation plan

EOF
)"
```

Only commit if the plan file is untracked/modified and you intend to keep it on the branch.

---

### Task 1: Stage A — Extract `research/` (request / plan / prompt)

**Files:**
- Create: `crates/moe-research-workflow/src/research/mod.rs`
- Create: `crates/moe-research-workflow/src/research/request.rs`
- Create: `crates/moe-research-workflow/src/research/plan.rs`
- Create: `crates/moe-research-workflow/src/research/prompt.rs`
- Delete after facade is solid: `crates/moe-research-workflow/src/research.rs`
- Modify: `crates/moe-research-workflow/src/lib.rs` (only if `mod research;` path needs no change — directory `research/` replaces `research.rs`)
- Touch imports only if needed: `crates/moe-research-workflow/src/workflow.rs`, `agent_loop.rs`, `validator.rs`, `tool_policy.rs`

**Interfaces:**
- Consumes: current `research.rs` contents
- Produces: public `research::*` identical to today; crate-private symbols still reachable as `crate::research::{...}`

Ownership split (move code **without rewriting logic**):

| Target file | Move these items exactly |
| --- | --- |
| `research/request.rs` | `ResearchContext` (+ `empty`), `ResearchPolicy`, `AspectRequest`, `ResearchTask`, `AspectResearchRequest`, `DeepResearchRequest`, `SUPPORTED_SCHEMA_VERSIONS` |
| `research/plan.rs` | `EffectiveResearchPlan`, `EffectiveAspectPlan`, `WorkflowValidationContext`, `normalize_for_execution` impls, `normalize_aspect_request`, `effective_research_limits`, `effective_agent_limits`, `stricter_limit`, provider/tool/name validators, `ensure_*` helpers used by normalization |
| `research/prompt.rs` | `ASPECT_PROMPT_MAX_BYTES`, `AspectPromptInput`, `AspectPromptAspect`, `AspectPromptEvidenceRequirements`, `AspectPromptOutputRequirements`, `From<&EffectiveAspectPlan>` |
| `research/mod.rs` | `mod request; mod plan; mod prompt;` + `pub use` of every public name previously exported from `research.rs` + `pub(crate) use` of private plan/prompt symbols other modules need |

- [ ] **Step 1: Create directory and scaffold facade without deleting `research.rs` yet**

```bash
mkdir -p crates/moe-research-workflow/src/research
```

Because Rust cannot have both `research.rs` and `research/mod.rs`, use this **safe two-step**:

1. Rename flat file to a temporary name:
```bash
git mv crates/moe-research-workflow/src/research.rs crates/moe-research-workflow/src/research/mod.rs
```
2. Immediately split content from `research/mod.rs` into `request.rs` / `plan.rs` / `prompt.rs`, leaving `mod.rs` as facade + re-exports only.

- [ ] **Step 2: Write `research/request.rs` (public DTOs only)**

Move the schema 0.2 request types. Keep attributes (`deny_unknown_fields`, derives) byte-identical.

`request.rs` must contain (signatures unchanged):

```rust
pub struct ResearchContext { /* fields unchanged */ }
impl ResearchContext {
    pub fn empty() -> Self { /* unchanged */ }
}
pub struct ResearchPolicy { /* unchanged */ }
pub struct AspectRequest { /* unchanged */ }
pub struct ResearchTask { /* unchanged */ }
pub struct AspectResearchRequest { /* unchanged */ }
pub struct DeepResearchRequest { /* unchanged */ }
pub(crate) const SUPPORTED_SCHEMA_VERSIONS: &[&str] = &["0.2"];
```

Imports: only what DTOs need (`serde`, `schemars`, `budget`/`policy` types referenced by fields). **No** normalize functions here.

- [ ] **Step 3: Write `research/plan.rs` (effective plans + normalization)**

Move normalization and limit merge without changing merge order or error messages.

Must expose:

```rust
pub(crate) struct EffectiveResearchPlan { /* unchanged fields */ }
pub(crate) struct EffectiveAspectPlan { /* unchanged fields */ }
pub(crate) struct WorkflowValidationContext<'a> { /* unchanged fields */ }
pub(crate) fn effective_research_limits(
    configured: &ResearchLimits,
    requested: Option<&ResearchLimits>,
) -> ResearchLimits { /* exact move */ }

impl AspectResearchRequest {
    pub(crate) fn normalize_for_execution(
        &self,
        ctx: &WorkflowValidationContext<'_>,
    ) -> Result<EffectiveAspectPlan> { /* exact move */ }
}

impl DeepResearchRequest {
    pub(crate) fn normalize_for_execution(
        &self,
        ctx: &WorkflowValidationContext<'_>,
    ) -> Result<EffectiveResearchPlan> { /* exact move */ }
}
```

Keep private helpers in the same file: `normalize_aspect_request`, `effective_agent_limits`, `stricter_limit`, `ensure_schema_version_supported`, `validate_explicit_*`, `ensure_runtime_tools_allowed`, `ensure_non_empty`.

Dependency rule: `plan` may use `request`, `budget`, `limit`, `policy`. It must **not** import `workflow` or `runtime`/`agent_loop`.

- [ ] **Step 4: Write `research/prompt.rs` (LLM-visible projection)**

```rust
pub(crate) const ASPECT_PROMPT_MAX_BYTES: usize = 64 * 1024;
pub(crate) struct AspectPromptInput<'a> { /* unchanged */ }
// private AspectPrompt* structs + From<&EffectiveAspectPlan>
```

Note: `normalize_aspect_request` currently enforces `ASPECT_PROMPT_MAX_BYTES`; after the split either:

- keep the const in `prompt.rs` and import it from `plan.rs`, or
- re-export the const from `research/mod.rs` and import via `super::ASPECT_PROMPT_MAX_BYTES`.

Do not change the numeric value or error strings.

- [ ] **Step 5: Write `research/mod.rs` facade**

```rust
//! Research request domain, effective plans, and prompt projection.

mod plan;
mod prompt;
mod request;

pub use request::{
    AspectRequest, AspectResearchRequest, DeepResearchRequest, ResearchContext, ResearchPolicy,
    ResearchTask,
};

pub(crate) use plan::{
    EffectiveAspectPlan, EffectiveResearchPlan, WorkflowValidationContext, effective_research_limits,
};
pub(crate) use prompt::{ASPECT_PROMPT_MAX_BYTES, AspectPromptInput};
pub(crate) use request::SUPPORTED_SCHEMA_VERSIONS;
```

Ensure every previous `crate::research::X` import still resolves. Prefer re-exporting rather than updating every call site.

- [ ] **Step 6: Fix internal imports if the compiler requires submodule paths**

```bash
cargo check -p moe-research-workflow
```

If other modules imported private items only available through the old monolithic file, keep re-exports in `research/mod.rs` so call sites stay:

```rust
use crate::research::{
    AspectResearchRequest, DeepResearchRequest, EffectiveAspectPlan, EffectiveResearchPlan,
    SUPPORTED_SCHEMA_VERSIONS, WorkflowValidationContext, effective_research_limits,
};
```

- [ ] **Step 7: Run Stage A tests**

```bash
cargo test -p moe-research-tests schema
cargo test -p moe-research-tests orchestrator
cargo test -p moe-research-tests deep_research
cargo test -p moe-research-tests policy_validator
cargo test -p moe-research-tests mcp
cargo clippy -p moe-research-workflow --all-targets -- -D warnings
```

Expected: PASS. No behavior changes; failures indicate a missed re-export, wrong visibility, or accidental logic edit — fix by restoring moved code, not by “improving” normalization.

- [ ] **Step 8: Commit Stage A**

```bash
git add crates/moe-research-workflow/src/research crates/moe-research-workflow/src/lib.rs
# include any import-only touchups in workflow/agent_loop/validator/tool_policy
git add -u crates/moe-research-workflow/src
git status
git commit -m "$(cat <<'EOF'
refactor(workflow): split research into request/plan/prompt modules

Behavior-preserving extraction for B4. Public research::* paths and
schema 0.2 normalization semantics unchanged.
EOF
)"
```

---

### Task 2: Stage B — Extract `report/` (DTOs + validator)

**Files:**
- Create: `crates/moe-research-workflow/src/report/mod.rs` (from current `report.rs`)
- Create: `crates/moe-research-workflow/src/report/validator.rs` (from current `validator.rs`)
- Delete: `crates/moe-research-workflow/src/report.rs`, `crates/moe-research-workflow/src/validator.rs`
- Modify: `crates/moe-research-workflow/src/lib.rs` — remove `mod validator;`; `pub mod report;` now maps to directory
- Modify imports: `crates/moe-research-workflow/src/agent_loop.rs` (today `use crate::validator::OutputValidator`)

**Interfaces:**
- Consumes: `report.rs` public DTOs; `validator.rs` `OutputValidator`
- Produces: public `report::*` unchanged; `OutputValidator` crate-private under `report::validator` with re-export path usable as `crate::report::validator::OutputValidator` **or** `pub(crate) use` from `report/mod.rs` if you prefer a single import path — pick one and use consistently

Recommended visibility (matches design: validator remains private):

```rust
// report/mod.rs
mod validator;
pub(crate) use validator::OutputValidator;
// all public DTO re-exports / definitions live here
```

Call sites then use `crate::report::OutputValidator` only if re-exported; otherwise update `agent_loop` to `crate::report::validator::OutputValidator`. Prefer **`pub(crate) use validator::OutputValidator`** so only one line changes in the runtime.

- [ ] **Step 1: Convert `report.rs` into `report/mod.rs`**

```bash
mkdir -p crates/moe-research-workflow/src/report
git mv crates/moe-research-workflow/src/report.rs crates/moe-research-workflow/src/report/mod.rs
git mv crates/moe-research-workflow/src/validator.rs crates/moe-research-workflow/src/report/validator.rs
```

- [ ] **Step 2: Wire `report/mod.rs`**

At top of `report/mod.rs` after the module docs / imports for DTOs:

```rust
mod validator;

pub(crate) use validator::OutputValidator;
```

Keep all public structs/enums/`pub use moe_research_model::TokenUsage` exactly as before. Do not add `deny_unknown_fields` to result DTOs (A7 is out of scope).

- [ ] **Step 3: Fix `report/validator.rs` imports**

Change:

```rust
use crate::report::{ ... };
use crate::research::AspectRequest;
```

to sibling/super paths as needed:

```rust
use super::{
    AspectReport, AspectResearchResult, Evidence, Finding, OpenQuestion, ValidationIssue,
    ValidationStatus,
};
use crate::research::AspectRequest; // still public research DTO
use crate::log_safe::{json_error_message_for_log, safe_evidence_id_for_log};
use crate::policy::{EvidencePolicy, OutputPolicy};
```

Do not change validation order, provenance byte-equality, or issue codes/messages.

- [ ] **Step 4: Update `lib.rs`**

Remove:

```rust
mod validator;
```

Keep:

```rust
pub mod report;
```

Root re-exports of report types must remain identical.

- [ ] **Step 5: Update runtime import of `OutputValidator`**

In `agent_loop.rs` (or later `runtime/agent.rs`):

```rust
use crate::report::OutputValidator;
```

- [ ] **Step 6: Run Stage B tests**

```bash
cargo test -p moe-research-tests policy_validator
cargo test -p moe-research-tests schema
cargo test -p moe-research-tests deep_research
cargo test -p moe-research-tests orchestrator
cargo clippy -p moe-research-workflow --all-targets -- -D warnings
```

Expected: PASS.

- [ ] **Step 7: Commit Stage B**

```bash
git add -u crates/moe-research-workflow/src
git status
git commit -m "$(cat <<'EOF'
refactor(workflow): nest report DTOs and OutputValidator under report/

Behavior-preserving move for report validation ownership. Public report
exports and validation timing unchanged.
EOF
)"
```

---

### Task 3: Stage C — Extract `workflow/` (aspect / deep / aggregation)

**Files:**
- Create: `crates/moe-research-workflow/src/workflow/mod.rs`
- Create: `crates/moe-research-workflow/src/workflow/aspect.rs`
- Create: `crates/moe-research-workflow/src/workflow/deep.rs`
- Create: `crates/moe-research-workflow/src/workflow/aggregation.rs`
- Delete: `crates/moe-research-workflow/src/workflow.rs`
- Modify: `crates/moe-research-workflow/src/lib.rs` only if root re-exports need adjustment (prefer facade re-exports)

**Interfaces:**
- Consumes: current `workflow.rs`; crate-private `AgentRuntime` from `agent_loop` (still flat until Stage D)
- Produces: public `workflow::aspect_research`, `workflow::deep_research`, public output/failure types; crate-root re-exports unchanged

Ownership split:

| Target | Items |
| --- | --- |
| `workflow/aspect.rs` | `AspectResearchOutput`, `AspectResearchFailure`, `aspect_research`, mapping helpers `from_runtime` / `top_level` for aspect |
| `workflow/deep.rs` | `DeepResearchFailure`, `deep_research`, `execute_aspects`, `run_aspect_runtime`, `aspect_requests`, shared budget construction, post-run `ensure_usage_within` handling, tracing for deep start/finish |
| `workflow/aggregation.rs` | `DeepResearchRun`, `record_aspect_result`, `record_aspect_success`, `namespace_aspect_evidence`, `finalize_deep_result`, `deep_result`, `order_failures_by_request`, `aspect_failure`, `confidence_summary` |
| `workflow/mod.rs` | submodule decls + `pub use` of public entrypoints/types |

Hard semantics to preserve while moving:

1. Standalone `aspect_research` builds research budget via `effective_research_limits(&budget_config.research, None)`, `record_agent_started()`, then `run_aspect_runtime`.
2. Deep path builds **one** `ResearchBudgetGuard::new(plan.limits.clone())` and clones/passes the **same** `Arc` (or existing shared handle) to every aspect.
3. `allow_partial_results` / `fail_fast` branches stay in the same order relative to recording and finalization.
4. Evidence namespacing remains `aspect_id:original_id` (exact existing implementation).
5. Failure ordering remains request order via `order_failures_by_request`.

- [ ] **Step 1: Convert flat `workflow.rs` into directory**

```bash
mkdir -p crates/moe-research-workflow/src/workflow
git mv crates/moe-research-workflow/src/workflow.rs crates/moe-research-workflow/src/workflow/mod.rs
```

- [ ] **Step 2: Extract `workflow/aggregation.rs` first (pure coordination)**

Move pure/recording helpers. No provider calls.

```rust
// aggregation.rs — illustrative surface; copy exact bodies from current workflow.rs
pub(super) struct DeepResearchRun { /* fields unchanged */ }

impl DeepResearchRun {
    pub(super) fn new() -> Self { /* unchanged */ }
}

pub(super) fn record_aspect_result(/* exact signature from current file */) { /* move */ }
pub(super) fn record_aspect_success(/* ... */) { /* move */ }
pub(super) fn namespace_aspect_evidence(/* ... */) { /* move */ }
pub(super) fn finalize_deep_result(/* ... */) { /* move */ }
pub(super) fn deep_result(/* ... */) { /* move */ }
pub(super) fn order_failures_by_request(/* ... */) { /* move */ }
pub(super) fn aspect_failure(/* ... */) { /* move */ }
pub(super) fn confidence_summary(/* ... */) { /* move */ }
```

Use `pub(super)` so only `workflow::{deep,aspect,mod}` see these. Do not make aggregation public outside the crate.

- [ ] **Step 3: Extract `workflow/aspect.rs`**

```rust
pub struct AspectResearchOutput { /* unchanged */ }
pub struct AspectResearchFailure { /* unchanged */ }

pub async fn aspect_research(
    request: AspectResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    budget_config: &BudgetConfig,
) -> std::result::Result<AspectResearchOutput, Box<AspectResearchFailure>> {
    // exact body from current workflow.rs
}
```

Keep module-level docs that explain standalone inherits operator research caps.

- [ ] **Step 4: Extract `workflow/deep.rs`**

```rust
pub struct DeepResearchFailure { /* unchanged */ }

pub async fn deep_research(
    request: DeepResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
    budget_config: &BudgetConfig,
) -> std::result::Result<DeepResearchResult, Box<DeepResearchFailure>> {
    // exact body; call aggregation helpers
}

pub(super) async fn execute_aspects(/* exact */) -> DeepResearchRun { /* move */ }
pub(super) async fn run_aspect_runtime(/* exact */) -> Result<AgentRuntimeOutput, AgentRuntimeFailure> {
    // still constructs AgentRuntime::new(...).run().await
}
pub(super) fn aspect_requests(request: &EffectiveResearchPlan) -> Vec<EffectiveAspectPlan> { /* move */ }
```

Import runtime as today:

```rust
use crate::agent_loop::{AgentRuntime, AgentRuntimeFailure, AgentRuntimeOutput};
use crate::runtime_budget::ResearchBudgetGuard;
```

(After Stage D these become `crate::runtime::{...}`.)

- [ ] **Step 5: Write `workflow/mod.rs` facade**

```rust
//! Workflow orchestration for standalone aspect and multi-aspect deep research.

mod aggregation;
mod aspect;
mod deep;

pub use aspect::{AspectResearchFailure, AspectResearchOutput, aspect_research};
pub use deep::{DeepResearchFailure, deep_research};
```

Ensure `lib.rs` still has:

```rust
pub use workflow::{
    AspectResearchFailure, AspectResearchOutput, DeepResearchFailure, aspect_research,
    deep_research,
};
```

- [ ] **Step 6: Run Stage C tests**

```bash
cargo test -p moe-research-tests orchestrator
cargo test -p moe-research-tests deep_research
cargo test -p moe-research-tests mcp
cargo test -p moe-research-tests policy_validator
cargo clippy -p moe-research-workflow --all-targets -- -D warnings
```

Expected: PASS. Failures in partial/fail_fast/budget tests mean aggregation or shared-guard wiring regressed — compare against pre-move `deep_research` / `execute_aspects` control flow.

- [ ] **Step 7: Commit Stage C**

```bash
git add -u crates/moe-research-workflow/src
git status
git commit -m "$(cat <<'EOF'
refactor(workflow): split workflow into aspect/deep/aggregation

Behavior-preserving extraction for B9. Shared research budget, partial
finalize, and public entrypoints unchanged.
EOF
)"
```

---

### Task 4: Stage D1 — Create `runtime/` support modules (budget, deadline, search_tool, model_turn)

**Files:**
- Create: `crates/moe-research-workflow/src/runtime/mod.rs`
- Create: `crates/moe-research-workflow/src/runtime/budget.rs` (from `runtime_budget.rs`)
- Create: `crates/moe-research-workflow/src/runtime/deadline.rs` (from `RuntimeDeadline` in `agent_loop.rs`)
- Create: `crates/moe-research-workflow/src/runtime/search_tool.rs` (from `tool_policy.rs` + search helpers in `agent_loop.rs`)
- Create: `crates/moe-research-workflow/src/runtime/model_turn.rs` (from model helpers in `agent_loop.rs`)
- Temporary keep: `crates/moe-research-workflow/src/agent_loop.rs` as thin orchestrator still compiling against new modules
- Delete when unused: `crates/moe-research-workflow/src/runtime_budget.rs`, `crates/moe-research-workflow/src/tool_policy.rs`
- Modify: `crates/moe-research-workflow/src/lib.rs` — replace `mod runtime_budget; mod tool_policy; mod agent_loop;` with `mod runtime;`

**Interfaces:**
- Consumes: live guards, tool policy, deadline, model/search helpers
- Produces: crate-private `runtime` module; `AgentRuntime` still available to `workflow` (via `agent_loop` temporarily or already re-exported from `runtime`)

**Important staging rule:** Do **not** rewrite reservation order. Budget consume → provider call → token record must stay identical. Deadline must wrap the same futures it wraps today.

- [ ] **Step 1: Create `runtime/` and move live budget guards**

```bash
mkdir -p crates/moe-research-workflow/src/runtime
git mv crates/moe-research-workflow/src/runtime_budget.rs crates/moe-research-workflow/src/runtime/budget.rs
```

`runtime/budget.rs` keeps:

```rust
pub struct AgentBudgetGuard { /* ... */ }
pub struct ResearchBudgetGuard { /* ... */ }
// all methods exact
```

Prefer `pub(crate)` on these types (they were `pub` only within a private module before — keep crate-private visibility equivalent: previously `runtime_budget` was private module with `pub struct`, so structs were crate-visible). Match prior visibility: `pub(crate) struct AgentBudgetGuard` / `ResearchBudgetGuard` if clippy/privacy allows; otherwise `pub struct` inside private `runtime` module is fine.

- [ ] **Step 2: Move `RuntimeDeadline` into `runtime/deadline.rs`**

Cut from `agent_loop.rs`:

```rust
pub(crate) struct RuntimeDeadline { /* fields */ }

impl RuntimeDeadline {
    pub(crate) fn new(timeout_ms: DurationLimitMs) -> Self { /* exact */ }
    pub(crate) fn remaining(&self) -> Result<Option<Duration>> { /* exact */ }
    pub(crate) async fn run<F, T>(&self, future: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    { /* exact — do not merge with AgentBudgetGuard clocks */ }
}
```

Also move `elapsed_ms` if only used by deadline/agent diagnostics — keep next to callers if shared.

- [ ] **Step 3: Fold `tool_policy.rs` into `runtime/search_tool.rs`**

```bash
git mv crates/moe-research-workflow/src/tool_policy.rs crates/moe-research-workflow/src/runtime/search_tool.rs
```

Then **append** (move, do not rewrite) search-related methods currently on `AgentRuntime`:

- `selected_search_provider`
- `search_request`
- `evidence_from_search`
- `evidence_from_result`
- `search_result_message`
- the body of `execute_tool_call` search path

Recommended shape after extraction (preserve call order inside functions):

```rust
pub(crate) const SEARCH_TOOL_NAME: &str = "search";
pub(crate) struct SearchToolArgs { /* exact */ }
pub(crate) struct ToolPolicyGuard { /* exact */ }
pub(crate) fn search_model_tool() -> ModelTool { /* exact */ }

// free functions or a small SearchToolExecutor struct — no new public trait
pub(crate) async fn execute_search_tool_call(/* parameters mirroring current execute_tool_call needs */) -> Result<ModelToolOutput> {
    // 1) ToolPolicyGuard::validate_search_call
    // 2) agent + research budget reservations (same order as today)
    // 3) single provider SearchService call
    // 4) evidence conversion + candidate_evidence extend
    // 5) tool output message
}
```

If extracting free functions is riskier mid-move, **first** only relocate `tool_policy` content into `search_tool.rs` and keep methods on `AgentRuntime` calling into it; complete method moves in Task 5. Minimum for this step: file exists and compiles; `SEARCH_TOOL_NAME` still reachable for workflow normalization context.

Update workflow import:

```rust
use crate::runtime::search_tool::SEARCH_TOOL_NAME;
// or re-export from runtime/mod.rs
```

- [ ] **Step 4: Create `runtime/model_turn.rs` with model request/dispatch helpers**

Move from `agent_loop.rs` without changing message construction:

```rust
pub(crate) fn aspect_response_format() -> ModelResponseFormat { /* exact */ }
pub(crate) fn add_token_usage(total: &mut Option<TokenUsage>, delta: Option<TokenUsage>) { /* exact */ }
pub(crate) fn sum_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> { /* exact */ }

// Prefer free functions taking the needed slices of state:
pub(crate) async fn complete_model(
    model_service: &ModelService,
    plan: &EffectiveAspectPlan,
    previous_response_id: Option<String>,
    input: Vec<ModelInputItem>,
    tools: Vec<ModelTool>,
) -> Result<ModelResponse> {
    // exact ModelRequest construction: System/User already in input;
    // provider from plan.task.model_provider; policy.model.apply_to
}

pub(crate) async fn complete_model_turn(
    // parameters: plan, services, state, budgets, tool_policy, ...
) -> Result<ModelResponse> {
    // exact order:
    // budget.consume_model_turn()?;
    // research_budget.try_consume_model_call()?;
    // complete_model(...).await;
    // add_token_usage + research_budget.record_token_usage
}
```

Prompt helpers that build System/User must keep:

```text
System = exact inline instructions (plan.task.instructions)
User   = serde_json pretty AspectPromptInput::from(plan)
```

Move `initial_input`, `system_prompt`, `user_prompt` either into `model_turn.rs` or keep on agent until Task 5; if moved, signatures:

```rust
pub(crate) fn system_prompt(plan: &EffectiveAspectPlan) -> &str {
    &plan.task.instructions
}
pub(crate) fn user_prompt(plan: &EffectiveAspectPlan) -> String { /* exact */ }
pub(crate) fn initial_input(plan: &EffectiveAspectPlan) -> Vec<ModelInputItem> { /* exact */ }
```

- [ ] **Step 5: Write temporary `runtime/mod.rs` and rewire `lib.rs`**

```rust
// runtime/mod.rs
pub(crate) mod budget;
pub(crate) mod deadline;
pub(crate) mod model_turn;
pub(crate) mod search_tool;

pub(crate) use budget::{AgentBudgetGuard, ResearchBudgetGuard};
pub(crate) use search_tool::SEARCH_TOOL_NAME;
// AgentRuntime still in agent_loop until Task 5 — re-export later
```

`lib.rs`:

```rust
mod agent_loop;
pub mod budget;
pub mod limit;
mod log_safe;
pub mod policy;
pub mod report;
pub mod research;
mod runtime;
pub mod workflow;
```

Remove `mod runtime_budget;` and `mod tool_policy;`.

Update all imports:

```rust
// old
use crate::runtime_budget::{AgentBudgetGuard, ResearchBudgetGuard};
use crate::tool_policy::{SearchToolArgs, ToolPolicyGuard, SEARCH_TOOL_NAME};
// new
use crate::runtime::{AgentBudgetGuard, ResearchBudgetGuard, SEARCH_TOOL_NAME};
use crate::runtime::search_tool::{SearchToolArgs, ToolPolicyGuard};
```

- [ ] **Step 6: Keep `agent_loop.rs` compiling as coordinator using new modules**

`agent_loop` may still own `AgentRuntime::run` loop at this step, but should call into `runtime::model_turn`, `runtime::search_tool`, `runtime::deadline`, `runtime::budget`.

- [ ] **Step 7: Run Stage D1 tests**

```bash
cargo test -p moe-research-tests orchestrator
cargo test -p moe-research-tests deep_research
cargo test -p moe-research-tests policy_validator
cargo test -p moe-research-tests mcp
cargo clippy -p moe-research-workflow --all-targets -- -D warnings
```

Expected: PASS. Pay special attention to timeout, budget_exceeded, tool_policy_denied, and single-provider tests.

- [ ] **Step 8: Commit Stage D1**

```bash
git add -u crates/moe-research-workflow/src
git status
git commit -m "$(cat <<'EOF'
refactor(workflow): extract runtime budget, deadline, search, model_turn

Behavior-preserving support-module extraction toward B1. Reservation and
deadline semantics unchanged; agent loop still coordinates turns.
EOF
)"
```

---

### Task 5: Stage D2 — Extract agent state machine into `runtime/agent.rs`

**Files:**
- Create: `crates/moe-research-workflow/src/runtime/agent.rs`
- Modify: `crates/moe-research-workflow/src/runtime/mod.rs` — own `AgentRuntime` boundary
- Delete: `crates/moe-research-workflow/src/agent_loop.rs`
- Modify: `crates/moe-research-workflow/src/lib.rs` — remove `mod agent_loop;`
- Modify: `crates/moe-research-workflow/src/workflow/deep.rs` (and aspect if needed) imports `crate::runtime::{AgentRuntime, ...}`

**Interfaces:**
- Consumes: Stage D1 modules
- Produces: final crate-private runtime boundary:

```rust
// runtime/mod.rs
mod agent;
pub(crate) mod budget;
pub(crate) mod deadline;
pub(crate) mod model_turn;
pub(crate) mod search_tool;

pub(crate) use agent::{AgentRuntime, AgentRuntimeFailure, AgentRuntimeOutput};
pub(crate) use budget::{AgentBudgetGuard, ResearchBudgetGuard};
pub(crate) use search_tool::SEARCH_TOOL_NAME;
```

Preserve constructor and run signature exactly:

```rust
impl<'a> AgentRuntime<'a> {
    pub(crate) fn new(
        model_service: &'a ModelService,
        search_service: &'a SearchService,
        request: &'a EffectiveAspectPlan,
        research_budget: Arc<ResearchBudgetGuard>,
    ) -> Self { /* exact fields */ }

    pub(crate) async fn run(&self) -> Result<AgentRuntimeOutput, AgentRuntimeFailure> {
        // state machine only: validate prompt, create deadline + AgentBudgetGuard,
        // loop: model_turn -> optional tools via search_tool -> finish/validate/partial
    }
}
```

`RuntimeState` (replay/input/evidence/token_usage) stays private inside `agent.rs`.

- [ ] **Step 1: Move remaining `agent_loop.rs` content into `runtime/agent.rs`**

```bash
git mv crates/moe-research-workflow/src/agent_loop.rs crates/moe-research-workflow/src/runtime/agent.rs
```

Strip pieces already moved to other runtime files; leave only:

- `AgentRuntime`, `AgentRuntimeOutput`, `AgentRuntimeFailure`
- `RuntimeState`
- `run` loop
- `validate_inline_prompt`
- `ensure_unique_tool_call_ids` (whole-batch before any tool side effect)
- `finish` + validation handoff via `OutputValidator`
- `partial_output` / `failure` / `untraced_failure`
- orchestration calls into `model_turn` / `search_tool` / `deadline` / `budget`

- [ ] **Step 2: Update `runtime/mod.rs` and delete root `agent_loop` module**

`lib.rs` final private runtime entry:

```rust
mod log_safe;
mod runtime;
```

No `mod agent_loop`.

- [ ] **Step 3: Point workflow at `crate::runtime`**

In `workflow/deep.rs` / any remaining references:

```rust
use crate::runtime::{AgentRuntime, AgentRuntimeFailure, AgentRuntimeOutput, ResearchBudgetGuard};
```

- [ ] **Step 4: Confirm dependency rules**

```bash
rg -n "use crate::workflow" crates/moe-research-workflow/src/runtime
rg -n "use crate::runtime" crates/moe-research-workflow/src/research
rg -n "use crate::runtime" crates/moe-research-workflow/src/report
```

Expected: no `runtime → workflow`, no `research/report → runtime/workflow`.

- [ ] **Step 5: Run Stage D2 tests (full workflow-relevant set)**

```bash
cargo test -p moe-research-tests orchestrator
cargo test -p moe-research-tests deep_research
cargo test -p moe-research-tests policy_validator
cargo test -p moe-research-tests schema
cargo test -p moe-research-tests mcp
cargo clippy -p moe-research-workflow --all-targets -- -D warnings
```

Expected: PASS. This stage closes **B1** structurally if `agent.rs` is the state machine and adapters live elsewhere.

- [ ] **Step 6: Commit Stage D2**

```bash
git add -u crates/moe-research-workflow/src
git status
git commit -m "$(cat <<'EOF'
refactor(workflow): move agent state machine into runtime::agent

Completes private runtime boundary for B1. Public API and agent-loop
semantics unchanged; model/search adapters and deadlines live beside agent.
EOF
)"
```

---

### Task 6: Stage E — `lib.rs` cleanup, docs, clippy, full gate

**Files:**
- Modify: `crates/moe-research-workflow/src/lib.rs`
- Modify: `crates/moe-research-workflow/CLAUDE.md`
- Modify: `/home/f4o3/EngineerProjects/Lapis/CLAUDE.md` (module index paths)
- Modify: `docs/development.md` only if it lists old workflow source files
- Modify: `docs/research-agent-product.md` — fix `SUPPORTED_SCHEMA_VERSIONS` path from `src/workflow.rs` to `src/research/request.rs` (or `research` module)
- Do **not** expand into unrelated README schema-version / PM tier drift

**Interfaces:**
- Consumes: final tree from Tasks 1–5
- Produces: documentation aligned with layout; clippy clean on workflow package and workspace

- [ ] **Step 1: Finalize `lib.rs` module list and re-exports**

Target `lib.rs` shape:

```rust
//! Research workflow boundary for MoeResearch.

pub mod budget;
pub mod limit;
pub mod policy;
pub mod report;
pub mod research;
pub mod workflow;

mod log_safe;
mod runtime;

pub use budget::{AgentLimits, BudgetConfig, ResearchLimits};
pub use limit::{CountLimit, DurationLimitMs, Limit, TokenLimit};
pub use policy::{
    EvidencePolicy, ExecutionPolicy, Freshness, ModelPolicy, OutputPolicy, SearchCategory,
    SearchContentLevel, SearchDepth, SearchPolicy, SearchRecency, ToolName,
};
pub use report::{
    AgentBudgetUsage, AspectFailure, AspectReport, AspectResearchResult, Confidence,
    ConfidenceSummary, CoverageSummary, DeepResearchResult, Evidence, Finding, FindingType,
    Importance, OpenQuestion, ResearchBudgetUsage, SourceType, TokenUsage, ValidationIssue,
    ValidationStatus,
};
pub use research::{
    AspectRequest, AspectResearchRequest, DeepResearchRequest, ResearchContext, ResearchPolicy,
    ResearchTask,
};
pub use workflow::{
    AspectResearchFailure, AspectResearchOutput, DeepResearchFailure, aspect_research,
    deep_research,
};
```

Verify no leftover `mod agent_loop|runtime_budget|tool_policy|validator`.

- [ ] **Step 2: Update `crates/moe-research-workflow/CLAUDE.md`**

Replace related file list and entry descriptions with the new layout:

- Core runtime: `src/runtime/agent.rs` (+ `model_turn`, `search_tool`, `budget`, `deadline`)
- Multi-aspect orchestration: `src/workflow/{aspect,deep,aggregation}.rs`
- Request/plan/prompt: `src/research/{request,plan,prompt}.rs`
- Report + validator: `src/report/{mod,validator}.rs`
- Note `SUPPORTED_SCHEMA_VERSIONS` lives under research request domain

Add changelog row dated 2026-07-10 for Phase 3 boundaries restructure.

- [ ] **Step 3: Update root `CLAUDE.md` module index cell for workflow**

Change entry/key files from:

`src/workflow.rs`, `src/agent_loop.rs`, `src/research.rs`, `src/report.rs`

to directory-oriented paths, e.g.:

`src/workflow/`, `src/runtime/`, `src/research/`, `src/report/`

- [ ] **Step 4: Fix product doc path for schema versions**

In `docs/research-agent-product.md`, update the sentence that points at `src/workflow.rs` for `SUPPORTED_SCHEMA_VERSIONS` to the research module path (actual file: `crates/moe-research-workflow/src/research/request.rs`).

- [ ] **Step 5: Optional characterization test only if public paths are not compiled by existing tests**

If `cargo test -p moe-research-tests schema` already constructs all public DTOs and MCP/orchestrator call entrypoints, **do not** add tests.

Only if a module-qualified path is unreferenced, add a small test in `crates/moe-research-tests/tests/schema_tests.rs`:

```rust
#[test]
fn public_module_paths_remain_reachable() {
    use moe_research_workflow::budget::BudgetConfig;
    use moe_research_workflow::limit::Limit;
    use moe_research_workflow::policy::ToolName;
    use moe_research_workflow::report::DeepResearchResult;
    use moe_research_workflow::research::DeepResearchRequest;
    use moe_research_workflow::workflow::{aspect_research, deep_research};
    let _ = std::any::type_name::<BudgetConfig>();
    let _ = std::any::type_name::<Limit<u32>>();
    let _ = std::any::type_name::<ToolName>();
    let _ = std::any::type_name::<DeepResearchResult>();
    let _ = std::any::type_name::<DeepResearchRequest>();
    let _ = std::any::type_name_of_val(&aspect_research);
    let _ = std::any::type_name_of_val(&deep_research);
}
```

Do not assert private file paths.

- [ ] **Step 6: Full verification gate**

```bash
cargo fmt --all -- --check
cargo test -p moe-research-tests orchestrator
cargo test -p moe-research-tests deep_research
cargo test -p moe-research-tests policy_validator
cargo test -p moe-research-tests schema
cargo test -p moe-research-tests mcp
cargo test --workspace
cargo clippy -p moe-research-workflow --all-targets -- -D warnings
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: all green. Remove obsolete `#[allow(clippy::too_many_lines)]` only when the function is actually short enough; do not silence new lints by allowing without shrinking.

- [ ] **Step 7: Self-check acceptance criteria (design §14)**

Manually confirm:

1. Control-plane modules still at `limit`/`budget`/`policy` paths  
2. `research/{request,plan,prompt}` ownership  
3. `workflow/{aspect,deep,aggregation}` ownership  
4. `runtime/{agent,model_turn,search_tool,budget,deadline}` ownership  
5. External crates compile without import changes  
6. Schema/error/partial semantics unchanged (tests green)  
7. Stricter limit merge still covered by orchestrator/deep tests  
8. Single provider per search still covered  
9. Shared global budget still covered  
10. Side-effect ordering tests still pass  
11. Prompt isolation / provenance tests still pass  
12. No new regression tests outside `moe-research-tests`  
13. fmt/test/clippy workspace gate green  
14. Docs for moved paths updated  

- [ ] **Step 8: Commit Stage E**

```bash
git add crates/moe-research-workflow/src/lib.rs \
  crates/moe-research-workflow/CLAUDE.md \
  CLAUDE.md \
  docs/research-agent-product.md \
  docs/development.md \
  crates/moe-research-tests/tests/schema_tests.rs
git add -u
git status
git commit -m "$(cat <<'EOF'
docs(workflow): align module docs with runtime/research/report layout

Phase 3 cleanup: lib module graph, CLAUDE indexes, and schema_version
path references match the responsibility-boundary tree.
EOF
)"
```

- [ ] **Step 9: Phase retrospective note (append to this plan file)**

Append a short section at the end of this file:

```markdown
## Phase 3 retrospective

- Branch: refactor/workflow-boundaries
- Findings closed: B1, B4, B9
- Final tree: <paste `find crates/moe-research-workflow/src -type f | sort`>
- Residual clippy allows: <list or none>
- Follow-ups deferred: dual limits (Phase 2 if not done), A7/A8/A9/B8 (Phase 4)
```

Commit:

```bash
git add docs/superpowers/plans/2026-07-10-phase3-workflow-boundaries.md
git commit -m "$(cat <<'EOF'
docs(plan): record Phase 3 workflow boundaries retrospective

EOF
)"
```

---

## Dependency rules cheat-sheet (enforce every stage)

```text
budget -> limit
policy -> provider-neutral model/search DTOs (existing)
research::plan -> research::request, limit, budget, policy
research::prompt -> research::request + EffectiveAspectPlan (plan types)
report -> research request DTOs only as needed for validator aspect identity
workflow -> research, report, runtime, budget, policy, log_safe
runtime -> research::{plan,prompt}, report::validator, budget, policy, model/search services, log_safe
```

Forbidden:

- `research`/`report` → `workflow`/`runtime`
- `runtime` → `workflow`
- new shared context mega-struct
- new public traits for extraction theater
- config crate depending on workflow

---

## Risk register (from design §13) — operational responses

| Risk | What to do in this plan |
| --- | --- |
| Missed re-export | Run schema + mcp + orchestrator after every facade extraction |
| Normalization precedence change | Move-only; do not rewrite `stricter_limit` |
| Validation order change | Keep whole-batch tool ID check before first tool future |
| Per-aspect global budgets | Deep path constructs one guard; pass same handle |
| Deadline clock merge | `deadline.rs` separate from `runtime/budget.rs` |
| Search fallback introduced | Keep explicit provider; no multi-provider loop |
| Lost partial evidence | Preserve `AgentRuntimeFailure.partial_output` and aggregation recording |
| Public DTO cleanup creep | Do not delete `ValidationStatus` / `TokenUsage` re-exports |
| Doc scope creep | Only path-invalidated docs |

---

## Self-review (plan vs design)

### Spec coverage map

| Design section | Plan task |
| --- | --- |
| §4.1 control-plane stay put | Global Constraints + final tree |
| §4.2 research request/plan/prompt | Task 1 Stage A |
| §4.3 report + validator | Task 2 Stage B |
| §4.4 workflow aspect/deep/aggregation | Task 3 Stage C |
| §4.5 runtime agent/model_turn/search_tool/budget/deadline | Tasks 4–5 Stages D1–D2 |
| §4.6 log_safe root | Global Constraints / final `lib.rs` |
| §5 dependency rules | cheat-sheet + Task 5 Step 4 |
| §6 data flow / AgentRuntime boundary | Tasks 3–5 signatures |
| §7 public compatibility | Task 0 inventory + Task 6 exports |
| §8 error/timeout/partial semantics | stage test filters + risk register |
| §9 migration stages 0–6 | Tasks 0–6 (A–E) |
| §10 test strategy | required commands every stage |
| §11 documentation | Task 6 |
| §14 acceptance criteria | Task 6 Step 7 |
| Audit B1/B4/B9 | B4→Task1, B9→Task3, B1→Tasks4–5 |

### Placeholder scan

No TBD/TODO steps; each stage lists exact paths, commands, commit messages, and the symbols to move.

### Type/export consistency

- Public entrypoints remain `aspect_research` / `deep_research` with the same argument lists and `Result`/`Box<Failure>` shapes.
- `AgentRuntime::new(model_service, search_service, request: &EffectiveAspectPlan, research_budget: Arc<ResearchBudgetGuard>)` preserved.
- `SEARCH_TOOL_NAME` remains the supported tool name fed into `WorkflowValidationContext`.
- Crate-root re-export list matches current `lib.rs`.

### Staging note vs design §9 numbering

Design stages 3–4 (runtime support then agent) map to plan Tasks 4–5 (D1/D2). Design stage 5 (workflow) is plan Task 3 (C) **before** full agent file deletion is acceptable only if workflow still compiles against transitional `agent_loop`; this plan prefers **workflow directory extraction while `agent_loop` still exists (Task 3), then runtime (Tasks 4–5)**, because workflow primarily *calls* the runtime boundary and benefits from early B9 closure without blocking on agent decomposition. If a conflict arises, keep Task 3’s `run_aspect_runtime` import path updated in Task 5 only — do not change scheduling logic twice.

---

## Findings closed when complete

| ID | Finding | Closed by |
| --- | --- | --- |
| B1 | `agent_loop.rs` god-module density | Tasks 4–5 (`runtime/*`) |
| B4 | `research.rs` mixed schema/normalize/prompt | Task 1 (`research/*`) |
| B9 | `workflow.rs` scheduling + finalize density | Task 3 (`workflow/*`) |
