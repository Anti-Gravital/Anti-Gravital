# PR: Fase 0 completa y Fase 1 PRs 1-7 (Shield MVP en construccion)

## Resumen

Fase 0 fundaciones y gobernanza completa en repositorio y siete capas del Shield MVP (HTTP, validacion, CORS, CSRF, rate-limit, JWT Ed25519, TLS 1.3) operativas con 73 tests verde y CI multiplataforma.

## Fase afectada

Fase 0 (Fundaciones y Gobernanza) y Fase 1 (Shield MVP, PRs 1-7 de 11 segun
RFC-0002).

La paralelizacion entre el cierre de las puertas externas de Fase 0 (primer
star externo, Discord con cinco miembros, landing page) y el inicio de
implementacion de Fase 1 esta autorizada explicitamente en
`docs/rfc/RFC-0001-paralelizar-fase-0-externa-y-fase-1.md`. La regla 1 del
proceso queda vigente para fases sucesivas; esta es una excepcion documentada.

## Tipo de cambio

- [x] Documentacion
- [x] Codigo
- [x] Infraestructura o CI
- [x] RFC nueva o actualizacion de RFC
- [x] ADR nuevo
- [x] Seguridad

## Documentos relacionados

- RFC: `docs/rfc/RFC-0001-paralelizar-fase-0-externa-y-fase-1.md`,
  `docs/rfc/RFC-0002-diseno-shield-mvp.md`.
- ADR: `docs/adr/0001-monorepo-workspace.md`,
  `docs/adr/0002-bilingual-documentation.md`,
  `docs/adr/0003-bdfl-governance.md`,
  `docs/adr/0004-descomposicion-de-maestros.md`,
  `docs/adr/0005-contact-identities.md`.
- Maestro afectado: ambos maestros markdown bajo `docs/master/`:
  - `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` (seccion 15.3
    actualizada con los correos reales).
  - `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` (entregables de Fase 0
    actualizados con los correos reales).
  - Hashes SHA-256 recomputados en `docs/master/VERSION.md` y en el
    workflow `.github/workflows/docs.yml`.

## Detalle por capa entregada

### Fase 0 (commits `0567faa`, `18f2487`)

- Instalacion verbatim de los tres documentos maestros en `docs/master/`.
- Descomposicion navegable de los maestros bajo `docs/architecture/` (20
  capitulos), `docs/roadmap/` (11 fases mas preambulo y reglas de oro),
  `docs/modules/<crate>/` (15 modulos), `docs/dsl/`, `docs/security/`,
  `docs/governance/` y `docs/benchmarks/`.
- `CLAUDE.md` con las 40 reglas de gobernanza tecnica.
- Bilinguidad: `README.md` con bloques espanol e ingles. Indices en
  `docs/es/` y `docs/en/`.
- Gobernanza: `CONTRIBUTING.md`, `GOVERNANCE.md`, `SECURITY.md`,
  `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1), `CHANGELOG.md`.
- Workspace Cargo con los 15 crates declarados y compilando.
- Workflows GitHub Actions: `ci.yml` matriz Linux x86-64 / Linux ARM64 /
  macOS ARM64 / Windows x64, `quality.yml` con fmt, clippy `-D warnings`,
  audit y deny, `docs.yml` con cargo doc, validacion de hashes de
  maestros, rechazo de evidencia IA y rechazo de emojis.
- Plantillas de issue (bug, feature, RFC), `PULL_REQUEST_TEMPLATE.md` y
  `CODEOWNERS`.
- Identidades de contacto oficiales: `anti@gravitalcloud.com` (raiz),
  `angelnereira@gravitalcloud.com` (BDFL), respaldados con ADR-0005 y
  reflejados en maestros, gobernanza y workflows.

### Fase 1 PR 1 (commit `81026f4`)

Bootstrap de `ag-core` con HTTP/1.1 y HTTP/2 sobre Axum + Tokio. Modulos
`error`, `config`, `runtime`, `shield` (con la primera capa, logging
estructurado) y `core` (placeholder estable). `AgError` con
`IntoResponse` y mapeo a status HTTP. `ShieldConfig` deserializable
desde TOML.

### Fase 1 PR 2 (commit `7ea6f94`)

Capa de validacion de payload (`shield::validation`). Trait `Validate`,
agregado `ValidationErrors` con `FieldError` serializable y extractor
`ValidatedJson<T>`. Fallos mapeados a `AgError::Validation` (422) con
detalle estructurado.

### Fix CI (commit `e130e73`)

`Unicode-3.0` anadido a `deny.toml` para destrabar el job
`quality/cargo-deny` tras la entrada de Axum y reqwest (cadena url ->
idna -> icu).

### Fase 1 PR 3 (commit `a5fc479`)

Capa CORS (`shield::cors`) sobre `tower_http::cors::CorsLayer`. API
publica refactorizada de `Shield::layer()` a `Shield::apply(router)`
para que cada capa nueva no rompa la firma de tipos de la pipeline.

### Fase 1 PR 4 (commit `cf2eb7c`)

Capa CSRF (`shield::csrf`) con patron double-submit cookie apatrida.
Skip en metodos seguros (GET, HEAD, OPTIONS, TRACE).

### Fase 1 PR 5 (commit `1a7226d`)

Capa rate-limit (`shield::rate_limit`) con governor sobre token bucket
por IP. Sin `ConnectInfo` la capa pasa transparente.

### Fase 1 PR 6 (commit `8272bdf`)

Capa de autenticacion JWT Ed25519 (`shield::auth`). Verifica
`Authorization: Bearer <token>` contra una clave publica Ed25519
cargada al arranque. Leeway forzado a 0. Inyecta `AuthContext` en las
extensiones del request. Extractor `Claims<T>` para handlers tipados.

### Fase 1 PR 7 (commit `26b6161`)

Capa TLS 1.3 (`shield::tls`) con rustls (provider ring). `TlsAcceptor`
construido desde cert/key PEM. Helper `Shield::serve(listener, router)`
que decide entre transporte plano o TLS segun config. Migracion de
`rustls-pemfile` (RUSTSEC-2025-0134) a `rustls-pki-types::PemObject`.

### Flujo de descriptores y autofill (commits posteriores)

Introduccion del flujo de descriptor pre-rellenado bajo
`docs/pr-drafts/<rama-aplanada>.md` y del workflow
`.github/workflows/pr-autofill.yml`. Al abrir o reabrir una PR, el
workflow busca el descriptor por nombre de rama (con `/` aplanada a
`-`) y reemplaza automaticamente el cuerpo del PR con su contenido.
Si falta, comenta el PR pidiendo crearlo y deja warning en el job.
Plantilla `.github/PULL_REQUEST_TEMPLATE.md` queda como aviso
visible solo cuando no hay descriptor. Regla incorporada a
`CLAUDE.md` y `CONTRIBUTING.md`.

## Plan de prueba

Desde la raiz del repositorio, sobre la rama
`phase-0/foundations-and-governance`:

```sh
# Workspace completo construye.
cargo build --workspace

