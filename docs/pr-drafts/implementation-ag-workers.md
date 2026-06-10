# ag-workers: Rust-native background execution engine (RFC-0012 / ADR-0013, Phase 4.6-D)

## Summary

Introduces `ag-workers`, the shared Rust-native background execution engine (typed jobs,
retries, DLQ, scheduling, worker pools), extracting the proven queue pattern from
`ag-mail`. Documentation first, then code in staged S1-S7 commits. No GA claim until the
pre-Phase-5 gate closes.

## Phase affected

Phase 4.6-D (additive pre-Phase-5 extraction/hardening, sibling of 4.6-A `mta` and
4.6-C `api`). Owner-authorized to proceed with the pre-Phase-5 release gate open
(ADR-0013); no production/GA claim is made.

## Type of change

- [x] Documentation (RFC-0012, ADR-0013, module doc, roster, roadmap, architecture)
- [x] Feature implementation (new crate `ag-workers`; additive, native-by-default,
  feature-gated) — staged S1-S7
- [x] DSL change (additive `worker` declaration in `ag-dsl`)
- [x] CLI change (additive `ag workers` subcommands, feature-gated)
- [x] Test (unit, integration via testcontainers, property, fuzz targets)
- [x] Cross-crate migration (S7): additive `workers`/`workers-postgres` features in
  `ag-mail` routing generic mail delivery through `ag-workers`; the duplicated
  `queue-persistent` generic queue is deprecated (not yet removed)

## Related documents

- `docs/rfc/RFC-0012-ag-workers.md`
- `docs/adr/0013-ag-workers-execution-model.md`
- `docs/modules/ag-workers/README.md`
- `docs/architecture/05-ecosistema-modulos.md`, `docs/architecture/08-modulos-batteries-included.md`
- `docs/roadmap/STATUS.md` (Phase 4.6-D)
- `crates/ag-mail/src/queue/{mod.rs,store.rs}` (the extracted pattern)
- CLAUDE.md rule 29 (tech debt tracked as GitHub Issues; `docs/DEBT.md` frozen)

## Test plan

- [x] `cargo fmt --all -- --check` — clean.
- [x] `cargo clippy -p ag-workers --all-features --all-targets -- -D warnings` — clean;
  also `-p ag-cli` and the four new examples clippy-clean with `-D warnings`. Full
  `--workspace --all-features` not re-run in this session.
- [x] `cargo test -p ag-workers` (memory backend: retry, poison guard, shutdown,
  ordering, runtime outcomes, scheduler, properties) — 15 tests green.
- [ ] `cargo test -p ag-workers --features postgres` (SKIP LOCKED leasing, lease-expiry +
  reaper, retry across restart, DLQ persistence, `enqueue_in_tx` rollback, scheduler
  singleton) — `#[ignore]`, needs a live `DATABASE_URL`. Tracked in #108.
- [x] `cargo test -p ag-dsl worker` (parse + generated stubs compile) — green.
- [x] `cargo test -p ag-mail --features workers` (S7/M2: Email payload roundtrip,
  retriable/permanent classification, enqueue-through-adapter delivered by a pool) — green.
- [ ] `cargo test -p ag-mail --features workers-postgres -- --ignored` (S7/M3 parity) —
  needs `TEST_DATABASE_URL`. Tracked in #109 (M4 removal in #103 depends on it).
- [x] Examples build, clippy-clean, and run: `workers-scheduled`, `workers-producer-edge`,
  `workers-mail-integration` run end to end; `workers-postgres` exits cleanly without
  `DATABASE_URL`.
- [x] `cargo bench -p ag-workers --no-run` — criterion throughput benchmarks compile
  (payload encode/decode, enqueue, batch lease; `benches/queue_throughput.rs`).
- [x] `fuzz_workers_payload` target added (rmp-serde decode boundary must not panic) and
  wired into the `quality` fuzz-smoke job (60s) alongside the DSL targets; verified by CI.
- [x] Coverage gate green at 82.28% (>= 80%); gate excludes examples/benches/fuzz and the
  feature-unified PostgreSQL backend (network/DB, `#[ignore]` tests).
- [x] `cargo audit` and `cargo deny check` — covered by the `quality` workflow on push.

Deferred to GitHub Issues (could not run here without a live database; this sandbox has
no usable Docker daemon and no running PostgreSQL): #108 (run/verify Postgres integration
tests), #109 (S7/M3 mail-job reconciliation), #103 (S7/M4 deprecated-queue removal),
#110 (`ag-data` canonical transaction handle for `enqueue_in_tx`). Follow-ups filed this
session: #112 (producer-only `ag-edge` wiring, deferred until a concrete consumer),
#113 (implement or reserve `RejectedRateLimited`), #114 (bulk DLQ re-drive, needs RFC).

## Exit criteria advanced (docs/roadmap/STATUS.md)

- Phase 4.6-D entry criteria (RFC/ADR approved; pattern present; `ag-data`/`ag-observe`
  available) — marked.
- Phase 4.6-D deliverables S1-S7 and exit criteria — advanced per landed stage. STATUS.md
  reconciled to reality this session: S1-S5 marked done; S6 partial (producer feature +
  example, `ag-edge` wiring deferred to #112); S7 M0-M2 done, M3/M4 blocked on a live
  database (#109/#103). Coverage and payload-fuzz exit criteria advanced.

## Final checklist

- [x] Belongs to the correct phase (4.6-D, additive pre-Phase-5).
- [x] Respects documentation (RFC-0012, ADR-0013; documentation lands first).
- [x] Does not break architecture (acyclic deps; `ag-core` untouched; `ag-mail` queue
  untouched at introduction).
- [x] No unnecessary complexity (extracts an existing pattern; one new dep `tokio-util`).
- [x] No circular dependencies.
- [x] Compiles (ag-workers, ag-cli, ag-mail, ag-dsl, and the five examples).
- [x] Tests pass (ag-workers memory/property/runtime/scheduler/poison-guard; ag-dsl
  worker; ag-mail `workers`). Postgres-backed tests are `#[ignore]`, tracked in #108/#109.
- [x] `cargo fmt` clean.
- [x] `cargo clippy -D warnings` clean (ag-workers `--all-features --all-targets`, ag-cli,
  examples).
- [x] Documentation updated in the same change (RFC/ADR/module/roster/roadmap/architecture).
