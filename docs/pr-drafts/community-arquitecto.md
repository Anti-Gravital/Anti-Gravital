# Resumen

Remove ag-ui from the roster and propose the Phase 5+ ag-data ORM/RLS design (issues #182, #154)

## Fase afectada

Phase 4.8 stage A (RFC-0019 scope decision, #182) and a design-only proposal
for Phase 5+ (#154). No phase is opened, advanced or implemented by #154.

## Tipo de cambio

- [x] Documentacion (maestros, README, CHANGELOG, RFC, grafos, derivados)
- [x] Retirada de un crate del workspace (solo `crates/ag-ui/`, sin codigo funcional)
- [x] Recalculo de hashes y de rangos del generador de derivados
- [ ] Codigo funcional
- [ ] Cambio de fase o de roadmap

## Documentos relacionados

- `RFC-0019`, seccion 4.7 y seccion 9 decision 2 (autoriza la retirada de
  `ag-ui` en un PR propio que actualice regla 14, tres maestros y workspace).
- `CLAUDE.md` regla 14; `docs/master/VERSION.md`; `.github/workflows/docs.yml`.
- `RFC-0025` (nueva propuesta) y la nota de diseno de datos
  `docs/modules/ag-data/data-layer-rls-tenancy.md`.
- `RFC-0019`, `RFC-0020` y `ADR-0013` como predecesores de `RFC-0025`.

## Plan de prueba

```sh
# En el worktree de la rama, con el entorno Cargo compartido:
bash tools/split-masters.sh && git diff --exit-code docs/roadmap docs/architecture
sha256sum docs/master/*.md   # coincide con VERSION.md y docs.yml
cargo fmt --all -- --check
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 cargo check --workspace --all-targets
cargo metadata --no-deps --format-version 1 | grep -c ag-ui   # 0
git diff --check
```

La retirada de `ag-ui` no toca dependencias en tiempo de ejecucion: el crate
era un esqueleto vacio y ningun template, ejemplo o manual lo referenciaba.

## Criterios de salida que avanza

- Issue #182: workspace y roster 19 crates; sin menciones activas de `ag-ui`;
  hashes de maestros y derivados sincronizados; `grep` de codigo limpio.
- Issue #154: `RFC-0025` disponible para periodo de comentarios; el issue
  permanece abierto porque la implementacion es Fase 5+ y no esta aceptada.

## Cierre de issues

Closes #182
Refs #154

## Checklist

- [x] Pertenece a la fase correcta.
- [x] Respeta la documentacion y la decision RFC-0019.
- [x] No rompe arquitectura ni anade dependencias.
- [x] Compila y pasa `cargo check --workspace --all-targets`.
- [x] Incluye documentacion y hashes actualizados.
