//! CRUD API template backed by ag-data.
//!
//! Default build: in-memory store, no database required.
//! With `--features postgres` and `DATABASE_URL`: real persisted store whose
//! schema is applied from `migrations/` through `ag_data::run_migrations`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ag_core::Shield;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
enum Store {
    Memory(Arc<Mutex<HashMap<String, Record>>>),
    #[cfg(feature = "postgres")]
    Postgres(ag_data::DbPool),
}

#[derive(Clone)]
struct AppState {
    store: Store,
    backend: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct Record {
    id: String,
    name: String,
    value: i64,
}

#[derive(Debug, Deserialize)]
struct CreateRecord {
    name: String,
    value: i64,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    backend: &'static str,
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: "record not found".into(),
        }
    }

    fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(serde_json::json!({ "error": self.message }))).into_response()
    }
}

impl Store {
    async fn create(&self, record: Record) -> Result<Record, ApiError> {
        match self {
            Store::Memory(map) => {
                let mut guard = map.lock().map_err(|e| {
                    ApiError::bad_request(&format!("store lock poisoned: {e}"))
                })?;
                guard.insert(record.id.clone(), record.clone());
                Ok(record)
            }
            #[cfg(feature = "postgres")]
            Store::Postgres(pool) => {
                sqlx::query("INSERT INTO records (id, name, value) VALUES ($1, $2, $3)")
                    .bind(&record.id)
                    .bind(&record.name)
                    .bind(record.value)
                    .execute(pool)
                    .await
                    .map_err(|e| ApiError::bad_request(&e.to_string()))?;
                Ok(record)
            }
        }
    }

    async fn get(&self, id: &str) -> Result<Record, ApiError> {
        match self {
            Store::Memory(map) => {
                let guard = map
                    .lock()
                    .map_err(|e| ApiError::bad_request(&format!("store lock poisoned: {e}")))?;
                guard.get(id).cloned().ok_or_else(ApiError::not_found)
            }
            #[cfg(feature = "postgres")]
            Store::Postgres(pool) => {
                use sqlx::Row;
                let row = sqlx::query("SELECT id, name, value FROM records WHERE id = $1")
                    .bind(id)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| ApiError::bad_request(&e.to_string()))?
                    .ok_or_else(ApiError::not_found)?;
                Ok(Record {
                    id: row.try_get("id").map_err(|e| ApiError::bad_request(&e.to_string()))?,
                    name: row.try_get("name").map_err(|e| ApiError::bad_request(&e.to_string()))?,
                    value: row.try_get("value").map_err(|e| ApiError::bad_request(&e.to_string()))?,
                })
            }
        }
    }

    async fn list(&self) -> Result<Vec<Record>, ApiError> {
        match self {
            Store::Memory(map) => {
                let guard = map
                    .lock()
                    .map_err(|e| ApiError::bad_request(&format!("store lock poisoned: {e}")))?;
                let mut items: Vec<Record> = guard.values().cloned().collect();
                items.sort_by(|a, b| a.id.cmp(&b.id));
                Ok(items)
            }
            #[cfg(feature = "postgres")]
            Store::Postgres(pool) => {
                use sqlx::Row;
                let rows = sqlx::query("SELECT id, name, value FROM records ORDER BY id")
                    .fetch_all(pool)
                    .await
                    .map_err(|e| ApiError::bad_request(&e.to_string()))?;
                rows.into_iter()
                    .map(|row| {
                        Ok(Record {
                            id: row.try_get("id").map_err(|e| ApiError::bad_request(&e.to_string()))?,
                            name: row.try_get("name").map_err(|e| ApiError::bad_request(&e.to_string()))?,
                            value: row.try_get("value").map_err(|e| ApiError::bad_request(&e.to_string()))?,
                        })
                    })
                    .collect()
            }
        }
    }
}

async fn build_store() -> Result<(Store, &'static str), Box<dyn std::error::Error>> {
    #[cfg(feature = "postgres")]
    if let Ok(url) = std::env::var("DATABASE_URL") {
        let config = ag_data::DataConfig {
            url,
            ..ag_data::DataConfig::default()
        };
        let pool = ag_data::connect(&config).await?;
        static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
        ag_data::run_migrations(&pool, &MIGRATOR).await?;
        return Ok((Store::Postgres(pool), "postgres"));
    }

    #[cfg(not(feature = "postgres"))]
    if std::env::var("DATABASE_URL").is_ok() {
        tracing::warn!("DATABASE_URL set but the postgres feature is off; using in-memory store");
    }

    let mut map = HashMap::new();
    let seed = Record {
        id: "00000000-0000-0000-0000-000000000001".into(),
        name: "seed record".into(),
        value: 42,
    };
    map.insert(seed.id.clone(), seed);
    Ok((Store::Memory(Arc::new(Mutex::new(map))), "memory"))
}

async fn create_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateRecord>,
) -> Result<(StatusCode, Json<Record>), ApiError> {
    if payload.name.trim().is_empty() {
        return Err(ApiError::bad_request("name must not be empty"));
    }
    let record = Record {
        id: uuid::Uuid::new_v4().to_string(),
        name: payload.name,
        value: payload.value,
    };
    let saved = state.store.create(record).await?;
    Ok((StatusCode::CREATED, Json(saved)))
}

async fn get_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Record>, ApiError> {
    Ok(Json(state.store.get(&id).await?))
}

async fn list_handler(State(state): State<AppState>) -> Result<Json<Vec<Record>>, ApiError> {
    Ok(Json(state.store.list().await?))
}

async fn health_handler(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        backend: state.backend,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let (store, backend) = build_store().await?;
    tracing::info!(backend, "store ready");

    let state = AppState { store, backend };
    let shield = Shield::with_defaults();
    let router = Router::new()
        .route("/records", get(list_handler).post(create_handler))
        .route("/records/:id", get(get_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    let app = shield.apply(router);
    let bind = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "listening");
    shield.serve(listener, app).await?;
    Ok(())
}
