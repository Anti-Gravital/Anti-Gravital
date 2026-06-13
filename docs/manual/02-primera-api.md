English | Espanol

---

# Your first API with Anti-Gravital

> Phase 2 — The Core MVP and full round trip
> Manual document: `docs/manual/02-primera-api.md`
> Previous: [01-shield-as-library.md](./01-shield-as-library.md)

---

## Goal

By the end of this chapter you will have a fully working REST API that
does CRUD against PostgreSQL, packaged as a static binary deployable
from `FROM scratch`.

The full flow is:

```
Request → Shield (auth, rate-limit, CORS) → Core (router, handlers) → PostgreSQL
```

All in a single Rust process: no IPC, no sidecar, no magic.

---

## Prerequisites

- Rust 1.95+ installed (`rustup update stable`)
- A local or remote PostgreSQL reachable
- The `ag` CLI built (`cargo install --path crates/ag-cli` from the
  repository)

---

## 1. Create the project

```sh
ag new mi-api --template rest
cd mi-api
```

This generates:

```
mi-api/
├── Cargo.toml
├── config.toml
└── src/
    └── main.rs
```

For an API with a database from the start:

```sh
ag new mi-api --template fullstack
```

The `fullstack` template adds `ag-data`, sqlx and a preconfigured
`migrations/` directory.

---

## 2. Run in development

```sh
export DATABASE_URL="postgresql://postgres:postgres@localhost/mi_api"
ag dev
```

The `ag dev` command detects whether `cargo-watch` is installed and
enables hot reload automatically. If it is not installed:

```sh
cargo install cargo-watch
```

The API is then available at `http://localhost:8080`.

---

## 3. Structure of a typical handler

```rust
use ag_core::{AgError, AgResult};
use ag_core::core::{State, Path, Query};
use axum::Json;

// Shared state: defined once, cloned by reference via Arc.
#[derive(Clone)]
struct AppState {
    db: ag_data::DbPool,
}

// Handler: takes typed extractors, returns Result<_, AgError>.
// Conversion to an HTTP response is automatic.
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AgError> {
    let user = sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AgError::Database(e.to_string()))?
        .ok_or_else(|| AgError::NotFound(format!("user {id}")))?;

    Ok(Json(user))
}
```

Extractors available from `ag_core::core`:

| Extractor | What it extracts |
| --- | --- |
| `State<T>` | Shared router state |
| `Path<T>` | Typed path segments (`/users/{id}`) |
| `Query<T>` | Query-string parameters (`?page=1`) |
| `ValidatedBody<T>` | JSON body with declarative validation |
| `Claims<T>` | JWT claims verified by Shield |

---

## 4. Connect to PostgreSQL

```rust
use ag_data::{DataConfig, connect, run_migrations};

let config = DataConfig {
    url: std::env::var("DATABASE_URL").expect("DATABASE_URL not set"),
    max_connections: 10,
    ..DataConfig::default()
};

let pool = connect(&config).await?;

// The macro embeds the SQL files into the binary at compile time.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
run_migrations(&pool, &MIGRATOR).await?;
```

Migrations are `.sql` files in the `migrations/` directory, numbered in
order: `0001_create_users.sql`, `0002_add_index.sql`, etc. They are
idempotent: applied only if they have not been applied before.

---

## 5. Register the router with Shield

```rust
use ag_core::{Shield, ShieldConfig};
use axum::{Router, routing::{get, post}};

let shield = Shield::with_defaults();

let router = Router::new()
    .route("/users", get(list_users).post(create_user))
    .route("/users/{id}", get(get_user).put(update_user).delete(delete_user))
    .with_state(AppState { db: pool });

// Shield applies: logging, rate-limit, CORS, CSRF, JWT auth.
let app = shield.apply(router);

let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
shield.serve(listener, app).await?;
```

---

## 6. Error system

`AgError` converts automatically into structured HTTP responses:

