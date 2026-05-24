//! Servidor HTTP Axum embebido del store Anti-Gravital.
//!
//! Se levanta en background cuando `StorageConfig::server_mode` es `true`.
//! Expone la API REST v1 con autenticacion Bearer token y rate limiting global.
//!
//! # API
//!
//! | Metodo | Ruta | Descripcion |
//! |---|---|---|
//! | `PUT` | `/v1/objects/*key` | Subir o reemplazar un objeto |
//! | `GET` | `/v1/objects/*key` | Descargar un objeto |
//! | `DELETE` | `/v1/objects/*key` | Borrar un objeto |
//! | `HEAD` | `/v1/objects/*key` | Verificar existencia |
//! | `GET` | `/v1/objects/` | Listar objetos (`?prefix=`) |
//! | `POST` | `/v1/copy` | Copiar (`?from=&to=`) |
//! | `GET` | `/v1/health` | Health check (sin auth) |

use super::{auth::bearer_auth_middleware, AgStore};
use crate::{StorageConfig, StorageError};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Path, Query, Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{delete, get, head, post, put},
    Json, Router,
};
use bytes::Bytes;
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use serde::{Deserialize, Serialize};
use std::{num::NonZeroU32, sync::Arc};
use tower::ServiceBuilder;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Arranca el servidor HTTP y bloquea hasta que el proceso termina.
pub async fn run_server(store: Arc<AgStore>, config: &StorageConfig) -> Result<(), StorageError> {
    let addr = format!("0.0.0.0:{}", config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(StorageError::Io)?;
    tracing::info!(port = config.server_port, "ag-storage server escuchando");
    let app = build_router(store, config);
    axum::serve(listener, app)
        .await
        .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))
}

// ---------------------------------------------------------------------------
// Router (pub para tests)
// ---------------------------------------------------------------------------

/// Construye el router Axum con todas las rutas y middlewares.
///
/// Separado de `run_server` para tests sin bind de puerto real.
pub fn build_router(store: Arc<AgStore>, config: &StorageConfig) -> Router {
    let rps = NonZeroU32::new(config.rate_limit_rps).unwrap_or(NonZeroU32::MIN);
    let limiter: Arc<DefaultDirectRateLimiter> =
        Arc::new(RateLimiter::direct(Quota::per_second(rps)));
    let token = Arc::new(config.store_token.clone());
    let max_body = config.max_object_size_mb as usize * 1024 * 1024;

    let protected = Router::new()
        .route("/v1/objects/", get(list_objects))
        .route("/v1/objects/*key", get(get_object))
        .route("/v1/objects/*key", put(put_object))
        .route("/v1/objects/*key", delete(delete_object))
        .route("/v1/objects/*key", head(head_object))
        .route("/v1/copy", post(copy_object))
        .with_state(store)
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(
                    limiter,
                    rate_limit_middleware,
                ))
                .layer(axum::middleware::from_fn_with_state(
                    token,
                    bearer_auth_middleware,
                )),
        );

    let public = Router::new().route("/v1/health", get(health));

    Router::new()
        .merge(protected)
        .merge(public)
        .layer(DefaultBodyLimit::max(max_body))
}

// ---------------------------------------------------------------------------
// Rate limit middleware
// ---------------------------------------------------------------------------

async fn rate_limit_middleware(
    State(limiter): State<Arc<DefaultDirectRateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if limiter.check().is_err() {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    next.run(request).await
}

// ---------------------------------------------------------------------------
// Content-Type seguro (lista positiva)
// ---------------------------------------------------------------------------

/// Retorna `(content_type, needs_attachment)` para una clave de objeto.
///
/// Cualquier extension no reconocida recibe `application/octet-stream`
/// con `Content-Disposition: attachment` para evitar ejecucion en navegador.
fn content_type_for(key: &str) -> (&'static str, bool) {
    let ext = std::path::Path::new(key)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "jpg" | "jpeg" => ("image/jpeg", false),
        "png" => ("image/png", false),
        "gif" => ("image/gif", false),
        "webp" => ("image/webp", false),
        // TECH-DEBT:
        // motivo: SVG puede contener JavaScript y disparar XSS en navegadores.
        // impacto: objetos SVG se sirven inline; usar un sanitizador o forzar
        //          attachment para mitigar XSS en clientes browser.
        // eliminacion esperada: segunda iteracion ag-storage con politica de CSP.
        "svg" => ("image/svg+xml", false),
        "ico" => ("image/x-icon", false),
        "txt" => ("text/plain; charset=utf-8", false),
        "html" | "htm" => ("text/html; charset=utf-8", false),
        "css" => ("text/css; charset=utf-8", false),
        "json" => ("application/json", false),
        "pdf" => ("application/pdf", false),
        "xml" => ("application/xml", false),
        _ => ("application/octet-stream", true),
    }
}

