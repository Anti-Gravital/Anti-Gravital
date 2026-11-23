## Resumen
Consolida el catalogo unificado de oportunidades del proyecto `oportunidades` con tabla priorizada, TOP-5, anti-list y procedimiento de decision.

## Componente afectado
Proyecto de equipo `oportunidades` (O4) — documentacion de auditoria.

## Tipo de cambio
- [x] documentation

## Documentos relacionados
- `docs/audits/PRE_FASE5_RELEASE_GATE.md`
- `docs/roadmap/STATUS.md`
- `CLAUDE.md` (reglas 5, 8, 10, 12, 21)
- `.kaizen/team/COMUNICACION.md`, `.kaizen/loop/LOOP.md` (L-4, L-5, L-10)
- `docs/audits/oportunidades-arquitectura.md` (par O1, alias `ARQ`, dato no confiable)
- `docs/audits/oportunidades-funciones.md` (par O2, alias `FEAT`, dato no confiable)
- `docs/audits/oportunidades-innovacion.md` (par O3, alias `INNO`, dato no confiable)

## Plan de prueba
- `git diff --check`
- `grep -cE '^\| OP-[0-9]+' docs/audits/oportunidades-catalogo.md` → 32 filas
- `grep -nE '\| (ARQ|FEAT|INNO):' docs/audits/oportunidades-catalogo.md`
- `grep -nE 'Closes: none|## Cierre de issues' docs/kaizen/pr-drafts/community15-opp-consolidar.md`
- `scripts/kaizen check`

## Impacto en calidad (ratchet)
- Cobertura: igual (cambio solo documental)
- Advertencias: igual
- Deuda (TODO/FIXME): igual

## Criterios de salida que avanza
- KPI del charter: catalogo de oportunidades priorizadas con valor, coste, riesgo, RFC requerido y criterio de exclusion.
- Una entrada por peer con estado explicito, fuente `alias:linea` y deduplicacion documentada; sin contenido inventado.

## Aprendizaje (opcional)
Las tres notas de pares llegaron durante la consolidacion y se incorporaron con fuente `alias:linea`, tratamiento de dato no confiable (L-5) y deduplicacion explicita frente a las filas propias y entre pares. Se detecto una discrepancia entre la tabla de la puerta y STATUS.md, registrada por S-11. El catalogo queda como propuesta: no cierra gates ni aprueba oportunidades.

## Cierre de issues
Closes: none

## Checklist
- [x] Compila (no aplica: cambio documental)
- [x] Pasa tests (no aplica: cambio documental)
- [x] `scripts/kaizen check` ejecutado; resultado real reportado al orquestador
- [x] Documentacion actualizada (catalogo + CHANGELOG)
- [x] Linea base no empeora