| Variant | HTTP | JSON code |
| --- | --- | --- |
| `NotFound(msg)` | 404 | `not_found` |
| `BadRequest(msg)` | 400 | `bad_request` |
| `Conflict(msg)` | 409 | `conflict` |
| `Validation(msg)` | 422 | `validation_error` |
| `Auth(msg)` | 401 | `auth_error` |
| `Database(msg)` | 500 | `database_error` |

Example error response:

```json
{
  "code": "not_found",
  "message": "not found: user 42"
}
```

---

## 7. Build for production

```sh
ag build
```

The resulting binary under `target/release/` is self-contained. For a
minimal Docker image:

```sh
# See examples/todo-api/Dockerfile for the complete Dockerfile
docker build -t mi-api .
docker run -e DATABASE_URL=postgresql://... -p 8080:8080 mi-api
```

The Dockerfile uses a MUSL build and `FROM scratch`, producing an image
under 20 MB.

---

## 8. Complete example: todo-api

The `examples/todo-api/` example in the repository implements a full
CRUD with all five endpoints:

```sh
# List tasks
curl http://localhost:8080/todos

# Create a task
curl -X POST http://localhost:8080/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Read the documentation"}'

# Get a task
curl http://localhost:8080/todos/1

# Update a task
curl -X PUT http://localhost:8080/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"done": true}'

# Delete a task
curl -X DELETE http://localhost:8080/todos/1
```

Run the example:

```sh
export DATABASE_URL="postgresql://postgres:postgres@localhost/todos"
cargo run -p todo-api
```

---

## What comes next

Phase 3 introduces the Anti-DSL: you define your models and endpoints in
`schema.ag` and the compiler generates all the code from this guide
automatically. See `docs/roadmap/fase-03-anti-dsl-alpha.md`.

---

# Tu primera API con Anti-Gravital

> Fase 2 — The Core MVP y roundtrip completo  
> Documento del manual: `docs/manual/02-primera-api.md`  
> Anterior: [01-shield-as-library.md](./01-shield-as-library.md)

---

## Objetivo

Al finalizar este capitulo tendras una API REST completamente funcional
que hace CRUD contra PostgreSQL, empaquetada en un binario estatico
deployable desde `FROM scratch`.

El flujo completo es:

```
Request → Shield (auth, rate-limit, CORS) → Core (router, handlers) → PostgreSQL
```

Todo en un proceso Rust, sin IPC, sin sidecar, sin magic.

---

## Prerequisitos

- Rust 1.95+ instalado (`rustup update stable`)
- PostgreSQL local o remoto accesible
- `ag` CLI compilado (`cargo install --path crates/ag-cli` desde el
  repositorio)

---

## 1. Crear el proyecto

```sh
ag new mi-api --template rest
cd mi-api
```

Esto genera:

```
mi-api/
├── Cargo.toml
├── config.toml
└── src/
    └── main.rs
```

Para una API con base de datos desde el inicio:

```sh
ag new mi-api --template fullstack
```

El template `fullstack` agrega `ag-data`, sqlx y un directorio
`migrations/` preconfigurado.

---

## 2. Arrancar en desarrollo

```sh
export DATABASE_URL="postgresql://postgres:postgres@localhost/mi_api"
ag dev
```

El comando `ag dev` detecta si `cargo-watch` esta instalado y activa
hot reload automaticamente. Si no esta instalado:

```sh
cargo install cargo-watch
```

La API queda disponible en `http://localhost:8080`.

---

## 3. Estructura de un handler tipico

```rust
use ag_core::{AgError, AgResult};
use ag_core::core::{State, Path, Query};
use axum::Json;

// Estado compartido: definido una vez, clonado por referencia Arc.
#[derive(Clone)]
struct AppState {
    db: ag_data::DbPool,
}

// Handler: recibe extractores tipados, devuelve Result<_, AgError>.
// La conversion a respuesta HTTP es automatica.
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AgError> {
    let user = sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AgError::Database(e.to_string()))?
        .ok_or_else(|| AgError::NotFound(format!("user {id}")))?;

    Ok(Json(user))
}
```

