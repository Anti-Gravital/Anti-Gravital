# Pre-Phase 5 Release Gate

> Formal gate that decides whether Anti-Gravital may advance to Phase 5
> (`ag-cloud`). Stage 11 / PR 8 deliverable of the master audit plan. This file
> is created as a **skeleton** in PR 1 (all rows `pending`) and is filled in,
> row by row, as each audit stage produces verifiable evidence. Phase 5 may not
> start until every blocking row is `pass`.

- **Created:** 2026-05-29 (PR 1, skeleton).
- **Status:** OPEN — gate not yet evaluated. No row may be marked `pass` without
  a reproducible command, a date, and (where applicable) a commit.

## Gate table

State values: `pending` (not yet evaluated), `pass`, `fail`. "Blocks Phase 5"
means a `fail` or `pending` on that row forbids advancing.

| Area | State | Evidence | Blocks Phase 5 | Stage |
| --- | --- | --- | --- | --- |
| Build | pass | `cargo clippy --workspace --all-targets --all-features` exit 0, 2026-05-29 (commit 635e623) | yes | 1 |
| Tests | pass | `cargo test --workspace --all-features`: 564 passed, 0 failed, 7 ignored, 2026-05-29 | yes | 1 |
| Clippy | pass | `cargo clippy --workspace --all-targets --all-features -- -D warnings` exit 0, 2026-05-29 | yes | 1 |
| Docs | pass | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features` exit 0, 2026-05-29 | yes | 1 |
| Audit | pass | `cargo audit` exit 0, 0 vulnerabilities / 591 deps, 2026-05-29 | yes | 3 |
| Deny | pass | `cargo deny check` exit 0 (advisories/bans/licenses/sources ok), 2026-05-29 | yes | 3 |
| Fuzz smoke | pass | 6 targets (3 DSL + fuzz_resp/storage_key/signed_url), ~9.3M runs, 0 crashes, 2026-05-29 | yes | 4 |
| Fuzz 24h | pending | manual gate; command in `pre-fase5-fuzzing.md` | yes | 4 |
| Coverage | pending | report | yes | 5 |
| Benchmarks | pending | docs | yes | 6 |
| Examples | pass | Stage 9 (`pre-fase5-examples.md`): 4 binaries build, self-contained ones run as documented, `ecommerce-api` regen reproducible (`diff -rq` identical), 3 `ag new` templates scaffold clean, docs honest, 2026-05-29 | yes | 9 |
| Security | pass | Stage 3 (`pre-fase5-security.md`) + Stage 8 per-module (`pre-fase5-security-modules.md`): all 10 functional crates pass, 2026-05-29 | yes | 3/8 |
| API/DX | pass | Stage 2 architecture (`pre-fase5-architecture.md`) + module API review (Stage 8); footguns documented, 2026-05-29 | yes | 2 |
| Docs honesty | pending | report | yes | 10 |
| Open debt | pending | `docs/DEBT.md` | yes | all |

## Findings severity classification

Used across all `docs/audits/pre-fase5-*.md` reports.

### Blocker — prevents Phase 5
Examples: workspace does not compile; tests fail; critical vulnerability; panic
on external input; false claim in README; broken auth/token; path traversal;
data loss; unbounded queue without backpressure; unresolved fuzz crash.

### Critical — must be fixed before public beta
Examples: error leaks secrets; trivial DoS; unbounded resource; vulnerable
dependency; misleading benchmark; behaviour inconsistent between docs and code.

### High — must be fixed before public release
Examples: API hard to use correctly; missing tests in a critical module; badly
designed feature flag; broken primary example.

### Medium — may go to documented debt
Tracked in `docs/DEBT.md`.

### Low — future improvement.

## "Ready to advance to Phase 5" definition

Anti-Gravital may advance to Phase 5 only if all of the following hold (mirrors
the plan's final checklist):

- [ ] Workspace compiles fully.
- [ ] Tests pass.
- [ ] Clippy passes with no warnings.
- [ ] Docs compile with no warnings.
- [ ] `cargo audit` passes.
- [ ] `cargo deny` passes.
- [ ] No known productive panics on external input.
- [ ] Fuzz smoke passes.
- [ ] Long manual fuzz executed or documented as a blocking pending item.
- [ ] Minimum coverage verified.
- [ ] Primary examples work.
- [ ] README claims are backed by evidence.
- [ ] Open technical debt is documented.
- [ ] `ag-cloud` remains a skeleton until this gate closes.

## Execution order (owner decision, 2026-05-29)

Stage 6 (benchmarks / memory / flamegraphs) is executed **last among the
technical stages, immediately before Stage 10 (docs)**, not as an intermediate
step. Rationale: benchmarks must measure stabilized code; running them before the
fuzzing (4), property (5), concurrency (7), per-module security (8) and examples
(9) fixes would produce numbers invalidated by later changes. Stage numbering and
gate rows are unchanged — only the execution sequence is:

`0 → 1 → 2 → 3 → 4 → 5 → 7 → 8 → 9 → 6 → 10 → 11`

## Audit document index

| Stage | Document | Status |
| --- | --- | --- |
| 0 | `docs/audits/pre-fase5-inventory.md` | done (PR 1) |
| 1 | `docs/audits/pre-fase5-build-gate.md` | done (PR 2) |
| 2 | `docs/audits/pre-fase5-architecture.md` | done (PR 3) |
| 3 | `docs/audits/pre-fase5-security.md` | done (PR 3) |
| 4 | `docs/audits/pre-fase5-fuzzing.md` | done (PR 4) |
| 5 | `docs/audits/pre-fase5-properties.md` | done (PR 5) |
| 6 | `docs/benchmarks/pre-fase5-performance.md` | pending |
| 7 | `docs/audits/pre-fase5-concurrency.md` | done (PR 5) |
| 8 | `docs/audits/pre-fase5-security-modules.md` | done (PR 5) |
| 9 | `docs/audits/pre-fase5-examples.md` | done (PR 6) |
| 10 | `docs/audits/pre-fase5-docs-reconciliation.md` | pending |
| 11 | `docs/audits/PRE_FASE5_RELEASE_GATE.md` | this file |
