# El Presidente (dueño del negocio y del proyecto)

Toda la comunidad trabaja **para** y **en nombre de** El Presidente: la persona que usa esta herramienta. No es un rol más del organigrama: es la autoridad final a la que reportan los 42 agentes.

## Autoridad

| Decisión | Quién la toma |
| --- | --- |
| Objetivo, alcance y KPI del proyecto | El Presidente (lo aprueba o corrige) |
| Auto-merge de bajo riesgo y bugfix verificado | Equipo, bajo nivel 3 y gates verdes |
| Cumplimiento, dinero, datos personales, seguridad, rutas protegidas | El Presidente o humano delegado, siempre (L-3/L-4, ADR-0002/0003) |
| Cambio de autonomía, modelo, presupuesto o herramientas | El Presidente mediante ADR |
| Cierre de un proyecto | El Presidente, previa verificación del KPI |

## Cómo se comunica con la comunidad

- **Reportes salientes**: resumen diario, decisiones pendientes, bloqueos, hallazgos de cumplimiento y verificación del KPI. Se envían por el adaptador configurado (webhook → n8n → WhatsApp, correo SMTP o archivo local). Ver `.kaizen/team/notify.env`.
- **Instrucciones entrantes**: mensajes de WhatsApp (o Telegram/correo) llegan al buzón local `scripts/kaizen-inbox`, se convierten en archivos en `.kaizen/team/inbox/` y el `kaizen-equipo` los transforma en proyecto/tarea/decisión con `source_ids`.
- **Trato**: la comunidad habla en español neutro de Panamá, con lenguaje ejecutivo: estado, evidencia, riesgo, decisión requerida y plazo. Nada de "he terminado" sin evidencia (K-10).

## Reglas para los agentes

1. Todo mensaje a El Presidente incluye: **qué pasó, evidencia, impacto, qué necesita él y hasta cuándo**.
2. Si hace falta una decisión, se propone una opción recomendada con sus riesgos; nunca se pregunta sin contexto.
3. Si un agente no puede actuar, lo reporta como `blocker`/`needs-human` con fingerprints para deduplicar (S-11).
4. Los datos del Presidente (número, correo, tokens) **nunca** se versionan: solo variables de entorno (S-1/S-2).
5. El Presidente puede pausar/reanudar todo con `scripts/kaizen-runtime down|up` o `scripts/kaizen-start stop|start`.

## Reporte mínimo esperado (formato)

```
ESTADO: <verde/ámbar/rojo> — <proyecto>
LOGRO: <métrica antes → después, con evidencia>
RIESGO: <hallazgo con source_id o comando>
DECISIÓN REQUERIDA: <opción recomendada + plazo>
SIGUIENTE: <qué hará el equipo sin intervención>
```
