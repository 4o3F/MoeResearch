# MoeResearch / Lapis Architecture Audit Report

**Date:** 2026-07-10  
**Status:** Audit for owner evaluation (not an implementation spec)  
**Scope:** Rust workspace code quality + Layer 1 (`skills/`, `prompts/`) responsibility split  
**Delivery form:** Dual-column full audit  
**Grading axes:** Column A = contract / behavior risk · Column B = structural debt / maintainability  
**Non-goals:** No code changes, no implementation plan, no redesign of schema 0.2

Related existing draft (narrower):  
`docs/superpowers/specs/2026-07-10-workflow-crate-responsibility-boundaries-design.md`  
covers only internal `moe-research-workflow` module boundaries. This audit is wider.

---

## 0. Executive summary

MoeResearch already has a **sound layered architecture**:

- acyclic crate graph
- thin MCP adapter
- domain-owned schemas (no contracts mega-crate)
- config isolated from runtime
- strong budget / policy / evidence-validation core
- serious integration-test contract net

The main architectural risk is **not** a broken dependency graph. It is:

1. **Cross-layer contract incompleteness** between Layer 1 assets and Rust runtime (packaging, partial semantics, doc/example drift).
2. **Density and duplication at seams** inside workflow, dual limit/error primitives, and CLI product surface.

**Bottom line:** treat the Rust core as a reliable execution engine; prioritize packaging/skill-contract fixes and workflow density reduction over graph redesign.

---

## 1. What is healthy (preserve)

1. **Acyclic layering**  
   `error → config/net → model/search → workflow → mcp → cli`.  
   Config does not depend on runtime crates. No production cycles.

2. **Thin MCP surface**  
   Only `aspect_research` and `deep_research`. Search is an internal model tool, not a top-level MCP tool.

3. **Domain-owned schemas**  
   Request/report DTOs in workflow; envelope in mcp; provider DTOs in model/search; TOML DTOs in config. Matches the historical “avoid contracts bucket” preference.

4. **Control-plane semantics**  
   - schema `0.2` with `deny_unknown_fields` on request/policy DTOs  
   - explicit single provider per aspect; allowlist ≠ fallback  
   - operator config ceiling ∩ request tightening; `Unlimited` does not override a finite peer  
   - byte-equal evidence provenance anti-tamper  
   - Layer 1 inlines `instructions`; Rust never does prompt file I/O

5. **Safety posture**  
   Public-safe errors, wire redaction in net, secrets out of envelopes, search content treated as untrusted.

6. **Composition-root discipline**  
   CLI alone maps config → services. Domain crates stay free of concrete host wiring.

7. **Test-as-contract strategy**  
   Central suite in `crates/moe-research-tests` (orchestrator / mcp / schema / search / budgets / partials). Behavior is trustworthy because of this net.

8. **PM path product completeness**  
   Capability routing, partial retry, host `HV-*` verification separated from MoeResearch evidence.

9. **Panic hygiene**  
   Production paths prefer `Result`; unwrap/expect density is low outside static invariants and tests.

---

## 2. Layer maps

### 2.1 Rust runtime

```text
Layer 1 (Claude Skill / Markdown assets)
        │  schema 0.2 DeepResearchRequest / AspectResearchRequest
        ▼
moe-research-mcp          ToolEnvelope + tool routing
        ▼
moe-research-workflow     normalize → policy/budget → agent loop → validate → aggregate
        ▼
moe-research-model / search
        ▼
moe-research-net          HTTP/SSE, retry, redaction, wire trace
        ▲
moe-research-cli          composition root (serve) + onboarding/assets product surface
moe-research-config       operator TOML + env validation
```

| Crate | Responsibility judgment | Size signal |
| --- | --- | --- |
| `moe-research-error` | Clean leaf | Small |
| `moe-research-config` | Clean, runtime-isolated | Medium |
| `moe-research-net` | Appropriate infrastructure | Large but justified (`reqwest_client`) |
| `moe-research-model` / `search` | Symmetric provider boundaries | Medium; Grok adapter heavy |
| **`moe-research-workflow`** | Correct core, internal density too high | **Primary complexity center** |
| `moe-research-mcp` | Thin adapter | Small |
| `moe-research-cli` | Composition root + product UX mixed | Large (`assets.rs` ~735) |
| `moe-research-tests` | Contract specification suite | ~11k LOC; high change cost |

### 2.2 Layer 1 assets

