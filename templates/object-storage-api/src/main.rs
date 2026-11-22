//! Object storage API scaffold built on `ag-storage`.
//!
//! The native filesystem backend is used by default so the service builds and
//! runs with no external service. Signed URLs are generated through
//! `AgStorage::signed_url` and verified with `verify_signed_url`.

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use ag_storage::{AgStorage, StorageConfig};
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

type ApiResult<T> = Result<T, (StatusCode, String)>;

#[derive(Debug, Deserialize)]
struct SignQuery {
    #[serde(default = "default_ttl")]
    ttl: u64,
}

fn default_ttl() -> u64 {
    300
}

#[derive(Debug, Deserialize)]
struct VerifyRequest {
    key: String,
    token: String,
}

async fn health() -> &'static str {
    "ok"
}

async fn put_object(
    State(storage): State<Arc<AgStorage>>,
    Path(key): Path<String>,
    body: Bytes,
) -> ApiResult<&'static str> {
    storage
        .put(&key, body)
        .await
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;
    Ok("stored")
}

async fn get_object(
    State(storage): State<Arc<AgStorage>>,
    Path(key): Path<String>,
) -> ApiResult<Bytes> {
    storage
        .get(&key)
        .await
        .map_err(|error| (StatusCode::NOT_FOUND, error.to_string()))
}

async fn sign_object(
    State(storage): State<Arc<AgStorage>>,
    Path(key): Path<String>,
    Query(query): Query<SignQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let ttl = query.ttl.clamp(1, 86_400);
    let token = storage
        .signed_url(&key, ttl)
        .map_err(|error| (StatusCode::SERVICE_UNAVAILABLE, error.to_string()))?;
    let path = storage
        .object_url(&key)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(Json(json!({ "key": key, "token": token, "path": path, "ttl_secs": ttl })))
}

async fn verify_object(
    State(storage): State<Arc<AgStorage>>,
    Json(request): Json<VerifyRequest>,
) -> ApiResult<&'static str> {
    storage
        .verify_signed_url(&request.key, &request.token)
        .map_err(|error| (StatusCode::FORBIDDEN, error.to_string()))?;
    Ok("valid")
}

async fn build_storage() -> Result<AgStorage, Box<dyn std::error::Error>> {
    let mut config = StorageConfig::default();
    config.root_path = env::var("STORAGE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./ag-store-data"));
    // This service owns its own HTTP surface; ag-storage's embedded server is
    // off so no second listener is opened.
    config.server_mode = false;
    config.sign_secret = env::var("STORAGE_SIGN_SECRET").unwrap_or_default();
    if config.sign_secret.is_empty() {
        tracing::warn!("STORAGE_SIGN_SECRET is not set; signed URLs are disabled");
    }
    Ok(AgStorage::new(config).await?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let storage = Arc::new(build_storage().await?);

    let app = Router::new()
        .route("/health", get(health))
        .route("/objects/*key", get(get_object).put(put_object))
        .route("/sign/*key", get(sign_object))
        .route("/verify", axum::routing::post(verify_object))
        .with_state(storage);

    let bind = env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "{{name}} listening");
    axum::serve(listener, app).await?;
    Ok(())
}
