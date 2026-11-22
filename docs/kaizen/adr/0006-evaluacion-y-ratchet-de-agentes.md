# ADR-0006: evaluación y ratchet de la calidad de los agentes

Estado: aceptado
Fecha: 2026-11-21
Aprobado por: Angel Nereira

## Contexto
La comunidad tiene 42 agentes con definiciones versionadas, pero sin una medida objetiva de calidad. Sin evaluación, una copia mal hecha, un KPI no medible o una referencia de conocimiento rota pueden degradar al equipo sin que nadie lo detecte. K-5 aplica al código; faltaba aplicarlo a los agentes.

## Decisión
1. Crear `scripts/kaizen-eval` con 10 dimensiones puntuables (0-10 cada una), umbral 70/100 y reporte por departamento.
2. Fijar `.kaizen/evals/baseline.json` como línea base; `ratchet` falla si cualquier agente empeora o si el promedio baja. Solo se relaja con ADR y `baseline --force`.
3. Registrar métricas observadas de decisiones (coste USD, minutos a verificación, revert) en `.kaizen/evals/metrics.jsonl`.
4. Integrar la puerta en `kaizen check --only eval` y en `kaizen-swarm check`.
5. Documentar la metodología en `docs/kaizen/org/EVALUACION.md`.

## Alternativas consideradas
- Evaluar solo con pruebas de ejecución: descartado; no detecta definiciones ambiguas ni knowledge roto.
- No medir coste/tiempo: descartado; la proactividad sin coste puede volverse ruido (KAIZEN-LOOP §11).

## Consecuencias
- Positivas: degradación detectable, priorización de mejoras por score y métricas reales para decidir autonomía.
- Negativas: un pase de enriquecimiento inicial fue necesario para elevar 19 definiciones al umbral; queda registrado en el diario y la línea base se recalibró con `--force` como mejora (no como relajación).

## Límites
La puntuación mide **la definición**, no el comportamiento en producción. Las métricas de decisión son observacionales: si no se registran, el reporte muestra 0 y lo declara. Ningún número se inventa (K-10).

## Cómo se verificará
- `bash tests/run.sh` incluye check, baseline, ratchet, métricas y detección de regresión.
- `kaizen check` y `kaizen-swarm check` en verde con la puerta `eval`.
