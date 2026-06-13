//! Embedded Axum HTTP server for the Anti-Gravital store.
//!
//! Started in the background when `StorageConfig::server_mode` is `true`.
//! Exposes the REST v1 API with Bearer token authentication and global rate limiting.
//!
//! # API
//!
//! | Method | Route | Description |
//! |---|---|---|
//! | `PUT` | `/v1/objects/*key` | Upload or replace an object |
//! | `GET` | `/v1/objects/*key` | Download an object |
//! | `DELETE` | `/v1/objects/*key` | Delete an object |
//! | `HEAD` | `/v1/objects/*key` | Check existence |
//! | `GET` | `/v1/objects/` | List objects (`?prefix=`) |
//! | `POST` | `/v1/copy` | Copy (`?from=&to=`) |
//! | `GET` | `/v1/health` | Health check (no auth) |

use super::{auth::bearer_auth_middleware, AgStore};
use crate::{StorageConfig, StorageError};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Path, Query, Request, State},
    http::{header, HeaderValue, StatusCode},
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

/// Starts the HTTP server and blocks until the process ends.
pub async fn run_server(store: Arc<AgStore>, config: &StorageConfig) -> Result<(), StorageError> {
    config.validate_server_security()?;
    let addr = std::net::SocketAddr::new(config.server_bind, config.server_port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(StorageError::Io)?;
    tracing::info!(%addr, authenticated = !config.store_token.is_empty(), "ag-storage server listening");
    let app = build_router(store, config);
    axum::serve(listener, app)
        .await
        .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))
}

// ---------------------------------------------------------------------------
// Router (pub for tests)
// ---------------------------------------------------------------------------

/// Builds the Axum router with all routes and middlewares.
///
/// Separated from `run_server` for tests without binding a real port.
pub fn build_router(store: Arc<AgStore>, config: &StorageConfig) -> Router {
    let rps = NonZeroU32::new(config.rate_limit_rps).unwrap_or(NonZeroU32::MIN);
    let limiter: Arc<DefaultDirectRateLimiter> =
        Arc::new(RateLimiter::direct(Quota::per_second(rps)));
    let auth_state = super::auth::AuthState::from_config(config);
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
                    auth_state,
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
// Safe Content-Type (allowlist)
// ---------------------------------------------------------------------------

/// Returns `(content_type, needs_attachment)` for an object key.
///
/// Any unrecognized extension gets `application/octet-stream` with
/// `Content-Disposition: attachment` to avoid execution in the browser.
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
        // User-controlled active content keeps its MIME type but must not render
        // inline on the storage origin.
        "svg" => ("image/svg+xml", true),
        "ico" => ("image/x-icon", false),
        "txt" => ("text/plain; charset=utf-8", false),
        "html" | "htm" => ("text/html; charset=utf-8", true),
        "css" => ("text/css; charset=utf-8", false),
        "json" => ("application/json", false),
        "pdf" => ("application/pdf", false),
        "xml" => ("application/xml", false),
        _ => ("application/octet-stream", true),
    }
}

fn etag_for(data: &Bytes) -> String {
    // Strong validator (RFC 7232): the full 256-bit blake3 digest, so distinct
    // contents cannot collide into the same ETag at scale.
    let hash = blake3::hash(data);
    format!("\"{}\"", hash.to_hex())
}

