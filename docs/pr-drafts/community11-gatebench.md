# Resumen

Make the Stage 6 benchmark gate reproducible: one-command runner with fail-closed reference host and rule-17 metadata (#157).

## Fase afectada

Fase 6 (benchmarks, memoria, flamegraphs) del gate previo a Fase 5. Ejecucion
ordenada tras la consolidacion de codigo estable. No avanza ningun gate: la
ejecucion de referencia aun no existe.

## Tipo de cambio

- [x] Documentacion de procedimiento y plantilla de evidencia.
- [x] Script de automatizacion reproducible (criterion + carga extremo a extremo).
- [ ] Codigo de crates (no aplica).
- [ ] Cambio de target de rendimiento (requeriria RFC; no se hace).

## Documentos relacionados

- `docs/benchmarks/reference-host.md` (nuevo): perfil, herramientas, comandos,
  artefactos y plantilla vacia de evidencia.
- `docs/benchmarks/verification-package.md`: procedimiento autoritativo; se
  anade la referencia al runner de un solo comando.
- `docs/benchmarks/wave9-host-results.md` y `docs/benchmarks/smoke-2026-11-22.md`:
  datos informativos previos, no evidencia de referencia.
- `docs/benchmarks/measurement-template.md`: metodologia y regla 17.
- `docs/audits/PRE_FASE5_RELEASE_GATE.md`: fila Benchmarks sigue `pending`.
- CLAUDE.md reglas 17, 22, 26, 36.

## Plan de prueba

```sh
bash -n scripts/run-benchmarks.sh
bash scripts/run-benchmarks.sh --help
AG_QUIET_HOST_ATTESTED=yes REPORTER=gatebench bash scripts/run-benchmarks.sh --reference   # debe rechazar este host
SAMPLE_SIZE=10 WARMUP_TIME=1 MEASUREMENT_TIME=1 OUT_DIR=docs/benchmarks/raw/<fecha>-informative \
  bash scripts/run-benchmarks.sh                                                            # carrera real corta
git diff --check
# escaneos de contenido prohibido y emojis sobre los archivos tocados
```

Resultado real: `bash -n` y `--help` correctos; `--reference` rechazo este host
con exit 3 y sin artefactos (6 checks fallidos); la carrera corta termino con
exit 0 en el commit `8ef5792` con arbol limpio. Evidencia en
`docs/benchmarks/gatebench-informative-2026-11-22.md` y
`docs/benchmarks/raw/2026-11-22-informative/`. Ningun numero de este host se
publica como objetivo de diseno.

## Criterios de salida que avanza

- Procedimiento de referencia listo para ejecutarse con un solo comando.
- Metadatos de la regla 17 capturados automaticamente, incluida la desviacion
  estandar de `estimates.json`.
- Modo `--reference` que se niega a etiquetar un host no conforme.
- Plantilla de evidencia vacia, sin numeros inventados.

## Cierre de issues

Refs #157
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (Stage 6, ultimo gate tecnico).
- [x] Respeta la documentacion y no altera targets (sin RFC).
- [x] No anade dependencias ni complejidad a los crates.
- [x] Verificado con `bash -n`, rechazo real en host no conforme y carrera real corta.
- [x] memoria descriptiva/archivos generados incluidos; gate no modificado.
