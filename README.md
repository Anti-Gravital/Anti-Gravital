# Anti-Gravital

**[English](#in-english) | [Espanol](#en-espanol)**

> Status: Phase 4.5 technical implementation complete — ag-mail (SMTP relay + native MTA), ag-domains (Cloudflare+ACME+SPF/DKIM/DMARC), DSL v0.7, ag-lsp v0.7, 14 cross-module E2E tests.
> Estado: Fase 4.5 implementacion tecnica completa — ag-mail (SMTP relay + native MTA), ag-domains (Cloudflare+ACME+SPF/DKIM/DMARC), DSL v0.7, ag-lsp v0.7, 14 tests E2E cross-module.

Anti-Gravital is an open source ecosystem for building high-performance
backend applications in pure Rust, with three core properties: no
external runtime, schema-first approach, and a modular architecture of
independent crates.

Anti-Gravital es un ecosistema de software libre para construir
aplicaciones backend de alto rendimiento en Rust puro, con tres
propiedades fundamentales: ausencia de runtime externo, enfoque
schema-first y arquitectura modular de crates independientes.

---

## In English

### What it is

A high-performance Rust backend runtime, a domain definition language
called Anti-DSL (`.ag` files), a unified CLI (`ag`), a set of batteries
included modules published as independent crates, a WASI plugin
system, a simplified deployment layer, native transactional email
(`ag-mail`) and domain plus TLS management (`ag-domains`) introduced
in Phase 4.5, typed SDK generators for TypeScript and Dart, and
importers from legacy frameworks.

### What it is not

It does not replace Kubernetes, Flutter, React Native, Next.js, Docker,
PostgreSQL, Redis, MinIO or NATS. It is not a game engine or a
scientific computing framework. `ag-mail` is not a full mail server:
it sends outbound transactional email; it does not host mailboxes and
does not implement IMAP/POP. A native outbound MTA core (direct MX
delivery, ESMTP+STARTTLS, Ed25519 DKIM signing, bounce classification)
is available as the opt-in `mta` feature per `ADR-0010` (Phase 4.6-A);
it is off by default and still hosts no mailboxes. `ag-domains` is not
a domain registrar: domains are
purchased externally. See the scope chapter at
`docs/architecture/03-alcance-y-limites.md`.

### Project status

Phases 1 through 4.5 have been technically completed. There is functional,
tested, and benchmarked code.

**Phase 1 — The Shield MVP:** complete. The `ag-core` crate contains
the operational `shield` module with HTTP/1.1, HTTP/2, TLS 1.3 (rustls),
JWT Ed25519 authentication, rate limiting, CORS, CSRF, payload
validation, and structured logging. Pipeline verified with E2E tests
and criterion benchmarks.

**Phase 2 — The Core MVP:** complete (technical implementation).
Axum router integrated with the Shield, typed extractors, `AgError`
error system, PostgreSQL connection pool via sqlx, embedded migrations,
and the `todo-api` example app with a full CRUD. The `ag` CLI provides
`new`, `dev`, and `build` with three templates (`rest`, `realtime`,
`fullstack`). The `todo-api` app deploys as a `FROM scratch` image of
2.49 MB. Benchmarks measured on real hardware: HTTP stack 89K req/s,
CRUD with PostgreSQL 14.5K req/s reads (bottleneck is PostgreSQL,
not the framework). Phase throughput and latency targets (40K req/s,
p99 <= 5 ms) require hardware with more cores or pgbouncer.

**Phase 3 — Anti-DSL alpha:** technical implementation complete (branch
`fase-3`). The `ag-dsl` compiler is operational with DSL v0.1 to v0.4:
models, endpoints, validations, and model relations (@references/@relation,
FOREIGN KEY SQL, Option<M>/Vec<M> Rust, $ref OpenAPI). The CLI exposes
`ag generate`, `ag schema lint`, and `ag schema diff`. LSP server
(`ag-lsp`) with real-time diagnostics, autocompletion, and hover. VS Code
plugin with syntax highlighting and LSP integration (`.vsix` packaged).
cargo-fuzz harness with 3 active targets in CI. 129 green tests (119
ag-dsl + 10 ag-lsp), 95.26% coverage. Real 2-hour benchmark against Neon
PostgreSQL: 255,805 requests, 0 errors, peak 43 req/s. External community
criteria pending.

**Phase 4 — Standard modules:** technical implementation complete (branch
`fase-4`). Five batteries-included modules operational as independent crates:

- `ag-auth`: WebAuthn/FIDO2 (registration+authentication, COSE ES256/EdDSA
  verification), OAuth2 PKCE (Google, GitHub), JWT Ed25519, BLAKE3 API keys,
  refresh tokens with in-memory blacklist. 32 tests.
- `ag-cache`: L1 in-memory cache with moka, tag-based invalidation, configurable TTL.
  >= 80% coverage. RFC-0005 (native RESP2 server) proposed, pending approval.
- `ag-realtime`: InProcess pub/sub event bus, external NATS client with 3-level TLS
  (system/custom CA/mTLS) and JetStream, Axum helpers for WebSocket and SSE
  (EventSource-compatible). `AgRealtime::new` is async. `realtime-chat` and
  `ai-backend` examples operational.
- `ag-storage`: native filesystem store with embedded Axum HTTP server, path-safe
  by construction, image processing (resize/thumbnail/webp), S3/MinIO backend via
  `object_store`, HMAC-SHA256 signed URLs. `AgStore` is a `Native | S3` enum.
- `ag-observe`: structured tracing, OTLP exporter, Prometheus metrics via
  `axum::Router`, custom layer, LogFormat (JSON/Text). Idempotent init.

DSL extended to v0.5 (auth/policies in endpoints) and v0.6 (declared events).
Updated generators: rust_gen (Claims extractor), openapi_gen (securitySchemes),
ts_gen (event payloads), new async_api_gen (AsyncAPI 2.6). 136 tests in ag-dsl,
95.88% coverage. `tests/integration` crate with 7 cross-module E2E tests
(auth+cache+realtime+storage+observe). >= 80% coverage in all modules. External
criteria (community, crates.io publication) pending.

**Phase 4.5 — ag-mail + ag-domains:** technical implementation complete
(2026-05-24). Two new crates introduced by ADR-0007:

- `ag-mail` (deferred standard): `MailSender` trait + `SmtpSender` (lettre +
  rustls) plus an opt-in native outbound MTA.
  `StringTemplate` engine for HTML/plaintext with compile-time var validation.
  Async queue with retries and exponential backoff (`InMemoryQueue`). Metrics
  towards `ag-observe` (feature `"metrics"`). Integration with `ag-auth` via
  `AuthMailer` for email verification, password recovery and magic links. 38
  tests.
- `ag-domains` (optional infra): `DnsProvider` trait + `CloudflareProvider`
  for A/AAAA/CNAME/TXT/MX records. ACME/Let's Encrypt via `instant-acme`
  (DNS-01 challenge, automatic renewal). SPF/DKIM/DMARC generation
  (`apply_mail_records`, idempotent). Propagation verification with
  `hickory-resolver`. 28 tests.

DSL extended to v0.7 (`mail`, `domain`, `template` blocks). `ag-lsp` updated
for the new blocks (hover, completions). CLI commands `ag domains check`,
`ag domains sync` and `ag mail test` operational. `auth-mail-demo` example
with three flows. 14 cross-module E2E tests total (7 Phase 4 + 7 Phase 4.5).

Detailed per-criterion status lives in `docs/roadmap/STATUS.md`.

### Quick start

Requires Rust 1.79+ (1.95+ recommended for all features).

```sh
# Install from source (Linux / macOS)
bash install.sh

# Or on Windows PowerShell:
# .\install.ps1

# Or directly:
# cargo install --path crates/ag-cli --locked

# Create a new project (prompts for template in interactive sessions)
ag new my-api

# Start in development mode
cd my-api
ag dev
```

Full installation guide and troubleshooting: `docs/manual/04-instalacion-y-onboarding.md`.

The app responds at `http://localhost:8080`. The `rest` template
generates a project with Shield, typed extractors, and a PostgreSQL
connection ready to configure via `DATABASE_URL`.

For the full CRUD example with PostgreSQL:

```sh
export DATABASE_URL="postgresql://user:pass@localhost/my_db"
cargo run -p todo-api
```

**DSL workflow (Phase 3, operational since v0.1+v0.2):**

Define your API in a `schema.ag` file and generate all artifacts:

```sh
ag generate --schema schema.ag --output ./generated
```

Produces: `src/models.rs`, `src/types.rs`, `src/handlers.rs`,
`src/router.rs`, `migrations/0001_initial.sql`,
`clients/typescript/types.ts`, `clients/typescript/client.ts`,
`openapi.json`.

Validate and diff schemas:

```sh
ag schema lint --schema schema.ag
ag schema diff old-schema.ag --schema schema.ag
```

### Measured performance

**Phase 2 — Ryzen 5 2500U local (2026-05-21)**

Measurements on AMD Ryzen 5 2500U (4C/8T), native PostgreSQL 18.4,
`rustc 1.95.0`, release profile with fat LTO. Full methodology in
`docs/benchmarks/measurement-2026-05-21-fase-2-crud-ryzen5-2500u.md`.

| Endpoint                     | req/s    | p99      | Notes                     |
| ---------------------------- | -------- | -------- | ------------------------- |
| GET /health (no DB)          | 88 930   | 3.2 ms   | Pure HTTP stack           |
| GET /todos/:id (SELECT PK)   | 14 478   | 14.6 ms  | Bottleneck: PostgreSQL    |
| POST /todos (INSERT)         | 8 934    | 9.4 ms   | synchronous_commit off    |

The 40K req/s target requires hardware with >= 8 physical cores or
pgbouncer in transaction mode.

**Phase 3 — Neon PostgreSQL serverless (2026-05-22)**

2-hour benchmark against Neon PostgreSQL (us-east-1, pooler). Mixed
operations: POST invoices with transactions, GET by id, GET filtered
list, PATCH status. Connection pool of 20. Full methodology in
`docs/benchmarks/measurement-2026-05-22-neon-real.md`.

| Metric                       | Value    | Notes                           |
| ---------------------------- | -------- | ------------------------------- |
| Total requests               | 255 805  | 120 min 11 s                    |
| Errors                       | 0        | Error rate 0.00%                |
| Average throughput           | 35.5 req/s | Includes cold-start           |
| Peak throughput              | 43.0 req/s | Cooldown phase (25 workers)   |

Saturation test: system stable up to 800 concurrent workers (0 errors).
Saturation at 1600 workers due to connection pool exhaustion (Neon was
at 10% CPU / 12% RAM throughout). See
`docs/benchmarks/measurement-2026-05-22-neon-saturacion.md`.

### Source of truth

The three master documents live in `docs/master/` and govern every
technical decision:

- `ANTI-GRAVITAL-Blueprint-v4.0.pdf` — vision, positioning, scope.
- `ANTI-GRAVITAL-Arquitectura-Tecnica.md` — how the system is built.
- `ANTI-GRAVITAL-Hoja-de-Ruta.md` — what is built and when.

These documents are decomposed into navigable files under
`docs/architecture/`, `docs/roadmap/`, `docs/modules/`, `docs/dsl/`,
`docs/security/`, `docs/governance/` and `docs/benchmarks/`. If a
derivative diverges from its master, the master wins.

### How to contribute

See `CONTRIBUTING.md` for the full guide. Quick summary:

1. Read the masters in `docs/master/` and current phase status in
   `docs/roadmap/STATUS.md`.
2. For architectural changes, open an RFC in `docs/rfc/` before
   touching code.
3. Keep pull requests small: titles up to 256 characters and a single
   logical unit of change.
4. Run `cargo fmt`, `cargo clippy -D warnings`, `cargo test`,
   `cargo audit` and `cargo deny check` before submitting.

### License

Apache 2.0. See `LICENSE`.

### Origin

Project started by Gravital Labs, the open source division of Nereira
Technology and Business Solutions, Republic of Panama. Initial
maintainer: Angel Nereira.

---

## En espanol

### Que es

Un runtime backend Rust de alto rendimiento, un lenguaje de definicion
de dominio llamado Anti-DSL (archivos `.ag`), una CLI unificada (`ag`),
un conjunto de modulos batteries-included publicados como crates
independientes, un sistema de plugins WASI, una capa de despliegue
simplificado, comunicacion transaccional nativa (`ag-mail`) y gestion
de dominios y TLS (`ag-domains`) introducidas en la Fase 4.5, generadores
de SDK tipados para TypeScript y Dart, e importadores desde frameworks
legacy.

### Que no es

No reemplaza Kubernetes. No reemplaza Flutter ni React Native. No
reemplaza Next.js. No reemplaza Docker. No reemplaza PostgreSQL,
Redis, MinIO ni NATS. No es un motor de juegos ni un framework de
computo cientifico. `ag-mail` no es un servidor de correo completo:
envia correo outbound transaccional, no aloja buzones ni implementa
IMAP/POP. Un nucleo de MTA outbound nativo (entrega MX directa,
ESMTP+STARTTLS, firma DKIM Ed25519, clasificacion de bounces) esta
disponible como feature opt-in `mta` segun `ADR-0010` (Fase 4.6-A);
esta apagado por defecto y sigue sin alojar buzones. `ag-domains`
no es un registrador de dominios: el dominio se compra externamente.
Vease el capitulo de alcance en
`docs/architecture/03-alcance-y-limites.md`.

### Estado del proyecto

El proyecto ha completado las fases 1, 2, 3, 4 y 4.5 de implementacion tecnica.
Existe codigo funcional, probado y benchmarkeado.

**Fase 1 — The Shield MVP:** completada. El crate `ag-core` contiene
el modulo `shield` operativo con HTTP/1.1, HTTP/2, TLS 1.3 (rustls),
autenticacion JWT Ed25519, rate limiting, CORS, CSRF, validacion de
payload y logging estructurado. Pipeline verificada con tests E2E y
benchmarks criterion.

**Fase 2 — The Core MVP:** completada (implementacion tecnica).
Router Axum integrado con la Shield, extractores tipados, sistema de
errores `AgError`, pool PostgreSQL via sqlx, migraciones embebidas y
la aplicacion de ejemplo `todo-api` con CRUD completo. La CLI `ag`
ofrece `new`, `dev` y `build` con tres templates (`rest`, `realtime`,
`fullstack`). La app `todo-api` se despliega como imagen `FROM scratch`
de 2.49 MB. Benchmarks medidos en hardware real: stack HTTP 89K req/s,
CRUD con PostgreSQL 14.5K req/s de lectura (cuello de botella en PG,
no en el framework). Los criterios de throughput y latencia de la fase
(40K req/s, p99 <= 5 ms) requieren hardware con mas nucleos o pgbouncer.

**Fase 3 — Anti-DSL alpha:** implementacion tecnica completa (rama `fase-3`).
El compilador `ag-dsl` esta operativo con DSL v0.1 a v0.4: modelos, endpoints,
validaciones y relaciones entre modelos (@references/@relation, FOREIGN KEY SQL,
Option<M>/Vec<M> Rust, $ref OpenAPI). La CLI expone `ag generate`, `ag schema lint`
y `ag schema diff`. Servidor LSP (`ag-lsp`) con diagnostics en tiempo real,
autocompletado y hover. Plugin VS Code con syntax highlighting e integracion
LSP (`.vsix` empaquetado). Harness cargo-fuzz con 3 targets activo en CI.
129 tests verdes (119 ag-dsl + 10 ag-lsp), cobertura 95.26%.
Benchmark real de 2 horas contra Neon PostgreSQL: 255.805 requests,
0 errores, peak 43 req/s. Criterios externos (comunidad) pendientes.

**Fase 4 — Modulos estandar:** implementacion tecnica completa (rama `fase-4`).
Los cinco modulos batteries-included estan operativos como crates independientes:

- `ag-auth`: WebAuthn/FIDO2 (registro+autenticacion, verificacion COSE ES256/EdDSA),
  OAuth2 PKCE (Google, GitHub), JWT Ed25519, API keys BLAKE3, refresh tokens con
  blacklist en memoria. 32 tests.
- `ag-cache`: cache L1 en memoria con moka, invalidacion por tags, TTL configurable.
  Cobertura >= 80%. RFC-0005 (servidor nativo RESP2) propuesto, pendiente de aprobacion.
- `ag-realtime`: bus de eventos InProcess pub/sub, cliente NATS externo con TLS 3 niveles
  (sistema/CA custom/mTLS) y JetStream, helpers Axum para WebSocket y SSE (EventSource-
  compatible). `AgRealtime::new` es asincrono. Examples `realtime-chat` y `ai-backend`
  operativos.
- `ag-storage`: store filesystem nativo con servidor HTTP Axum embebido, seguridad de path
  por construccion, procesamiento de imagen (resize/thumbnail/webp), backend S3/MinIO via
  `object_store`, URLs firmadas con HMAC-SHA256. `AgStore` es un enum `Native | S3`.
- `ag-observe`: tracing estructurado, exporter OTLP, metricas Prometheus via `axum::Router`,
  layer personalizado, LogFormat (JSON/Text). Init idempotente.

DSL ampliado a v0.5 (auth/politicas en endpoints) y v0.6 (eventos declarados). Generadores
actualizados: rust_gen (Claims extractor), openapi_gen (securitySchemes), ts_gen
(payload de eventos), nuevo async_api_gen (AsyncAPI 2.6). 136 tests en ag-dsl, cobertura
95.88%. Crate `tests/integration` con 7 tests E2E cross-module (auth+cache+realtime+storage+observe).
Cobertura >= 80% en todos los modulos. Criterios externos (comunidad, publicacion en crates.io) pendientes.

**Fase 4.5 — ag-mail + ag-domains:** implementacion tecnica completa (2026-05-24).
Dos crates nuevos introducidos por ADR-0007:

- `ag-mail` (estandar diferido): trait `MailSender` + `SmtpSender` (lettre + rustls)
  mas un MTA outbound nativo opt-in. Motor `StringTemplate`
  para HTML/plaintext con validacion de vars en compile-time. Cola asincrona con
  reintentos y backoff exponencial (`InMemoryQueue`). Metricas hacia `ag-observe`
  (feature `"metrics"`). Integracion con `ag-auth` via `AuthMailer` para verificacion,
  recuperacion y magic links. 38 tests.
- `ag-domains` (opcional infra): trait `DnsProvider` + `CloudflareProvider` para
  registros A/AAAA/CNAME/TXT/MX. ACME/Let's Encrypt via `instant-acme` (DNS-01,
  renovacion automatica). Generacion SPF/DKIM/DMARC (`apply_mail_records`,
  idempotente). Verificacion de propagacion con `hickory-resolver`. 28 tests.

