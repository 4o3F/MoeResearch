# Architecture Remediation Roadmap

> **For agentic workers:** Execute **one phase plan at a time**, in order, unless the owner explicitly parallelizes independent doc-only tasks inside Phase 1. Each phase has its own plan file under `docs/superpowers/plans/`.

**Goal:** Close all Column A (contract/behavior) and Column B (structural debt) findings from `docs/superpowers/specs/2026-07-10-architecture-audit-report.md` without redesigning schema 0.2 or the crate dependency graph.

**Architecture:** Four sequential phases. Phase 1 is Layer-1/docs/packaging contract repair. Phase 2 fences CLI composition and dual primitive mapping. Phase 3 is a behavior-preserving `moe-research-workflow` internal split (new git branch). Phase 4 hardens residual contract edges and test pressure. No phase introduces a `contracts` crate or `config → workflow` dependency.

**Tech Stack:** Rust 2024 workspace (`moe-research-*`), Claude Code skills/prompts Markdown, CLI assets packaging (Rust + optional Node pack script), `cargo test -p moe-research-tests`.

## Global Constraints

- Source of truth for findings: `docs/superpowers/specs/2026-07-10-architecture-audit-report.md`
- Workflow internal target layout (Phase 3): `docs/superpowers/specs/2026-07-10-workflow-crate-responsibility-boundaries-design.md`
- No broad `contracts` / `common` / `utils` crate
- Config must not depend on workflow
- CLI remains the only composition root; mapping stays CLI-local
- Single search provider per call; no silent multi-provider fallback
- Workflow regressions stay in `crates/moe-research-tests`
- Large structural refactors (Phase 3+) run on a **new git branch**, not directly on `main`
- Schema `0.2` public fields stay stable unless a phase explicitly marks a breaking change (none do in Phases 1–3)
- Prefer smallest fix that closes the finding (YAGNI / simple design)
- User-facing interaction language for plan docs may be Chinese summaries; plan task bodies stay English for implementers

## Locked decisions (audit open questions)

These are locked so plans are not blocked. Owner can override before a phase starts.

| # | Question | Locked decision |
| --- | --- | --- |
| 1 | Generic product path | **Ship Generic** on Claude/repo installs: include generic L1/L2 prompts in packaging |
| 2 | Second host | **MCP+CLI only** for now; extract `compose` module, no library host API |
| 3 | Limit types | **Keep dual types**; single CLI mapping module with explicit `From`/`map_limit`; no new leaf crate |
| 4 | CLI identity | **Product binary** with fenced modules (`compose`, `assets`, onboarding) |
| 5 | Partial envelope | **Freeze asymmetry**; document only in Phase 1 (no breaking envelope unify) |
| 6 | Provider growth | **Static registration table** in CLI compose; still OpenAI + Exa/Grok/Tavily |
| 7 | Lapis naming | **Drop `lapis-*` from project docs/memory pointers** during Phase 1 hygiene |
| 8 | Tests | Keep central integration suite; Phase 4 adds pure unit tests only for pure helpers |

## Phase index

| Phase | Plan file | Findings closed | Branch policy | Depends on |
| --- | --- | --- | --- | --- |
| **1 — Contract integrity** | `2026-07-10-phase1-contract-integrity.md` | A1, A2, A3, A4, A5, A6, A10, B2, B12 | may use short-lived branch or main-sized PR if owner prefers; still prefer branch | none |
| **2 — Composition seams** | `2026-07-10-phase2-composition-seams.md` | B3, B5, B6, B7, B11 (rename only if cheap) | new branch recommended | Phase 1 done or at least packaging truth stable |
| **3 — Workflow boundaries** | `2026-07-10-phase3-workflow-boundaries.md` | B1, B4, B9 | **required new branch** | Phase 2 preferred (cleaner imports); can start after Phase 1 if staffing forces |
| **4 — Hardening** | `2026-07-10-phase4-hardening.md` | A7, A8, A9, A11, B8, B10, B13, B14 residual | new branch | Phases 1–3 (A8/A9 may be cut if owner rejects scope) |

## Definition of done per phase

Each phase is done only when:

1. Every finding listed for that phase is closed or explicitly deferred in the phase plan’s “Out of scope” with owner reason.
2. `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` pass (Phase 1 may be docs/assets-heavy; still run tests if Rust touched).
3. Docs/skills/packaging stay isomorphic for any path the phase claims to support.
4. Phase plan checkboxes are completed and a short phase retrospective note is appended to the plan file or a follow-up commit message.

## Explicitly out of roadmap

- Central schema/contracts crate
- Config depending on workflow
- Multi-provider search aggregation/fallback
- Exposing search as MCP tool
- Unifying deep vs aspect partial envelope shapes (breaking)
- Full test-suite rewrite
- New model providers beyond wiring hygiene

## Execution order for humans/agents

```text
Phase 1  →  Phase 2  →  Phase 3  →  Phase 4
   │            │            │
   └─ may fix docs-only tasks in parallel inside the phase
```

Do not start Phase 3 file moves until Phase 1 packaging/skill contracts are stable enough that integration tests are green on `main` or the integration branch.

---

## Coverage matrix (finding → phase)

| ID | Summary | Phase |
| --- | --- | --- |
| A1 | Generic path not shipped | 1 |
| A2 | Partial contract under-taught | 1 |
| A3 | Dual packaging allowlists | 1 |
| A4 | PM vs common evidence dup | 1 |
| A5 | Product doc drift | 1 |
| A6 | Limit skeleton drift | 1 |
| A7 | Result DTO unknown-field drop | 4 |
| A8 | No enabled-provider discovery | 4 |
| A9 | Config ceiling opacity | 4 |
| A10 | Thin academic/technical skills | 1 |
| A11 | MCP partial `expect` | 4 |
| B1 | `agent_loop` density | 3 |
| B2 | Dual packaging ownership | 1 |
| B3 | Dual limit primitives | 2 |
| B4 | `research.rs` mixed roles | 3 |
| B5 | CLI root vs product surface | 2 |
| B6 | Dual Grok effort + stringly providers | 2 |
| B7 | Dual ErrorCode enums | 2 |
| B8 | Integration test monolith pressure | 4 |
| B9 | `workflow.rs` density | 3 |
| B10 | Grok adapter weight | 4 |
| B11 | `log_safe` name collision | 2 or 4 |
| B12 | Lapis naming residue | 1 |
| B13 | Profile investment imbalance | 1 (partial) / 4 (depth) |
| B14 | Clippy allows as symptom | 3–4 (as modules shrink) |
