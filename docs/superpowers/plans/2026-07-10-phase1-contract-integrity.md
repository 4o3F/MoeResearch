# Phase 1 — Contract Integrity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close Phase 1 contract findings (A1, A2, A3, A4, A5, A6, A10, B2, B12) by shipping Generic prompts, enforcing one packaging allowlist truth, documenting frozen partial-envelope asymmetry, consolidating PM evidence modules onto `common/`, canonicalizing limit tiers, fixing product-doc drift, and removing in-repo `lapis-*` residue — without changing schema 0.2 wire shapes.

**Architecture:** Layer-1/docs/packaging contract repair only. Rust runtime semantics for partial envelopes stay frozen (document asymmetry; no unify). Packaging remains Node packer + Rust install validator, kept isomorphic by a checked-in expected allowlist asserted in `cli_assets_tests`. Skills and docs become the operational contract surface for Generic, budgets, and partial/failed handling. No new crates, no `config → workflow` dependency, no envelope breaking change.

**Tech Stack:** Rust 2024 workspace (`moe-research-cli`, `moe-research-tests`), Node packaging script (`scripts/package-research-skills-assets.mjs`), Markdown skills/prompts under `skills/` and `prompts/`, docs under `docs/`.

## Global Constraints

- Source of truth for findings: `docs/superpowers/specs/2026-07-10-architecture-audit-report.md`
- Workflow internal target layout (Phase 3 only): `docs/superpowers/specs/2026-07-10-workflow-crate-responsibility-boundaries-design.md`
- No broad `contracts` / `common` / `utils` crate
- Config must not depend on workflow
- CLI remains the only composition root; mapping stays CLI-local
- Single search provider per call; no silent multi-provider fallback
- Workflow regressions stay in `crates/moe-research-tests`
- Large structural refactors (Phase 3+) run on a **new git branch**, not directly on `main`
- Schema `0.2` public fields stay stable unless a phase explicitly marks a breaking change (none do in Phases 1–3)
- Prefer smallest fix that closes the finding (YAGNI / simple design)
- User-facing interaction language for plan docs may be Chinese summaries; plan task bodies stay English for implementers
- **Phase 1 locked decisions:** Ship Generic; freeze partial envelope asymmetry (document only); single packaging truth via test-enforced isomorphism; PM→`common/` consolidation; fix product-doc false claims; one canonical budget table; academic/technical partial handling parity; in-repo `lapis-*` hygiene

## Out of scope (do not implement in this plan)

- Workflow file splits (`agent_loop`, `research.rs`, `workflow.rs`) — Phase 3
- Dual `Limit` / `ConfigLimit` merge — Phase 2
- `ErrorCode` / `ToolErrorCode` merge — Phase 2
- Provider discovery MCP tool — Phase 4
- Result DTO `deny_unknown_fields` — Phase 4
- Unifying deep vs aspect partial envelope shapes (breaking)
- Full academic/technical methodology rewrite to PM depth
- Editing agent memory outside the repo (only note it; optional local follow-up)

---

## File map

| Path | Action | Responsibility |
| --- | --- | --- |
| `crates/moe-research-cli/src/commands/assets.rs` | Modify | Add Generic root prompt files to `ALLOWED_ASSET_FILES` (and dirs only if archive emits them) |
| `scripts/package-research-skills-assets.mjs` | Modify | Ship same Generic files via `ROOTS` + `ALLOWED_FILES` |
| `crates/moe-research-tests/tests/cli_assets_tests.rs` | Modify | Expected Generic paths, isomorphic allowlist test, Claude layout assertions |
| `skills/deep-research.md` | Modify | Partial contract section; limit-tier pointer; Generic path verification notes |
| `skills/pm-deep-research.md` | Modify | Point evidence modules at `common/`; partial section alignment; limit-tier pointer + skeleton |
| `skills/academic-deep-research.md` | Modify | Add partial/failed handling section |
| `skills/technical-evaluation.md` | Modify | Add partial/failed handling section |
| `docs/research-skills.md` | Modify | Install tree includes Generic roots; profiles table |
| `docs/mcp-usage.md` | Modify | Canonical budget tier table; expanded partial-status contract |
| `docs/research-agent-product.md` | Modify | Fix multi-model Anthropic / `schema/*` / prompt-tree claims to runtime truth |
| `docs/pm-deep-research.md` | Modify only if it still points PM evidence modules at PM copies or omits Generic | Keep install trees accurate |
| `prompts/layer1/common/{claim-ledger,evidence-postprocess,evidence-verifier,host-verification-backfill}.md` | Modify | Absorb unique PM product-decision content still required after consolidation |
| `prompts/layer1/pm-deep-research/{claim-ledger,evidence-postprocess,evidence-verifier,host-verification-backfill}.md` | Delete | After skill paths + common merge |
| `CLAUDE.md` / other in-repo docs with incorrect `lapis-*` crate/binary names | Modify | Naming hygiene (B12); keep “Lapis” as monorepo codename only if intentional |
| `docs/superpowers/plans/2026-07-10-phase1-contract-integrity.md` | Create (this file) | Execution checklist |

### Generic assets that MUST ship (A1)

Exact allowlist **file** entries to add (not prefixes — these are root files, and Rust validation requires exact file match or a child under a prefix):

```text
prompts/layer1/task-decomposition.md
prompts/layer1/final-report.md
prompts/layer2/aspect-agent.md
prompts/layer2/search-planner.md
prompts/layer2/evidence-extractor.md
```