DSL ampliado a v0.7 (bloques `mail`, `domain`, `template`). `ag-lsp` actualizado para
los bloques nuevos (hover, completions). Comandos CLI `ag domains check`,
`ag domains sync` y `ag mail test` operativos. Example `auth-mail-demo` con tres
flujos. 14 tests E2E cross-module en total (7 Fase 4 + 7 Fase 4.5).

El estado detallado de cada criterio vive en `docs/roadmap/STATUS.md`.

### Inicio rapido

Requiere Rust 1.79+ (se recomienda 1.95+).

```sh
# Instalar desde fuente (Linux / macOS)
bash install.sh

# En Windows PowerShell:
# .\install.ps1

# O directamente:
# cargo install --path crates/ag-cli --locked

# Crear un proyecto nuevo (pide la plantilla si el terminal es interactivo)
ag new mi-api

# Arrancar en modo desarrollo
cd mi-api
ag dev
```

Guia de instalacion completa y troubleshooting: `docs/manual/04-instalacion-y-onboarding.md`.

La app responde en `http://localhost:8080`. El template `rest` genera
un proyecto con Shield, extractores tipados y conexion a PostgreSQL
lista para configurar con `DATABASE_URL`.

Para el ejemplo completo CRUD con PostgreSQL:

```sh
export DATABASE_URL="postgresql://usuario:clave@localhost/mi_db"
cargo run -p todo-api
```

