# Pre-Phase 5 audit, stabilization and hardening

Pre-Phase 5 technical gate: audit and harden everything built up to Phase 4.5
before advancing to Phase 5 (`ag-cloud`). No new features; scope frozen per the
master audit plan. Baseline includes the in-flight native-server work
(RFC-0005 / DEBT-004).

## Affected phase

Phase 4.5 closure -> pre-Phase 5 release gate. Does not start Phase 5.

## Change type

- Documentation (audit reports under `docs/audits/`, benchmarks, reconciliation).
- Tests / fuzz / property tests / benchmarks (no public API changes intended).
- Bug fixes, panic removal, error hardening, CI hardening where required.

## Related documents

- Master audit plan (external PDF).
- `docs/roadmap/STATUS.md`, `docs/DEBT.md`.
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`,
  `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- ADR-0008 (English canonical), ADR-0009 (real-state / self-sufficiency).

## Test plan

Per stage (recorded with command + date + commit in the matching
`docs/audits/pre-fase5-*.md`):

- Stage 1: `cargo fmt --all -- --check`, `cargo check`, `cargo clippy -D warnings`,
  `RUSTDOCFLAGS="-D warnings" cargo doc`, `cargo test` (workspace, all features).
- Stage 3: `cargo audit`, `cargo deny check`.
- Stage 4: fuzz smoke (300s) + new targets; long fuzz documented.
- Stage 6: criterion benches + HTTP benchmarks (honest hardware caveats).

## Exit criteria advanced

Populates `docs/audits/PRE_FASE5_RELEASE_GATE.md`. Phase 5 is blocked until every
gate row is `pass`.

## Final checklist

- [x] Stage 0 inventory: `docs/audits/pre-fase5-inventory.md`.
- [x] Release gate skeleton: `docs/audits/PRE_FASE5_RELEASE_GATE.md`.
- [x] Stage 1 build/lint/test/docs gate (3 fixes: ag-storage DEL-byte DoS,
  ag-realtime thiserror build, clippy lints). All rows pass.
- [x] Stage 2 architecture audit (clean: no cycles, ag-core isolated, optional
  coupling, feature hygiene). `pre-fase5-architecture.md`.
- [x] Stage 3 security + supply chain (audit/deny clean; fixed Critical 5xx
  error-leak in ag-core; secrets review clean). `pre-fase5-security.md`.
- [x] Stage 4 fuzzing (3 new targets; fixed Critical RESP2 unbounded-alloc DoS;
  smoke ~9.3M runs 0 crashes). `pre-fase5-fuzzing.md`. 24h gate pending (manual).
- [ ] Stage 5 property tests.
- [ ] Stage 6 performance.
- [ ] Stage 7 concurrency.
- [ ] Stage 8 per-module security.
- [ ] Stage 9 examples/templates.
- [ ] Stage 10 docs honesty reconciliation.
- [ ] Stage 11 release gate evaluated.
