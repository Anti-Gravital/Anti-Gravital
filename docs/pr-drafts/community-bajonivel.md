# Resumen

Add low-level review of the mail stack upgrade (#178) and chumsky RFC (#180)

## Fase afectada

Phase 4.6 (native MTA, `ag-mail`) and Phase 3 (`ag-dsl` parser). Documentation
only; no roadmap box is marked complete by this change.

## Tipo de cambio

- [ ] Code
- [ ] Configuration / CI
- [x] Documentation / review
- [ ] RFC / ADR
- [ ] Test
- [ ] Refactor

## Cambios

- Add `docs/audits/low-level-review-178-180.md` with evidence-based findings
  on DKIM signing semantics, `mail-auth` 0.15 crypto feature selection, the
  #181 process-level ring provider fix, and the low-level correctness inputs
  for the chumsky 0.9 to 0.13 RFC (parser API, error recovery, performance
  measurement, fuzz gates and MSRV).
- Add the audit section to `docs/INDEX.md` and an entry to `CHANGELOG.md`.
- No file owned by dev2 or innovacion was modified.

## Documentos relacionados

- `docs/audits/pre-fase5-fuzzing.md` (fuzz smoke and 24h gate state)
- `docs/rfc/RFC-0003-librerias-compilador-ag-dsl.md` (chumsky base decision)
- `docs/audits/PRE_FASE5_RELEASE_GATE.md` (#181 Tests row)
- `CLAUDE.md` rule 17 (benchmark evidence metadata)

## Plan de prueba

- `cargo test -p ag-mail --all-features --lib`
- `cargo test -p ag-dsl --lib`
- `grep -rn "ClientConfig::builder()\|ServerConfig::builder()" crates`
- Review of crate source metadata for `mail-auth` 0.15.1, `mail-send` 0.7.0
  and `chumsky` 0.13.0 from the local registry.

Verification outcome: `cargo test -p ag-mail --all-features --lib` passed
(128 passed, 0 failed, 1 ignored) and `cargo test -p ag-dsl --lib` passed
(165 passed, 0 failed, 0 ignored); combined exit code 0. The earlier
shared-target lock contention cleared. All static checks in this descriptor
were completed.

## Criterios de salida que avanza

- #178 Done-when item 2 (independent DKIM verifier): clarified that a
  cross-version `mail-auth` check is not independent; no box is marked done.
- #180 Done-when item 1 (RFC records the decision): inputs and constraints
  supplied to the primary owner; no box is marked done.
- #181 Done-when item 2 (no implicit process-default rustls config): the
  workspace grep was re-run and remains clean; no new defect found.

## Cierre de issues

Closes: none
Refs #178
Refs #180
Refs #181

## Checklist

- [x] Pertenece a la fase correcta (review de Fase 3 y Fase 4.6)
- [x] Respeta la documentacion y la politica de idioma tecnico en ingles
- [x] No rompe arquitectura ni anade dependencias
- [x] No edita archivos de otros roles
- [x] Sin evidencia de herramientas IA, sin emojis, sin atribucion automatica
