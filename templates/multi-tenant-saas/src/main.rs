//! Multi-tenant API template.
//!
//! The request context (tenant id) is resolved once in middleware and
//! propagated to handlers through request extensions. Every stored record is
//! keyed by `(tenant_id, id)`, so a caller can never read another tenant's
//! data. Row-level security in PostgreSQL is documented in README.md; this
//! template deliberately ships no real RLS SQL.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ag_core::Shield;
use axum::{
    body::Body,
    extract::{Extension, Path, State},
    http::{header::HeaderName, HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

const TENANT_HEADER: &str = "x-tenant-id";
const MAX_TENANT_LEN: usize = 64;

#[derive(Clone, Debug)]
struct TenantId(String);

#[derive(Clone)]
struct AppState {
    resources: Arc<Mutex<HashMap<(String, String), Resource>>>,
}

#[derive(Clone, Debug, Serialize)]
struct Resource {
    id: String,
    tenant_id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CreateResource {
    name: String,
}

#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
    isolation: &'static str,
}

struct ApiError {
    status: StatusCode,
    message: &'static str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(serde_json::json!({ "error": self.message }))).into_response()
    }
}

fn missing_tenant() -> ApiError {
    ApiError {
        status: StatusCode::BAD_REQUEST,
        message: "X-Tenant-Id header is required",
    }
}

fn invalid_tenant() -> ApiError {
    ApiError {
        status: StatusCode::BAD_REQUEST,
        message: "X-Tenant-Id header is invalid",
    }
}

fn not_found() -> ApiError {
    ApiError {
        status: StatusCode::NOT_FOUND,
        message: "resource not found in this tenant",
    }
}

/// Tenant ids are restricted to an explicit allow-list of characters. The
/// same validation would be applied to a JWT `tenant_id` claim in production.
fn valid_tenant_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_TENANT_LEN
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn tenant_from_headers(headers: &HeaderMap) -> Result<TenantId, ApiError> {
    let raw = headers
        .get(HeaderName::from_static(TENANT_HEADER))
        .ok_or_else(missing_tenant)?
        .to_str()
        .map_err(|_| invalid_tenant())?;
    if !valid_tenant_id(raw) {
        return Err(invalid_tenant());
    }
    Ok(TenantId(raw.to_owned()))
}

/// Resolves and validates the tenant context once, then inserts it into the
/// request extensions so every downstream handler receives the same value.
async fn tenant_context_middleware(mut request: Request<Body>, next: Next) -> Response {
    // Keep health checks tenant-agnostic.
    if request.uri().path() == "/health" {
        return next.run(request).await;
    }

    match tenant_from_headers(request.headers()) {
        Ok(tenant) => {
            request.extensions_mut().insert(tenant);
            next.run(request).await
        }
        Err(error) => error.into_response(),
    }
}

async fn create_handler(
    State(state): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Json(payload): Json<CreateResource>,
) -> Result<(StatusCode, Json<Resource>), ApiError> {
    if payload.name.trim().is_empty() {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            message: "name must not be empty",
        });
    }
    let resource = Resource {
        id: uuid::Uuid::new_v4().to_string(),
        tenant_id: tenant.0,
        name: payload.name,
    };
    let key = (resource.tenant_id.clone(), resource.id.clone());
    let mut guard = state
        .resources
        .lock()
        .map_err(|_| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "store lock poisoned",
        })?;
    guard.insert(key, resource.clone());
    Ok((StatusCode::CREATED, Json(resource)))
}

async fn list_handler(
    State(state): State<AppState>,
    Extension(tenant): Extension<TenantId>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    let guard = state.resources.lock().map_err(|_| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: "store lock poisoned",
    })?;
    let mut items: Vec<Resource> = guard
        .values()
        .filter(|resource| resource.tenant_id == tenant.0)
        .cloned()
        .collect();
    items.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(Json(items))
}

async fn get_handler(
    State(state): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(id): Path<String>,
) -> Result<Json<Resource>, ApiError> {
    let guard = state.resources.lock().map_err(|_| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: "store lock poisoned",
    })?;
    guard
        .get(&(tenant.0, id))
        .cloned()
        .map(Json)
        .ok_or_else(not_found)
}

async fn health_handler() -> Json<Health> {
    Json(Health {
        status: "ok",
        isolation: "tenant-key-prefix-memory",
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

    if std::env::var("DATABASE_URL").is_ok() {
        tracing::info!("DATABASE_URL declared; default build keeps the in-memory isolation store");
    }

    let state = AppState {
        resources: Arc::new(Mutex::new(HashMap::new())),
    };

    let shield = Shield::with_defaults();
    let router = Router::new()
        .route("/resources", get(list_handler).post(create_handler))
        .route("/resources/:id", get(get_handler))
        .route("/health", get(health_handler))
        .with_state(state)
        .layer(middleware::from_fn(tenant_context_middleware));

    let app = shield.apply(router);
    let bind = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "listening");
    shield.serve(listener, app).await?;
    Ok(())
}
