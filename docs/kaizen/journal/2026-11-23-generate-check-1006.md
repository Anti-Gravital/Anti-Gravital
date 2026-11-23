# 2026-11-23 G1 - `ag generate --check` (RFC-0019 PR 4)

- Tarea: G1-generate-check (proyecto fase48-comandos), rama `community21/generate-check`.
- Decision: añadir `#[arg(long)] check: bool` al subcomando `Generate` existente,
  no un comando nuevo. RFC-0019 seccion 4.4/PR 4 ya autoriza `--check`, asi que
  no hace falta otro RFC (CLAUDE.md regla 28). Se reutiliza `ag_dsl::generate`
  mediante `compile_artifacts` para que escritor y verificador no puedan diverger.

## Evidencia (comandos y resultados reales)
- `cargo test -p ag-cli`: 260 tests en verde (11 binarios), RC=0.
- `cargo test -p ag-cli --test generate_check`: 9 tests en verde, RC=0.
- `cargo test --workspace --all-features`: 1310 passed, 0 failed (89 lineas
  `test result: ok`), RC=0.
- `cargo fmt --all -- --check`: limpio. `cargo clippy --workspace --all-targets
  -- -D warnings`: limpio.
- `bash scripts/ag-check`: PASS=8 FAIL=0 SKIP=0 (descriptors, fmt, clippy,
  test, audit, deny, masters, prohibited), RC=0.
- `bash tools/check-file-size.sh`: 314 archivos Rust, 14 excepciones, ninguno
  sobre 600 lineas, RC=0. `git diff --check`: limpio.
- Humo manual con el binario recompilado: match -> `Check passed: 6 artifact(s)
  ...`, RC=0; deriva de un byte -> `changed   src/models.rs`, RC=1, y el byte
  derivado sobrevive al check.

## Resultado
- Codigo: flag `--check` en `cli.rs`, ruteo en `dispatch.rs`, `cmd_generate_check`
  en `commands/schema.rs` (comparacion byte a byte, estados `missing`/`changed`/
  `unexpected`, informe determinista, cero escrituras).
- Tests: `crates/ag-cli/tests/generate_check.rs` + 4 tests de parseo en `cli.rs`.
- Documentacion: README EN+ES, CHANGELOG, apendice de estado de RFC-0019,
  STATUS.md y descriptores en `docs/kaizen/pr-drafts/` y `docs/pr-drafts/`.
- Sin push, PR ni merge (el orquestador los hace).

## Aprendizaje
Una guardia de codigo generado solo es honesta si consume exactamente la misma
salida en memoria que el escritor. Una asercion de codigo de salida no basta:
snapshotear el directorio antes y despues detecta escrituras accidentales que
una prueba de `status` ignoraria. El binario `debug/ag` puede estar desactualizado
frente al binario hashed que usa `CARGO_BIN_EXE_ag`; para evidencia manual hay que
recompilar el binario explicitamente.
