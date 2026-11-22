# Aprendizaje continuo: de errores a experiencia

Los agentes no repiten el mismo error dos veces sin dejar constancia. Cada fallo, blocker, revert o decisión se convierte en una **lección**; las lecciones se deduplican, se agregan en **experiencia** y, cuando se repiten, se promueven a **regla** de la definición del agente (ADR-0007).

## Ciclo

```
trabajar → fallo/decisión → kaizen-learn record → lección (fingerprint)
   ↑                                      ↓
   └── kaizen-org feed ← digest ← experiencia ← (occurrences ≥ 2) → apply → definición v+1
```

## Comandos

```bash
# Registrar (lo hace el runtime automáticamente en fallos; también puedes hacerlo a mano)
scripts/kaizen-learn record --agent cumplimiento-panama --kind blocker \
  --title "Fuente sin Gaceta" --lesson "Toda norma debe citar source_id verificado" --source "PA-001"

# Convertir lecciones en conocimiento del agente
scripts/kaizen-learn digest cumplimiento-panama
scripts/kaizen-learn digest all

# Promover lecciones repetidas a la definición (sube versión, requiere ADR)
scripts/kaizen-learn apply cumplimiento-panama

# Métricas
scripts/kaizen-learn report
scripts/kaizen-learn status
```

## Estructura

```
.kaizen/learning/
  learning.env
  lessons/<agente>/YYYY-MM-DD-NNN-<kind>.md   # fingerprint, occurrences, applied
  experience/<agente>.md                      # resumen saneado, alimentado al agente
  report.md
```

## Reglas

1. **Fingerprint estable**: `agente|kind|titulo`; si se repite, sube `occurrences` y se anota la recurrencia. Nunca se crea un duplicado.
2. **Evidencia obligatoria**: toda lección cita `source_ids` o un comando; si no, queda marcada como pendiente y no se promueve.
3. **Umbral de promoción**: 2 recurrencias antes de `apply`. `KL_AUTO_APPLY=0` por defecto: un humano aprueba con ADR.
4. **Ratchet**: `apply` sube la versión del agente con `kaizen-org save`; si la definición empeora, ADR-0006 bloquea.
5. **Límites**: el aprendizaje no cambia autonomía, vetos ni reglas de seguridad por repetición; eso siempre es decisión humana.

## Resultado esperado

Cuanto más se usa el equipo: más lecciones, más experiencia disponible antes de cada propuesta, definiciones más precisas y menos repetición de errores. Eso es mejora continua aplicada a los propios agentes.
