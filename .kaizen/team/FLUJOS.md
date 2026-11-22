# Modelo de flujo de trabajo del equipo

Estado por proyecto en `.kaizen/team/projects/<slug>/state.json`. El orquestador (`kaizen-team`) mueve el estado; los agentes proponen, no se auto-avanzan.

## Máquina de estados

```
intake → plan → squad → execute → review → decide → deliver → verify → closed
   ↑                                                        │
   └───────────── needs-human / regression-suspect ─────────┘
```

| Etapa | Dueño | Entrada | Salida | Puerta para avanzar |
| --- | --- | --- | --- | --- |
| `intake` | facilitador | objetivo + restricciones | `charter.md` con KPI y alcance | KPI verificable |
| `plan` | arquitecto + analista-negocios | charter | `plan.md` con tareas y dependencias | contratos en `contracts/` |
| `squad` | gerente | plan | escuadra asignada con roles y channels | todos los miembros activos |
| `execute` | devs + especialistas | tareas con lease | artefactos + handoffs | acuses de handoff |
| `review` | QA + seguridad + cumplimiento + DX | diff consolidado | dictámenes en `reviews/` | sin BLOCK abierto |
| `decide` | facilitador + dueños | propuesta + `source_ids` | decisión con hash y fecha | `kaizen-team verify` en verde |
| `deliver` | Dev5 + finops | decisión aprobada | descriptor + PR del orquestador | `kaizen check` verde |
| `verify` | QA + cumplimiento | cambio fusionado | métrica antes/después (L-9) | KPI mejoró o rollback |
| `closed` | gerente | verificación | retro y lecciones | — |

## Proactividad

- Cada agente tiene `triggers` y `channels`. Ante un disparador, **emite** una propuesta/finding; no espera a que le pidan.
- El facilitador publica `standup` diario y detecta tareas sin dueño o con lease caducado (`kaizen-swarm reap`).
- El monitor de Gaceta detecta cambios normativos y emite `alert` con `source_ids`; eso puede reabrir un proyecto `closed` en `review`.
- El verificador de evidencia revalida fuentes trimestralmente y marca `stale` cuando vence la fecha de corte.

## Autonomía vs. humano

Nivel 3 permite auto-merge de bajo riesgo y bugfix con regresión. **Nunca** sin humano: cumplimiento, dinero, datos personales, seguridad, rutas protegidas, cambio de autonomía y decisiones con fuentes en `pending`/`conflict` (REG-EV-01, L-3/L-4, ADR-0002/0003).

## Fallos

- `verify` en rojo → etapa vuelve a `review` y se abre `needs-human`.
- KPI no mejora → `regression-suspect` + revert propuesto (L-9).
- Agente caído → lease reaperado y tarea reasignada; el proyecto no se detiene.
- Ningún fallo se abandona en silencio: S-11.