**Decision:** Ship all five. Generic runtime path requires the first three (`skills/deep-research.md` Generic row). Product doc historically advertises `search-planner.md` and `evidence-extractor.md`; they exist in-repo and remain optional Layer-2 helper prompts (not required as separate inlined instructions when `aspect-agent.md` already covers the loop). Include them in packaging for repo completeness and doc honesty.

Do **not** add overly broad prefixes such as `prompts/layer1/` or `prompts/layer2/` (would admit arbitrary future files without review).

### Canonical budget tiers (A6) — locked numbers

Authoritative table lives in `docs/mcp-usage.md` §5.3 (or a new subsection immediately after). Skills reference that section; they do not invent competing “standard” numbers.

| Tier | Research `limits` | Per-aspect `limits` | Intended use |
| --- | --- | --- | --- |
| `quick` | `max_agents=2`, `max_concurrent_agents=1`, `max_total_model_calls=12`, `max_total_search_calls=8`, `total_timeout_ms=300000`, `max_tokens=-1` | `max_turns=4`, `max_tool_calls=4`, `max_search_calls=2`, `timeout_ms=180000` | Smoke / narrow question |
| `standard` | `max_agents=4`, `max_concurrent_agents=2`, `max_total_model_calls=32`, `max_total_search_calls=20`, `total_timeout_ms=600000`, `max_tokens=-1` | `max_turns=8`, `max_tool_calls=12`, `max_search_calls=6`, `timeout_ms=600000` | Default Generic / deep-research skeleton |
| `deep` | `max_agents=6`, `max_concurrent_agents=3`, `max_total_model_calls=70`, `max_total_search_calls=56`, `total_timeout_ms=1260000`, `max_tokens=-1` | `max_turns=8`, `max_tool_calls=8`, `max_search_calls=4`, `timeout_ms=600000` | Default PM deep skeleton |

Notes for implementers:

- Operator TOML ceilings still win on stricter-merge; skills must say so next to the table.
- `aspect_research` has no top-level research `limits`; only aspect `task.limits` apply (inherit research caps from operator config only).
- Skeletons in skills must match a named tier, not free-hand numbers.

### Frozen partial envelope contract (A2) — document only

Do **not** change Rust. Document this exact product contract everywhere Layer 1 acts on envelopes:

| Tool | `status` | `data` | `error` | Host action |
| --- | --- | --- | --- | --- |
| `deep_research` partial | `partial` | present; `failed_aspects` + possibly evidence from failures | **`null`** | Keep completed aspects; treat failures as gaps; **one** targeted `aspect_research` retry per failed aspect |
| `aspect_research` partial | `partial` | present (frozen evidence; typically empty findings) | **present** (`ToolError`) | Usable evidence is in `data`; do not discard because `error` is set; at most one retry if retryable |
| either with `allow_partial_results=false` | `failed` | null / no partial payload | present | Hard fail; no partial report path |
| hard fail (`provider_unavailable`, process down, etc.) | `failed` | null | present | Surface code + retryable; stop; no host-only MoeResearch substitute |

Claude Code: read stable payload from `result.structuredContent`.

### PM vs common merge rule (A4)

| Module | Current | Action |
| --- | --- | --- |
| `claim-ledger.md` | DIFFER (PM richer) | Merge PM decision-intent / PR-FAQ / product-decision wording into `common/claim-ledger.md` as optional product-decision guidance without breaking academic/technical neutrality; delete PM copy |
| `evidence-postprocess.md` | DIFFER | Same merge pattern; delete PM copy |
| `evidence-verifier.md` | DIFFER | Same merge pattern; delete PM copy |
| `host-verification-backfill.md` | DIFFER | Same merge pattern; delete PM copy |
| `report-annex.md` | common only | Keep; PM skill may reference if needed |
| PM-only methodology (`decision-closure.md`, `chinese-product-report-structure.md`, task/final-report variants) | PM tree | **Keep** under `prompts/layer1/pm-deep-research/` |

After merge, every skill path for the four evidence modules must be `../prompts/layer1/common/<name>.md`.

---

### Task 1: Packaging allowlist single truth + ship Generic prompts

**Files:**
- Modify: `crates/moe-research-cli/src/commands/assets.rs` (`ALLOWED_ASSET_FILES`, possibly `ALLOWED_ASSET_DIRS` only if tests show dir entries needed)
- Modify: `scripts/package-research-skills-assets.mjs` (`ROOTS`, `ALLOWED_FILES`)
- Test: `crates/moe-research-tests/tests/cli_assets_tests.rs`

**Interfaces:**
- Consumes: existing `validate_owned_asset_file_path` exact-file + prefix-child rules
- Produces: isomorphic allowlist policy that includes Generic root prompt files; package script ships them; install accepts them

- [ ] **Step 1: Write failing tests first (TDD)**

In `crates/moe-research-tests/tests/cli_assets_tests.rs`:

1. Extend constants:

```rust
const GENERIC_LAYER1_TASK: &str = "prompts/layer1/task-decomposition.md";
const GENERIC_LAYER1_FINAL: &str = "prompts/layer1/final-report.md";
const GENERIC_LAYER2_ASPECT: &str = "prompts/layer2/aspect-agent.md";
const GENERIC_LAYER2_SEARCH_PLANNER: &str = "prompts/layer2/search-planner.md";
const GENERIC_LAYER2_EVIDENCE_EXTRACTOR: &str = "prompts/layer2/evidence-extractor.md";
```

2. Add all five to `EXPECTED_FILES`.

3. Update `is_research_asset_path` to accept the five exact paths (keep existing prefix rules).

4. In `assets_install_default_claude_code_layout_rewrites_skill_prompt_paths`, assert installed files exist:

```rust
assert!(skill_root.join("prompts/layer1/task-decomposition.md").is_file());
assert!(skill_root.join("prompts/layer1/final-report.md").is_file());
assert!(skill_root.join("prompts/layer2/aspect-agent.md").is_file());
assert!(skill_root.join("prompts/layer2/search-planner.md").is_file());
assert!(skill_root.join("prompts/layer2/evidence-extractor.md").is_file());
```

5. Add new test `packaging_allowlists_are_isomorphic` that:

- Reads `crates/moe-research-cli/src/commands/assets.rs` and `scripts/package-research-skills-assets.mjs` as UTF-8 text from `workspace()`.
- Extracts the string-literal members of Rust `ALLOWED_ASSET_FILES` and `ALLOWED_ASSET_PREFIXES` (simple line scan for quoted paths inside those const blocks is enough; do not invent a shared parser crate).
- Extracts Node `ALLOWED_FILES` / `ALLOWED_PREFIXES` the same way.
- Asserts both file-sets equal and both prefix-sets equal.
- Asserts the five Generic file paths are members of the shared file-set.

6. Optionally strengthen `package_script_manifest_covers_expanded_research_roots` by asserting the five Generic paths appear in the generated manifest (already covered if they are in `EXPECTED_FILES` and real package run collects them — for the live package test, files must exist on disk under repo root, which they do).

- [ ] **Step 2: Run tests and confirm failure**

```bash
cargo test -p moe-research-tests --test cli_assets_tests packaging_allowlists_are_isomorphic -- --nocapture
cargo test -p moe-research-tests --test cli_assets_tests package_script_manifest_covers_expanded_research_roots -- --nocapture
```

Expected: FAIL — Generic paths missing from allowlists / package roots / `is_research_asset_path`, or isomorphism mismatch after expected-set expansion.

- [ ] **Step 3: Update Rust allowlist**

In `crates/moe-research-cli/src/commands/assets.rs`, change `ALLOWED_ASSET_FILES` to:

```rust
const ALLOWED_ASSET_FILES: &[&str] = &[
    "skills/deep-research.md",
    "skills/pm-deep-research.md",
    "skills/academic-deep-research.md",
    "skills/technical-evaluation.md",
    "prompts/layer1/task-decomposition.md",
    "prompts/layer1/final-report.md",
    "prompts/layer2/aspect-agent.md",
    "prompts/layer2/search-planner.md",
    "prompts/layer2/evidence-extractor.md",
];
```

Leave `ALLOWED_ASSET_PREFIXES` unchanged (profile trees + common). Do **not** add broad `prompts/layer1/` or `prompts/layer2/` prefixes.

Only if archive extraction tests start requiring directory entries for parent paths, add the minimal dirs:

```text
prompts/layer1
prompts/layer2
```

to `ALLOWED_ASSET_DIRS` (they already exist). No other dir changes.

- [ ] **Step 4: Update Node packer**

In `scripts/package-research-skills-assets.mjs`:

1. Extend `ROOTS` with the five Generic files (files, not directories):

```js
const ROOTS = [
  'skills/deep-research.md',
  'skills/pm-deep-research.md',
  'skills/academic-deep-research.md',
  'skills/technical-evaluation.md',
  'prompts/layer1/task-decomposition.md',
  'prompts/layer1/final-report.md',
  'prompts/layer2/aspect-agent.md',
  'prompts/layer2/search-planner.md',
  'prompts/layer2/evidence-extractor.md',
  'prompts/layer1/common',
  'prompts/layer1/pm-deep-research',
  'prompts/layer2/pm-deep-research',
  'prompts/layer1/academic-deep-research',
  'prompts/layer2/academic-deep-research',
  'prompts/layer1/technical-evaluation',
  'prompts/layer2/technical-evaluation',
];
```

2. Extend `ALLOWED_FILES` with the same five prompt paths (mirrors Rust exact files).

3. Keep `ALLOWED_PREFIXES` identical to Rust prefixes.

- [ ] **Step 5: Run tests and confirm pass**

```bash
cargo test -p moe-research-tests --test cli_assets_tests -- --nocapture
```

Expected: PASS for allowlist isomorphism, package script, repo layout, Claude layout (including Generic files).

Manual sanity (optional but recommended once):

```bash
node scripts/package-research-skills-assets.mjs --version 0.0.0-phase1 --output-dir /tmp/moeresearch-assets-phase1
python3 - <<'PY'
import json
m=json.load(open('/tmp/moeresearch-assets-phase1/research-skills-assets-v0.0.0-phase1.manifest.json'))
need=[
 'prompts/layer1/task-decomposition.md',
 'prompts/layer1/final-report.md',
 'prompts/layer2/aspect-agent.md',
 'prompts/layer2/search-planner.md',
 'prompts/layer2/evidence-extractor.md',
]
paths={f['path'] for f in m['files']}
missing=[p for p in need if p not in paths]
assert not missing, missing
print('ok', len(paths), 'files')
PY
```

- [ ] **Step 6: Commit**

```bash
git add \
  crates/moe-research-cli/src/commands/assets.rs \
  scripts/package-research-skills-assets.mjs \
  crates/moe-research-tests/tests/cli_assets_tests.rs
git commit -m "$(cat <<'EOF'
fix(assets): ship Generic L1/L2 prompts and enforce allowlist isomorphism

Close A1/A3/B2 for packaging: include generic root prompts in CLI and
Node allowlists, and lock both lists together via cli_assets_tests.
EOF
)"
```

---

### Task 2: Skill Generic path verification + research-skills install tree

