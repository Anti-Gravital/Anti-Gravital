# Resumen

Add a reproducible real-mailbox verification kit for ag-mail with an opt-in, honest delivery gate checker; keep issue #178 open.

- Fase afectada: no altera fases del roadmap; completa la documentacion de verificacion de la puerta externa de la Fase 4.6-A (ola 12).
- Tipo de cambio: [x] documentacion y procedimiento de verificacion  [x] script opt-in  [ ] codigo de crate  [ ] DSL  [ ] API publica.
- Documentos relacionados: `docs/security/REAL_MAILBOX_VERIFICATION.md`,
  `docs/security/REAL_MAILBOX_RESULTS_TEMPLATE.md`,
  `docs/security/DKIM_INDEPENDENT_VERIFICATION.md`,
  `docs/audits/dkim-independent-ci.md`, `docs/audits/mail-delivery-gate.md`,
  RFC-0009 secciones 4.3 y 4.8,
  RFC-0035, `CHANGELOG.md`, CLAUDE.md reglas 17, 18, 19, 26 y 28.

## Cambios

- `docs/security/REAL_MAILBOX_VERIFICATION.md` (nuevo): procedimiento paso a
  paso para ejecutar `MtaSender` contra un sumidero local de bucle y contra un
  buzón real; campos exactos de `Authentication-Results` (`header.d`,
  `header.s`, `header.a`, `header.b`) y de `DKIM-Signature` (`d=`, `s=`, `a=`,
  `b=`); tabla de resultados del checker y condicion exacta de cierre de #178.
- `docs/security/REAL_MAILBOX_RESULTS_TEMPLATE.md` (nuevo): plantilla vacia,
  sin inferencia de valores y con estado `gate_claimed=false`.
- `scripts/check-mail-delivery.sh` (nuevo): checker opt-in con PASS/SKIP/FAIL
  honestos. SKIP si no hay opt-in, egress de puerto 25, credenciales o
  evidencia; FAIL si la evidencia existe pero es contradictoria; PASS de
  `wire` siempre imprime `gate.real_mailbox=NOT_SATISFIED`.
- `docs/security/DKIM_INDEPENDENT_VERIFICATION.md`: enlace al kit de la puerta
  pendiente. `docs/audits/mail-delivery-gate.md`: evidencia real de SKIP y
  pruebas controladas de FAIL/wire. `CHANGELOG.md`: entrada bajo `[Unreleased]`.

## Plan de prueba

- `bash -n scripts/check-mail-delivery.sh` -> 0.
- Ejecucion real sin opt-in en este entorno -> `RESULT: SKIP reason=not-opted-in`, exit 0; nunca PASS.
- Ejecucion real con opt-in sin objetivo de sonda -> `SKIP reason=probe-target-not-configured`.
- Ejecucion real con opt-in y sonda `127.0.0.1:25` -> `SKIP reason=port-25-egress-unavailable`.
- Ejecucion real con opt-in en modo `wire` sin sumidero -> `SKIP reason=sink-unavailable`.
- Prueba controlada con evidencia falsa en `/tmp` y escucha local:
  `dkim=fail` -> `FAIL reason=receiver-rejected`, exit 1; tags de firma
  validos en modo `wire` -> `PASS scope=wire` con `gate.real_mailbox=NOT_SATISFIED`.
  Ninguno de estos ficheros esta en el repositorio ni constituye evidencia real.
- `git diff --check` y escaneos de contenido prohibido (emojis, evidencia de
  herramientas IA, marcas retiradas segun ADR-0011, prefijos de descriptor).

## Criterios de salida que avanza

- Hace ejecutable y verificable el segundo paso de Verification de #178 sin
  afirmar que se completo: sigue siendo una puerta externa y manual.
- No cambia codigo Rust, dependencias, CI por defecto ni contratos publicos.
- La plantilla vacia y los SKIP honestos impiden que una ejecucion sin evidencia
  se lea como aprobacion de la puerta.

## Cierre de issues

Refs #178
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (documentacion de verificacion, ola 12).
- [x] Respeta la documentacion y no anade features no contempladas.
- [x] No rompe arquitectura ni anade dependencias circulares.
- [x] No anade complejidad innecesaria: un script y dos documentos.
- [x] No aplica compilacion: sin codigo Rust; shell validado con `bash -n`.
- [x] Pasa `git diff --check` y los escaneos de contenido.
- [x] Tiene documentacion (procedimiento, plantilla, enlace y changelog).
- [x] No inventa resultados ni cierra la puerta; #178 permanece abierta.
