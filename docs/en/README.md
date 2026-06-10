# Documentation in English

English is the canonical language of the code and of the deep technical
documentation (ADR-0008, which supersedes the Spanish-default rule of
ADR-0002). Pre-existing Spanish technical documents migrate to English
gradually as they are touched.

## Shortcuts

- General index: `docs/INDEX.md`.
- Masters (bilingual EN+ES, English canonical): `docs/master/`.
- Architecture: `docs/architecture/`.
- Roadmap: `docs/roadmap/`.
- Modules: `docs/modules/`.

## Language policy (ADR-0008)

- Code comments and identifiers: English, no exceptions for new code.
- Deep technical documentation (`docs/architecture/`, `docs/modules/`,
  `docs/dsl/`, `docs/rfc/`, `docs/adr/`, `docs/benchmarks/`,
  `docs/security/`, `docs/governance/`): English canonical; legacy
  Spanish content migrates when touched.
- Showcase documents (root `README.md`, the three masters in
  `docs/master/`, the chapters of `docs/manual/`): bilingual EN+ES in
  the same file, English section first and canonical.
- If the English and Spanish sections diverge, English wins and the
  Spanish section is marked as pending an update.

## Status

This folder predates ADR-0008 and is kept as a navigation shortcut; no
separate English translation tree is needed now that English is the
canonical language in place.
