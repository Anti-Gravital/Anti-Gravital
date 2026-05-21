# Anti-Gravital

> Estado: Fase 3 en curso — Anti-DSL alpha. DSL v0.1, v0.2 y v0.3 operativos.
> Status: Phase 3 in progress — Anti-DSL alpha. DSL v0.1, v0.2 and v0.3 operational.

Anti-Gravital es un ecosistema de software libre para construir
aplicaciones backend de alto rendimiento en Rust puro, con tres
propiedades fundamentales: ausencia de runtime externo, enfoque
schema-first y arquitectura modular de crates independientes.

Anti-Gravital is an open source ecosystem for building high-performance
backend applications in pure Rust, with three core properties: no
external runtime, schema-first approach, and a modular architecture of
independent crates.

---

## En espanol

### Que es

Un runtime backend Rust de alto rendimiento, un lenguaje de definicion
de dominio llamado Anti-DSL (archivos `.ag`), una CLI unificada (`ag`),
un conjunto de modulos batteries-included publicados como crates
independientes, un sistema de plugins WASI, una capa de despliegue
simplificado, generadores de SDK tipados para TypeScript y Dart, e
importadores desde frameworks legacy.

### Que no es

No reemplaza Kubernetes. No reemplaza Flutter ni React Native. No
reemplaza Next.js. No reemplaza Docker. No reemplaza PostgreSQL,
Redis, MinIO ni NATS. No es un motor de juegos ni un framework de
computo cientifico. Vease el capitulo de alcance en
`docs/architecture/03-alcance-y-limites.md`.

### Estado del proyecto

El proyecto ha completado las fases 1 y 2 de implementacion tecnica.
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

**Fase 3 — Anti-DSL alpha:** en curso (rama `fase-3`). El compilador
`ag-dsl` esta operativo con DSL v0.1, v0.2 y v0.3. Define modelos,
endpoints, tipos de peticion y respuesta, y anotaciones de validacion
(`@min`, `@max`, `@email`, `@regex`, `@length`) en un archivo `.ag`.
Genera structs Rust con serde y metodos `validate()`, migraciones SQL
con CHECK constraints, interfaces TypeScript, un cliente HTTP tipado,
y documentacion OpenAPI 3.1 con constraints de validacion.
La CLI expone `ag generate`, `ag schema lint` y `ag schema diff`.

El estado detallado de cada criterio vive en `docs/roadmap/STATUS.md`.

### Inicio rapido

Requiere Rust 1.95+ y PostgreSQL.

```sh
# Instalar la CLI
cargo install --path crates/ag-cli

# Crear un proyecto nuevo
ag new mi-api

# Arrancar en modo desarrollo
cd mi-api
ag dev
```

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

Mediciones del 2026-05-21 en AMD Ryzen 5 2500U (4C/8T), PostgreSQL 18.4
nativo, `rustc 1.95.0`, perfil release con LTO fat. Metodologia completa
en `docs/benchmarks/measurement-2026-05-21-fase-2-crud-ryzen5-2500u.md`.

| Endpoint                     | req/s    | p99      | Notas                     |
| ---------------------------- | -------- | -------- | ------------------------- |
| GET /health (sin DB)         | 88 930   | 3.2 ms   | Stack HTTP puro           |
| GET /todos/:id (SELECT PK)   | 14 478   | 14.6 ms  | Cuello de botella: PG     |
| POST /todos (INSERT)         | 8 934    | 9.4 ms   | synchronous_commit off    |

El objetivo de 40K req/s requiere hardware con >= 8 nucleos fisicos o
uso de pgbouncer en transaction mode. Ver analisis en el documento
de benchmarks.

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

## In English

### What it is

A high-performance Rust backend runtime, a domain definition language
called Anti-DSL (`.ag` files), a unified CLI (`ag`), a set of batteries
included modules published as independent crates, a WASI plugin
system, a simplified deployment layer, typed SDK generators for
TypeScript and Dart, and importers from legacy frameworks.

### What it is not

It does not replace Kubernetes, Flutter, React Native, Next.js, Docker,
PostgreSQL, Redis, MinIO or NATS. It is not a game engine or a
scientific computing framework. See the scope chapter at
`docs/architecture/03-alcance-y-limites.md`.

### Project status

Phases 1 and 2 have been technically completed. There is functional,
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

**Phase 3 — Anti-DSL alpha:** in progress (branch `fase-3`). The
`ag-dsl` compiler is operational with DSL v0.1, v0.2, and v0.3. Define
models, endpoints, request and response types, and validation annotations
(`@min`, `@max`, `@email`, `@regex`, `@length`) in a `.ag` file.
Generates Rust structs with serde and `validate()` methods, SQL
migrations with CHECK constraints, TypeScript interfaces, a typed HTTP
client, and a full OpenAPI 3.1 document with validation constraints.
The CLI exposes `ag generate`, `ag schema lint`, and `ag schema diff`.

Detailed per-criterion status lives in `docs/roadmap/STATUS.md`.

### Quick start

Requires Rust 1.95+ and PostgreSQL.

```sh
# Install the CLI
cargo install --path crates/ag-cli

# Create a new project
ag new my-api

# Start in development mode
cd my-api
ag dev
```

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

Measurements from 2026-05-21 on AMD Ryzen 5 2500U (4C/8T), native
PostgreSQL 18.4, `rustc 1.95.0`, release profile with fat LTO. Full
methodology in
`docs/benchmarks/measurement-2026-05-21-fase-2-crud-ryzen5-2500u.md`.

| Endpoint                     | req/s    | p99      | Notes                     |
| ---------------------------- | -------- | -------- | ------------------------- |
| GET /health (no DB)          | 88 930   | 3.2 ms   | Pure HTTP stack           |
| GET /todos/:id (SELECT PK)   | 14 478   | 14.6 ms  | Bottleneck: PostgreSQL    |
| POST /todos (INSERT)         | 8 934    | 9.4 ms   | synchronous_commit off    |

The 40K req/s target requires hardware with >= 8 physical cores or
pgbouncer in transaction mode. See the benchmark document for analysis.

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

## Calendario / Calendar

| Fase / Phase | Hito / Milestone              | Estado / Status                                         |
| ------------ | ----------------------------- | ------------------------------------------------------- |
| 0            | Fundaciones y gobernanza      | En curso (externos pendientes) / In progress (externals pending) |
| 1            | The Shield MVP                | Implementacion completa / Technical implementation complete |
| 2            | The Core MVP                  | Implementacion completa / Technical implementation complete |
| 3            | Anti-DSL alpha                | En curso / In progress — DSL v0.1+v0.2 operativos       |
| 4            | Modulos estandar              | Pendiente / Pending                                     |
| 5            | ag-cloud y version 0.5 beta   | Pendiente / Pending                                     |
| 6            | ag-ai y Knowledge Graph       | Pendiente / Pending                                     |
| 7            | ag-migrate importadores       | Pendiente / Pending                                     |
| 8            | ag-mobile Flutter bridge      | Pendiente / Pending                                     |
| 9            | Sistema de plugins WASI       | Pendiente / Pending                                     |
| 10           | Endurecimiento y version 1.0  | Pendiente / Pending                                     |

Detalle completo en `docs/roadmap/STATUS.md` y `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`.
