# TEAM.md — contrato del equipo de agentes

Complementa `.kaizen/RULES.md`, `.kaizen/org/ORGANIZATION.md` y `.kaizen/swarm/PROTOCOL.md`. Define cómo los agentes trabajan **juntos**, no solo qué hace cada uno.

## Principios

1. **Escuadra, no multitud**: por proyecto se forma un equipo mínimo con un facilitador, un dueño de evidencia y los roles necesarios; el resto colabora por canal cuando se le convoca.
2. **Memoria compartida**: `blackboard.md` del proyecto contiene solo hechos verificados (`source_ids` o comandos reproducibles).
3. **Decisión = evidencia + consenso + registro**: sin `source_ids` verificados no hay decisión (`.kaizen/team/EVIDENCIA.md`).
4. **Comunicación con sobre**: `id`, `correlation_id`, `channel`, `intent`, `expects_ack`, `deadline`, `source_ids`.
5. **Proactividad con frenos**: iniciar hilos está permitido; auto-aprobar, cambiar etapa o publicar sin humano en áreas sensibles no (L-3/L-4).
6. **Rotación de turnos y acuses**: evita que dos agentes escriban a ciegas y que un handoff muera sin responder.

## Rol del facilitador

`facilitador-equipo`: convoca standups, asigna tareas, vigila acuses y leases, ordena discrepancias y escala a humano. No decide por los especialistas salvo en `resource`.

## Role del verificador

`verificador-evidencias`: valida `source_ids`, hashes y fechas de corte; ejecuta `kaizen-team verify`; puede bloquear decisiones con fuente `pending`, `conflict` o `stale`.

## Role del monitor normativo

`monitor-gaceta`: vigila Gaceta Oficial y órganos competentes, abre expedientes `kaizen-org research` y emite `alert` cuando cambia una norma.

## Interfaces con el resto del sistema

- `.kaizen/swarm/` gestiona la ejecución por issue y worktrees.
- `.kaizen/org/` gestiona el inventario, conocimiento, consenso y capacidades.
- `.kaizen/team/` gestiona la **coordinación continua del proyecto** y la evidencia de decisiones.
- `scripts/kaizen-team` es el orquestador de equipo; todo lo que hace es auditable en archivos versionados.
