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
