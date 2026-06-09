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

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test -p ag-workers` (memory backend: retry, poison guard, shutdown, ordering)
- [ ] `cargo test -p ag-workers --features postgres` (testcontainers: SKIP LOCKED leasing,
  lease-expiry + reaper, retry across restart, DLQ persistence, `enqueue_in_tx` rollback,
  scheduler singleton across N processes)
- [ ] `cargo test -p ag-dsl worker` (parse + semantic + generated stubs compile)
- [ ] `cargo test -p ag-mail --features workers` (S7/M2: Email payload roundtrip,
  retriable/permanent classification, enqueue-through-adapter delivered by a pool)
- [ ] `cargo test -p ag-mail --features workers-postgres -- --ignored` (S7/M3 parity:
  mail job persists as `kind=mail.delivery`, delivered, survives restart; needs
  `TEST_DATABASE_URL`)
- [ ] `cargo test --workspace --all-features` (no circular deps; cross-crate green)
- [ ] `cargo audit` and `cargo deny check`

## Exit criteria advanced (docs/roadmap/STATUS.md)

- Phase 4.6-D entry criteria (RFC/ADR approved; pattern present; `ag-data`/`ag-observe`
  available) — marked.
- Phase 4.6-D deliverables S1-S7 and exit criteria — advanced per landed stage.

## Final checklist

- [x] Belongs to the correct phase (4.6-D, additive pre-Phase-5).
- [x] Respects documentation (RFC-0012, ADR-0013; documentation lands first).
- [x] Does not break architecture (acyclic deps; `ag-core` untouched; `ag-mail` queue
  untouched at introduction).
- [x] No unnecessary complexity (extracts an existing pattern; one new dep `tokio-util`).
- [x] No circular dependencies.
- [ ] Compiles (per staged commit).
- [ ] Tests pass (per staged commit).
- [ ] `cargo fmt` clean.
- [ ] `cargo clippy -D warnings` clean.
- [x] Documentation updated in the same change (RFC/ADR/module/roster/roadmap/architecture).
