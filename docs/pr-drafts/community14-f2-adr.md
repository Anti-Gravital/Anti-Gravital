## Resumen
Registrar ADR-0001 de autonomia nivel 3 como decision ya vigente, sin cambiar autonomia

Nota: este archivo es el espejo que exige `scripts/ag-check` (`docs/pr-drafts/<rama-aplanada>.md`). El descriptor canonico K-4 esta en `docs/kaizen/pr-drafts/community14-f2-adr.md`; ambos declaran el mismo cierre y las mismas pruebas.

## Componente afectado
Kaizen / gobernanza (documentacion de ADR), tarea F2 del proyecto kaizen-gates-green

## Tipo de cambio
- [x] documentation

## Documentos relacionados
- `docs/kaizen/adr/0001-autonomia-nivel-3.md` (nuevo)
- `docs/kaizen/pr-drafts/community14-f2-adr.md` (descriptor canonico)

Refs: LOOP.md:9-13, PROTOCOL.md seccion 7, ADR-0002, ADR-0003, F2/kaizen-gates-green.

## Plan de prueba
- `scripts/kaizen-swarm check`
- `bash scripts/ag-check`
- `git diff --check`

## Impacto en calidad (ratchet)
- Cobertura, advertencias y deuda: igual (solo documentacion).

## Criterios de salida que avanza
- ADR-0001 existe y `scripts/kaizen-swarm check` ya no reporta `falta ADR-0001 (autonomia nivel 3)`.

## Verificacion real

- `bash scripts/kaizen-swarm check` → exit 1, sin el fallo ADR-0001; queda `kaizen-gov check en rojo` (preexistente, F3, ya en outbox).
- `bash scripts/ag-check` → exit 0, `PASS=8 FAIL=0 SKIP=0`, todas las comprobaciones en verde (incluido `masters`).
- `git diff --check` → exit 0. Nada se silencio ni relajo; sin push/PR/merge.

## Aprendizaje (opcional)
Decision documental, no cambio de autonomia; detalle de "bajo riesgo" pendiente de decision humana.

## Cierre de issues
Closes: none

## Checklist
- [ ] `scripts/kaizen-swarm check` sin el fallo ADR-0001
- [ ] Documentacion actualizada
- [ ] Linea base no empeora