| Surface | Path | Intended ownership |
| --- | --- | --- |
| Unified entry | `skills/deep-research.md` | Routing, payload assembly, acceptance, final NL report |
| Profile skills | `skills/pm|academic|technical-*.md` | Domain methodology only |
| L1 prompts | `prompts/layer1/**` | Decomposition / allocation / final report / evidence postprocess |
| L2 prompts | `prompts/layer2/**` | Inlined as `AspectRequest.instructions` |
| Install path | `cli assets` + `scripts/package-research-skills-assets.mjs` | Version-pinned delivery |

**Intended split (correct):** Skill = orchestration + methodology; Rust = execution, budgets, schema, providers, immutable evidence.

**Actual tension:** release packaging, docs, skill examples, and Rust contracts are not fully isomorphic.

---

## 3. Severity legend

| Severity | Meaning |
| --- | --- |
| **P0** | User-visible break or high-confidence contract hole under current documented path |
| **P1** | Likely mis-operation, silent drift, or high blast-radius maintainability trap |
| **P2** | Real debt that slows evolution; not currently known-broken |
| **P3** | Hygiene / imbalance / confusion; low urgency |

Columns A and B are ranked **independently**. An item may appear in only one column.

---

## 4. Column A — Contract / behavior risks

### A-P0

#### A1. Generic profile path is broken on release installs

**Evidence**

- `skills/deep-research.md` routes Generic to:
  - `prompts/layer1/task-decomposition.md`
  - `prompts/layer1/final-report.md`
  - `prompts/layer2/aspect-agent.md`
- Install allowlists omit those roots:
  - `crates/moe-research-cli/src/commands/assets.rs` `ALLOWED_ASSET_PREFIXES` / `ALLOWED_ASSET_FILES`
  - `scripts/package-research-skills-assets.mjs` same policy
- Claude layout installs only specialized profile trees + common under `~/.claude/skills/deep-research/`.

**Failure scenario**  
User installs via `moeresearch assets install research-skills`, then runs generic multi-aspect research through the unified skill. Routing selects Generic; required prompt files are absent → fail-fast or improvised host-only behavior, contradicting skill policy.

**Why P0**  
Documented happy path for non-PM/academic/technical research does not ship.

---

### A-P1

#### A2. Partial-result contract under-specified outside PM

**Evidence**

- Runtime asymmetry in `crates/moe-research-mcp/src/tools.rs` + `crates/moe-research-workflow/src/workflow.rs`:
  - **deep partial:** `status=partial`, top-level `error=null`, failures in `data.failed_aspects`
  - **aspect partial:** `status=partial`, **both** `data` and `error` set
  - `allow_partial_results=false` suppresses partial payload into hard fail
- Skill guidance:
  - `skills/pm-deep-research.md`: strong (keep completed aspects, one `aspect_research` retry)
  - `skills/deep-research.md`: one-bullet handling
  - `skills/academic-deep-research.md` / `technical-evaluation.md`: almost none

**Failure scenario**  
Host treats deep partial as full success because `error` is null; or treats aspect partial as hard fail because `error` is present and discards usable `data`.

#### A3. Dual source of truth for asset packaging allowlists

**Evidence**  
Near-duplicate allowlists in:

- `crates/moe-research-cli/src/commands/assets.rs`
- `scripts/package-research-skills-assets.mjs`

**Failure scenario**  
Release tarball and CLI install validator diverge; package contains files install rejects, or install expects files package never ships (A1 is already an instance of this class).

#### A4. PM evidence modules duplicated against `common/`

**Evidence**  
Parallel methodology files:

- `prompts/layer1/common/{claim-ledger,evidence-postprocess,evidence-verifier,host-verification-backfill}.md`
- `prompts/layer1/pm-deep-research/` copies of the same roles

Academic/technical point at **common**; PM points at **PM copies**.

**Failure scenario**  
Verification / claim-ledger rules diverge by profile; same product claim audited differently depending on route.

#### A5. Product architecture doc drift

**Evidence** (`docs/research-agent-product.md` vs code):

- multi-model / Anthropic-compatible narrative vs CLI wiring only `"openai"` in `serve.rs`
- residual central-schema mental model vs removed contracts/schema crate
- prompt tree emphasis on generic `search-planner` / `evidence-extractor` vs shipped profile-heavy pack

Contract truth currently lives in `docs/mcp-usage.md` + workflow types, not fully in product doc.

**Failure scenario**  
Contributors implement against fiction (extra providers, wrong paths, wrong Layer 2 tool model).

