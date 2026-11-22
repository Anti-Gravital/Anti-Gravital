# Diario Kaizen — 2026-11-22 17:20 UTC

## Tarea
Instalar Kaizen en modo local en Anti-Gravital-org, levantar cuatro equipos alineados al proposito de hacer operativo y funcional el proyecto, y avanzar la ola de trabajo aprobada por el humano (borradores RFC + seguimiento local, sin remoto).

## Decision
- El framework canónico existe en `/home/angel-nereira/Github-local-angelnereira/opencode` (VERSION 1.0.0). Se instalo solo en este repo con `scripts/kaizen-installer repo <destino>`.
- No se reinstalaron los agentes globales: ya estan presentes y son mas recientes (47 vs 21 definidos por el framework). Reiniciarlos seria una degradacion. No se modifications `opencode.json` global.
- No se ejecuto `git push`, `git add`, `git commit` ni ninguna accion remota. Nada de Kaizen se publica.
- Se uso el sustrato existente (docs/rfc, docs/adr, Issues de GitHub ya presentes) mas el orquestador Kaizen local para tareas, no se invento infraestructura adicional.
- Para secretos se respeto RFC-0020 (aceptada): no se crea crate nuevo; se extienden `ag-core` y `ag-cli`.
- Para benchmarks se adopto la decision humana: escalera de concurrencia 200 -> 1k -> 5k, RPS como resultado medido, hardware informativo no publicable hasta hardware de referencia.

## Evidencia (comandos y resultados reales)
- `scripts/kaizen-installer check` -> `todo en verde`.
- `scripts/kaizen-installer repo <destino>` -> `126 archivos/bloques creados, 4 ya existentes (no sobrescritos)`; `23 agentes instalados` en `.opencode/agents`.
- `scripts/kaizen-installer status` -> `agentes definidos: 21`, `globales: 47`.
- `scripts/kaizen-team check` -> `todo en verde (23 fuentes verificadas)`.
- `scripts/kaizen check` -> FALLO en cuatro puertas: tests, swarm, organizacion, ratchet (deuda 4 > 0). Causas documentadas en `.kaizen/loop/outbox/`:
  - `kaizen-test-cmd-inexistente.md` (KZ_TEST_CMD apunta a `tests/run.sh` inexistente).
  - `kaizen-swarm-adr0001-faltante.md` (falta ADR-0001 en el bootstrap).
  - `kaizen-org-catalogo-desincronizado.md` (43 vs 21).
  - `kaizen-ratchet-deuda-codegen.md` (TODO en literales de codegen).
  - `kaizen-team-assign-handoff-jq.md` (bug `now` vs `$now` en `kaizen-team assign`).
- Cuatro proyectos creados con `kaizen-team init` y standup del dia; tareas asignadas con `kaizen-team assign`; `sync` reporta 1 tarea abierta por proyecto y etapa `execute`.

## Entregables
- Instalacion local de Kaizen (sin remoto).
- Cuatro equipos: `secretos-config-tui`, `observabilidad-proyecto`, `benchmark-carga`, `errores-diagnosticabilidad`, cada uno con KPI verificable, miembros y tarea asignada.
- Tres borradores de RFC: RFC-0037 (error contract), RFC-0038 (scopes y health de observabilidad), RFC-0039 (protocolo de carga y medicion de instalacion). RFC-0020 sigue siendo la decision vigente para secretos.
- Indice `docs/rfc/README.md` actualizado con los tres borradores.

## Resultado
Instalacion completada y equipos levantados. No se inicio implementacion de codigo ni instalacion de tooling (requiere aprobacion humana). `kaizen check` sigue en rojo por fallos de configuracion del framework y del repo preexistentes a esta iteracion; no se relajo ni silencio nada (K-5/L-10).

## Aprendizaje
Instalar un framework de gobernanza en un repo maduro puede introducir fallos de puerta por desalineacion entre la configuracion por defecto y la realidad del repositorio (comandos de test inexistentes, catalogos desincronizados, heuristicos de deuda que cuentan cadenas generadas). La respuesta correcta es documentar con evidencia y escalar a humano, nunca relajar la puerta.

## Decisiones humanas pendientes
- Arreglar `KZ_TEST_CMD`, ADR-0001, sincronizacion de catalogo y baseline de deuda (rutas protegidas L-4).
- Corregir el bug `now` -> `$now` en `scripts/kaizen-team` (ruta protegida L-4).
- Aprobar o rechazar RFC-0037, RFC-0038 y RFC-0039.
- Autorizar instalacion de `oha`, hardware de referencia y despliegue de collectors/Grafana si se decide.
