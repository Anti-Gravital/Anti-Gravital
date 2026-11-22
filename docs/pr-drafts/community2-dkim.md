# Descriptor de PR: community2/dkim

## Resumen

Add independent dkimpy verification of the ag-mail DKIM vector and a fresh signature (issue #178)

## Fase afectada

Fase 4.6-D (ag-mail, MTA nativo), cierre de deuda de verificacion de #178. Sin cambio de fase.

## Tipo de cambio

- [x] Codigo (test ignorado y herramienta de verificacion)
- [x] Documentacion de seguridad
- [x] Auditoria / evidencia de verificacion
- [ ] Cambio de arquitectura
- [ ] Cambio de DSL
- [ ] Cambio de dependencies de Cargo

## Documentos relacionados

- `docs/audits/low-level-review-178-180.md` (seccion 2, hallazgo 2)
- `docs/security/DKIM_INDEPENDENT_VERIFICATION.md` (nuevo)
- `CHANGELOG.md`
- Issue #178 (Done-when item 2)

## Plan de prueba

Comandos ejecutados en este worktree con el target de Cargo compartido:

1. `cargo test -p ag-mail --all-features` (sin el test ignorado; hermetico).
2. `cargo test -p ag-mail --all-features dkim_signatures_verify_with_dkimpy -- --ignored --nocapture`
   (dos lineas `dkim=pass`, una para la firma 0.9 grabada y otra para la firma 0.15 fresca).
3. `cargo fmt --all -- --check`
4. `cargo clippy -p ag-mail --all-features -- -D warnings`
5. `cargo audit`
6. `cargo deny --all-features check`

La instalacion de dkimpy y PyNaCl es documentada y reproducible en
`docs/security/DKIM_INDEPENDENT_VERIFICATION.md`. No se anadio ninguna
dependencia de Cargo: dkimpy es BSD-like y PyNaCl es Apache-2.0.

## Criterios de salida que avanza

- Done-when item 2 de #178: una firma de la pila nueva verifica con un
  verificador independiente, con comando y salida grabados.
- No avanza la puerta manual de buzon real / Mail A (egreso puerto 25 bloqueado).

## Cierre de issues

Refs #178
Closes: none

## Checklist

- [x] Pertenece a la fase correcta.
- [x] Respeta la documentacion.
- [x] No rompe arquitectura.
- [x] Compila.
- [x] Pasa tests (incluido el verificador independiente ignorado).
- [x] Pasa fmt.
- [x] Pasa clippy.
- [x] Pasa audit.
- [x] Pasa deny.
- [x] Tiene documentacion.
- [x] Mantiene coherencia con Anti-Gravital v4.1.
