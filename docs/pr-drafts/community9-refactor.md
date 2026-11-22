# Split oversized ag-cli deploy and dns modules into focused submodules

## Resumen

Split the `ag-cli` deploy and dns command modules into focused submodules with no behaviour change.

## Fase afectada

Mantenimiento de deuda tecnica de la Fase 4 (RFC-0022, guardrail de longitud de archivos).
No altera ninguna fase del roadmap ni anade funcionalidad.

## Tipo de cambio

- [x] Refactorizacion interna sin cambio de comportamiento
- [x] Reduccion de deuda tecnica
- [x] Documentacion (CHANGELOG y lista de excepciones)
- [ ] Nueva funcionalidad
- [ ] Cambio de API publica o CLI

## Documentos relacionados

- RFC-0022, seccion 4.3 (limite de 600 lineas por archivo fuente)
- Issue #189 (propietario de las excepciones de longitud)
- `tools/file-size-allowlist.txt`
- `docs/pr-drafts/README.md`

## Cambios concretos

- `commands/deploy.rs`: 768 -> 30 lineas. Submodulos `deploy/model.rs` (199),
  `deploy/plan.rs` (224), `deploy/apply.rs` (85), `deploy/status.rs` (81) y
  `deploy/tests.rs` (204).
- `commands/dns.rs`: 1234 -> 34 lineas. Submodulos `dns/model.rs` (297),
  `dns/manifest.rs` (330), `dns/state.rs` (82), `dns/provider.rs` (99),
  `dns/commands.rs` (189) y `dns/tests.rs` (245).
- La API `pub(crate)` de ambos modulos se conserva con re-exportaciones;
  `main.rs` y `cli.rs` no se tocan.
- `tools/file-size-allowlist.txt`: 20 -> 18 excepciones (se retiran las dos
  entradas de deploy.rs y dns.rs).

## Plan de prueba

```sh
bash tools/check-file-size.sh
cargo test -p ag-cli
cargo test --workspace --all-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo audit
cargo deny check
```

## Criterios de salida que avanza

- Ningun archivo fuente de ag-cli supera las 600 lineas sin excepcion.
- La lista de excepciones del issue #189 se reduce en dos entradas.
- 100% de los tests de ag-cli y del workspace pasan; clippy sin advertencias.

## Verificacion real

- `bash tools/check-file-size.sh`: OK, 290 archivos verificados, 18 excepciones.
- `cargo test -p ag-cli`: verde (incluye 10 tests de deploy y 14 de dns).
- `cargo test --workspace --all-features`: verde.
- `cargo fmt --all -- --check`: limpio.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: limpio.
- `cargo audit`: FALLA por RUSTSEC-2026-0592 (`url` 2.5.8 transitiva),
  preexistente y ajena a este cambio. No se modifica Cargo.lock.
- `cargo deny check`: FALLA por advertencias `advisory-not-detected` en
  `deny.toml` (ignorados ya resueltos), preexistente y ajena al cambio.

## Cierre de issues

Closes: none
Refs #189 (avanza la reduccion de excepciones; #189 sigue abierto hasta que
los demas archivos de la lista se reduzcan en sus propias ramas)

## Checklist

- [x] Sin cambio de comportamiento ni de API observable
- [x] Compila, pasa tests, fmt y clippy
- [x] Documentacion actualizada en el mismo cambio
- [x] Sin dependencias nuevas ni circulares
- [x] Sin evidencia de herramientas IA, emojis ni mensajes largos
