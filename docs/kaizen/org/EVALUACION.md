# Evaluación de agentes, métricas de decisión y ratchet

Los agentes también se degradan: definiciones incompletas, KPI no medibles, knowledge desactualizado o autonomía ambigua. Esta capa aplica el principio K-5 a la **calidad de los propios agentes** (ADR-0006).

## 1. Las 10 dimensiones (10 puntos cada una)

| Dimensión | Qué comprueba |
| --- | --- |
| `proposito` | Sección Propósito presente, no plantilla, suficientemente concreta |
| `responsabilidades` | Al menos dos responsabilidades verificables |
| `kpi_medible` | Evidencia y KPI con número, porcentaje, `source_ids` o verificación |
| `evidencia` | Exige `source_ids`, `verificador-evidencias` o `kaizen-team verify` |
| `autonomia` | `autonomy` + `decision_authority` + humano en decisiones sensibles |
| `colaboracion` | `correlation_id` y ack en handoffs |
| `recursion` | Delegación con `KAZEN_AGENT_DEPTH` y límite `KR_MAX_DEPTH` |
| `conocimiento` | Todas las rutas de `knowledge` existen en el repositorio |
| `seguridad` | No credenciales y contenido externo datado/no confiable (S-1/S-15) |
| `reglas_vigentes` | Reglas K-1..K-14, L-1..L-17 y S-1..S-16 |

Umbral aprobado: **70/100** (`KE_MIN_SCORE`). Estado actual: 42 agentes, promedio 93.1.

## 2. Comandos

```bash
scripts/kaizen-eval check                 # puntúa los 42 agentes y falla si alguno < umbral
scripts/kaizen-eval baseline --force      # fija la línea base tras mejora o ADR
scripts/kaizen-eval ratchet               # falla si cualquier agente empeoró
scripts/kaizen-eval report                # reporte por departamento + métricas
scripts/kaizen-eval metrics add --agent cumplimiento-panama --decision DEC-0007 \
  --cost 0.12 --minutes 18 --reverted 0
scripts/kaizen-eval all                   # check + ratchet + report
```

## 3. Ratchet

- Una definición **no puede bajar de puntuación** sin ADR y `baseline --force`.
- Una mejora sube la línea base directamente (`baseline` sin `--force` lo permite).
- `kaizen check --only eval` y `kaizen-swarm check` ejecutan la puerta; un agente degradado bloquea la integración.

## 4. Métricas de decisión (observadas, no estimadas)

Cada decisión verificada puede registrar:

- `cost_usd`: coste de modelo/herramientas.
- `minutes`: tiempo desde propuesta hasta `verify` en verde.
- `reverted`: 1 si hubo revert (L-9).

El reporte agrega **coste medio por decisión**, **tiempo medio a verificación** y **tasa de revert**. Sin datos, muestra 0 y lo dice; nunca inventa cifras (K-10).

## 5. Mejora continua

1. La retro (K-9) identifica el agente con peor puntuación o mayor coste.
2. Se copia/especializa con `kaizen-org copy`, se refina y se guarda con `kaizen-org save` (sube versión).
3. Se registra una métrica nueva y se ejecuta `kaizen-eval all`.
4. Si la puntuación sube, `baseline` fija el nuevo mínimo. Si baja, ADR + humano.

Esto convierte al equipo en un sistema que **se mide y se auto-mejora**, con evidencia en `.kaizen/evals/`.
