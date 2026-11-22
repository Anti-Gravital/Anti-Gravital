# Resumen

Document blocked-issue evidence and reconcile tracking parents for ACME staging, instant-acme 0.8, the repository audit and ag-domains.

## Fase afectada

Fase 4.6-D / 4.7 documentation follow-up (RFC-0011 phases E-F). No phase state changes; no code behavior changes.

## Tipo de cambio

- [ ] Code
- [ ] Bug fix
- [ ] Dependency / stack change (requires RFC; not attempted)
- [x] Documentation
- [x] Issue reconciliation / evidence
- [ ] CI
- [ ] RFC / ADR

## Documentos relacionados

- `docs/ag-domains/how-to/acme-staging-test.md` (new; contract for #87).
- `docs/ag-domains/reference/instant-acme-0.8-migration.md` (new; blocked port #179).
- `docs/ag-domains/README.md` (index updated).
- `CHANGELOG.md` (Unreleased entry).
- `docs/rfc/RFC-0011-ag-domains-control-plane.md` (phases E-F).
- `docs/audits/PRE_FASE5_RELEASE_GATE.md` and `docs/audits/pre-fase5-docs-reconciliation.md` (audit context for #142).
- `CLAUDE.md` rules 29, 26 and the descriptor policy.

## Cambios concretos

- Verified that #87 and #179 are OPEN, carry the `blocked` label, and that #179 already has the coordinator comment linking #87.
- Reconciled all children of #142 and #76 against the live issue board: #76 has one open child (#87); #142 has six open children (#141, #149, #152, #154, #156, #157).
- Posted a checklist comment on both tracking parents. Updated only the outdated status line of #142; no child descriptions were duplicated.
- Added the staging procedure with prerequisites, exact commands and an evidence template. The `#[ignore]` test does not exist yet, so no pass is claimed.
- Added the nine-point 0.8 API migration reference based on the crate source and a scratch `cargo check` (0.8.0 and 0.8.5: eight compiler errors in the current tree).

## Plan de prueba

```
git diff --check
cargo fmt --all -- --check
cargo clippy -p ag-domains --all-targets --all-features -- -D warnings
cargo test -p ag-domains --all-features
```

No code paths changed; the `instant-acme` dependency remains at 0.7. The credentialed staging run of #87 cannot be executed in this environment and stays blocked.

## Criterios de salida que avanza

- #87: reproducible staging procedure published; issue stays open with `blocked`.
- #179: blocking dependency and nine migration points documented; no unverified code change.
- #142: parent body and checklist now reflect the six children still open; parent stays open.
- #76: checklist confirms every child except #87 is verified closed; parent stays open.

## Cierre de issues

Closes: none
Refs #87
Refs #179
Refs #142
Refs #76

## Checklist

- [x] Cambio minimo y documentado.
- [x] Sin evidencia de herramientas IA ni emojis.
- [x] Sin dependencias nuevas ni cambios de codigo.
- [x] Mensajes de commit <= 256 caracteres.
- [x] Issues bloqueados siguen abiertos con etiqueta `blocked`.
- [x] No se cierra ningun padre con hijos abiertos.