**Flujo DSL (Fase 3, operativo desde v0.1+v0.2):**

Define tu API en un archivo `schema.ag`:

```ag
config {
    project_name "mi-api"
    database "postgres"
}

model User {
    id    UUID   @primary @auto
    email String @unique
    name  String
}

request CreateUserRequest { email String  name String }
response UserResponse     { id UUID  email String  name String }
error EmailTaken { status 409 message "Email ya registrado" }

endpoint CreateUser {
    method   POST
    path     /users
    body     CreateUserRequest
    response UserResponse
    errors   [EmailTaken]
}
```

Genera todos los artefactos con un comando:

```sh
ag generate --schema schema.ag --output ./generated
```

Produce: `src/models.rs`, `src/types.rs`, `src/handlers.rs`,
`src/router.rs`, `migrations/0001_initial.sql`,
`clients/typescript/types.ts`, `clients/typescript/client.ts`,
`openapi.json`.

Valida el schema sin generar:

```sh
ag schema lint --schema schema.ag
ag schema diff schema-anterior.ag --schema schema.ag
```

### Rendimiento medido

**Fase 2 — Ryzen 5 2500U local (2026-05-21)**

Mediciones en AMD Ryzen 5 2500U (4C/8T), PostgreSQL 18.4 nativo,
`rustc 1.95.0`, perfil release con LTO fat. Metodologia completa en
`docs/benchmarks/measurement-2026-05-21-fase-2-crud-ryzen5-2500u.md`.