**Files:**
- Modify: `docs/research-skills.md`
- Modify: `skills/deep-research.md` (routing/assets notes only; partial/limits may wait for Tasks 3–5 if already planned — keep Generic install notes here)
- Modify: `docs/pm-deep-research.md` only if install tree omits Generic / still wrong

**Interfaces:**
- Consumes: Task 1 shipped paths
- Produces: Docs/skills that describe the real Claude + repo install trees including Generic roots

- [ ] **Step 1: Update `docs/research-skills.md` profiles + trees**

1. In **Installed Skill Profiles**, add Generic row:

```markdown
| Generic DeepResearch | Multi-aspect research that does not fit PM/academic/technical. | `prompts/layer1/task-decomposition.md`, `prompts/layer1/final-report.md`, `prompts/layer2/aspect-agent.md` |
```

2. Mention optional helpers:

```markdown
Optional Generic Layer-2 helpers (not required when `aspect-agent.md` is inlined): `prompts/layer2/search-planner.md`, `prompts/layer2/evidence-extractor.md`.
```

3. Expand **Claude Code Layout** tree to:

```text
~/.claude/skills/deep-research/
  SKILL.md
  prompts/
    layer1/
      task-decomposition.md
      final-report.md
      common/
      pm-deep-research/
      academic-deep-research/
      technical-evaluation/
    layer2/
      aspect-agent.md
      search-planner.md
      evidence-extractor.md
      pm-deep-research/
      academic-deep-research/
      technical-evaluation/
```

4. Under **Choosing a Profile**, keep “Other multi-aspect research → generic deep-research” and add: after `moeresearch assets install research-skills`, Generic prompts must be present; if missing, reinstall matching binary version.

- [ ] **Step 2: Update `skills/deep-research.md` Generic verification note**

Near the Generic routing row / after routing plan emission, add an explicit fail-fast check:

```markdown
Before Generic execution, verify these assets resolve from the skill workspace:
- `../prompts/layer1/task-decomposition.md`
- `../prompts/layer1/final-report.md`
- `../prompts/layer2/aspect-agent.md`

If any are missing, stop and instruct the user to run
`moeresearch assets install research-skills` for this `moeresearch` version.
Do not improvise Generic orchestration without those files.
```

Keep Claude layout rewrite behavior in mind: installed skill uses `./prompts/...` after path rewrite; repo layout keeps `../prompts/...`.

- [ ] **Step 3: Grep docs for stale install trees**

```bash
rg -n "Claude Code Layout|layer1/pm-deep-research|generic|task-decomposition.md" docs/research-skills.md docs/pm-deep-research.md README.md
```

Fix any install tree that still claims only profile subtrees without Generic roots.

- [ ] **Step 4: Commit**

```bash
git add docs/research-skills.md skills/deep-research.md docs/pm-deep-research.md
git commit -m "$(cat <<'EOF'
docs(skills): document Generic install tree and fail-fast asset checks

Align research-skills install docs with shipped Generic L1/L2 prompts.
EOF
)"
```

---

### Task 3: Partial-status contract across skills + mcp-usage

**Files:**
- Modify: `docs/mcp-usage.md` §8 (and cross-links in §10 / validation notes)
- Modify: `skills/deep-research.md` **Failure handling**
- Modify: `skills/pm-deep-research.md` **Failure handling** + workflow bullet that already mentions partial
- Modify: `skills/academic-deep-research.md` (add section; full A10 depth in Task 7 if preferred, but put the shared partial block here)
- Modify: `skills/technical-evaluation.md` (same)

**Interfaces:**
- Consumes: Frozen asymmetry table in this plan; existing MCP tests as behavioral truth
- Produces: Identical host semantics text across four skills + mcp-usage

- [ ] **Step 1: Expand `docs/mcp-usage.md` §8**

After the existing short `status=partial` paragraph, insert a subsection **Partial-status host contract (frozen)**:

```markdown
### Partial-status host contract (frozen)

Schema 0.2 intentionally keeps deep vs aspect partial asymmetry. Do not “normalize” envelopes in the client by dropping fields.

| Case | `status` | `data` | `error` | Host must |
| --- | --- | --- | --- | --- |
| `deep_research` partial | `partial` | present (`aspects`, `failed_aspects`, `evidence_index`, …) | **null** | Keep completed aspects; treat `failed_aspects[]` as gaps; at most one `aspect_research` retry per failed aspect |
| `aspect_research` partial | `partial` | present (frozen evidence; findings usually empty) | **present** | Use `data`; do not treat as pure hard-fail discard; inspect `error.retryable` / code for one retry |
| Partials disabled (`allow_partial_results=false`) | `failed` | null / no partial payload | present | Stop; no partial report path |
| Hard transport/config failure | `failed` | null | present | Surface stable code; no host-only substitute for MoeResearch execution |

Claude Code direct tools: read `result.structuredContent`.
Raw MCP: unwrap `tools/call` result per §4.2, then the same envelope.
```

Keep existing wording about `run_id` for deep partials.

- [ ] **Step 2: Replace thin Failure handling in `skills/deep-research.md`**

Replace the three bullets under **Failure handling** with the shared contract (English, skill voice):

