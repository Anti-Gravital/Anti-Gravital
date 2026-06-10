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

## Issues addressed in this branch

- #57 `ag-observe`: `/metrics` now renders the real Prometheus snapshot from a
  stored `PrometheusHandle`; uninitialized exporter state returns an explicit
  service-unavailable response instead of an empty body.
- #58 `ag-observe`: `OTEL_EXPORTER_OTLP_ENDPOINT` now returns
  `ObserveError::OtlpSetup` until a real OTLP exporter exists, avoiding silent
  trace drops.
- #61 MSRV/toolchain: workspace MSRV is aligned to Rust 1.95.0 in Cargo,
  clippy config, contributor docs, onboarding docs and CI with a dedicated
  `cargo check --workspace --all-targets` job.

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

Commands run for the issue fixes:

- `cargo fmt --package ag-observe`
- `cargo test -p ag-observe`
- `cargo clippy -p ag-observe --all-targets -- -D warnings`
- `cargo check --workspace --all-targets`
- `git diff --check`

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
- [x] Stage 5 property tests (proptest: ag-storage signed-URL crypto + path
  confinement, 6 properties green). `pre-fase5-properties.md`.
- [ ] Stage 6 performance.
- [x] Stage 7 concurrency (no unbounded channels; mail queue bounded; storage
  I/O non-blocking; fixed missing reqwest timeouts on Cloudflare+Resend).
  `pre-fase5-concurrency.md`.
- [x] Stage 8 per-module security (10 crates pass; JWT alg-pinned, API keys
  constant-time, SQL parameterized; fixed metric-cardinality footgun doc).
  `pre-fase5-security-modules.md`.
- [x] Stage 9 examples (5 examples build/run as documented; fixed ag-realtime
  publish-with-no-subscriber 500; regenerated ecommerce-api English+reproducible;
  added 2 missing READMEs; fixed stale index docs). `pre-fase5-examples.md`.
- [ ] Stage 10 docs honesty reconciliation.
- [ ] Stage 11 release gate evaluated.
