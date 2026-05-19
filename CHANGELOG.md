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
- Bootstrap del crate `ag-core` con HTTP/1.1 y HTTP/2 funcionales via
  Axum + Tokio (sin TLS aun): modulos `error`, `config`, `runtime`,
  `shield` (capa de logging estructurado), `core` (placeholder).
- `AgError` y `AgResult` con mapeo automatico a respuestas HTTP via
  `IntoResponse`.
- `ShieldConfig` deserializable desde TOML con defaults seguros.
- Dependencias compartidas del workspace declaradas en
  `[workspace.dependencies]` (axum, tokio, tower, tower-http, tracing,
  serde, thiserror, hyper, http, bytes, toml, pin-project-lite).
- Tests: 12 unit tests por modulo, 2 tests E2E con servidor real, 1
  doctest. Todos en verde con `cargo fmt`, `cargo clippy -D warnings`
  y `cargo doc --no-deps` limpios.
- Capa de validacion de payload (`shield::validation`) detras de la
  feature `validation` activa por defecto. Trait `Validate`, agregado
  `ValidationErrors` con `FieldError` serializable y extractor
  `ValidatedJson<T>` que mapea fallos a `AgError::Validation` con
  detalle estructurado por campo (status 422). 4 unit tests
  adicionales y 3 tests E2E sobre `/projects`.
- Capa CORS (`shield::cors`) detras de la feature `cors` activa por
  defecto. Wraps `tower_http::cors::CorsLayer` con configuracion
  declarativa via `CorsConfig` en `ShieldConfig`. Defaults seguros:
  CORS deshabilitado salvo declaracion explicita. Errores de
  configuracion mapeados a `AgError::Cors` con codigo `cors_error`
  (status 403). 4 unit tests sobre construccion y 4 tests E2E sobre
  preflight, origenes listados y rechazados.
- Tower-http feature `cors` activada en el workspace.

Cambiado:

- API publica de `Shield`: `Shield::layer()` reemplazado por
  `Shield::apply(router)`. La nueva firma oculta la complejidad de
  tipos de la pipeline y permite agregar capas sin romper la
  superficie publica en cada PR.
- `Shield::try_new(config)` valida la configuracion en construccion
  (origenes, metodos y headers de CORS); `Shield::new(config)` mantiene
  semantica de panic para casos de prototipado.
- Workflow `quality.yml`: `cargo deny` ya pasa tras anadir
  `Unicode-3.0` a `deny.toml` (commit anterior).

[Unreleased]: https://github.com/anti-gravital/anti-gravital/compare/HEAD..HEAD