# Tests pasan: 45 unit + 27 E2E + 1 doctest = 73 verdes en ag-core.
cargo test --workspace

# Formato sin diferencias.
cargo fmt --all -- --check

# Clippy estricto sin warnings.
cargo clippy --workspace --all-targets -- -D warnings

# Politica de dependencias y advisories al dia.
cargo deny check

# Documentacion sin warnings.
cargo doc --workspace --no-deps

# Integridad de los documentos maestros.
sha256sum docs/master/ANTI-GRAVITAL-Blueprint-v4.0.pdf \
          docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md \
          docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
# Los valores devueltos deben coincidir con docs/master/VERSION.md.

# Ausencia de atribuciones a herramientas IA en archivos del repo.
grep -RIE "Co-Authored-By: (Claude|GPT|Copilot)|claude\.ai/code/session_" \
  --exclude-dir=.git --exclude-dir=target .

# Ausencia de emojis en archivos del repo.
perl -ne 'if (/[\x{1F300}-\x{1FAFF}]|[\x{2600}-\x{27BF}]|[\x{1F000}-\x{1F2FF}]/) { print "$ARGV:$.:$_" }' \
  $(find . -type f \( -name '*.md' -o -name '*.rs' -o -name '*.toml' -o -name '*.yml' -o -name '*.yaml' -o -name '*.sh' \) -not -path './target/*' -not -path './.git/*')
```

## Criterios de salida que avanza

De `docs/roadmap/STATUS.md`, esta unidad de cambio marca:

Fase 0:

- [x] Repositorio creado y publico.
- [x] LICENSE Apache 2.0.
- [x] README.md bilingue.
- [x] CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md, GOVERNANCE.md.
- [x] CI en cuatro plataformas (pendiente de verde en el primer run).
- [x] Plantillas de issue, PR y RFC.
- [x] Estructura del monorepo definida.
- [x] Workspace Cargo con 15 crates vacios.
- [x] Email institucional operativo (`anti@gravitalcloud.com`).

Fase 1:

- [x] Crate `ag-core` con modulo `shield` operativo.
- [x] Soporte HTTP/1.1 y HTTP/2 via Axum + Tokio.
- [x] Terminacion TLS 1.3 con rustls.
- [x] Middleware de validacion de payload basico.
- [x] Middleware de autenticacion JWT con verificacion Ed25519.
- [x] Middleware de rate limiting con governor.
- [x] Middleware CORS y CSRF con defaults seguros.
- [x] Middleware de logging estructurado con `tracing`.

Queda fuera de este PR y pendiente para PRs siguientes de Fase 1:
configuracion minima desde archivo TOML completa (PR 8), tests con
cobertura >=80% del crate (en buen camino; PR 10), tests E2E del
pipeline completo (PR 10), benchmark Hello World con criterion (PR 9),
documentacion API en docs.rs (PR 11), capitulo del manual de usuario
(PR 11), y las metricas duras de cierre de Fase 1 (>=300K req/s en
hardware de referencia, p99 <=1ms, memoria idle <=15MB, arranque
<=100ms, blog post, 10 stars).

## Checklist

- [x] Titulo de PR de 256 caracteres o menos.
- [x] Sin emojis en ningun archivo modificado.
- [x] Sin atribuciones de herramientas IA.
- [x] Documentacion actualizada en el mismo PR (CHANGELOG, STATUS,
  RFCs, ADRs, READMEs por modulo).
- [x] CHANGELOG.md actualizado bajo `[Unreleased]` con seccion Fase 0 y
  seccion Fase 1.
- [x] CLAUDE.md respetado: alcance (Fase 0 + Fase 1 dentro de scope),
  fase (transicion documentada en RFC-0001), dependencias (solo las
  declaradas en RFC-0002), seguridad (defaults seguros, leeway JWT 0,
  CORS deshabilitado por defecto, sin `unsafe` en codigo propio).