| Endpoint                     | req/s    | p99      | Notas                     |
| ---------------------------- | -------- | -------- | ------------------------- |
| GET /health (sin DB)         | 88 930   | 3.2 ms   | Stack HTTP puro           |
| GET /todos/:id (SELECT PK)   | 14 478   | 14.6 ms  | Cuello de botella: PG     |
| POST /todos (INSERT)         | 8 934    | 9.4 ms   | synchronous_commit off    |

El objetivo de 40K req/s requiere hardware con >= 8 nucleos fisicos o
uso de pgbouncer en transaction mode.

**Fase 3 — Neon PostgreSQL serverless (2026-05-22)**

Benchmark de 2 horas contra Neon PostgreSQL (us-east-1, pooler). Mix
de operaciones: POST facturas con transacciones, GET por id, GET lista
filtrada, PATCH estado. Pool de 20 conexiones. Metodologia completa en
`docs/benchmarks/measurement-2026-05-22-neon-real.md`.

| Metrica                      | Valor    | Notas                           |
| ---------------------------- | -------- | ------------------------------- |
| Total requests               | 255 805  | 120 min 11 s                    |
| Errores                      | 0        | Error rate 0.00%                |
| Throughput promedio          | 35.5 req/s | Incluye cold-start              |
| Throughput pico              | 43.0 req/s | Fase cooldown (25 workers)      |