Los extractores disponibles desde `ag_core::core`:

| Extractor | Que extrae |
| --- | --- |
| `State<T>` | Estado compartido del router |
| `Path<T>` | Segmentos de ruta tipados (`/users/{id}`) |
| `Query<T>` | Parametros de query string (`?page=1`) |
| `ValidatedBody<T>` | Body JSON con validacion declarativa |
| `Claims<T>` | Claims JWT verificados por Shield |

---

## 4. Conectar a PostgreSQL

```rust
use ag_data::{DataConfig, connect, run_migrations};

let config = DataConfig {
    url: std::env::var("DATABASE_URL").expect("DATABASE_URL no definida"),
    max_connections: 10,
    ..DataConfig::default()
};

let pool = connect(&config).await?;

// El macro embebe los archivos SQL en el binario al compilar.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
run_migrations(&pool, &MIGRATOR).await?;
```

Las migraciones son archivos `.sql` en el directorio `migrations/`,
numerados en orden: `0001_create_users.sql`, `0002_add_index.sql`, etc.
Son idempotentes: se aplican solo si no han sido aplicadas antes.

---

## 5. Registrar el router con Shield

```rust
use ag_core::{Shield, ShieldConfig};
use axum::{Router, routing::{get, post}};

let shield = Shield::with_defaults();

let router = Router::new()
    .route("/users", get(list_users).post(create_user))
    .route("/users/{id}", get(get_user).put(update_user).delete(delete_user))
    .with_state(AppState { db: pool });

// Shield aplica: logging, rate-limit, CORS, CSRF, auth-JWT.
let app = shield.apply(router);

let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
shield.serve(listener, app).await?;
```

---

## 6. Sistema de errores

`AgError` convierte automaticamente a respuestas HTTP estructuradas:

| Variante | HTTP | Codigo JSON |
| --- | --- | --- |
| `NotFound(msg)` | 404 | `not_found` |
| `BadRequest(msg)` | 400 | `bad_request` |
| `Conflict(msg)` | 409 | `conflict` |
| `Validation(msg)` | 422 | `validation_error` |
| `Auth(msg)` | 401 | `auth_error` |
| `Database(msg)` | 500 | `database_error` |

Ejemplo de respuesta de error:

```json
{
  "code": "not_found",
  "message": "not found: user 42"
}
```

---

## 7. Compilar para produccion

```sh
ag build
```

El binario resultante en `target/release/` es autocontenido.
Para una imagen Docker minima:

```sh
# Ver examples/todo-api/Dockerfile para el Dockerfile completo
docker build -t mi-api .
docker run -e DATABASE_URL=postgresql://... -p 8080:8080 mi-api
```

El Dockerfile usa compilacion MUSL y `FROM scratch`, produciendo una
imagen de menos de 20 MB.

---

## 8. Ejemplo completo: todo-api

El ejemplo `examples/todo-api/` del repositorio implementa un CRUD
completo con los cinco endpoints:

```sh
# Listar tareas
curl http://localhost:8080/todos

# Crear tarea
curl -X POST http://localhost:8080/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Leer la documentacion"}'

# Obtener tarea
curl http://localhost:8080/todos/1

# Actualizar tarea
curl -X PUT http://localhost:8080/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"done": true}'

# Eliminar tarea
curl -X DELETE http://localhost:8080/todos/1
```

Arrancar el ejemplo:

```sh
export DATABASE_URL="postgresql://postgres:postgres@localhost/todos"
cargo run -p todo-api
```

---

## Que sigue

La Fase 3 introduce el Anti-DSL: defines tus modelos y endpoints en
`schema.ag` y el compilador genera todo el codigo de esta guia
automaticamente. Ver `docs/roadmap/fase-03-anti-dsl-alpha.md`.