#### A6. Limit skeleton / example drift across Layer 1 and docs

**Evidence**  
Different “standard” budgets in:

- `skills/deep-research.md` payload skeleton
- `skills/pm-deep-research.md` payload skeleton
- `docs/mcp-usage.md` examples

**Failure scenario**  
Not a wire bug by itself, but operators hit unexpected `budget_exceeded` after config tightening, with little envelope detail beyond the code. Amplifies A2.

---

### A-P2

#### A7. Result DTO unknown-field dropping

**Evidence**  
Request/policy DTOs use `deny_unknown_fields`; `crates/moe-research-workflow/src/report.rs` result structs generally do not.

**Failure scenario**  
Model emits extra/renamed fields; serde drops them; validator never sees the drift. Request-side is strict; response-side is soft. Detection cost moves into integration tests only.

#### A8. No enabled-provider discovery surface for Layer 1

**Evidence**  
Providers are config/env selected and string-matched in CLI composition; MCP exposes no “what is enabled?” tool.

**Failure scenario**  
Skill guesses provider names; gets `provider_unavailable` / invalid-input style failures instead of a discoverable allowlist.

#### A9. Config ceiling can silently dominate Layer 1 requested limits

**Evidence**  
Normalizer stricter-merge in `research.rs` / budget validation; `aspect_research` has no request-level `ResearchLimits` and inherits operator research caps.

**Failure scenario**  
Skill believes it set generous limits; operator TOML is tighter; run aborts with `budget_exceeded` without a Layer 1-visible “requested vs effective” projection in the envelope.

---

### A-P3

#### A10. Academic / technical skills are thin relative to PM

**Evidence**  
File depth and failure-handling coverage differ sharply across `skills/*.md`.

**Impact**  
Onboarding and operational quality uneven by profile; not a core runtime defect.

#### A11. Aspect partial path uses `expect` after guard

**Evidence**  
`tools.rs`: `failure.partial_output.take().expect("partial output")` after `is_some()` check.

**Impact**  
Currently safe; brittle style if refactor reorders checks.

---

## 5. Column B — Structural debt / code quality

### B-P0

None at pure structure level that independently equals A-P0 user breakage.  
The closest structural twin of A1 is **B2** (dual packaging truth), graded P1 because the break is already captured as contract risk A1.

---

### B-P1

#### B1. `agent_loop.rs` god-module density

**Evidence**  
`crates/moe-research-workflow/src/agent_loop.rs` ~817 lines; owns:

- turn state machine / replay input
- model dispatch
- search tool execution + evidence conversion
- deadlines + budget interaction
- output validation handoff
- partial failure packaging

Also marked `clippy::too_many_lines`.

**Why P1**  
Every partial/budget/timeout/tool bug concentrates here. Highest cognitive load in the system. Existing workflow-boundary draft correctly targets this.

#### B2. Asset packaging policy dual-owned (structure view of A3)

**Evidence**  
Rust CLI + Node packaging script both encode allowlists (~735 LOC assets command).

**Why P1**  
Two writers for one policy; install/package divergence is a process bug class, not a one-off content miss.

#### B3. Dual limit primitives across config and workflow

**Evidence**

- `crates/moe-research-config/src/limit.rs` — `ConfigLimit<T>`
- `crates/moe-research-workflow/src/limit.rs` — `Limit<T>`
- Same alias names (`CountLimit`, `DurationLimitMs`, `TokenLimit`) as **different types**
- CLI field mapping: `build_workflow_budget` / `map_limit` in `serve.rs`

Plus three vocabulary layers:

1. Config: `LimitsConfig` / `ResearchLimitsConfig` / `AgentLimitsConfig`
2. Workflow DTO: `BudgetConfig` / `ResearchLimits` / `AgentLimits`
3. Runtime: `AgentBudgetGuard` / `ResearchBudgetGuard`

**Why P1**  
Intentional isolation (config must not depend on workflow) is correct, but clone cost and name collision are ongoing footguns.

#### B4. `research.rs` mixes public schema, normalization, and prompt projection

**Evidence**  
~489 LOC combining:

- public request DTOs
- effective plans
- limit merge
- provider/tool validation
- `AspectPromptInput` projection