```markdown
## Failure handling

Envelope semantics (frozen; do not reinterpret):

1. **Hard fail** (`status=failed`, no usable partial payload; codes such as `provider_unavailable`, `network_failed`, process/tool missing): surface the stable error code, `retryable`, and the smallest safe next action. Stop. There is no host-only fallback for MoeResearch execution.
2. **`deep_research` partial** (`status=partial`, `data` present, **`error` is null**): this is not full success. Keep completed aspects from `data`. Treat each entry in `data.failed_aspects` as a gap. For each failed aspect, you may run **one** targeted `aspect_research` retry with the same aspect plan and inline instructions. Then write a partial report that lists remaining gaps.
3. **`aspect_research` partial** (`status=partial`, **`data` and `error` both set**): keep frozen evidence in `data`. Do not discard `data` because `error` is present. Retry at most once if `error.retryable` is true and the failure is not a Layer-1 schema/prompt bug (`schema_validation_failed` → fix request/prompt, do not blind-retry).
4. **`allow_partial_results=false`**: expect hard `failed` with no partial payload; do not invent a partial report.
5. **Insufficient evidence after success/partial**: do not invent conclusions; emit gap list and follow-up searches.

Read Claude Code tool results from `result.structuredContent`. See `docs/mcp-usage.md` §8 for the full envelope table.
```

- [ ] **Step 3: Align PM skill**

Ensure `skills/pm-deep-research.md` workflow bullet 6 and **Failure handling** use the same deep/aspect asymmetry (PM already has strong retry language — extend it to mention aspect partial `error` co-presence and `structuredContent`). Do not weaken PM’s one-retry rule.

- [ ] **Step 4: Add the same Failure handling section to academic + technical**

Copy the deep-research Failure handling block into:

- `skills/academic-deep-research.md` (new section before or after Policy boundaries)
- `skills/technical-evaluation.md` (same)

Tailor only the intro line (“Academic profile…” / “Technical profile…”); keep the five rules identical.

- [ ] **Step 5: Grep for conflicting partial guidance**

```bash
rg -n "status=partial|failed_aspects|allow_partial_results|Failure handling" \
  skills docs/mcp-usage.md
```

Fix contradictions (e.g. “partial means success”, “discard when error set”).

- [ ] **Step 6: Commit**

```bash
git add docs/mcp-usage.md \
  skills/deep-research.md \
  skills/pm-deep-research.md \
  skills/academic-deep-research.md \
  skills/technical-evaluation.md
git commit -m "$(cat <<'EOF'
docs(skills): freeze and publish partial envelope host contract

Document deep vs aspect partial asymmetry across mcp-usage and all
research skills without changing schema 0.2 wire shapes.
EOF
)"
```

---

### Task 4: PM → common evidence module consolidation

**Files:**
- Modify: `prompts/layer1/common/claim-ledger.md`
- Modify: `prompts/layer1/common/evidence-postprocess.md`
- Modify: `prompts/layer1/common/evidence-verifier.md`
- Modify: `prompts/layer1/common/host-verification-backfill.md`
- Modify: `skills/pm-deep-research.md` (all paths that currently point at PM copies)
- Modify: `docs/pm-deep-research.md` and `docs/research-skills.md` if they list PM copies as evidence homes
- Delete:  
  - `prompts/layer1/pm-deep-research/claim-ledger.md`  
  - `prompts/layer1/pm-deep-research/evidence-postprocess.md`  
  - `prompts/layer1/pm-deep-research/evidence-verifier.md`  
  - `prompts/layer1/pm-deep-research/host-verification-backfill.md`

**Interfaces:**
- Consumes: Existing common modules used by academic/technical; richer PM copies
- Produces: Single evidence-module source under `prompts/layer1/common/`; PM skill path updates; no contracts crate

- [ ] **Step 1: Diff and merge (do this before deleting)**

```bash
for f in claim-ledger evidence-postprocess evidence-verifier host-verification-backfill; do
  diff -u "prompts/layer1/common/$f.md" "prompts/layer1/pm-deep-research/$f.md" | less
done
```

Merge rules:

1. Prefer **common** as the canonical file path.
2. Port any PM-only operational requirements that academic/technical also need (load-bearing claim rules, HV-* separation, CiteEval sampling, source tiers).
3. Keep product-decision / PR-FAQ examples as clearly labeled optional subsections (“When used from PM DeepResearch…”) so academic/technical remain usable.
4. Do not leave “see PM copy” pointers.
5. Keep titles domain-neutral (`Layer 1 Common Module: …`).

- [ ] **Step 2: Update PM skill paths**

In `skills/pm-deep-research.md`, replace every:

```text
../prompts/layer1/pm-deep-research/evidence-postprocess.md
../prompts/layer1/pm-deep-research/claim-ledger.md
../prompts/layer1/pm-deep-research/host-verification-backfill.md
../prompts/layer1/pm-deep-research/evidence-verifier.md
```

with:

```text
../prompts/layer1/common/evidence-postprocess.md
../prompts/layer1/common/claim-ledger.md
../prompts/layer1/common/host-verification-backfill.md
../prompts/layer1/common/evidence-verifier.md
```

Also update **Assets** and **Product-requirements module order** tables so module names resolve under `common/`. Leave PM-only files (`decision-closure.md`, `chinese-product-report-structure.md`, task/final-report variants) on the PM path.

- [ ] **Step 3: Update docs references**

```bash
rg -n "pm-deep-research/(claim-ledger|evidence-postprocess|evidence-verifier|host-verification-backfill)" \
  skills docs prompts
```

Zero hits remaining after edits (except historical audit report under `docs/superpowers/specs/`, which may stay as audit evidence).

- [ ] **Step 4: Delete PM duplicates**

```bash
git rm \
  prompts/layer1/pm-deep-research/claim-ledger.md \
  prompts/layer1/pm-deep-research/evidence-postprocess.md \
  prompts/layer1/pm-deep-research/evidence-verifier.md \
  prompts/layer1/pm-deep-research/host-verification-backfill.md
```

- [ ] **Step 5: Package + assets tests still pass**

