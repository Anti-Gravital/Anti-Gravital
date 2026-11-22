# RFC-0020 stage B: Secret<T> and declared environment loading in ag-core

## Resumen / Summary

Add `Secret<T>` (redacted Debug/Display, no Serialize, zeroize on drop,
explicit expose) and an additive declared-environment loader to `ag-core`.
Stage B only: no encrypted file and no `ag env` command.

## Fase afectada / Affected phase

Additive Phase 4.8, stage B (RFC-0020, PR 1 of the implementation plan).

## Tipo de cambio / Change type

- [x] New public API (`Secret<T>`, `ag_core::env` module).
- [x] Tests (unit + compile-fail doctest).
- [x] Documentation (crate docs, CHANGELOG, RFC-0020 stage status).
- [ ] CLI command.
- [ ] Template change.
- [ ] Breaking change.

## Documentos relacionados / Related documents

- `docs/rfc/RFC-0020-environment-and-secrets.md` (sections 4.1-4.3, 4.7, 5).
- `CLAUDE.md` rules 15, 16, 19, 26.
- `ADR-0008` (English canonical for code and technical docs).

## Plan de prueba / Test plan

```sh
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
cargo test -p ag-core
cargo test --workspace --all-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo audit
cargo deny --all-features check
```

## Criterios de salida que avanza / Exit criteria advanced

- RFC-0020 section 5, PR 1: `Secret<T>` and the typed loader in `ag-core`,
  with tests.
- No module was migrated off `std::env::var`: that is PR 4 and stays open.

## Cierre de issues

Closes: none

## Checklist

- [x] Belongs to the correct phase and follows the documentation.
- [x] No new dependency graph (`zeroize` and `url` were already in Cargo.lock).
- [x] Compiles, passes tests, fmt and clippy with `-D warnings`.
- [x] Public API documented in the same change.
- [x] `docs/master/` untouched; README state unchanged (no observable phase
      or CLI change).
