# fix(ag-workers,ag-cache): qualify RETURNING columns in lease() and fix manual_is_multiple_of lint

## Summary

Fixes two open issues in priority order. Both are resolvable without external
infrastructure. Issues requiring a live database (#108, #109, #103), a real
domain (#87) or a concrete consumer use case (#112) remain untouched and
correctly documented as blocked/deferred.

- **#121 (p1, bug) ag-workers PostgreSQL RETURNING ambiguity:** `PostgresQueue::lease`
  used bare `RETURNING id, ...` inside an `UPDATE … FROM due` where both the target
  table alias `j` and the `due` CTE expose `id`, causing PostgreSQL to reject every
  lease attempt with `column reference "id" is ambiguous`. The `RETURNING` list is
  now qualified with `j.` through a new `SELECT_COLUMNS_QUALIFIED_J` constant.
  The unused, now-redundant `SELECT_COLUMNS` constant is removed. Column output
  names are unchanged so `row_to_envelope` is unaffected.
- **#119 (p3, lint) ag-cache clippy manual_is_multiple_of:** The Rust 1.95
  `clippy::manual_is_multiple_of` default lint flagged `(args.len() - 1) % 2 != 0`
  in `cmd_mset`. Replaced with `!(args.len() - 1).is_multiple_of(2)` (stable since
  1.87, within MSRV 1.95). This clears the last workspace-wide clippy error, making
  `cargo clippy --workspace --all-targets --all-features -- -D warnings` fully green.

## Phase affected

Phase 4.6-D (ag-workers durable backend) and Phase 4.5 (ag-cache correctness).
No phase transition; changes are corrective and additive.

## Type of change

- [ ] Security fix
- [x] Bug fix
- [ ] Tests
- [ ] Documentation
- [ ] New feature
- [ ] Breaking public API change

## Related documents

- `docs/rfc/RFC-0012-ag-workers-background-jobs.md` (PostgreSQL backend, §13)
- `crates/ag-workers/tests/postgres_queue.rs` (integration tests, #[ignore])
- `docs/roadmap/STATUS.md`

## Test plan

- [x] `cargo fmt --check` — no diffs.
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` — exit 0.
- [x] `cargo test -p ag-workers --all-features` — all in-memory tests pass; Postgres
      `#[ignore]` tests unchanged (still require live DB, tracked in #108).
- [x] `cargo test -p ag-cache --all-features` — 13 lib tests + 2 doc tests pass.
- Note: Postgres integration tests (#108) are not run here (no live DB); the SQL fix
  is structurally correct and mirrors the qualified `s.` pattern already used by
  `scheduler.rs` in the same crate.

## Exit criteria advanced

- #121 (p1) closed: `lease()` RETURNING ambiguity resolved.
- #119 (p3) closed: workspace clippy gate fully green.
- Still blocked on external infrastructure (untouched): #108, #109, #103 (live
  PostgreSQL), #87 (real-domain ACME staging).
- Design-deferred (untouched): #112 (ag-edge producer wiring, no consumer yet).
- Tracking: #76 (ag-domains remaining work) unaffected.

## Final checklist

- [x] Belongs to correct phase
- [x] Respects documentation
- [x] Does not break architecture
- [x] No unnecessary complexity added
- [x] No circular dependencies
- [x] Compiles (full workspace, all features)
- [x] Tests pass (`cargo test -p ag-workers --all-features`, `cargo test -p ag-cache --all-features`)
- [x] `cargo fmt` passes
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [x] Documentation updated in same PR
- [x] No emojis
- [x] No AI attribution
- [x] Commit messages under 256 characters
- [x] PR descriptor present
