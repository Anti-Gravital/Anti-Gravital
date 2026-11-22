//! Capability-scoped file upload service built on ag-storage.
//!
//! Two layers of confinement are demonstrated:
//! 1. `ag-storage` roots every object in a directory and uses `cap-std` with
//!    symlink-safe key validation, so a key cannot escape the store.
//! 2. Callers must present a signed capability bound to a key prefix and a
//!    scope (`r`, `w` or `rw`). Every object request is checked against it.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use ag_core::Shield;
use ag_storage::{signed::{sign_url, verify_signed_url, SignedUrlError}, AgStorage, StorageConfig};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

const CAP_PREFIX_HEADER: &str = "x-capability-prefix";
const CAP_SCOPE_HEADER: &str = "x-capability-scope";
const CAP_TOKEN_HEADER: &str = "x-capability-token";
const DEFAULT_TTL_SECS: u64 = 3600;
const MAX_PREFIX_LEN: usize = 128;

#[derive(Clone)]
struct AppState {
    storage: Arc<AgStorage>,
    secret: Arc<String>,
}

#[derive(Debug, Deserialize)]
struct CapabilityRequest {
    prefix: String,
    scope: String,
    #[serde(default)]
    ttl_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
struct Capability {
    prefix: String,
    scope: String,
    expires_at: u64,
    token: String,
}

#[derive(Debug, Serialize)]
struct ObjectInfo {
    key: String,
    size: usize,
    url: String,
}

#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
    backend: &'static str,
    capability: &'static str,
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn unauthorized(message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: "object not found".into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(serde_json::json!({ "error": self.message }))).into_response()
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Prefixes are relative, dot-free and always end with `/`. This is the
/// application-level half of the confinement check.
fn valid_prefix(prefix: &str) -> bool {
    !prefix.is_empty()
        && prefix.len() <= MAX_PREFIX_LEN
        && prefix.ends_with('/')
        && !prefix.starts_with('/')
        && !prefix.contains("..")
        && !prefix.contains('\\')
        && prefix.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '.')
        })
}

fn valid_scope(scope: &str) -> bool {
    !scope.is_empty()
        && scope.len() <= 2
        && scope.chars().all(|c| c == 'r' || c == 'w')
}

fn descriptor(prefix: &str, scope: &str) -> String {
    format!("{prefix}|{scope}")
}

/// Checks that the key lives inside the authorized prefix and that the
/// caller's signed capability covers the required operation.
fn authorize(
    secret: &str,
    key: &str,
    required: char,
    headers: &HeaderMap,
) -> Result<(), ApiError> {
    let prefix = headers
        .get(CAP_PREFIX_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("missing capability prefix"))?;
    let scope = headers
        .get(CAP_SCOPE_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("missing capability scope"))?;
    let token = headers
        .get(CAP_TOKEN_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("missing capability token"))?;

    if !valid_prefix(prefix) || !valid_scope(scope) {
        return Err(ApiError::bad_request("invalid capability descriptor"));
    }
    if !key.starts_with(prefix) {
        return Err(ApiError::unauthorized("key is outside the granted prefix"));
    }
    if !scope.contains(required) {
        return Err(ApiError::unauthorized("capability scope does not allow this operation"));
    }

    match verify_signed_url(secret, &descriptor(prefix, scope), token) {
        Ok(()) => Ok(()),
        Err(SignedUrlError::Expired) => Err(ApiError::unauthorized("capability expired")),
        Err(_) => Err(ApiError::unauthorized("invalid capability signature")),
    }
}

async fn create_capability_handler(
    State(state): State<AppState>,
    Json(payload): Json<CapabilityRequest>,
) -> Result<Json<Capability>, ApiError> {
    if !valid_prefix(&payload.prefix) || !valid_scope(&payload.scope) {
        return Err(ApiError::bad_request("invalid prefix or scope"));
    }
    let ttl = payload.ttl_secs.unwrap_or(DEFAULT_TTL_SECS).min(86400);
    let expires_at = now_secs() + ttl;
    let token = sign_url(
        &state.secret,
        &descriptor(&payload.prefix, &payload.scope),
        expires_at,
    )
    .map_err(|e| ApiError::bad_request(&e.to_string()))?;
    Ok(Json(Capability {
        prefix: payload.prefix,
        scope: payload.scope,
        expires_at,
        token,
    }))
}

async fn put_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(key): axum::extract::Path<String>,
    body: Bytes,
) -> Result<(StatusCode, Json<ObjectInfo>), ApiError> {
    authorize(&state.secret, &key, 'w', &headers)?;
    let size = body.len();
    state
        .storage
        .put(&key, body)
        .await
        .map_err(|e| ApiError::bad_request(&e.to_string()))?;
    let url = state
        .storage
        .object_url(&key)
        .map_err(|e| ApiError::bad_request(&e.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(ObjectInfo { key, size, url }),
    ))
}

async fn get_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(key): axum::extract::Path<String>,
) -> Result<Bytes, ApiError> {
    authorize(&state.secret, &key, 'r', &headers)?;
    state.storage.get(&key).await.map_err(|e| match e {
        ag_storage::StorageError::NotFound(_) => ApiError::not_found(),
        other => ApiError::bad_request(&other.to_string()),
    })
}

async fn health_handler() -> Json<Health> {
    Json(Health {
        status: "ok",
        backend: "ag-storage-native",
        capability: "hmac-sha256-signed-prefix",
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

    let secret = std::env::var("STORAGE_SIGN_SECRET")?;
    if secret.is_empty() {
        return Err("STORAGE_SIGN_SECRET must not be empty".into());
    }
    let config = StorageConfig {
        root_path: std::env::var("STORAGE_ROOT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("./ag-store-data")),
        sign_secret: secret.clone(),
        ..StorageConfig::default()
    };
    let storage = AgStorage::new(config).await?;

    let state = AppState {
        storage: Arc::new(storage),
        secret: Arc::new(secret),
    };

    let shield = Shield::with_defaults();
    let router = Router::new()
        .route("/capabilities", post(create_capability_handler))
        .route("/objects/*key", put(put_handler).get(get_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    let app = shield.apply(router);
    let bind = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "listening");
    shield.serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_validation_rejects_escapes() {
        assert!(valid_prefix("tenants/acme/"));
        assert!(!valid_prefix("../etc/"));
        assert!(!valid_prefix("/absolute/"));
        assert!(!valid_prefix("tenants/acme"));
    }

    #[test]
    fn scope_validation_is_restrictive() {
        assert!(valid_scope("r"));
        assert!(valid_scope("rw"));
        assert!(!valid_scope("admin"));
        assert!(!valid_scope(""));
    }
}
