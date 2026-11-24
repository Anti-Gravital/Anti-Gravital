## Resumen
Reconcile parents #76 and #142 against the live board; additive wave-29 audit addendum, no closure.

## Componente afectado
Governance / issue-board audit (docs/audits); tracking parents #76 y #142.

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [x] documentation
- [ ] refactor

## Documentos relacionados
- docs/audits/issue-board-reconciliation.md (addendum wave 29)
- RFC-0011 / ADR-0012 (#76); CLAUDE.md rule 29 / PR #159, #163 (#142)

## Plan de prueba
- `git diff --check`
- `gh issue list --state all --limit 500 --json number,state` (recount 122 board issues)
- `gh issue view <n> --json number,state` para los 47 hijos (#78-#93, #125-#157 subset)
- `git merge-base --is-ancestor <commit> origin/main` para las 15 evidencias de #76
- Link check and ASCII/no-emoji/no-AI-attribution content scan

## Impacto en calidad (ratchet)
- Cobertura: igual (cambio solo de documentacion)
- Advertencias: igual
- Deuda (TODO/FIXME): igual

## Criterios de salida que avanza
- Matriz verificada de los 16 hijos de #76 (15 CLOSED, 1 OPEN: #87)
- Matriz verificada de los 31 hijos de #142 (26 CLOSED, 5 OPEN)
- Condicion exacta de cierre de cada padre documentada (todos los hijos verificados CLOSED)

## Aprendizaje (opcional)
`gh issue view` sigue roto por el sunset de Projects classic; la forma --json funciona. El tablero crecio de 114 a 122 issues, sin afectar a estos padres.

## Cierre de issues
Closes: none (Refs #76, Refs #142). Ningun padre se cierra con hijos abiertos.

## Checklist
- [ ] Compila (no aplica, solo documentacion)
- [ ] Pasa tests (no aplica, solo documentacion)
- [ ] `scripts/kaizen check` en verde
- [ ] Documentacion actualizada
- [ ] Linea base no empeora
