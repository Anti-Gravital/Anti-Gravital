# fix/verify: ag-workers PostgreSQL lease fix and live-DB verification of the ag-mail S7 migration

## Summary

Resolves the open issues that are reproducible in this environment, in priority
then precedence order. Critically, this session **disproved the "blocked on a
live database" premise** of #108/#109/#103: PostgreSQL 16 was stood up directly
in the ephemeral container (initdb + server start as the `postgres` user — no
System V IPC failure), the `#[ignore]` integration suites were run against it,
and the results recorded below.

- **#121 (p1, bug) ag-workers PostgreSQL lease() ambiguity:** `lease()` used a
  bare `RETURNING id, ...` inside `UPDATE … FROM due`; both the target alias `j`
  and the `due` CTE expose `id`, so every claim failed with `column reference
  "id" is ambiguous`. Qualified the `RETURNING` list with `j.`. Reproduced the
  exact error and the fix directly in psql before/after, then confirmed with the
  live suite.
- **#108 (was blocked) ag-workers live PostgreSQL verification:** all 6 `#[ignore]`
  tests in `postgres_queue.rs` pass against a live PostgreSQL 16 (enqueue/lease/ack,
  `SKIP LOCKED` concurrency, lease-expiry + reaper, DLQ persistence, bulk DLQ,
  `enqueue_in_tx` rollback). Test reset now also drops `_sqlx_migrations` for rerun
  determinism; header documents `--test-threads=1`.
- **#109 (was blocked) ag-mail S7/M3 parity:** found the parity suites did not even
  compile under their documented command (they import `NullSender` from
  `test_utils`, gated behind `test-utils`/`cfg(test)`, which does not reach the lib
  in an integration-test build). Added a self dev-dependency enabling `test-utils`.
  Fixed an incorrect assertion (the backend acks by deleting the row; there is no
  persisted `succeeded` status). Both parity tests pass live: mail jobs persist as
  `kind=mail.delivery`, deliver once, and survive a simulated restart.
- **#103 (was blocked, depends on #109) ag-mail S7/M4 removal:** with parity proven,
  removed the duplicated `queue::store::PersistentQueue`, the `queue-persistent`
  feature, the legacy `persistent_queue.rs` test and `migrations/0001_mail_queue.sql`,
  and the now-unused optional deps `ag-data`/`chrono`/`uuid` (`sqlx` stays for the
  parity suite). The shared `ag-workers` PostgreSQL backend is the sole durable path.
- **#119 (p3, lint) ag-cache clippy:** `(args.len()-1) % 2 != 0` →
  `!(args.len()-1).is_multiple_of(2)`, clearing the last Rust-1.95 workspace lint.

Issues left untouched and correctly so: **#87** (live ACME staging — needs a real
controlled domain + DNS, not available here), **#112** (design-deferred `ag-edge`
producer wiring — no concrete consumer; Postgres availability does not change the
design rationale), **#76** (ag-domains tracking — its children are unrelated).

## Phase affected

Phase 4.6-D (ag-workers durable backend; `ag-mail` S7 migration now M0-M4 complete)
and Phase 4.5 (ag-cache correctness). No phase transition; corrective + cleanup.

## Type of change

- [x] Bug fix
- [x] Tests
- [x] Documentation
- [ ] New feature
- [ ] Security fix
- [ ] Breaking public API change

Note: removing the `queue-persistent` feature is a removal of a previously
`#[deprecated]` opt-in surface, executed per RFC-0012 S7/M4 once parity was proven.

## Related documents

- `docs/rfc/RFC-0012-ag-workers.md` (S7/M3-M4), `docs/adr/0013-ag-workers-execution-model.md`
- `docs/roadmap/STATUS.md` (4.6-D exit criteria + S7), `README.md` (root, bilingual)
- `crates/ag-mail/README.md`, `crates/ag-workers/README.md`, `docs/modules/ag-mail/README.md`

## Test plan (executed against a live PostgreSQL 16 in this environment)

- [x] `cargo fmt --check` — clean.
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` — exit 0.
- [x] `cargo check --workspace --all-features` — exit 0 (no cross-crate breakage from
      the removed feature).
- [x] ag-workers live: `DATABASE_URL=… cargo test -p ag-workers --features postgres
      --test postgres_queue -- --ignored --test-threads=1` — **6 passed**.
- [x] ag-mail live: `TEST_DATABASE_URL=… cargo test -p ag-mail --features
      workers-postgres --test workers_postgres -- --ignored --test-threads=1` —
      **2 passed**.
- [x] ag-mail legacy parity (pre-removal, for equivalence): `persistent_queue.rs`
      `message_survives_and_is_sent` — **1 passed** live, same observable outcome.
- [x] `cargo test -p ag-mail --all-features` and `-p ag-cache --all-features` — green.
- [x] Adversarial check: reproduced `column reference "id" is ambiguous` in psql with
      the unqualified query and confirmed the qualified query parses + executes.

## Exit criteria advanced

- #121, #119 closed.
- #108, #109, #103 closed with live-database evidence (premise that they were blocked
  on external infrastructure disproved for this environment).
- STATUS.md 4.6-D: PostgreSQL backend exit criterion flips to verified; S7 marked
  complete (M0-M4).
- Still untouched (genuinely out of reach here): #87 (real domain), #112 (no consumer).

## Final checklist

- [x] Belongs to correct phase
- [x] Respects documentation
- [x] Does not break architecture
- [x] No unnecessary complexity added
- [x] No circular dependencies
- [x] Compiles (full workspace, all features)
- [x] Tests pass (workspace + live PostgreSQL integration suites)
- [x] `cargo fmt` passes
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [x] Documentation updated in same PR (README sync, STATUS, module docs)
- [x] No emojis
- [x] No AI attribution
- [x] Commit messages under 256 characters
- [x] PR descriptor present
