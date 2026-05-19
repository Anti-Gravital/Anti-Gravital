# Changelog

Formato basado en Keep a Changelog (https://keepachangelog.com/) y
semver (https://semver.org/). El repositorio aun no publica ninguna
version; las entradas viven bajo `[Unreleased]` hasta que se libere la
primera version etiquetada.

## [Unreleased]

### Fase 0 - Fundaciones y gobernanza

Anadido:

- Documentos maestros instalados en `docs/master/`:
  `ANTI-GRAVITAL-Blueprint-v4.0.pdf`,
  `ANTI-GRAVITAL-Arquitectura-Tecnica.md`,
  `ANTI-GRAVITAL-Hoja-de-Ruta.md` y `VERSION.md` con hashes SHA-256.
- Constitucion tecnica del repositorio en `CLAUDE.md`.
- README bilingue espanol e ingles.
- Documentos de gobernanza: `CONTRIBUTING.md`, `GOVERNANCE.md`,
  `SECURITY.md`, `CODE_OF_CONDUCT.md`.
- Estructura de documentacion: `docs/architecture/`, `docs/roadmap/`,
  `docs/modules/`, `docs/dsl/`, `docs/benchmarks/`, `docs/security/`,
  `docs/governance/`, `docs/examples/`, `docs/rfc/`, `docs/adr/`,
  `docs/diagrams/`, `docs/graph/`, `docs/es/`, `docs/en/`.
- Descomposicion verbatim de los maestros en archivos navegables por
  capitulo, fase y modulo.
- Workspace Cargo con 15 crates vacios: `ag-core`, `ag-dsl`,
  `ag-cli`, `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`,
  `ag-storage`, `ag-observe`, `ag-ui`, `ag-cloud`, `ag-ai`,
  `ag-mobile`, `ag-migrate`, `ag-wasm-host`.
- Configuracion de toolchain: `rust-toolchain.toml`, `rustfmt.toml`,
  `clippy.toml`, `deny.toml`.
- Workflows de CI multiplataforma: `ci.yml`, `quality.yml`, `docs.yml`.
- Plantillas de issue, pull request y RFC en `.github/`.
- ADRs iniciales: `0001-monorepo-workspace.md`,
  `0002-bilingual-documentation.md`, `0003-bdfl-governance.md`,
  `0004-descomposicion-de-maestros.md`,
  `0005-contact-identities.md`.
- Tablero vivo del proyecto en `docs/roadmap/STATUS.md`.
- Lista de entregables externos pendientes en
  `docs/governance/external-deliverables.md`.

Cambiado:

- Identidades de contacto oficiales del proyecto. Los placeholders
  `security@gravital.io` y `hello@antigravital.dev` de los maestros se
  reemplazan por `anti@gravitalcloud.com` (correo raiz) y
  `angelnereira@gravitalcloud.com` (BDFL inicial) en
  `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` (15.3) y
  `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` (Fase 0). Hashes
  recomputados en `docs/master/VERSION.md` con entrada de historial.
  Derivados verbatim regenerados. Registrado en
  `docs/adr/0005-contact-identities.md`.

Sin codigo funcional. El primer hito tecnico (Shield MVP) se entrega
en Fase 1.

### Fase 1 - The Shield MVP (en curso)

Anadido:

- RFC-0001 que autoriza la paralelizacion de las puertas externas de
  Fase 0 con la implementacion de Fase 1 mientras el BDFL trabaja en
  solitario.
- RFC-0002 con el diseno detallado del Shield MVP: stack, modulos,
  features Cargo, configuracion TOML, sistema de errores y plan de
  implementacion en 11 PRs incrementales.
- Estado vivo de Fase 1 reflejado en `docs/roadmap/STATUS.md`.

[Unreleased]: https://github.com/anti-gravital/anti-gravital/compare/HEAD..HEAD