```bash
cargo test -p moe-research-tests --test cli_assets_tests -- --nocapture
node scripts/package-research-skills-assets.mjs --version 0.0.0-phase1-a4 --output-dir /tmp/moeresearch-assets-a4
```

Expected: PASS; package no longer lists the four deleted paths; common files still present.

- [ ] **Step 6: Commit**

```bash
git add prompts/layer1/common skills/pm-deep-research.md docs/pm-deep-research.md docs/research-skills.md
git commit -m "$(cat <<'EOF'
refactor(prompts): consolidate PM evidence modules onto common/

Merge unique PM verification guidance into prompts/layer1/common and
point the PM skill at the shared modules; remove duplicate PM copies.
EOF
)"
```

---

### Task 5: Limit skeleton canonicalization

**Files:**
- Modify: `docs/mcp-usage.md` (§5.3 ResearchLimits / AgentLimits — add tier table; align examples)
- Modify: `skills/deep-research.md` (skeleton → `standard` tier; pointer)
- Modify: `skills/pm-deep-research.md` (skeleton → `deep` tier; pointer)
- Optionally one-line pointers in academic/technical if they gain skeletons later (not required if they have no numeric skeleton)

**Interfaces:**
- Consumes: Locked tier table in this plan
- Produces: One canonical table; skill skeletons match named tiers

- [ ] **Step 1: Insert canonical tier table in `docs/mcp-usage.md`**

Immediately after the ResearchLimits / AgentLimits JSON examples in §5.3, add:

```markdown
### Recommended Layer-1 budget tiers

Layer 1 should pick a named tier (`quick` | `standard` | `deep`) and copy these numbers unless the user overrides. Operator config still applies stricter-merge: a finite operator ceiling wins over a more generous request; `-1` does not override a finite peer.

| Tier | `limits` (deep_research) | Per-aspect `task.aspects[].limits` / `task.limits` |
| --- | --- | --- |
| `quick` | agents 2, concurrent 1, model calls 12, search calls 8, total_timeout_ms 300000, max_tokens -1 | turns 4, tool_calls 4, search_calls 2, timeout_ms 180000 |
| `standard` | agents 4, concurrent 2, model calls 32, search calls 20, total_timeout_ms 600000, max_tokens -1 | turns 8, tool_calls 12, search_calls 6, timeout_ms 600000 |
| `deep` | agents 6, concurrent 3, model calls 70, search calls 56, total_timeout_ms 1260000, max_tokens -1 | turns 8, tool_calls 8, search_calls 4, timeout_ms 600000 |

Skill payload skeletons must cite this table. Do not invent a second “standard” budget in profile skills.
```

Align the §7 `deep_research` example object to **`standard`** numbers (today it is close: agents 5 → change to 4, model 30 → 32 for isomorphism, or keep §7 as a minimal illustration and mark it “illustrative, not a tier” — prefer **making §7 = standard** to avoid A6 recurrence).

- [ ] **Step 2: Align `skills/deep-research.md` skeleton to `standard`**

Current skeleton already matches standard (agents 4 / concurrent 2 / model 32 / search 20 / total 600000; aspect 8/12/6/600000). Add an explicit label above the JSON:

```markdown
Default skeleton = **standard** tier from `docs/mcp-usage.md` §5.3.
For `quick` or `deep`, substitute that tier’s numbers instead of editing ad hoc.
```

- [ ] **Step 3: Align `skills/pm-deep-research.md` skeleton to `deep`**

Current PM skeleton already matches deep (agents 6 / concurrent 3 / model 70 / search 56 / total 1260000; aspect 8/8/4/600000). Label it:

```markdown
Default PM skeleton = **deep** tier from `docs/mcp-usage.md` §5.3.
Use `standard` or `quick` only when the user explicitly wants a cheaper run.
```

Ensure aspect retry skeleton uses the same per-aspect deep numbers.

- [ ] **Step 4: Cross-check no third budget set remains**

```bash
rg -n "max_agents|max_total_model_calls|max_tool_calls|total_timeout_ms" \
  skills/deep-research.md skills/pm-deep-research.md docs/mcp-usage.md
```

Every numeric skeleton must match a named tier row.

- [ ] **Step 5: Commit**

```bash
git add docs/mcp-usage.md skills/deep-research.md skills/pm-deep-research.md
git commit -m "$(cat <<'EOF'
docs(limits): canonicalize quick/standard/deep budget tiers

Make mcp-usage the single budget table; label deep-research as standard
and PM as deep so Layer 1 skeletons stop drifting.
EOF
)"
```

---

### Task 6: Product doc drift fix (A5)

**Files:**
- Modify: `docs/research-agent-product.md`

**Interfaces:**
- Consumes: Runtime truth from CLI serve (OpenAI model + Exa/Grok/Tavily search), domain-owned schemas (no `schema/*` crate), prompt tree after Task 1
- Produces: Product doc that no longer claims Anthropic multi-model runtime, central schema paths, or a Generic-only thin prompt tree

- [ ] **Step 1: Fix model-provider claims**

In §4 goals, §5 non-goals, and §6.2 Reasoning Layer:

- Replace “Anthropic-compatible / local model gateway / multi-model runtime today” claims with:

```markdown
Current shipping model provider wiring is OpenAI Responses API (`openai`) via
`moe-research-cli` composition. The architecture remains provider-trait based so
additional model adapters can be added later; they are not present in the default binary today.
```

- Keep “not GPT-only forever” as a **design goal**, not a current feature checklist item.

- [ ] **Step 2: Fix schema path mental model**