fn etag_for(data: &Bytes) -> String {
    // TECH-DEBT:
    // motivo: ETag truncado a 16 hex chars (64 bits de blake3). RFC 7232
    //         recomienda hash completo para ETags fuertes; 64 bits pueden
    //         colisionar a escala alta.
    // impacto: posibles falsos cache-hits con objetos distintos.
    // eliminacion esperada: segunda iteracion ag-storage con ETag completo.
    let hash = blake3::hash(data);
    format!("\"{}\"", &hash.to_hex()[..16])
}

// ---------------------------------------------------------------------------
// Error de aplicacion
// ---------------------------------------------------------------------------

enum AppError {
    Storage(StorageError),
}

impl From<StorageError> for AppError {
    fn from(e: StorageError) -> Self {
        AppError::Storage(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Storage(StorageError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
            AppError::Storage(StorageError::InvalidKey(_))
            | AppError::Storage(StorageError::PathEscape(_)) => {
                StatusCode::BAD_REQUEST.into_response()
            }
            AppError::Storage(StorageError::TooLarge { .. }) => {
                StatusCode::PAYLOAD_TOO_LARGE.into_response()
            }
            AppError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn put_object(
    State(store): State<Arc<AgStore>>,
    Path(key): Path<String>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    store.put(&key, body).await?;
    Ok(StatusCode::CREATED)
}

async fn get_object(
    State(store): State<Arc<AgStore>>,
    Path(key): Path<String>,
) -> Result<Response, AppError> {
    let data = store.get(&key).await?;
    let (ct, is_attachment) = content_type_for(&key);
    let etag = etag_for(&data);
    let len = data.len();
    let mut builder = Response::builder()
        .header(header::CONTENT_TYPE, ct)
        .header(header::CONTENT_LENGTH, len)
        .header("X-AG-Store-Key", &key)
        .header(header::ETAG, etag);
    if is_attachment {
        let fname = std::path::Path::new(&key)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        builder = builder.header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{fname}\""),
        );
    }
    Ok(builder.body(Body::from(data)).unwrap())
}

async fn delete_object(
    State(store): State<Arc<AgStore>>,
    Path(key): Path<String>,
) -> Result<StatusCode, AppError> {
    store.delete(&key).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn head_object(
    State(store): State<Arc<AgStore>>,
    Path(key): Path<String>,
) -> Result<StatusCode, AppError> {
    if store.exists(&key).await? {
        Ok(StatusCode::OK)
    } else {
        Err(AppError::Storage(StorageError::NotFound(key)))
    }
}

#[derive(Deserialize)]
struct ListParams {
    prefix: Option<String>,
}

#[derive(Serialize)]
struct ListResponse {
    keys: Vec<String>,
    prefix: Option<String>,
    count: usize,
}

async fn list_objects(
    State(store): State<Arc<AgStore>>,
    Query(params): Query<ListParams>,
) -> Result<Json<ListResponse>, AppError> {
    let prefix = params.prefix.as_deref();
    let keys = store.list(prefix).await?;
    let count = keys.len();
    Ok(Json(ListResponse {
        keys,
        prefix: prefix.map(String::from),
        count,
    }))
}

#[derive(Deserialize)]
struct CopyParams {
    from: String,
    to: String,
}

async fn copy_object(
    State(store): State<Arc<AgStore>>,
    Query(params): Query<CopyParams>,
) -> Result<StatusCode, AppError> {
    store.copy(&params.from, &params.to).await?;
    Ok(StatusCode::CREATED)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;

    fn temp_store() -> (tempfile::TempDir, Arc<AgStore>) {
        let dir = tempfile::tempdir().unwrap();
        let cfg = crate::StorageConfig {
            root_path: dir.path().to_path_buf(),
            ..crate::StorageConfig::default()
        };
        let store = Arc::new(AgStore::new(&cfg).unwrap());
        (dir, store)
    }

    #[tokio::test]
    async fn server_health_check() {
        let (_dir, store) = temp_store();
        let config = crate::StorageConfig::default();
        let app = build_router(store, &config);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn server_put_get_roundtrip() {
        let (_dir, store) = temp_store();
        let config = crate::StorageConfig::default();
        let app = build_router(Arc::clone(&store), &config);

        // PUT
        let put_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/v1/objects/test/hello.txt")
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Body::from("hola mundo"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(put_res.status(), StatusCode::CREATED);

        // GET
        let get_res = app
            .oneshot(
                Request::builder()
                    .uri("/v1/objects/test/hello.txt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_res.status(), StatusCode::OK);
        assert_eq!(get_res.headers()["X-AG-Store-Key"], "test/hello.txt");
    }

    #[tokio::test]
    async fn put_and_get_object() {
        let (_dir, store) = temp_store();
        let config = crate::StorageConfig::default();
        let app = build_router(store, &config);

        let put_req = Request::builder()
            .method("PUT")
            .uri("/v1/objects/myfile.txt")
            .body(Body::from("hello"))
            .unwrap();
        let resp = app.clone().oneshot(put_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let get_req = Request::builder()
            .uri("/v1/objects/myfile.txt")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(get_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn delete_existing_object() {
        let (_dir, store) = temp_store();
        store
            .put("todelete.txt", bytes::Bytes::from("bye"))
            .await
            .unwrap();
        let config = crate::StorageConfig::default();
        let app = build_router(store, &config);

        let req = Request::builder()
            .method("DELETE")
            .uri("/v1/objects/todelete.txt")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn head_existing_object_returns_200() {
        let (_dir, store) = temp_store();
        store
            .put("head.txt", bytes::Bytes::from("data"))
            .await
            .unwrap();
        let config = crate::StorageConfig::default();
        let app = build_router(store, &config);

        let req = Request::builder()
            .method("HEAD")
            .uri("/v1/objects/head.txt")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn head_missing_object_returns_404() {
        let (_dir, store) = temp_store();
        let config = crate::StorageConfig::default();
        let app = build_router(store, &config);

        let req = Request::builder()
            .method("HEAD")
            .uri("/v1/objects/nonexistent.bin")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_objects_returns_200() {
        let (_dir, store) = temp_store();
        store
            .put("ns/a.bin", bytes::Bytes::from("a"))
            .await
            .unwrap();
        let config = crate::StorageConfig::default();
        let app = build_router(store, &config);

        let req = Request::builder()
            .uri("/v1/objects/?prefix=ns/")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn copy_object_returns_201() {
        let (_dir, store) = temp_store();
        store
            .put("src.txt", bytes::Bytes::from("data"))
            .await
            .unwrap();
        let config = crate::StorageConfig::default();
        let app = build_router(store, &config);

        let req = Request::builder()
            .method("POST")
            .uri("/v1/copy?from=src.txt&to=dst.txt")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn get_missing_object_returns_404() {
        let (_dir, store) = temp_store();
        let config = crate::StorageConfig::default();
        let app = build_router(store, &config);

        let req = Request::builder()
            .uri("/v1/objects/missing.txt")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn bearer_auth_protects_objects() {
        let (_dir, store) = temp_store();
        let config = crate::StorageConfig {
            store_token: "secret".to_string(),
            ..crate::StorageConfig::default()
        };
        let app = build_router(store, &config);

        let req = Request::builder()
            .uri("/v1/objects/test.txt")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
