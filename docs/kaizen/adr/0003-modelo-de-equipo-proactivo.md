# ADR-0003: modelo de equipo proactivo con evidencia verificada

Estado: aceptado
Fecha: 2026-11-21
Aprobado por: Angel Nereira (responsable del repositorio)

## Contexto
El inventario de agentes y el swarm por issue cubren la ejecución, pero no bastan para que un proyecto se ejecute como un **equipo**: faltan canales, turnos, acuses, estado compartido, reuniones asíncronas y una puerta que impida decidir con base en documentos no verificados (por ejemplo, una norma citada sin Gaceta Oficial).

## Decisión
1. Añadir `.kaizen/team/` con `TEAM.md`, `COMUNICACION.md`, `FLUJOS.md` y `EVIDENCIA.md`.
2. Crear `scripts/kaizen-team` con: `init`, `standup`, `assign`, `sync`, `decide`, `verify`, `retro`, `status`, `check`.
3. Máquina de estados por proyecto: `intake → plan → squad → execute → review → decide → deliver → verify → closed`, con escalado `needs-human`.
4. Toda decisión declarada con `source_ids` debe pasar `kaizen-team verify` contra `.kaizen/org/knowledge/SOURCES.md`; fuente `pending`, `conflict` o `stale` bloquea (REG-EV-01).
5. Enriquecer las definiciones de los agentes con `channels`, `squad_role`, `decision_authority` y una sección `## Colaboración`; añadir los puestos `facilitador-equipo`, `verificador-evidencias` y `monitor-gaceta`.

## Alternativas consideradas
- Un único agente "manager" que lo decide todo: descartado; concentra fallos y carece de contraste de evidencia.
- Comunicación informal en chat: descartado; no es auditable ni reanudable por otro agente.

## Consecuencias
- Positivas: colaboración trazable, decisiones verificables contra fuente oficial (Gaceta Oficial incluida), reanudación por cualquier agente al leer archivos.
- Negativas: más artefactos y una puerta más; se compensa con comandos que generan plantillas y validación automática.

## Límites
La autonomía 3 no elimina el humano en cumplimiento, dinero, datos personales, seguridad ni rutas protegidas. Los contenidos externos siguen siendo datos no confiables (S-15/L-5).

## Cómo se verificará
- `bash tests/run.sh` comprueba el flujo completo y que una decisión sin fuente verificada se bloquea.
- `kaizen-team check`, `kaizen check`, `kaizen-org check` y `kaizen-swarm check` en verde.
- Catálogo regenerado con los 32 agentes y consistente con el registro.
