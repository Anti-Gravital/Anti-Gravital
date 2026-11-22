# Descriptor de PR

## Resumen

Add offline alpha release smoke script and real end-to-end acceptance evidence

## Fase afectada

Ola 5 (alpha release). No cambia ninguna fase de la Hoja de Ruta ni ningun
crate de produccion; anade una herramienta de verificacion y su evidencia.

## Tipo de cambio

- [ ] Feature
- [x] Verificacion / QA
- [x] Documentacion
- [ ] Bug fix
- [ ] Refactor
- [ ] Cambio de arquitectura (requiere RFC)

## Documentos relacionados

- `docs/audits/alpha-smoke-evidence.md` (evidencia real, nueva).
- `scripts/release-smoke.sh` (script reproducible, nuevo).
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` (fase alpha, sin cambios).
- `docs/pr-drafts/README.md` (convencion de descriptor).

No se modifican RFC, ADR ni documentos maestros: el cambio es aditivo y no
altera contratos arquitectonicos.

## Plan de prueba

Desde la raiz del repositorio, con el entorno Cargo de la ola 5:

```
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
bash -n scripts/release-smoke.sh
bash scripts/release-smoke.sh
cargo test --locked -p ag-cli --test wave4_regression
cargo fmt --all -- --check
git diff --check
```

Resultado real registrado en `docs/audits/alpha-smoke-evidence.md`: 10 pasos
PASS, 0 FAIL, codigo de salida 0, sin red y sin servicios externos.

## Criterios de salida que avanza

- Evidencia reproducible de que el alpha es ejecutable y usable sin red:
  build `--locked`, scaffold, DSL generate, build del proyecto generado y
  respuesta HTTP real en loopback.
- No cierra ninguna casilla de `docs/roadmap/STATUS.md` que dependa de CI en
  cuatro plataformas, instalador con binarios precompilados o servicios
  externos; esas siguen pendientes y se enumeran en la evidencia.

## Cierre de issues

Closes: none

## Checklist final

- [x] Pertenece a la ola correcta (alpha release).
- [x] No rompe arquitectura ni anade dependencias.
- [x] Todo cambio observable esta documentado en la misma rama.
- [x] Verification real ejecutada; ningun resultado inventado.
- [x] Sin emojis, sin evidencia de herramientas IA, mensajes cortos.
