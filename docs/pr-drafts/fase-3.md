# feat(fase-3): completa cargo-fuzz, ag-lsp y plugin VS Code

## Resumen

Cierre de todos los entregables tecnicos de la Fase 3 — Anti-DSL alpha.
El compilador DSL v0.1-v0.4 esta completo (modelos, endpoints, validaciones,
relaciones). Esta PR anade los tres entregables finales: harness de fuzzing
(cargo-fuzz), servidor LSP (ag-lsp) y plugin VS Code con syntax highlighting,
diagnostics en tiempo real, autocompletado y hover.

## Fase afectada

Fase 3 — Anti-DSL alpha

## Tipo de cambio

- [x] Nueva feature
- [x] Bugfix (lexer panic en enteros > i64::MAX)
- [ ] Refactor
- [ ] Documentacion

## Documentos relacionados

- `docs/superpowers/specs/2026-05-21-fase3-pendientes-design.md`
- `docs/fuzz/README.md`
- `docs/dsl/lsp-roadmap.md`
- `docs/roadmap/STATUS.md`

## Cambios principales

### fix(dsl): lexer panic en overflow i64

`IntLit` usaba `.unwrap()` en el parse de enteros. Con inputs arbitrarios
(detectado por cargo-fuzz), enteros > i64::MAX causaban panic. Corregido
con `.ok()`: logos descarta el token y genera un error lexico controlado.
Test de regresion: `fuzz_crash_repro_tab_comment_number` en lib.rs.

### test(dsl): harness cargo-fuzz

3 targets libfuzzer en `fuzz/`:
- `fuzz_lexer`: lexer no panics con UTF-8 arbitrario
- `fuzz_parser`: parser no panics con entrada arbitraria
- `fuzz_compile`: pipeline completo lint+compile+generate no panics
Smoke test local: 42.404 runs en 11s sin crashes post-fix.

### ci(fuzz): job fuzz-smoke en quality.yml

Ejecuta cada target por 60s en cada PR/push con nightly Rust.
Sube artefactos si detecta crash. Gate manual de 24h pendiente antes
de mergear (ver `docs/fuzz/README.md`).

### feat(lsp): ag-lsp — servidor LSP Anti-Gravital alpha

Nuevo crate `crates/ag-lsp/` con tower-lsp 0.20:
- `textDocument/initialize`: server info + capabilities
- `textDocument/didOpen` + `didChange`: diagnostics en tiempo real via ag_dsl::lint()
- `textDocument/completion`: keywords, tipos, anotaciones, nombres de modelos del schema
- `textDocument/hover`: descripcion de tipos primitivos y anotaciones

10 tests unitarios. Smoke test del protocolo LSP: responde initialize correctamente.

### feat(vscode): plugin Anti-Gravital DSL

Plugin VS Code en `tools/vscode-anti-gravital/`:
- Syntax highlighting con tmLanguage para archivos `.ag`
- Integracion con ag-lsp via vscode-languageclient 8
- Deteccion automatica de ag-lsp en PATH con fallback a `cargo install ag-lsp`
- `.vsix` empaquetado y verificado: `anti-gravital-0.1.0.vsix`

## Plan de prueba

- [x] `cargo test --workspace`: 129+ tests verdes (119 ag-dsl + 10 ag-lsp + otros)
- [x] `cargo clippy --workspace --all-targets -- -D warnings`: limpio
- [x] `cargo fmt --all -- --check`: limpio
- [x] `cargo fuzz run fuzz_compile -- -max_total_time=10`: 42K runs sin crash
- [x] `npx tsc --noEmit` en tools/vscode-anti-gravital: TypeScript OK
- [x] `npx vsce package --no-dependencies`: anti-gravital-0.1.0.vsix generado
- [x] Smoke test LSP: ag-lsp responde initialize con serverInfo.name = "ag-lsp"

## Criterios de salida que avanza

- [x] Harness de fuzzing activo en CI (criterio 3.2.x)
- [x] ag-lsp binario funcional con diagnostics en tiempo real
- [x] Plugin VS Code empaquetado
- [ ] Gate manual 24h fuzzing (pendiente ejecucion en hardware Linux x86-64)
- [ ] Publicacion en VS Code marketplace (requiere repo publico)

## Checklist final

- [x] Pertenece a la fase correcta (Fase 3).
- [x] Respeta la documentacion.
- [x] No rompe arquitectura (ag-lsp depende solo de ag-dsl y tower-lsp).
- [x] No anade complejidad innecesaria.
- [x] No crea dependencias circulares.
- [x] Compila.
- [x] Pasa tests.
- [x] Pasa fmt.
- [x] Pasa clippy.
- [x] Tiene documentacion (docs/fuzz/README.md, docs/dsl/lsp-roadmap.md).
- [x] Tiene manejo de errores correcto.
- [x] Mantiene coherencia con Anti-Gravital v4.0.
