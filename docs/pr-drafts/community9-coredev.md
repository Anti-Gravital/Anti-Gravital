# Descriptor de PR: community9/coredev

## Resumen

Harden RFC-0020 stage B in ag-core: remove unzeroized secret clones and add opt-in fail-fast env validation in config loading

## Fase afectada

Additive Phase 4.8 stage B (RFC-0020). No roadmap or master document is changed; this implements and hardens the accepted design already in the tree.

## Tipo de cambio

- [ ] Feature (additive opt-in API, no new module or crate)
- [x] Fix (wave-4 review observation O5: transient unzeroized secret clones)
- [ ] Refactor
- [x] Documentation
- [x] Seguridad

## Documentos relacionados

- `docs/rfc/RFC-0020-environment-and-secrets.md` (sections 4.2, 4.3, 4.8, section 5 PR 1)
- `docs/audits/wave4-review.md` (observation O5, section 5)
- `CLAUDE.md` rules 15, 16, 18, 19, 22, 24, 26, 28, 31
- `ADR-0008` (English canonical code comments and technical docs)

## Plan de prueba

```bash
cargo test --locked -p ag-core
cargo test --locked --workspace --all-features
cargo fmt --all -- --check
cargo clippy --locked -p ag-core --all-features --all-targets -- -D warnings
cargo audit
cargo deny check
```

Key assertions: `EnvLoader::resolve` takes raw values out of the source map (no clone) and validates without allocating a parsed copy; exact raw bytes of typed secrets reach `Secret::expose`; `Debug`/`Display`/formatting flags and containers never leak; a `Secret` is not `Clone` nor comparable (compile-fail doctests); fail-closed errors list every problem with names and types only; plain `from_path` reads no environment variables; opt-in `from_path_with_env` and `from_toml_str_with_env` return `(ShieldConfig, LoadedEnv)` and fail on missing/malformed declarations without printing values. Full evidence: `/tmp/opencode/w9-coredev-*.log`.

## Criterios de salida que avanza

- `Secret<T>` keeps redaction and zeroization guarantees with no second, unzeroized copy of the material.
- Declared environment validation can be bound to config loading before a listener opens, as an opt-in that does not change defaults.
- No value reaches an error, log or formatted output path.
- All 82 direct `std::env::var` call sites remain unchanged.

## Coordinacion

`clidev` receives the exact API shape for `ag env list|check` in `/tmp/opencode/agents/w9-coredev.md`. No CLI file is touched by this branch.

## Cierre de issues

Closes: none

O5 is a wave-4 review observation with no open issue; no issue is closed falsely.

## Checklist

- [x] Pertenece a la fase correcta (additive Phase 4.8 stage B)
- [x] Respeta la documentacion (RFC-0020, wave-4 review, ADR-0008)
- [x] No rompe arquitectura (additive methods; `ShieldConfig::from_path` signature unchanged)
- [x] No crea dependencias circulares (no new crate or dependency; patch-only lock bump)
- [x] Compila
- [x] Pasa tests
- [x] Pasa fmt
- [x] Pasa clippy
- [x] Pasa audit
- [x] Pasa deny
- [x] Tiene documentacion (crate/module docs, ag-core README, CHANGELOG)
- [x] Tiene trazabilidad (descriptor, notes, RFC and review references)
