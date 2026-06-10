# audit: consolidate verified Phase 0-4.5 capabilities

## Summary

Audits the Phase 0-4.5 contract against the current implementation, fixes
verified correctness and production-readiness gaps outside the active
`ag-domains` workstream, and reorganizes public status and onboarding around
reproducible evidence.

## Phase affected

Phases 0, 1, 2, 3, 4, and 4.5.

## Type of change

- Audit and documentation reconciliation
- Correctness fixes
- Async and resource hardening
- Installation and developer-experience fixes
- Regression tests

## Related documents

- `CLAUDE.md`
- `docs/master/ANTI-GRAVITAL-Blueprint-v4.0.pdf`
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`
- `docs/audits/PRE_FASE5_RELEASE_GATE.md`
- `docs/DEBT.md`

## Audit issues

- #70 fix(ag-dsl): make generated Rust compile as a coherent module unit
- #71 docs(roadmap): replace completed phase claims with evidence-based status
- #72 fix(tooling): make installation reproducible and integrity guidance real
- #73 fix(ag-realtime): remove blocking per-event file I/O from event persistence
- #74 fix(ag-cache): bound native RESP2 server connections
- #75 docs(ag-cli): align implemented commands and onboarding with approved scope

## Test plan

- [x] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`
- [ ] `cargo audit`
- [ ] `cargo deny check`
- [/] Installer checks (`bash -n install.sh` passed; controlled stub execution and `pwsh` were unavailable locally)
- [x] Generated Rust compile regression (`tests/generated-rust-fixture` is a workspace crate; `cargo test -p ag-generated-rust-fixture`)

Additional targeted checks run during consolidation:

- [x] `cargo test -p ag-dsl` (157 passed, 1 doc-test)
- [x] `cargo test -p ag-realtime --features event-persistence` (33 passed, 1 ignored manual scalability gate, 1 doc-test)
- [x] `cargo test -p ag-cache --features native-server` (29 unit passed, 1 ignored benchmark, 13 RESP2 integration passed, 2 doc-tests)
- [x] `git diff --check`

Local runner note: global clippy and rustdoc commands were attempted but produced no output for several minutes and were terminated. Audit and deny availability could not be confirmed without additional command approval. These checks remain intentionally unchecked.

## Exit criteria advanced

- Phase status reflects actual evidence.
- Generated Phase 3 Rust is a compilable module unit.
- Open Phase 4 realtime/cache hardening debt is reduced.
- Installation instructions are reproducible and integrity guidance exists.
- Public README is bilingual, concise, and aligned with verified capabilities.

## Final checklist

- [x] Belongs to the correct phases
- [x] Respects the master documentation and approved RFCs/ADRs
- [x] Excludes active `ag-domains` and `ag-edge` implementation work
- [x] Adds no future-phase feature
- [x] Compiles
- [x] Tests pass
- [x] Format passes
- [ ] Clippy passes
- [ ] Audit passes
- [ ] Deny passes
- [x] Documentation updated with behavior
- [x] No emojis
- [x] No tool attribution
- [x] PR descriptor present