Prueba de saturacion: sistema estable hasta 800 workers concurrentes
(0 errores). Saturacion a 1600 workers por agotamiento del pool de
conexiones (Neon estaba al 10% CPU / 12% RAM). Ver
`docs/benchmarks/measurement-2026-05-22-neon-saturacion.md`.

### Fuente de verdad

Los tres documentos maestros viven en `docs/master/` y gobiernan toda
decision tecnica del proyecto:

- `ANTI-GRAVITAL-Blueprint-v4.0.pdf` — vision, posicionamiento y alcance.
- `ANTI-GRAVITAL-Arquitectura-Tecnica.md` — como se construye.
- `ANTI-GRAVITAL-Hoja-de-Ruta.md` — que se construye y cuando.

Esta documentacion se descompone en archivos navegables bajo
`docs/architecture/`, `docs/roadmap/`, `docs/modules/`, `docs/dsl/`,
`docs/security/`, `docs/governance/` y `docs/benchmarks/`. Si existe
divergencia entre un derivado y el maestro, el maestro gana.

### Como contribuir

Vease `CONTRIBUTING.md` para la guia completa. Resumen rapido:

1. Lea los maestros bajo `docs/master/` y la fase actual en
   `docs/roadmap/STATUS.md`.
2. Para cambios arquitectonicos, abra una RFC en `docs/rfc/` antes de
   tocar codigo.
