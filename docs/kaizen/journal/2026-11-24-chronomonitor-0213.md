# Diario - 2026-11-24 0213 UTC - chronomonitor (community30)

## Tarea

Corregir el monitor de deriva de chrono (issue #185) para que un error de
herramienta/red de `cargo deny` no se reporte como FAIL de deriva. No cerrar
#185 (sigue bloqueado por aguas arriba).

## Decision

Clasificar el resultado de `cargo deny` ANTES de hashear: exit no cero CON
`deny.status` vacio (sin veredicto de politica) -> `RESULT: ERROR
reason=deny-tool-error-no-verdict`, exit 2, tambien en `--print-baseline` para
que una ejecucion rota no se hornee en una baseline nueva. Exit 0 o exit no cero
CON veredicto mantiene la comparacion de hash original: FAIL si hay deriva real,
PASS si coincide. Fail-closed preservado: ERROR nunca es PASS. Hash de baseline
`dd4fb95d...` sin cambios. Se usa `Refs #185`, nunca `Closes #185`.

El archivo de hallazgo S-11 referenciado no existia en el historial de esta rama
(se escribio en `community29/i185`); se reconstruyo su sustancia desde
`docs/audits/chrono-edge-monitor.md` seccion 7.3 y este diario, y se anadio el
outbox en esta rama.

## Evidencia (comandos y resultados reales)

- `bash -n scripts/check-chrono-edge.sh` -> exit 0.
- `git diff --check` -> exit 0.
- `bash scripts/check-chrono-edge.sh` (cache caliente real, cargo 1.99.1,
  cargo-deny 0.20.2) -> `RESULT: PASS`, exit 0,
  `hash.current = hash.baseline = dd4fb95d5...ccc83f`.
- Stub `cargo` en PATH (sin red), error de fetch en cache fria -> `deny.exit=1`,
  `deny.status=none`, `deny.verdict=absent`,
  `RESULT: ERROR reason=deny-tool-error-no-verdict`, exit 2.
- Stub, deriva real (`advisories fail`) -> `deny.verdict=present`, hashes
  distintos, `RESULT: FAIL reason=drift-detected`, exit 1.
- Stub, limpio -> `deny.exit=0`, hashes iguales, `RESULT: PASS`, exit 0.
- Stub cache fria + `--print-baseline` -> `RESULT: ERROR`, exit 2, sin hash.
- Argumento desconocido -> `RESULT: ERROR reason=unknown-argument`, exit 2.
- `bash scripts/ag-check` -> PASS=6 (descriptors, fmt, clippy, test, masters,
  prohibited), FAIL=2 (audit, deny) por falta de red/DNS al descargar la base de
  datos de RustSec (fallo del entorno, no del cambio). Hallazgo S-11 en
  `.kaizen/loop/outbox/ag-check-audit-deny-offline.md`.
- `bash scripts/ag-check --skip audit,deny` -> PASS=6, SKIP=2, exit 0 (aislamiento
  de los checks dependientes de red; no aprueba audit/deny).
- `scripts/kaizen check` -> todo en verde, exit 0.

## Resultado

Commit preparado en `community30/chronomonitor` con: fix en
`scripts/check-chrono-edge.sh`, seccion 8 y tabla de semantica actualizada en
`docs/audits/chrono-edge-monitor.md`, entrada de CHANGELOG, descriptor en
`docs/kaizen/pr-drafts/` y `docs/pr-drafts/`, hallazgo S-11 en el outbox y esta
entrada de diario. Sin push/PR/merge. #185 sigue abierto y bloqueado.

## Aprendizaje

Un exit code no cero de una herramienta es ambiguo: clasificar la SALIDA
(veredicto presente o ausente) antes de hashear evita confundir un fallo de
infraestructura con deriva de politica, sin debilitar la deteccion real.
La verificacion con stubs controlados permite ejercitar las tres ramas sin red.
