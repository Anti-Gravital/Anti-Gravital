# Resumen

Make the issue #157 benchmark handoff executable in one session on a reference host without claiming results.

- Fase afectada: documentacion y tooling de verificacion (ola 12); no altera
  fases del roadmap, targets de rendimiento ni el estado de la puerta.
- Tipo de cambio: [x] documentacion  [x] script de verificacion  [ ] codigo de crate  [ ] DSL  [ ] API publica.
- Documentos relacionados: `docs/benchmarks/REFERENCE_RUNBOOK.md`,
  `docs/benchmarks/reference-host.md`,
  `docs/benchmarks/gatebench-informative-2026-11-22.md`,
  `docs/audits/PRE_FASE5_RELEASE_GATE.md`, `CHANGELOG.md`,
  `docs/pr-drafts/README.md`, CLAUDE.md reglas 17, 19, 22 y 36, ADR-0008.

## Cambios

- `docs/benchmarks/REFERENCE_RUNBOOK.md` (nuevo): handoff unico para un
  mantenedor con el host de referencia. Incluye comprobaciones de preflight con
  comandos exactos, preparacion del host, rehearsal `--dry-run`, comando de
  ejecucion real, verificacion y commit de artefactos, curacion del informe,
  edicion de la fila `Benchmarks`, edicion del README EN/ES, checklist de
  cierre y anti-patrones. No contiene resultados ni mueve la puerta.
- `scripts/run-benchmarks.sh`: nuevo flag `--dry-run` que implica `--reference`.
  Ejecuta la misma validacion fail-closed del host e imprime los comandos y
  artefactos que produciria una corrida real, sin ejecutar benchmarks ni
  escribir artefactos. Sale 0 si el host pasa y 3 si falla el perfil. No
  requiere `REPORTER` ni attestation por adelantado porque no escribe nada; la
  attestation ausente se reporta como `FAIL quiet_attestation`.
- Correccion del gate: `quiet_attestation` ya no pasa siempre; antes usaba el
  literal `yes` como resultado en lugar del valor observado.
- Correccion de nombres: la salida del generador de carga usa
  `<tool>-run-N.txt` (`oha` o `wrk`), coherente con `reference-host.md` y con
  las rutas escritas en `e2e-runs.csv`.
- `docs/benchmarks/reference-host.md`: paso de preflight `--dry-run`, enlace al
  runbook y glob de artefactos que acepta ambos generadores de carga.
- `CHANGELOG.md` e indice de descriptores: registro y navegacion.

## Plan de prueba

- `bash -n scripts/run-benchmarks.sh` (exit 0).
- `./scripts/run-benchmarks.sh --help` muestra `--dry-run` y su semantica.
- `./scripts/run-benchmarks.sh --dry-run` sin attestation: imprime los `FAIL`
  del perfil (incluido `quiet_attestation no`) y sale 3; sin artefactos.
- `AG_QUIET_HOST_ATTESTED=yes REPORTER=gatebench ./scripts/run-benchmarks.sh --dry-run`
  en este host: rechazo real por hardware con exit 3; `git status --porcelain`
  sin cambios y directorio de salida no creado.
- Verificacion de contenido prohibido de `.github/workflows/docs.yml`: evidencia
  de herramientas IA, prefijos de rama IA en descriptores, emojis y marcas
  retiradas (ADR-0011).
- `git diff --check`.
- No se ejecuto ninguna corrida `--reference` ni se invento ningun numero de
  hardware de referencia; esto es capacidad de ejecucion, no evidencia.

## Criterios de salida que avanza

- Hace ejecutable en una sesion el procedimiento documentado en la issue #157
  sin afirmar que la puerta paso. La fila `Benchmarks` sigue `pending` y la
  issue abierta con `blocked`.
- Cumple la regla 17 (el runbook exige todos los campos y la desviacion
  estandar) y la regla 36 (reproducibilidad con commit y comando exactos).
- Respeta ADR-0008: documentacion tecnica en ingles, descriptores en espanol
  segun la convencion del repositorio.

## Cierre de issues

Refs #157
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (ola 12, handoff de puerta, sin cambio de fase).
- [x] Respeta la documentacion y no anade features no contempladas.
- [x] No rompe arquitectura ni anade dependencias circulares.
- [x] No anade complejidad innecesaria: reutiliza el validador existente.
- [x] Compila / no aplica (sin codigo Rust): shell validado con `bash -n`.
- [x] Pasa `git diff --check` y los escaneos de contenido.
- [x] Tiene documentacion (runbook, procedimiento, indice y changelog).
- [x] No inventa resultados ni cierra la puerta ni la issue.