// ---------------------------------------------------------------------------
// Application error
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
    // The object key is untrusted input and may contain bytes that are invalid
    // in an HTTP header value (control chars, non-ASCII). Echo the raw key when
    // it is a valid value; otherwise fall back to a percent-encoded form. This
    // prevents a panic (and a trivial DoS) when building the response.
    let key_header = match HeaderValue::from_str(&key) {
        Ok(value) => value,
        Err(_) => HeaderValue::from_str(&percent_encode_attr(&key))
            .unwrap_or_else(|_| HeaderValue::from_static("")),
    };
    let mut builder = Response::builder()
        .header(header::CONTENT_TYPE, ct)
        .header(header::CONTENT_LENGTH, len)
        .header("X-AG-Store-Key", key_header)
        .header(header::ETAG, etag);
    if is_attachment {
        let fname = std::path::Path::new(&key)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        builder = builder.header(header::CONTENT_DISPOSITION, content_disposition(&fname));
    }
    // Headers above are guaranteed valid, so this never errors in practice; map
    // any residual error to a 500 instead of panicking on external input.
    match builder.body(Body::from(data)) {
        Ok(response) => Ok(response),
        Err(error) => {
            tracing::error!(%error, "failed to build object response");
            Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
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
// Header encoding helpers
// ---------------------------------------------------------------------------

/// Percent-encodes `input`, keeping only RFC 5987 attr-chars unencoded, so the
/// result is always valid ASCII usable in an HTTP header value. Used to make
/// untrusted object keys safe to echo back in headers without panicking.
fn percent_encode_attr(input: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    // RFC 5987 attr-char set: ALPHA / DIGIT plus a handful of symbols.
    const SAFE: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!#$&+-.^_`|~";
    let mut out = String::with_capacity(input.len());
    for &byte in input.as_bytes() {
        if SAFE.contains(&byte) {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }
    out
}

/// Builds a safe `Content-Disposition` value. ASCII filenames use the plain
/// `filename="..."` form; names with non-token bytes use the RFC 5987
/// `filename*=UTF-8''...` form so untrusted keys never yield an invalid header.
fn content_disposition(filename: &str) -> String {
    let plain_safe = !filename.is_empty()
        && filename
            .bytes()
            .all(|b| (b.is_ascii_graphic() || b == b' ') && b != b'"' && b != b'\\');
    if plain_safe {
        format!("attachment; filename=\"{filename}\"")
    } else {
        format!(
            "attachment; filename*=UTF-8''{}",
            percent_encode_attr(filename)
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;

    #[test]
    fn etag_is_strong_full_blake3_and_collision_resistant() {
        let a = etag_for(&Bytes::from_static(b"hello"));
        let b = etag_for(&Bytes::from_static(b"hellp"));
        // Strong validator: quoted, full 64 hex chars (256-bit blake3).
        assert!(a.starts_with('"') && a.ends_with('"'));
        assert_eq!(a.trim_matches('"').len(), 64);
        // Distinct contents produce distinct ETags; identical contents match.
        assert_ne!(a, b);
        assert_eq!(a, etag_for(&Bytes::from_static(b"hello")));
    }

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
    async fn get_object_with_del_byte_key_does_not_panic() {
        // Regression: a key containing 0x7F (DEL) passes `validate_key` (which
        // only rejects bytes < 0x20) but is rejected by `HeaderValue`, so the
        // old `.body().unwrap()` panicked when echoing the key in a header -- a
        // trivial DoS reachable from the URL path (`%7F`).
        let (_dir, store) = temp_store();
        store
            .put("a\u{7f}b.txt", bytes::Bytes::from("data"))
            .await
            .unwrap();
        let config = crate::StorageConfig::default();
        let app = build_router(store, &config);

        // %7F decodes to the DEL control byte in the key.
        let req = Request::builder()
            .uri("/v1/objects/a%7Fb.txt")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        // The untrusted byte is percent-encoded in the echoed header.
        assert_eq!(resp.headers()["X-AG-Store-Key"], "a%7Fb.txt");
    }

    #[test]
    fn percent_encode_attr_encodes_unsafe_bytes() {
        assert_eq!(percent_encode_attr("plain-1.txt"), "plain-1.txt");
        assert_eq!(percent_encode_attr("café.png"), "caf%C3%A9.png");
        assert_eq!(percent_encode_attr("a b"), "a%20b");
    }

    #[test]
    fn content_disposition_uses_rfc5987_for_non_ascii() {
        assert_eq!(
            content_disposition("a.png"),
            "attachment; filename=\"a.png\""
        );
        assert!(content_disposition("café.png").starts_with("attachment; filename*=UTF-8''"));
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
    async fn active_content_is_served_as_attachment() {
        for (key, body, content_type) in [
            (
                "page.html",
                "<script>alert(1)</script>",
                "text/html; charset=utf-8",
            ),
            (
                "image.svg",
                r#"<svg onload="alert(1)"></svg>"#,
                "image/svg+xml",
            ),
        ] {
            let (_dir, store) = temp_store();
            store.put(key, Bytes::from(body)).await.unwrap();
            let app = build_router(store, &crate::StorageConfig::default());

            let response = app
                .oneshot(
                    Request::builder()
                        .uri(format!("/v1/objects/{key}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers()[header::CONTENT_TYPE], content_type);
            assert!(response.headers()[header::CONTENT_DISPOSITION]
                .to_str()
                .unwrap()
                .starts_with("attachment;"));
        }
    }

    #[test]
    fn active_content_types_require_attachment() {
        assert_eq!(
            content_type_for("page.html"),
            ("text/html; charset=utf-8", true)
        );
        assert_eq!(
            content_type_for("page.htm"),
            ("text/html; charset=utf-8", true)
        );
        assert_eq!(content_type_for("image.svg"), ("image/svg+xml", true));
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
