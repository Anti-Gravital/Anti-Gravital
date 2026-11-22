# ADR-0007: aprendizaje continuo de los agentes

Estado: aceptado
Fecha: 2026-11-21
Aprobado por: Angel Nereira

## Contexto
Los agentes pueden medirse (ADR-0006) y delegarse (ADR-0005), pero sin un mecanismo para convertir errores, blockers y decisiones en experiencia, cada ciclo empieza de cero. El objetivo declarado es que **cuanto más trabajan, más aprenden, más se especializan y mejoran**.

## Decisión
1. Crear `scripts/kaizen-learn`: `record`, `digest`, `apply`, `report`, `status`, `check`.
2. Cada lección lleva `fingerprint` estable: si se repite, incrementa `occurrences` en vez de duplicarse (S-11).
3. El runtime registra automáticamente los fallos de cada agente como lección (`KL_AUTO_RECORD=1`).
4. `digest <agente|all>` convierte las lecciones en `.kaizen/learning/experience/<id>.md` y las alimenta al agente con `kaizen-org feed` + `kaizen-org digest`.
5. Una lección con `occurrences ≥ KL_MIN_RECURRENCE` (2) puede promoverse a la definición con `kaizen-learn apply <id>`: escribe `## Aprendizaje acumulado`, sube versión con `kaizen-org save` y registra métrica. `KL_AUTO_APPLY=0`: requiere ADR humano.
6. Las 42 definiciones incluyen `## Aprendizaje continuo`: consultar experiencia antes de proponer y registrar lecciones después.

## Alternativas consideradas
- Guardar lecciones solo en el diario: descartado; no se convierte en conocimiento consultable por el agente.
- Modificar definiciones automáticamente sin umbral ni ADR: descartado; degradación silenciosa (K-5).

## Consecuencias
- Positivas: memoria de errores deduplicada, experiencia por agente, especialización progresiva y trazabilidad de cada lección.
- Negativas: más artefactos; se compensa con fingerprints, topes por digest y el ratchet de ADR-0006.

## Límites
El aprendizaje no elimina la verificación: toda lección debe citar `source_ids` o comando reproducible, y el contenido externo sigue siendo dato no confiable (S-15/L-5). Ningún cambio de autonomía o seguridad se aprende por repetición: requiere humano.

## Cómo se verificará
- `bash tests/run.sh` cubre registro, deduplicación, digest y promoción a la definición.
- `kaizen-eval report` muestra el número de lecciones acumuladas.