Replace residual `schema/model.rs`, `schema/search.rs`, `schema/` module language with domain-owned locations:

```markdown
- Request/report DTOs: `moe-research-workflow`
- MCP envelope: `moe-research-mcp`
- Model DTOs: `moe-research-model`
- Search DTOs: `moe-research-search`
- TOML DTOs: `moe-research-config`
```

State explicitly: there is **no** central contracts/schema crate (by design).

- [ ] **Step 3: Fix §14 Skill & Prompt asset tree**

Replace the thin Generic-only tree with the real multi-profile tree, including Generic roots and profile directories, e.g.:

```text
skills/
  deep-research.md
  pm-deep-research.md
  academic-deep-research.md
  technical-evaluation.md
prompts/
  layer1/
    task-decomposition.md
    final-report.md
    common/
    pm-deep-research/
    academic-deep-research/
    technical-evaluation/
  layer2/
    aspect-agent.md
    search-planner.md
    evidence-extractor.md
    pm-deep-research/
    academic-deep-research/
    technical-evaluation/
```

Document:

- Generic uses `task-decomposition.md` + `final-report.md` + inlined `aspect-agent.md`.
- `search-planner.md` / `evidence-extractor.md` are **optional helper prompts** for Generic refinement; current Generic path may rely on `aspect-agent.md` alone.
- Profile personas live under profile subdirectories and are inlined as `AspectRequest.instructions`.

- [ ] **Step 4: Grep remaining fiction**

```bash
rg -n "Anthropic-compatible|schema/model|schema/search|contracts crate|multi-model" docs/research-agent-product.md
```

Expected: no false “currently supported” claims remain. Historical “future evolution” language is OK if labeled as future.

- [ ] **Step 5: Commit**

```bash
git add docs/research-agent-product.md
git commit -m "$(cat <<'EOF'
docs(product): align research-agent-product with runtime truth

Remove Anthropic multi-model and central schema path claims; document the
real prompt tree including Generic roots and profile packs.
EOF
)"
```

---

### Task 7: Academic / technical skill failure-handling parity (A10 partial)

**Files:**
- Modify: `skills/academic-deep-research.md`
- Modify: `skills/technical-evaluation.md`

**Interfaces:**
- Consumes: Task 3 shared partial contract (if Task 3 already inserted the section, this task only verifies quality parity and adds any missing operational bullets)
- Produces: Academic/technical skills with partial/failed quality comparable to PM/deep-research **for envelope semantics** (not full PM methodology)

- [ ] **Step 1: Verify Failure handling exists**

If Task 3 already added the shared block, extend each skill with a short operational checklist after it:

```markdown
### Operational checklist

- Prefer `deep_research` for multi-aspect work; use `aspect_research` only for a single focused retry.
- On `deep_research` partial: keep completed aspects; one `aspect_research` retry per failed aspect max.
- On `aspect_research` partial: preserve frozen evidence; fix Layer-1 prompt/schema bugs before retrying `schema_validation_failed`.
- After MoeResearch returns, continue with `../prompts/layer1/common/` evidence modules; host WebSearch/WebFetch remains HV-* only.
- If MCP tools are missing: stop and direct the user to `moeresearch mcp register` / `moeresearch assets install research-skills`.
```

- [ ] **Step 2: Ensure Assets section lists common evidence modules**

Add explicit common module list matching academic workflow step 6:

```markdown
Common: `../prompts/layer1/common/evidence-postprocess.md`, `claim-ledger.md`,
`host-verification-backfill.md`, `evidence-verifier.md`, `report-annex.md`.
```

- [ ] **Step 3: Length/quality check (manual)**

Academic/technical need not match PM line count. Required: Failure handling + operational checklist + common paths + policy boundaries (already present). No full PR-FAQ methodology port.

- [ ] **Step 4: Commit**

```bash
git add skills/academic-deep-research.md skills/technical-evaluation.md
git commit -m "$(cat <<'EOF'
docs(skills): raise academic/technical partial and failure handling

Give non-PM profiles the same envelope semantics and retry checklist as
deep/PM without rewriting full PM methodology.
EOF
)"
```

---

### Task 8: In-repo naming hygiene (B12)

**Files:**
- Modify: any project docs still teaching `lapis-*` crate/binary names incorrectly
- Note only: agent memory under `~/.claude/projects/.../memory/` is outside the repo

**Interfaces:**
- Consumes: Actual crate names `moe-research-*`, binary `moeresearch`
- Produces: In-repo docs free of incorrect `lapis-*` identifiers

- [ ] **Step 1: Search in-repo residue**

```bash
rg -n "lapis-|lapis_|crates/lapis|lapis-tests|lapis-workflow" \
  --glob '!target/**' --glob '!.git/**' --glob '!docs/superpowers/**'
```

Also check root title/context:

```bash
rg -n "\bLapis\b" README.md CLAUDE.md docs/*.md crates -g '!target/**'
```

- [ ] **Step 2: Fix incorrect identifiers**

Rules:

- Wrong: `lapis-workflow`, `lapis-tests`, `lapis-*` crate paths, binary name `lapis` → replace with `moe-research-workflow`, `moe-research-tests`, `moe-research-*`, `moeresearch`.
- Acceptable: monorepo codename “Lapis” in `CLAUDE.md` title (`MoeResearch / Lapis`) **if** it does not claim crate names. Optionally rephrase to “MoeResearch (repo codename: Lapis)” once for clarity.
- Do **not** rewrite historical audit findings under `docs/superpowers/specs/` that correctly describe past residue.
- Do **not** edit user agent memory files from this plan; if memory still says `lapis-*`, leave a one-line note in the phase retrospective that local memory may need a manual update outside the repo.

