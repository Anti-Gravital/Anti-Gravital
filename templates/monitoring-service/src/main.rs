//! Metrics and health endpoint scaffold.
//!
//! The default build is fully offline: it installs a tracing subscriber and
//! serves a small Prometheus text endpoint backed by atomic counters. Enabling
//! the `ag-observe` feature swaps in ag-observe's init, request recorder and
//! Prometheus handler. Both paths expose the same `/health`, `/ready` and
//! `/metrics` routes.

use std::env;
use std::time::Instant;

#[cfg(not(feature = "ag-observe"))]
use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{Response, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

#[cfg(not(feature = "ag-observe"))]
static REQUESTS: AtomicU64 = AtomicU64::new(0);
#[cfg(not(feature = "ag-observe"))]
static SERVER_ERRORS: AtomicU64 = AtomicU64::new(0);

fn init_observability() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "ag-observe")]
    {
        ag_observe::init(&ag_observe::ObserveConfig::from_env())?;
    }
    #[cfg(not(feature = "ag-observe"))]
    {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .try_init()
            .map_err(|error| format!("subscriber init: {error}"))?;
    }
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn ready() -> Json<serde_json::Value> {
    Json(json!({ "status": "ready" }))
}

async fn metrics() -> Response {
    #[cfg(feature = "ag-observe")]
    {
        ag_observe::metrics_handler().await.into_response()
    }
    #[cfg(not(feature = "ag-observe"))]
    {
        let body = format!(
            "# HELP http_requests_total Total HTTP requests.\n\
             # TYPE http_requests_total counter\n\
             http_requests_total {}\n\
             # HELP http_server_errors_total Total HTTP 5xx responses.\n\
             # TYPE http_server_errors_total counter\n\
             http_server_errors_total {}\n",
            REQUESTS.load(Ordering::Relaxed),
            SERVER_ERRORS.load(Ordering::Relaxed),
        );
        IntoResponse::into_response((
            [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
            body,
        ))
    }
}

async fn record_metrics(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_owned();
    let started = Instant::now();
    let response = next.run(request).await;
    let status = response.status().as_u16();

    #[cfg(feature = "ag-observe")]
    ag_observe::record_request(&method, &path, status, started.elapsed().as_secs_f64());

    #[cfg(not(feature = "ag-observe"))]
    {
        let _ = (method, path, started);
        REQUESTS.fetch_add(1, Ordering::Relaxed);
        if status >= 500 {
            SERVER_ERRORS.fetch_add(1, Ordering::Relaxed);
        }
    }

    response
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_observability()?;
    tracing::info!("observability initialized");

    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .layer(axum::middleware::from_fn(record_metrics));

    let bind = env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "{{name}} listening");
    axum::serve(listener, app).await?;
    Ok(())
}
