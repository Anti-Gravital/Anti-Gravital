//! Two-service starter with a shared typed contract.
//!
//! Set `SERVICE_ROLE=gateway` (default) or `SERVICE_ROLE=worker` to choose the
//! service a process runs. Each service has its own bind address and routes;
//! both compile the same `contract` module, so the wire format cannot drift.
//! In production, deploy the two roles as separate processes or containers.

mod contract;

use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use contract::{EnqueueRequest, EnqueueResponse, HealthResponse, SCHEMA_VERSION};
use serde_json::json;

#[derive(Clone)]
struct GatewayState {
    sequence: Arc<AtomicU64>,
}

async fn gateway_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "gateway",
        status: "ok",
    })
}

async fn enqueue(
    State(state): State<GatewayState>,
    Json(request): Json<EnqueueRequest>,
) -> Json<EnqueueResponse> {
    let sequence = state.sequence.fetch_add(1, Ordering::Relaxed);
    let job_id = contract::new_job_id(sequence);
    tracing::info!(%job_id, job_type = %request.job_type, "job accepted");
    Json(EnqueueResponse {
        job_id,
        accepted: true,
        schema_version: SCHEMA_VERSION,
    })
}

async fn worker_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "worker",
        status: "ok",
    })
}

async fn process(Json(request): Json<EnqueueRequest>) -> Json<serde_json::Value> {
    tracing::info!(job_type = %request.job_type, "job processed");
    Json(json!({
        "job_type": request.job_type,
        "schema_version": SCHEMA_VERSION,
        "processed": true,
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let role = env::var("SERVICE_ROLE").unwrap_or_else(|_| "gateway".into());
    if role != "gateway" && role != "worker" {
        return Err(format!("unknown SERVICE_ROLE: {role}").into());
    }

    let (bind, app): (String, Router) = if role == "gateway" {
        let bind = env::var("GATEWAY_BIND").unwrap_or_else(|_| "127.0.0.1:8081".into());
        let state = GatewayState {
            sequence: Arc::new(AtomicU64::new(1)),
        };
        let app = Router::new()
            .route("/health", get(gateway_health))
            .route("/enqueue", axum::routing::post(enqueue))
            .with_state(state);
        (bind, app)
    } else {
        let bind = env::var("WORKER_BIND").unwrap_or_else(|_| "127.0.0.1:8082".into());
        let app = Router::new()
            .route("/health", get(worker_health))
            .route("/process", axum::routing::post(process));
        (bind, app)
    };

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, role = %role, "{{name}} listening");
    axum::serve(listener, app).await?;
    Ok(())
}
