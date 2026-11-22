# ag new --with MODULE and ag env list|check (RFC-0019/0020)

Resumen: CLI roadmap gaps: ag new --with MODULE and ag env list|check (RFC-0019/0020)

Fase afectada:

- Additive Phase 4.8, stage A (RFC-0019) and stage B (RFC-0020).
- No change to the canonical masters or to the roadmap phases.

Tipo de cambio:

- [x] New CLI flag (`ag new --with`, repeatable).
- [x] New CLI command group (`ag env list|check`).
- [x] Tests (unit + non-interactive integration).
- [x] Documentation (CHANGELOG, README EN+ES, module README, RFC status).
- [ ] Dependency or lockfile change (none made).
- [ ] Breaking change (none: default scaffold is byte-identical).

Documentos relacionados:

- `docs/rfc/RFC-0019-project-layered-architecture.md` (section 4.3, 11).
- `docs/rfc/RFC-0020-environment-and-secrets.md` (sections 4.1, 4.6, 5).
- `templates/CATALOG.md`, `README.md`, `crates/ag-cli/README.md`,
  `docs/modules/ag-cli/README.md`, `CHANGELOG.md`.

Plan de prueba:

```bash
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0
cargo test -p ag-cli --all-features
cargo test --workspace --all-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo audit
cargo deny --all-features check
```

Criterios de salida que avanza:

- RFC-0019 stage A: module selection is recorded in the generated `ag.toml`
  `[modules]` table without changing the default scaffold.
- RFC-0020 stage B PR 2: `ag env list` and `ag env check` exist as read-only
  views through the core loader; no value is printed and no file is written.

Verification evidence (2026-11-22):

- `cargo test -p ag-cli --all-features`: 157 unit + 10 env_cli + all existing
  integration suites, 0 failures.
- `cargo test --workspace --all-features`: 0 failures.
- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean.
- `cargo audit` / `cargo deny --all-features check`: BLOCKED by the pre-existing
  RUSTSEC-2026-0592 advisory (`url` 2.5.8, fixed in 2.5.9) present in the base
  Cargo.lock. This branch changes no Cargo.toml/Cargo.lock, so the failure is
  not introduced here. Not claimed as a pass.

Cierre de issues:

Refs #154
Closes: none

Checklist:

- [x] Pertenece a la fase correcta (additive 4.8-A/B).
- [x] Respeta la documentacion (RFC status updated in the same change).
- [x] No anade complejidad innecesaria (no new dependency, no new crate).
- [x] Compila, pasa tests, pasa fmt, pasa clippy.
- [x] Audit/deny: pre-existing blocker documented above, not faked.
- [x] Tiene documentacion EN+ES y trazabilidad.
