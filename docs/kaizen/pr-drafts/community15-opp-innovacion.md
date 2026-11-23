## Resumen

Catalog nine low-risk tooling opportunities with evidence and decisions; reject hype and unverified packages.

## Componente afectado

Project `oportunidades`, task O3 (innovation / tech watch). Documentation only.

## Tipo de cambio

- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [x] documentation

## Documentos relacionados

- `docs/audits/oportunidades-innovacion.md` (new catalog)
- ADR-0021, RFC-0026, RFC-0027, RFC-0031, RFC-0036 (evidence sources)
- `docs/audits/chumsky-migration-spike.md`, `chrono-unblock-analysis.md`,
  `instant-acme-08-compile-spike.md`, `wave4-review.md`
- `docs/benchmarks/REFERENCE_RUNBOOK.md`, `reference-host.md`,
  `wave9-host-results.md`, `gatebench-informative-2026-11-22.md`
- `docs/fuzz/24h-gate-runbook.md`, `docs/fuzz/24h-gate-verification.md`
- CHANGELOG.md (entry added in the same commit)

## Plan de prueba

- `git diff --check`
- Content scans: emojis, AI-tool attribution and session URLs, prohibited
  descriptor branch prefixes, retired third-party brand names, descriptor
  closing keywords (patterns from `.github/workflows/docs.yml`).
- `scripts/kaizen check --only descriptors`, `--only policy`, `--only ratchet`
- `scripts/kaizen check` (full result reported verbatim in the journal)
- Confirm `git diff --name-only` touches no dependency, manifest, lockfile,
  source, CI or release-gate file.

## Impacto en calidad (ratchet)

- Cobertura: igual (documentation-only change; no source code).
- Advertencias: igual (no source or CI change).
- Deuda (TODO/FIXME): igual.

## Criterios de salida que avanza

- Catalog with what / why / evidence / S-9 risk / decision / reason per opportunity.
- Each row cites a documented spike, a measured number, or a labelled hypothesis.
- Explicit `Hipos a rechazar` section against rules 10/12/21.
- No install, fetch, network command, gate flip or issue closure.

## Aprendizaje (opcional)

Most "upgrade now" opportunities already have a measured deferral in the repo;
the real low-risk work is the prerequisites (golden tests, a tracked benchmark,
a reference-host run), not the dependency bump itself.

## Cierre de issues

Closes: none

This is a board task (`oportunidades` O3), not a GitHub issue. It closes no
gate and no issue; every row that touches a gated item keeps `Refs` semantics.

## Checklist

- [x] Compila (no code changed)
- [x] Pasa tests (documentation-only; checks recorded in the journal)
- [ ] `scripts/kaizen check` en verde (result recorded verbatim in the journal)
- [x] Documentacion actualizada (new audit + CHANGELOG)
- [x] Linea base no empeora