**Why P1**  
Schema evolution (#34 style) currently touches runtime-adjacent code; blast radius larger than necessary. Matches the existing workflow-boundary draft split (`request` / `plan` / `prompt`).

---

### B-P2

#### B5. CLI identity split: composition root vs product surface

**Evidence**  
`serve.rs` is the real DI root; CLI also owns init/check/onboard/mcp-register and a large assets installer. `assets.rs` alone exceeds `serve.rs` LOC.

**Why P2**  
Architecturally valid as a product binary, but dilutes “cli = composition root” mental model and raises review cost for unrelated serve changes.

#### B6. Dual `GrokReasoningEffort` + stringly provider registration

**Evidence**

- config enum + search provider enum + manual map in `serve.rs`
- hard-coded `"openai" | "exa" | "grok" | "tavily"` switches in composition root

**Why P2**  
Fine while provider set is tiny; becomes copy-paste tax on each new provider/host.

#### B7. Dual error-code enums at MCP edge

**Evidence**  
`moe_research_error::ErrorCode` and `mcp::ToolErrorCode` + manual mapping/`as_str` tables.

**Why P2**  
Transport isolation is reasonable; any new code needs two edits + tests. Currently aligned by contract tests.

#### B8. Integration-test monolith pressure

**Evidence**  
Approx sizes:

- `orchestrator_tests.rs` ~1866
- `search_tests.rs` ~1501
- `wire_trace_tests.rs` ~1149
- `mcp_tests.rs` ~1029  
  Total tests crate ~11k LOC, comparable to all production Rust.

**Why P2**  
Excellent safety net; expensive feedback and merge conflicts. Pure helpers (`stricter_limit`, provenance compare) are forced through public API fakes.

#### B9. `workflow.rs` scheduling + finalize density

**Evidence**  
~533 LOC combining public entrypoints, concurrency, fail_fast, partial finalize, evidence namespacing, budget post-checks.

**Why P2**  
Less severe than `agent_loop`, still a multi-concern module. Existing draft’s `workflow/{aspect,deep,aggregation}` split is directionally right.

#### B10. Search / model adapter weight imbalance

**Evidence**  
`search/provider/grok.rs` ~729; OpenAI model adapter ~540; Exa/Tavily smaller.

**Why P2**  
Not a layering violation; signals provider complexity lives in the right crate but may need internal submodules before the next Grok protocol change.

---

### B-P3

#### B11. `log_safe` name collision across net and workflow

Different purposes (wire redaction vs error scrubbing); same name confuses navigation.

#### B12. Historical Lapis vs MoeResearch naming residue

Code/crates/binary are `moe-research-*` / `moeresearch`. Some agent memory and root title still say Lapis / `lapis-*`. Cheap confusion source for agents and humans.

#### B13. Thin academic/technical skill maintainability imbalance

Structural twin of A10; profile engineering investment is PM-centric.

#### B14. Clippy `too_many_lines` / `struct_excessive_bools` allows concentrated in CLI and workflow hotspots

Symptom of B1/B5, not an independent defect.

---

## 6. Cross-impact matrix

| If you address… | Likely also touches… | Notes |
| --- | --- | --- |
| A1 generic packaging | A3/B2 allowlists, possibly A5 docs | Content + packaging policy together |
| A2 partial skill contract | A6 budgets, mcp-usage examples | Doc/skill only unless envelope is intentionally changed |
| A4 PM vs common modules | A10/B13 profile depth | Methodology consolidation |
| B1 agent_loop split | B9 workflow split, B8 tests | Prefer behavior-preserving moves; existing draft |
| B3 dual limits | B5/B6 composition mapping | Do **not** make config depend on workflow |
| B7 dual error codes | MCP schema tests | Generate or wrap; keep public strings stable |
| B8 test pressure | Any control-plane change | Add pure unit tests only where pure |

**Hard constraints from project memory / CLAUDE.md (still valid):**

- no broad `contracts` / `common` / `utils` crate
- config must not depend on workflow
- CLI-local mapping for config → runtime remains preferred
- single provider per search call; no silent fallback
- workflow regressions stay in `moe-research-tests`
- large refactors on a new branch, not directly on `main`

---

## 7. Highest-ROI disposition guidance (suggestions only)

This section is **not** an implementation plan. It is a suggested evaluation order for the owner.

### First wave — contract integrity (Column A)

1. **Ship or stop advertising Generic** (A1): either add generic L1/L2 roots to packaging allowlists, or change unified skill routing/docs so Generic is not a release path.  
2. **One packaging allowlist source** (A3/B2): eliminate dual writers.  
3. **Normalize partial-status guidance** across all skills + mcp-usage (A2).  
4. **Collapse or explicitly version PM vs common evidence modules** (A4).  
5. **Refresh product doc against runtime truth** (A5).

### Second wave — core maintainability (Column B)

6. **Decompose `agent_loop` / `research` / deep finalize** along already-visible seams (B1/B4/B9); the existing workflow-boundary draft is a ready candidate once prioritized.  
7. **Fence composition mapping** into an explicit CLI module without inventing a second framework (B3/B5/B6).  
8. **Decide limit-type strategy**: keep dual types with a single `From` mapping module, or a tiny pure leaf for limit wire types only (B3). Avoid config→workflow.  
9. **Generate or wrap MCP error codes** from one source of truth (B7).

### Explicitly deprioritize for now

- central schema/contracts crate
- multi-provider search aggregation/fallback
- expanding MCP tool count for search
- large test architecture rewrite without pure-function extraction first
- speculative multi-host library API unless a second host is a real goal

---

## 8. Open questions for the product owner

1. **Is Generic a supported product path on Claude install, or repo-only?** This decides A1 fix direction.  
2. **Is MCP+CLI the forever host, or is library-style embedding first-class?** Decides how hard to extract composition mapping.  
3. **Limit type strategy preference:** dual types forever, or a tiny shared pure leaf?  
4. **CLI identity:** primarily MCP server binary with helpers, or broader product CLI (assets/skills platform)?  
5. **Partial envelope product contract freeze:** keep deep/aspect asymmetry, or unify later as a breaking change?  
6. **Provider growth horizon:** OpenAI-only for long, or registration table soon?  
7. **Naming:** is “Lapis” only a local monorepo codename to drop from agent memory/docs?  
8. **Test budget:** keep integration suite as the primary specification, or reclaim pure-core unit tests?

---

## 9. Ranked shortlists (for scanning)

### Column A top five

1. A1 Generic path not shipped  
2. A2 Partial contract under-taught  
3. A3 Dual packaging allowlists  
4. A4 PM vs common evidence duplication  
5. A5 Product doc drift  

### Column B top five

1. B1 `agent_loop` density  
2. B2 Dual packaging ownership  
3. B3 Dual limit primitives  
4. B4 `research.rs` mixed responsibilities  
5. B5 CLI composition root vs product surface  

---

## 10. Relation to existing workflow-boundary draft

The draft dated 2026-07-10 proposes:

```text
research/{request,plan,prompt}
report/{validator}
workflow/{aspect,deep,aggregation}
runtime/{agent,model_turn,search_tool,budget,deadline}
```

That draft is a **strong candidate answer to B1/B4/B9 only**.  
It does **not** solve A1–A6 packaging/skill contracts, B2 dual packaging, B3 dual limits, or B5 CLI product surface.  
If adopted later, keep it behavior-preserving and branch-isolated per project policy.

---

## 11. Audit method notes

Sources used:

- workspace `Cargo.toml` and per-crate dependency edges
- crate `CLAUDE.md` + root architecture docs
- LOC hotspots across production and tests
- skill/prompt/package allowlist comparison
- partial envelope implementation in mcp tools + workflow finalize
- project memories: simple design, avoid contracts bucket, config-driven limits, CLI I/O boundary, workflow split preferences, test-crate regressions
- recent commit themes: control-plane simplification (#34), assets packaging, timeout/retry hardening, MCP contract clarifications

Confidence:

- **High** for dependency graph, packaging allowlist gap, partial envelope asymmetry, dual limit/error types, module size hotspots  
- **Medium** for product-doc staleness breadth (spot-checked, not full doc rewrite audit)  
- **Low/None** claimed as runtime bugs without failing tests; this audit is structural/contract-oriented

---

## 12. Closing judgment

| Question | Answer |
| --- | --- |
| Is the crate graph wrong? | **No.** Keep it. |
| Is MCP too fat? | **No.** Keep it thin. |
| Is workflow the complexity center? | **Yes.** Density, not misplacement. |
| Where do users get hurt first? | **Layer 1 packaging + incomplete skill contracts**, not agent-loop internals. |
| Where do maintainers get hurt first? | **`agent_loop` / dual limits / dual packaging / giant integration tests.** |
| Should you open a contracts crate? | **No.** |
| Should you make config depend on workflow? | **No.** |

**Recommended owner posture:** freeze core runtime semantics; fix A-column packaging/skill contract holes first; then take B-column workflow/composition density as a planned refactor branch.
