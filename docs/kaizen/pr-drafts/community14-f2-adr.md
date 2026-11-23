## Resumen
Registrar ADR-0001 de autonomia nivel 3 como decision ya vigente, sin cambiar autonomia

## Componente afectado
Kaizen / gobernanza (documentacion de ADR), tarea F2 del proyecto kaizen-gates-green

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [x] documentation
- [ ] refactor

## Documentos relacionados
- `docs/kaizen/adr/0001-autonomia-nivel-3.md` (nuevo; este cambio)
- `.kaizen/loop/LOOP.md` L-3/L-4 (fuente canonica, no modificada)
- `.kaizen/swarm/PROTOCOL.md` seccion 7; `docs/kaizen/adr/0002-...`, `0003-...`
- `.kaizen/loop/outbox/kaizen-swarm-adr0001-faltante.md` (hallazgo S-11 preexistente)
- CHANGELOG.md, `docs/pr-drafts/community14-f2-adr.md` (espejo para ag-check)

Refs: LOOP.md:9-13, PROTOCOL.md seccion 7, ADR-0002, ADR-0003, CONSENSUS.md:18, kaizen-swarm:253, F2/kaizen-gates-green.

## Plan de prueba
- `scripts/kaizen-swarm check` (verificar que desaparece `falta ADR-0001 (autonomia nivel 3)`)
- `bash scripts/ag-check`
- `git diff --check`

## Impacto en calidad (ratchet)
- Cobertura: igual (solo documentacion)
- Advertencias: igual
- Deuda (TODO/FIXME): igual

## Criterios de salida que avanza
- El archivo `docs/kaizen/adr/0001-autonomia-nivel-3.md` existe con Estado `aprobado (registro de decision ya vigente; no introduce cambios de autonomia)`.
- Los limites literales de L-3 y la lista de rutas protegidas de L-4 se incluyen verbatim como restricciones.
- `scripts/kaizen-swarm check` ya no reporta el fallo ADR-0001; cualquier otro fallo se registra abajo.

## Verificacion real

- `bash scripts/kaizen-swarm check` → exit 1. **Ya no reporta `falta ADR-0001 (autonomia nivel 3)`** (criterio de salida cumplido). Queda un fallo preexistente y ajeno a este cambio: `kaizen-gov check en rojo` — cuatro agentes sin gobernanza (`oss-observability-specialist`, `container-k8s-specialist`, `rust-lowlevel-specialist`, `rust-perf-benchmarker`). Ya registrado en `.kaizen/loop/outbox/kaizen-gov-especialistas-y-regresion.md`. No se toca `.kaizen/` (L-4); corresponde a la tarea F3.
- `bash scripts/ag-check` → exit 0, resumen `PASS=8 FAIL=0 SKIP=0`: `descriptors`, `fmt`, `clippy`, `test`, `audit`, `deny`, `masters` y `prohibited` en verde. En una ejecucion intermedia sobre la base anterior, `masters` estaba en rojo por SHA-256 desactualizado de `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`; otro agente lo corrigio en `main` antes de recrearse este worktree. Hallazgo historico en `.kaizen/loop/outbox/ag-check-falsos-positivos-push.md`.
- `git diff --check` → exit 0, sin salida.
- Nada se silencio, salto ni relajo (S-10/L-10). No se hizo push, PR ni merge.

## Aprendizaje (opcional)
La decision ya estaba aprobada y citada en L-3; el defecto era exclusivamente documental. Se evita redactar un ADR de cambio de autonomia (requeriria humano, L-4/K-13) y se senala como pendiente de decision humana la definicion cuantitativa de "bajo riesgo", que ninguna fuente canonica especifica. El hallazgo S-11 ya estaba registrado en el outbox del repo principal, por lo que no se duplica ni se toca `.kaizen/` (L-4). Durante la ejecucion el orquestador retiro el worktree y la rama originales; se recrearon desde `main` y se rehizo el trabajo (ver incidencia en el outbox).

## Cierre de issues
Closes: none

## Checklist
- [ ] Compila (no aplica: solo documentacion)
- [ ] Pasa tests (no aplica: solo documentacion)
- [ ] `scripts/kaizen-swarm check` sin el fallo ADR-0001
- [ ] Documentacion actualizada
- [ ] Linea base no empeora
