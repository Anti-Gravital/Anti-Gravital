# 2026-11-23 Fase 5 - revalidacion de issues diferidos

- Tarea: revalidar #178, #152 y #154 (proyecto atender-issues).
- Decision: solo auditoria aditiva. No se implementa Fase 5, no se envia correo,
  no se cierra ningun issue y no se fabrica evidencia de buzón real.

## Evidencia (comandos y resultados reales)
- `gh api repos/Anti-Gravital/Anti-Gravital/issues/{178,152,154}`: los tres OPEN;
  etiquetas `tech-debt,blocked,p2,ag-mail` (#178),
  `enhancement,tech-debt,p2,ag-mail` (#152) y
  `enhancement,tech-debt,p3,ag-data` (#154).
- `bash scripts/check-mail-delivery.sh`: `RESULT: SKIP reason=not-opted-in`, exit 0.
- `AG_MAIL_DELIVERY_OPT_IN=1 bash scripts/check-mail-delivery.sh`: SKIP
  `probe-target-not-configured`, exit 0.
- Opt-in con `AG_MAIL_DELIVERY_PROBE_TARGET=127.0.0.1:25`: SKIP
  `port-25-egress-unavailable`, exit 0.
- `bash scripts/check-dkim-independent.sh`: `RESULT: PASS` con dkimpy 1.1.8 y
  PyNaCl 1.6.2 sobre dos vectores offline. NO es evidencia de entrega real ni
  de cabecera `Authentication-Results`; se registra como tal.
- Gate pre-Fase 5: Audit/Deny `pass` (commit `800b671`); Fuzz 24h, Benchmarks y
  Open debt `pending` (`docs/audits/PRE_FASE5_RELEASE_GATE.md`).

## Resultado
- Anadido `docs/audits/fase5-deferred-issues-revalidation.md` con estado actual,
  brecha exacta, condicion de cierre por issue y advertencia explicita
  "preparado pero no ejecutado" para buzón real y autorizacion de Fase 5.
- Descriptor `docs/kaizen/pr-drafts/community29-i178.md` y espejo
  `docs/pr-drafts/community29-i178.md` con `Closes: none` y `Refs #178/#152/#154`.
- Sin cambios de codigo ni de estado de gate. Los tres issues quedan OPEN.

## Aprendizaje
Un verificador offline puede PASS y no probar la puerta externa. La honestidad
consiste en separar "verificacion de vectores" de "respuesta de un MTA
receptor": solo `RESULT: PASS scope=real-mailbox` cierra #178 y habilita el
gate Mail A de #152.
