# Split oversized CLI, config, env and JSON test modules (issue #189)

## Resumen

Split the five remaining #189 files below 600 lines with no behaviour change.

## Fase afectada

Mantenimiento de deuda tecnica (RFC-0022 section 4.3 guardrails); no roadmap
phase changes. Toca `ag-cli` y `ag-core`, sin nuevos crates ni comandos CLI.

## Tipo de cambio

- [ ] Nueva funcionalidad
- [x] Refactorizacion sin cambio de comportamiento
- [x] Correccion de deuda tecnica
- [ ] Cambio de arquitectura
- [ ] Cambio de DSL
- [x] Documentacion

## Documentos relacionados

- `tools/file-size-allowlist.txt` (RFC-0022 section 4.3).
- `CHANGELOG.md` (registro de lineas antes/despues).
- `CLAUDE.md` reglas 12, 15, 18, 19, 26 y 29.
- Issue #189 (tech-debt, p3).

## Plan de prueba

```sh
bash tools/check-file-size.sh
cargo test -p ag-cli
cargo test -p ag-core
cargo test --workspace --all-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo audit
cargo deny check
```

Evidencia real en esta rama: check-file-size OK (310 archivos, 14 excepciones);
todos los comandos anteriores terminaron con codigo 0. Los nombres de los tests
se compararon contra HEAD y son identicos en los cinco archivos.

## Criterios de salida que avanza

- Issue #189 done-when 1: los cinco archivos estan a 487, 522, 591, 429 y 598
  lineas; cada uno por debajo del limite de 600.
- Issue #189 done-when 2: las cinco entradas `#189` se eliminaron del allowlist.
- Issue #189 done-when 3: `bash tools/check-file-size.sh` pasa con 14
  excepciones (antes 19).
- Issue #189 done-when 4: tests all-features, fmt y clippy en verde.

## Cierre de issues

Closes #189

## Checklist

- [x] Compila y pasa tests.
- [x] Pasa fmt y clippy con `-D warnings`.
- [x] Pasa audit y deny.
- [x] Documentacion y CHANGELOG actualizados en el mismo cambio.
- [x] Sin evidencia de herramientas IA, emojis ni marcas prohibidas.