- [ ] **Step 3: Re-grep**

```bash
rg -n "lapis-|lapis_|crates/lapis" \
  --glob '!target/**' --glob '!.git/**' --glob '!docs/superpowers/**'
```

Expected: no incorrect crate/binary identifiers in active project docs/code comments.

- [ ] **Step 4: Commit**

```bash
git add CLAUDE.md README.md docs crates
git commit -m "$(cat <<'EOF'
docs: remove incorrect lapis-* naming residue from project docs

Keep MoeResearch crate/binary names authoritative in-repo; note agent
memory outside the repository is out of band for this change.
EOF
)"
```

---

### Task 9: Phase 1 verification gate

**Files:**
- None new (verification only)
- Optionally append a short retrospective note at the bottom of this plan file after success

- [ ] **Step 1: Format / lint / tests**

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p moe-research-tests --test cli_assets_tests
```

Expected: all green. Phase 1 is docs/assets-heavy; still run full workspace tests because Rust allowlist constants changed.

- [ ] **Step 2: Contract grep checklist**

```bash
# A1: Generic files packaged
node scripts/package-research-skills-assets.mjs --version 0.0.0-phase1-final --output-dir /tmp/moeresearch-phase1-final
rg -n "task-decomposition.md|aspect-agent.md" /tmp/moeresearch-phase1-final/*.manifest.json

# A3/B2: allowlists isomorphic (covered by test)
cargo test -p moe-research-tests --test cli_assets_tests packaging_allowlists_are_isomorphic

# A4: no PM evidence duplicates
test ! -f prompts/layer1/pm-deep-research/claim-ledger.md
test ! -f prompts/layer1/pm-deep-research/evidence-postprocess.md
test ! -f prompts/layer1/pm-deep-research/evidence-verifier.md
test ! -f prompts/layer1/pm-deep-research/host-verification-backfill.md
rg -n "prompts/layer1/common/(claim-ledger|evidence-postprocess|evidence-verifier|host-verification-backfill)" skills/pm-deep-research.md

# A2/A10: failure handling present
rg -n "Failure handling|error is null|data and \`error\` both set" skills/*.md docs/mcp-usage.md

# A6: tier table present
rg -n "Recommended Layer-1 budget tiers|standard.*max_agents|deep.*max_agents" docs/mcp-usage.md skills/deep-research.md skills/pm-deep-research.md

# A5: product doc no false Anthropic/schema claims
rg -n "Anthropic-compatible|schema/model.rs|schema/search.rs" docs/research-agent-product.md || true

# B12
rg -n "lapis-|lapis_" --glob '!target/**' --glob '!.git/**' --glob '!docs/superpowers/**' || true
```

- [ ] **Step 3: Phase retrospective (append to this plan)**

Append:

```markdown
## Phase 1 retrospective

- Date completed:
- Commits:
- Deviations from plan:
- Residual risks:
- Follow-ups deferred to Phase 2+:
```

- [ ] **Step 4: Final commit only if retrospective edited**

```bash
git add docs/superpowers/plans/2026-07-10-phase1-contract-integrity.md
git commit -m "docs(plan): record Phase 1 contract integrity retrospective"
```

---

## Self-review

### 1. Spec coverage (findings A1–A6, A10, B2, B12)

| ID | Finding | Task(s) | Closed by |
| --- | --- | --- | --- |
| A1 | Generic path not shipped | 1, 2 | Exact file allowlist + package ROOTS + install tree docs + skill fail-fast |
| A2 | Partial contract under-taught | 3, 7 | Frozen asymmetry table in mcp-usage + all four skills |
| A3 | Dual packaging allowlists | 1 | Same entries in Rust + Node + `packaging_allowlists_are_isomorphic` test |
| A4 | PM vs common evidence dup | 4 | Merge into common, retarget PM skill, delete PM copies |
| A5 | Product doc drift | 6 | Provider/schema/prompt-tree fixes in `docs/research-agent-product.md` |
| A6 | Limit skeleton drift | 5 | Canonical tier table + labeled standard/deep skeletons |
| A10 | Thin academic/technical | 3, 7 | Shared Failure handling + operational checklist (not full PM rewrite) |
| B2 | Dual packaging ownership | 1 | Test-enforced single truth (YAGNI; no new shared crate) |
| B12 | Lapis naming residue | 8 | In-repo identifier cleanup; out-of-repo memory noted |

### 2. Placeholder scan

- No TBD/TODO steps.
- Exact file paths, exact allowlist strings, exact tier numbers, exact commands.
- Out-of-scope items listed with phase pointers rather than empty stubs.

### 3. Consistency checks

- Generic shipped files: three required + two optional helpers — product doc and allowlists agree.
- Partial envelope: document-only; no Rust tool changes; matches audit freeze decision.
- PM evidence modules: path target is always `prompts/layer1/common/*` after Task 4.
- Budget tiers: deep-research skeleton = standard; PM skeleton = deep; mcp-usage owns the table.
- Packaging: Node packs, Rust validates, tests enforce isomorphism — no second policy writer without a failing test.
- No contracts crate; no config→workflow; schema 0.2 stable.

### 4. Execution notes for agents

- Prefer a short-lived branch for Phase 1 even though the roadmap allows a main-sized PR.
- Run Task 1 before relying on package contents in later manual checks.
- Task 4 deletes files — do not delete until path rewrites and common merges are done.
- If `packaging_allowlists_are_isomorphic` parsing is brittle, keep the extractor dumb (quoted-string harvest inside named const blocks) rather than introducing a shared schema format in Phase 1.
