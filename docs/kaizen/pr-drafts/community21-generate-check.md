## Resumen
Add read-only `ag generate --check` (RFC-0019 PR 4) with a deterministic drift report, a shared codegen path and full integration coverage.

## Componente afectado
`ag-cli` (fase 4.8-A, proyecto fase48-comandos, tarea G1-generate-check).

## Tipo de cambio
- [x] enhancement
- [x] documentation

## Documentos relacionados
- `docs/rfc/RFC-0019-project-layered-architecture.md` section 4.4 / PR 4 (authorizes `--check`) and its stage-A status appendix.
- `docs/roadmap/STATUS.md` (fase 4.8.2).
- `README.md` (EN+ES), `CHANGELOG.md`.
- `.kaizen/RULES.md` K-1/K-7/K-26; `CLAUDE.md` rules 8, 20, 22, 28; `.kaizen/security/SECURITY-RULES.md` S-1/S-3/S-4.

## Plan de prueba
- `CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p ag-cli`: 260 passed, 0 failed (11 binaries).
- `cargo test -p ag-cli --test generate_check`: 9 passed, 0 failed.
- `cargo test --workspace --all-features`: 1310 passed, 0 failed (89 `test result: ok` lines); log `/tmp/opencode/generate-check-workspace-test.log`.
- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `bash scripts/ag-check`: PASS (all checks).
- `bash tools/check-file-size.sh`: PASS (no new exceptions).
- `git diff --check`: clean.
- Content scans: no AI attribution, no emoji, no retired brand, no secret values.

## Impacto en calidad (ratchet)
- Cobertura: sube (9 integration tests + 4 CLI parsing tests for the new flag).
- Advertencias: igual (`clippy -D warnings` clean).
- Deuda (TODO/FIXME): igual (ningun marcador nuevo).

## Criterios de salida que avanza
- RFC-0019 section 4.4: `ag generate --check` exits 0 when artifacts match and non-zero when a file is `missing`, `changed` or `unexpected`.
- Read-only: temp-dir tests assert byte-for-byte snapshots are unchanged after a check, that drifted bytes survive, and that a fresh check does not create the output directory.
- Writer and verifier share `compile_artifacts`, so generate and check cannot drift.
- `--output-format` selector kept intact; `--check` forces the prose path so it exits on drift rather than on the format selector.
- K-7 documentation updated in the same change (README EN+ES, CHANGELOG, RFC status appendix, STATUS.md).

## Aprendizaje (opcional)
A code guard is only honest if it consumes the exact same in-memory generator output as the writer. Snapshotting the directory before and after the check catches accidental writes that a status-code-only assertion would miss.

## Cierre de issues
Closes: none (tarea G1-generate-check de fase48-comandos; Refs RFC-0019 PR 4)

## Checklist
- [x] Compila
- [x] Pasa tests
- [x] `bash scripts/ag-check` en verde
- [x] Documentacion actualizada
- [x] Linea base no empeora
