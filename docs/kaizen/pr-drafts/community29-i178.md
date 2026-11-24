## Resumen
Revalidate deferred issues #178, #152 and #154 with honest gate evidence and exact close conditions; keep all three open.

## Componente afectado
Documentacion de auditoria (issues #178, #152, #154; proyecto atender-issues).

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [x] documentation
- [ ] refactor

## Documentos relacionados
- `docs/security/REAL_MAILBOX_VERIFICATION.md` y `docs/security/DKIM_INDEPENDENT_VERIFICATION.md`
- `docs/architecture/ag-mail-managed-api-design.md` y `docs/modules/ag-mail/managed-api-dx-design.md`
- RFC-0025, ADR-0020 y `docs/modules/ag-data/phase5-implementation-readiness.md`
- `docs/audits/PRE_FASE5_RELEASE_GATE.md` y `docs/audits/issue-board-reconciliation.md`
- Nuevo: `docs/audits/fase5-deferred-issues-revalidation.md`

## Plan de prueba
- `bash scripts/check-mail-delivery.sh` -> `RESULT: SKIP reason=not-opted-in`, exit 0
- `AG_MAIL_DELIVERY_OPT_IN=1 bash scripts/check-mail-delivery.sh` -> SKIP `probe-target-not-configured`, exit 0
- `AG_MAIL_DELIVERY_OPT_IN=1 AG_MAIL_DELIVERY_PROBE_TARGET=127.0.0.1:25 bash scripts/check-mail-delivery.sh` -> SKIP `port-25-egress-unavailable`, exit 0
- `bash scripts/check-dkim-independent.sh` -> PASS solo offline (vectores dkimpy); NO es evidencia de buzón real
- `git diff --check`
- `scripts/kaizen check`
- `scripts/ag-check`

## Impacto en calidad (ratchet)
- Cobertura: igual (cambio solo documental)
- Advertencias: igual
- Deuda (TODO/FIXME): igual

## Criterios de salida que avanza
- Ninguna casilla de #178, #152 ni #154 se marca como completada; los tres siguen OPEN.
- Ninguna fila de `docs/audits/PRE_FASE5_RELEASE_GATE.md` cambia de estado; Fuzz 24h, Benchmarks y Open debt siguen `pending`.
- Se documenta la brecha exacta: buzón real (preparado, no ejecutado) y autorizacion de Fase 5 (preparada, no ejecutada).
- No se fabrica ningun `Authentication-Results` ni se escribe ningun secreto.

## Aprendizaje
El wrapper de DKIM puede PASS con dkimpy instalado y seguir sin probar entrega real: un PASS offline no debe leerse como cierre de puerta. Solo el checker de buzón real con `scope=real-mailbox` puede aportar esa evidencia.

## Cierre de issues
Closes: none
Refs #178
Refs #152
Refs #154

## Checklist
- [x] Compila (sin cambios de codigo)
- [x] Pasa tests (verificaciones documentales registradas)
- [ ] `scripts/kaizen check` en verde
- [x] Documentacion actualizada
- [x] Linea base no empeora
