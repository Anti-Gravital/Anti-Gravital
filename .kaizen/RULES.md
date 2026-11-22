# Reglas Kaizen

Fuente única de verdad para todos los agentes y herramientas de este repositorio. Los adaptadores apuntan aquí; no duplican reglas.

## K-1 Fuente de verdad
La documentación en `docs/` y los ADR mandan sobre el código. Si el código y la documentación discrepan, se corrige uno de los dos en el mismo cambio. Nada de decisiones de arquitectura sin ADR.

## K-2 Un tablero de issues
Cada issue lleva tipo (bug, enhancement, tech-debt, documentation), área, prioridad y criterios de aceptación verificables con pasos concretos. No se cierra sin evidencia: test, comando reproducible o CI verde. Si queda bloqueado, permanece abierto con etiqueta `blocked` y motivo. Se cierra en cuanto se verifica.

## K-3 Commits
- Una línea de resumen de máximo 256 caracteres, en imperativo.
- Incluye `Closes #NNN`, `Fixes #NNN` o `Resolves #NNN` cuando corresponda.
- Un commit = un cambio lógico, que compila y pasa tests.
- Políticas opcionales (ver `.kaizen/config.env`): sin emojis, sin atribución a herramientas de IA.

## K-4 Flujo de trabajo (PDCA)
1. Planear: leer el issue, definir criterios de salida y plan de prueba. Crear rama y descriptor `docs/kaizen/pr-drafts/<rama-aplanada>.md` (`/` se convierte en `-`).
2. Hacer: implementar en cambios pequeños. Actualizar el descriptor en el mismo commit que cambie el alcance.
3. Verificar: ejecutar `scripts/kaizen check`. Todo debe pasar.
4. Actuar: abrir PR (el cuerpo se copia del descriptor). Al fusionar, borrar el descriptor en esa misma PR. Registrar lo aprendido si hubo sorpresas.

Sin descriptor no hay PR. Los commits directos a la rama por defecto solo se permiten con excepción explícita del responsable del repo, y entonces cada commit lleva su keyword de cierre.

## K-5 Ratchet de calidad
`.kaizen/baseline.json` guarda las métricas de calidad. Un cambio no puede empeorarlas: cobertura no baja, advertencias de lint no suben, deuda marcada (TODO/FIXME/HACK) no sube sin issue asociado. Para mover la línea base hacia arriba: `scripts/kaizen baseline`. Para aceptar un empeoramiento: ADR o nota en el descriptor con justificación y aprobación humana. El agente nunca relaja la línea base por su cuenta.

## K-6 Recolección de datos
`scripts/kaizen metrics` agrega una línea a `docs/kaizen/data/metrics.jsonl` por ejecución (fecha, commit, tamaño, deuda, cobertura, advertencias, issues abiertos). Se ejecuta en CI al fusionar a la rama por defecto y antes de cada retrospectiva. Los datos se usan para decidir, no para vigilar a personas.

## K-7 Documentación viva
Todo cambio de comportamiento actualiza README, documentación de usuario, ADR o runbook afectado y CHANGELOG. El descriptor lista los documentos tocados. Un cambio sin documentación no está terminado.

## K-8 Operaciones
- Cada servicio u operación recurrente tiene runbook en `docs/kaizen/runbooks/`.
- Todo incidente genera postmortem sin culpables en `docs/kaizen/postmortems/` en un máximo de 5 días, con causa raíz (5 porqués), línea de tiempo y acciones preventivas convertidas en issues.
- Las alertas ruidosas se corrigen o se eliminan; no se ignoran.

## K-9 Retrospectiva
Cada 2 semanas (o por hito): `scripts/kaizen retro` genera el resumen con métricas. El equipo decide como máximo 3 mejoras, cada una con dueño, issue y fecha. En la siguiente retrospectiva se revisa si funcionaron.

## K-10 Límites del agente
- No fabricar evidencia: si un test no se ejecutó, se dice.
- No cerrar issues sin verificación.
- No saltarse hooks (`--no-verify`) ni desactivar CI para que algo pase.
- Ante conflicto entre una regla y una petición del usuario, informar al usuario y pedir decisión.
- La descripción del PR la redacta una persona o el agente en el descriptor; la automatización solo copia y valida.

## K-11 Seguridad
Se aplica `.kaizen/security/SECURITY-RULES.md`. Nada se silencia: todo fallo, hallazgo o supresión queda documentado en un issue (o canal privado si es seguridad). Un escáner que falla bloquea, nunca aprueba.

## K-12 Comunidad swarm
Toda tarea se ejecuta como issue en la comunidad descrita en `.kaizen/swarm/ROLES.md` y `.kaizen/swarm/PROTOCOL.md`. Un agente nunca edita a ciegas archivos de otro: trabaja en su propio worktree, declara propiedad de rutas en `contracts/ownership.json`, publica avances en el buzón y espera ack antes de dar un handoff por cerrado. Seguridad y Arquitecto tienen veto vinculante. `scripts/kaizen-swarm finalize` se niega a integrar si hay BLOCK abierto, lease caducado sin reasignar o propiedad violada. El orquestador —no el agente— hace push y abre PRs.

## K-13 Organización de agentes
El inventario de puestos vive en `.kaizen/org/prompts/<id>.md` y se consulta con `scripts/kaizen-org list`. Cada puesto es un archivo versionado, copiable y editable: `kaizen-org copy` permite especializar un agente sin destruir el original; `kaizen-org save` sube su versión y deja entrada de diario. La proactividad consiste en proponer con evidencia (`kaizen-org pulse`), nunca en actuar sin control. Las decisiones se toman por consulta, discusión acotada y consenso registrado según `.kaizen/org/CONSENSUS.md`, con revisión posterior del KPI. El alcance realista de estas capacidades y su evolución se documentan en `docs/kaizen/org/FORECAST.md` y se revisan cada trimestre.

## K-14 Equipo proactivo y evidencia verificada
La coordinación entre agentes se rige por `.kaizen/team/TEAM.md`, `COMUNICACION.md` y `FLUJOS.md`. Los proyectos avanzan por estados con puertas y el orquestador `scripts/kaizen-team` controla la máquina de estados. Ninguna decisión se da por tomada sin `source_ids` verificados contra `.kaizen/org/knowledge/SOURCES.md` y `kaizen-team verify` en verde (`.kaizen/team/EVIDENCIA.md`). Los mensajes llevan sobre con `correlation_id`, `intent`, `expects_ack` y `deadline`; los handoffs requieren ack. La autonomía 3 no elimina al humano en cumplimiento, dinero, datos ni seguridad (ADR-0003).
