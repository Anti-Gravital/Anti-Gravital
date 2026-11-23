# Diario - 2026-11-23 0219 UTC - estado-honesto E3

## Tarea

Inventariar con evidencia la deuda, los issues abiertos y la salud de pruebas
del proyecto `estado-honesto` (tarea `E3-deuda-y-issues`, owner `qa`, en
`.kaizen/team/projects/estado-honesto/tasks.json`).

## Decision

Publicar `docs/audits/estado-honesto-deuda.md` con tres grupos de issues
(bloqueados externamente, diferidos por decision, accionables ahora) mas dos
padres de seguimiento, cada uno con tipo, razon de bloqueo, condicion exacta de
cierre y owner; un inventario de deuda (DEBT.md congelado, 4 hallazgos del
ratchet `mktemp XXXXXX`, 14 excepciones de tamano, ignore de chrono, marcadores
TECH-DEBT) y la salud de pruebas (40 targets, cobertura citada, 4 fallas de
`kaizen check` con su referencia S-11). Autonomia `propose`: no se cerro ningun
issue, no se movio etapa ni puerta, no se relajo la linea base.

## Evidencia (comandos y resultados reales)

- `gh issue list --repo Anti-Gravital/Anti-Gravital --state open --json ...` ->
  14 abiertos (208, 185, 180, 179, 178, 157, 156, 154, 152, 142, 141, 112,
  87, 76). `gh issue list --repo Anti-Gravital/Gravital-Fabric --state open` ->
  1 abierto (#2).
- `gh issue view 208 ...` -> ci-failure autodetectado, creado
  2026-11-22T18:34:52Z, run 45055852575, commit 5ff8e5d. `gh issue view 3
  ... Fabric` -> CLOSED; #189 y #198 (AG) -> CLOSED. El tablero vivo (15)
  difiere del snapshot wave-11 (14 AG + 2 Fabric).
- `grep -c '^### DEBT-' docs/DEBT.md` -> 25; `docs/DEBT.md:1-14` confirma
  registro historico congelado (regla 29 de CLAUDE.md).
- `bash scripts/kaizen check --only ratchet` -> `FALLA: deuda subio: 4 > 0`
  (exit 1). `grep -rEI 'TODO|FIXME|HACK|XXX' scripts tests` -> exactamente 4
  lineas: `scripts/validate-deploy.sh:90`,
  `scripts/validate-domain-loop.sh:95`, `scripts/release-smoke.sh:152`,
  `scripts/validate-templates.sh:70`, todas plantillas `mktemp ....XXXXXX`.
  Causa: `scripts/kaizen:15-17` cuenta el token `XXX` sobre
  `KZ_SRC_GLOBS="scripts tests"`; `.kaizen/baseline.json` -> `debt_max: 0`.
- `bash tools/check-file-size.sh` -> `OK: 313 tracked Rust files checked, 14
  allowed exceptions, none over 600 lines.` Las 14 entradas de
  `tools/file-size-allowlist.txt` referencian #142 y miden 606-1507 lineas.
  `diff` de ese archivo entre rama y `main` -> sin cambios.
- `deny.toml:21-26` y `docs/audits/chrono-unblock-analysis.md:1-228` -> ignore
  RUSTSEC-2026-0543 con condicion de retirada e issue #185; reporte upstream
  abierto y sin PR.
- `grep` de `TECH-DEBT` sobre `crates/` -> 0 coincidencias; la convencion
  sigue en `CLAUDE.md:744` y `Cargo.toml:154`; #135 CLOSED.
- `cargo metadata --no-deps --format-version 1` -> 40 targets `test`; 189
  archivos con anotaciones de test en `crates/`; 10 `.rs` en `tests/`.
  `docs/audits/pre-fase5-build-gate.md:21` -> 52 suites / 564 passed /
  7 ignored (snapshot 2026-05-29, no repetido aqui).
  `docs/audits/PRE_FASE5_RELEASE_GATE.md:32` -> cobertura `pass` 80.05%
  (3210/4010, 2026-05-30, no repetida aqui).
- `bash scripts/kaizen check` antes del descriptor -> `ok: descriptores
  pendientes` mas 4 fallas: `tests`, `swarm`, `gobernanza`, `deuda subio:
  4 > 0`, y `FALLA: falta el descriptor ...`. Tras anadir el descriptor ->
  `ok: descriptores validados` y las mismas 4 fallas preexistentes (exit 1).
  Cada falla tiene su outbox S-11 y su tarea correctiva en
  `.kaizen/team/projects/kaizen-gates-green/tasks.json`.
- `git diff --check` y `git diff HEAD --check` -> exit 0. Escaneo de emoji,
  atribucion a IA y supresiones sobre los archivos cambiados -> sin
  coincidencias tras corregir una frase que mencionaba literalmente el token
  de omision de hooks.

## Resultado

Entregable listo para commit en `community15/status-deuda`. Verificacion
propia en verde para `git diff --check` y escaneo de contenido. `kaizen check`
permanece en rojo por cuatro fallas preexistentes, no introducidas por este
cambio; se reportan sin supresion (S-10) y escalan al orquestador. Sin push,
PR ni merge.

## Aprendizaje

El ratchet de deuda cuenta el token `XXX`, asi que cuatro plantillas
`mktemp ....XXXXXX` aparecen como deuda: un falso positivo de la configuracion
de globs, no del codigo. Conviene leer `count_debt()` antes de aceptar el
numero. Ademas, el estado de un tablero se degrada con el tiempo: los addenda
wave-11/12 ya no describen el tablero vivo (#189, #198 y Fabric #3 cerrados;
#208 abierto), por lo que toda cifra debe reconsultarse con `gh` en el corte de
evidencia.
