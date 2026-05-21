//! Todo API: ejemplo de CRUD completo con ag-core y ag-data.
//!
//! Demuestra el roundtrip Request -> Shield -> Core -> Handler -> Respuesta
//! con conexion real a PostgreSQL. Ejecutar con:
//!
//! ```sh
//! export DATABASE_URL="postgresql://usuario:clave@localhost/todos"
//! cargo run -p todo-api
//! ```
//!
//! Endpoints disponibles:
//!
//! | Metodo | Ruta         | Descripcion              |
//! | ------ | ------------ | ------------------------ |
//! | GET    | /todos       | Lista todas las tareas   |
//! | POST   | /todos       | Crea una tarea nueva     |
//! | GET    | /todos/:id   | Obtiene una tarea        |
//! | PUT    | /todos/:id   | Actualiza una tarea      |
//! | DELETE | /todos/:id   | Elimina una tarea        |
//! | GET    | /health      | Verificacion de estado   |

#![allow(missing_docs)]

mod handlers;
mod models;

use ag_core::Shield;
use ag_data::{DataConfig, DbPool};
use axum::{routing::get, Router};
use handlers::{create_todo, delete_todo, get_todo, health, list_todos, update_todo};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: DbPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/todos".into());

    let max_connections: u32 = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    tracing::info!(max_connections, "conectando a la base de datos");
    let config = DataConfig {
        url: db_url,
        max_connections,
        ..DataConfig::default()
    };
    let pool = ag_data::connect(&config).await?;

    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
    ag_data::run_migrations(&pool, &MIGRATOR).await?;

    let state = AppState { db: pool };

    let shield = Shield::with_defaults();
    let router = Router::new()
        .route("/health", get(health))
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/:id",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .with_state(state);

    let app = shield.apply(router);

    let bind = std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(addr = %bind, "servidor iniciado");

    shield.serve(listener, app).await?;
    Ok(())
}
