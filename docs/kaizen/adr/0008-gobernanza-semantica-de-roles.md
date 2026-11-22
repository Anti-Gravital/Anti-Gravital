# ADR-0008: gobernanza semántica de roles, obligaciones y participación

Estado: aceptado
Fecha: 2026-11-21
Aprobado por: Angel Nereira

## Contexto
Los agentes tenían prompts ricos en responsabilidades, pero no una forma legible por máquina de responder: **quién ejecuta, quién responde, quién consulta, quién informa, quién puede vetar y cuándo interviene un humano** en decisiones, despliegues, cambios, incidentes y cumplimiento. Sin eso, la colaboración depende de interpretación del modelo.

## Decisión
1. Crear `.kaizen/gov/governance.json` como fuente de verdad semántica: 9 procesos (decisión, despliegue, cambio, incidente, cumplimiento, investigación, ciclo de agente, presupuesto y retro), etapas con RACI (R/A/C/I) + V (veto) + H (humano), puerta de avance y obligaciones compartidas.
2. Cada agente declara `responsibilities`, `obligations`, `participates_in` y `decision_rights` (approve/recommend/execute/veto/escalate), derivados de su departamento, autonomía y veto.
3. Cada definición incluye `governance`, `obligations_ref`, `participation_ref` y la sección `## Participación estructural`.
4. `scripts/kaizen-gov` expone `check`, `raci`, `who`, `can`, `obligations`, `report`, `status` y `regenerate`.
5. El humano —**El Presidente**— es la autoridad final (`A`) en despliegues, cumplimiento, presupuesto y decisiones sensibles. Los vetos de seguridad, arquitectura, QA, cumplimiento, alcance y recurso se mantienen (ADR-0002/0003).

## Alternativas consideradas
- RACI solo en prosa: descartado; no es verificable ni consultable por un agente.
- RACI estático sin derivación de las definiciones: descartado; se desincroniza al copiar o especializar agentes.

## Consecuencias
- Positivas: decisiones y despliegues con responsable explícito, consulta automatizable (`kaizen-gov can`) y escalado humano verificable.
- Negativas: un artefacto más que mantener; se regenera con `kaizen-gov regenerate` desde las definiciones y se valida en `kaizen check`.

## Límites
La semántica no otorga autonomía nueva: `approve` solo existe donde la autonomía y el ADR lo permiten. Ningún proceso puede quitar el humano en seguridad, dinero, datos personales, cumplimiento o rutas protegidas (L-3/L-4).

## Cómo se verificará
- `kaizen-gov check` y `kaizen check --only gov` en verde.
- `bash tests/run.sh` valida matriz RACI, `can` y detección de rol desconocido.
