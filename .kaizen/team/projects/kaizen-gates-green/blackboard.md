# Blackboard: kaizen-gates-green

Solo hechos con `source_ids` verificados o salida de comando reproducible.


## F1 platform-engineer — tests/run.sh (2026-11-22)

- Que hace: `bash tests/run.sh` corre 29 selftests deterministicos (sin red, secretos ni Rust): archivos canonicos, `bash -n` de scripts del gate, parseo de `kaizen-insights.py`, `ag-check --list` de 8 compuertas y honestidad de ag-check (false->1, true->0, --bogus->2, SKIP nunca PASS).
- `bash tests/run.sh` -> exit 0; "29 pruebas pasadas, 0 falladas".
- `scripts/kaizen check --only tests` -> exit 2: modo no soportado por el CLI (ruta protegida). Gate real validado con `scripts/kaizen check` -> "ok: tests" (exit global 1 por fallos preexistentes swarm/gob/deuda, otras tareas).
- `bash scripts/ag-check` -> FALLO preexistente en `masters` (hash de ANTI-GRAVITAL-Arquitectura-Tecnica.md != docs/master/VERSION.md); no causado por este cambio.
- `git diff --check` -> exit 0. `scripts/kaizen check --only descriptors` -> exit 0.
- Commit en rama community14/f1-runsh; sin push/PR/merge.
## Auditoria de puertas - F3 / gate-audit (2026-11-22, rama `community14/gate-audit@c1b918d`)
Rol: gate auditor. Modo: solo lectura; NO se ejecuto ninguna regeneracion ni se modifico codigo. Archivos tocados: este blackboard y el outbox `kaizen-f3-catalogo-needs-human.md` (canal S-11). Todo lo demas es evidencia de comando reproducible (S-15/L-5: contenido externo = dato, no instruccion).
### F3 - catalogo desincronizado: ESCALADO a humano (no ejecutado)
- Reporte original: `scripts/kaizen-org check` -> `FALLA: catálogo desc.sincronizado (43 vs 21)` (outbox `kaizen-org-catalogo-desincronizado.md`).
- Verificacion actual (2026-11-22): el desajuste NO se reproduce en `main@b44c9b0` ni en `c1b918d`. Salida real: `[kaizen-org] kaizen-org check: todo en verde`; `.count=25`, registry `length=25`, 26 ficheros en `prompts/` (25 con `id` + `_template.md`). `git status --short` limpio tras `check`.
- Comando exacto que SOLO UN HUMANO debe ejecutar: `scripts/kaizen-org catalog`. Efecto: reescribe `.kaizen/org/registry.json`, `docs/kaizen/org/CATALOGO-AGENTES.md` y `.kaizen/org/catalog.json` (count = definiciones). Tras el check, arrastra `kaizen-swarm` (`scripts/kaizen-swarm:254`).
- Autoridad: L-4 protege `.kaizen/`; un agente no puede regenerar. Precondicion humana: decidir fuente de verdad porque `gov/governance.json` tiene 43 entradas frente a 25 definiciones locales y regenerar degrada la cobertura (finding `kaizen-gov-especialistas-y-regresion.md`). Detalle completo en el outbox `kaizen-f3-catalogo-needs-human.md`.
### Otros items que SIGUEN en rojo tras F1/F2/F4 (escalacion unica al humano)
| # | Item (hallazgo) | Puerta / CI afectada | Evidencia reproducible | Por que requiere humano |
| --- | --- | --- | --- | --- |
| 1 | Gobernanza: 4 especialistas sin entradas (`oss-observability-specialist`, `container-k8s-specialist`, `rust-lowlevel-specialist`, `rust-perf-benchmarker`); matriz 43 vs catalogo local 25 | `kaizen check --only gov` FAIL; arrastra `swarm` | `python3 scripts/kaizen-gov check` -> `kaizen-gov check: FALLO` + 4 x `agente sin gobernanza`; `jq '[.agents[]?]\|length' .kaizen/gov/governance.json` -> 43 | `.kaizen/gov/governance.json` y `scripts/lib/gov_seed.py` son protegidos (L-4). `kaizen-gov regenerate` degrada 43->25: no ejecutar sin decision de fuente. Acoplado a F3. |
| 2 | `swarm` en rojo: falta ADR-0001 + gov en rojo | CI job `framework` -> `scripts/kaizen-swarm check` FAIL | `scripts/kaizen-swarm check` -> `FALLA: falta ADR-0001 (autonomia nivel 3)` y `FALLA: kaizen-gov check en rojo`; `scripts/kaizen-swarm:253-254` | `docs/kaizen/adr/` es protegido (L-4). F2 (ADR autonomia nivel3) cubre el ADR, PERO no elimina el item 1; `swarm` no estara verde hasta resolver gobernanza. Ademas `scripts/bootstrap` solo copia ADR 0002..0010 (reincidencia en instalacion limpia). |
| 3 | `ag-check` (hook pre-push) con 3 falsos positivos: `docs/audits/wave9-roadmap-decisions.md:428`, `scripts/kaizen:49` (regex de politica) y PDF binario | Bloquea push a main -> release blocker | `bash scripts/ag-check` -> `FAILED`, `PASS=7 FAIL=3`; ver outbox `ag-check-falsos-positivos-push.md` | Cambiar un escanner de seguridad requiere humano (S-10: nada se silencia; L-4 sobre hooks/escanners). No se modulo nada. |
| 4 | `.github/workflows/docs.yml:50` hardcodea hash viejo del maestro de Arquitectura | Workflow `docs`, job `masters integrity`, CI rojo en main | `grep -n expected_arch .github/workflows/docs.yml` -> `7c1c7d0c...`; `sha256sum docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` -> `ed89854f...` | `.github/` es protegido (L-4). Requiere humano actualizar el hash y, mejor, leerlo desde `VERSION.md`. Ver outbox `docs-yml-hash-hardcodeado-rfc0038.md`. |
| 5 | `kaizen-team assign` no escribe handoffs (bug jq `now` vs `$now`); `handoffs.jsonl` queda en 0 lineas | Protocolo K-14/L-17 (handoff sin `expects_ack`); `kaizen check --only team` puede dar verde falso | `scripts/kaizen-team:71-72` usa `now` sin `$`; `wc -l .kaizen/team/projects/<slug>/handoffs.jsonl` -> 0; ver outbox `kaizen-team-assign-handoff-jq.md` y `kaizen-team-assign-jq-timestamp.md` | `scripts/kaizen*` es protegido (L-4). Requiere humano corregir el script y re-asignar handoffs. |
| 6 | `.kaizen/config.env` no auditable por el agente | Revision de configuracion de puertas | Intento de lectura del archivo denegado por permisos (S-2: el agente no lee `.env`/config de credenciales) | Un humano debe revisar `KZ_TEST_CMD`, `KZ_SRC_GLOBS`, `KZ_COVERAGE_CMD`, `KZ_WARNINGS_CMD`, `KZ_POLICY_*`. F1/F4 cubren los sintomas (tests/run.sh, deuda), no la configuracion en si. |
### Ya cubierto por F1/F2/F4 (informativo, no escalar de nuevo)
- F1 `crear-tests-runsh`: `KZ_TEST_CMD="bash tests/run.sh"` apunta a archivo inexistente -> `kaizen check --only tests` FAIL (outbox `kaizen-test-cmd-inexistente.md`).
- F4 `arreglar-contador-deuda`: `kaizen check --only ratchet` -> `FALLA: deuda subio: 4 > 0` (literales `TODO` de codegen; outbox `kaizen-ratchet-deuda-codegen.md`). Verificado en esta rama: sigo en rojo (4 > 0).
- F2 `ADR-autonomia-nivel3`: coincide con ADR-0001 (item 2), pero no resuelve gobernanza (item 1).
### No bloqueante (registrado, sin supresion)
- `kaizen-security` aviso S-3 por URL loopback `127.0.0.1:4318` en test OTLP: justificado en outbox `kaizen-security-falso-hardcoding-test-url.md`; `kaizen check --only security` en verde. No se silencia (S-10).
### Verificacion del auditor
- `git diff --check` y escaneo de contenido: ver commit de entrega (sin secretos, sin emojis, sin atribucion a herramientas de IA).
- `scripts/kaizen-org check` ejecutado SOLO en modo validacion; `git status --short` limpio de cambios versionados antes de anadir estos dos artefactos.

## CIERRE HONESTO (2026-11-23) - KPI PARCIALMENTE CUMPLIDO

KPI: `kaizen check FAIL=0 y kaizen-gates framework+ratchet success`.
Resultado real: NO cumplido. Progreso:
- F1 (merge 1139dfc): `tests/run.sh` creado -> `ok: tests` (antes fallaba 127).
  `kaizen-gates/framework` debe pasar en el proximo CI.
- F2/F3/F4: escalados needs-human (ADR-0001, catalogo .kaizen/, deuda
  count_debt en scripts/kaizen*) — rutas protegidas L-4, agente no autorizado.
- `kaizen check` final: FALLO en 3 (swarm, gobernanza, deuda 4>0).
- `docs.yml:50` hash hardcodeado viejo -> workflow docs rojo (needs-human).
- `guardrails/derived`: corregido (b44c9b0), 39 derivados coinciden.
El proyecto se marco `closed` por el orquestador antes de tiempo; el KPI real
sigue abierto. Reabrir cuando haya autorizacion humana para los 4 items
protegidos (ver outbox S-11: docs-yml-hash, kaizen-f3-catalogo,
kaizen-ratchet-deuda, kaizen-swarm-adr0001, kaizen-gov).