3. Mantenga sus pull requests cortas: titulo de hasta 256 caracteres y
   una unica unidad logica de cambio.
4. Pase `cargo fmt`, `cargo clippy -D warnings`, `cargo test`,
   `cargo audit` y `cargo deny check` antes de proponer cambios.

### Licencia

Apache 2.0. Vease `LICENSE`.

### Origen

Proyecto iniciado por Gravital Labs, division open source de Nereira
Technology and Business Solutions, Republica de Panama. Mantenedor
inicial: Angel Nereira.

---

## Calendario / Calendar

| Fase / Phase | Hito / Milestone              | Estado / Status                                         |
| ------------ | ----------------------------- | ------------------------------------------------------- |
| 0            | Fundaciones y gobernanza      | En curso (externos pendientes) / In progress (externals pending) |
| 1            | The Shield MVP                | Implementacion completa / Technical implementation complete |
| 2            | The Core MVP                  | Implementacion completa / Technical implementation complete |
| 3            | Anti-DSL alpha                | Implementacion completa / Technical implementation complete |
| 4            | Modulos estandar              | Implementacion completa / Technical implementation complete |
| 4.5          | ag-mail + ag-domains: comunicacion y dominios | Implementacion completa / Technical implementation complete |
| 5            | ag-cloud y version 0.5 beta   | Pendiente / Pending                                     |
| 6            | ag-ai y Knowledge Graph       | Pendiente / Pending                                     |
| 7            | ag-migrate importadores       | Pendiente / Pending                                     |
| 8            | ag-mobile Flutter bridge      | Pendiente / Pending                                     |
| 9            | Sistema de plugins WASI       | Pendiente / Pending                                     |
| 10           | Endurecimiento y version 1.0  | Pendiente / Pending                                     |

Detalle completo en `docs/roadmap/STATUS.md` y `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`.
