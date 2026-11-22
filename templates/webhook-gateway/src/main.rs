//! Inbound webhook gateway template.
//!
//! Every request body is verified with HMAC-SHA256 before any work is done.
//! The comparison runs inside `hmac::Mac::verify_slice`, which is constant
//! time. A small in-memory event log and delivery-id dedup show where a real
//! gateway would hand off to a queue.

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use ag_core::Shield;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

const SIGNATURE_HEADER: &str = "x-signature-256";
const DELIVERY_HEADER: &str = "x-delivery-id";
const MAX_EVENTS: usize = 1000;

#[derive(Clone)]
struct AppState {
    secret: Arc<String>,
    events: Arc<Mutex<VecDeque<Event>>>,
    seen: Arc<Mutex<HashSet<String>>>,
}

#[derive(Clone, Debug, Serialize)]
struct Event {
    id: String,
    delivery_id: Option<String>,
    body: String,
}

#[derive(Debug, Serialize)]
struct Accepted {
    status: &'static str,
    id: String,
    duplicate: bool,
}

#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
    algorithm: &'static str,
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

/// Verifies `sha256=<hex>` (or bare hex) against the raw request body.
///
/// Returns false on a malformed header, wrong length or bad signature. The
/// final comparison is delegated to `verify_slice` so it is constant time.
fn verify_signature(secret: &str, body: &[u8], header: &str) -> bool {
    let encoded = header.strip_prefix("sha256=").unwrap_or(header);
    let Ok(provided) = hex::decode(encoded) else {
        return false;
    };
    if provided.len() != 32 {
        return false;
    }
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    mac.verify_slice(&provided).is_ok()
}

async fn webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Accepted>), ApiError> {
    let signature = headers
        .get(SIGNATURE_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError {
            status: StatusCode::UNAUTHORIZED,
            message: "missing signature header",
        })?;

    if !verify_signature(&state.secret, &body, signature) {
        tracing::warn!("rejected webhook with invalid HMAC signature");
        return Err(ApiError {
            status: StatusCode::UNAUTHORIZED,
            message: "invalid signature",
        });
    }

    let delivery_id = headers
        .get(DELIVERY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    if let Some(id) = &delivery_id {
        let mut seen = state.seen.lock().map_err(|_| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "dedup lock poisoned",
        })?;
        if !seen.insert(id.clone()) {
            return Ok((
                StatusCode::ACCEPTED,
                Json(Accepted {
                    status: "duplicate",
                    id: id.clone(),
                    duplicate: true,
                }),
            ));
        }
    }

    let id = uuid::Uuid::new_v4().to_string();
    let event = Event {
        id: id.clone(),
        delivery_id,
        body: String::from_utf8_lossy(&body).into_owned(),
    };
    {
        let mut events = state.events.lock().map_err(|_| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "event lock poisoned",
        })?;
        if events.len() >= MAX_EVENTS {
            events.pop_front();
        }
        events.push_back(event);
    }
    tracing::info!(id = %id, "webhook accepted");
    Ok((
        StatusCode::OK,
        Json(Accepted {
            status: "accepted",
            id,
            duplicate: false,
        }),
    ))
}

async fn events_handler(State(state): State<AppState>) -> Result<Json<Vec<Event>>, ApiError> {
    let events = state.events.lock().map_err(|_| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: "event lock poisoned",
    })?;
    Ok(Json(events.iter().rev().cloned().collect()))
}

async fn health_handler() -> Json<Health> {
    Json(Health {
        status: "ok",
        algorithm: "hmac-sha256",
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

    let secret = std::env::var("WEBHOOK_SECRET")?;
    if secret.is_empty() {
        return Err("WEBHOOK_SECRET must not be empty".into());
    }
    let state = AppState {
        secret: Arc::new(secret),
        events: Arc::new(Mutex::new(VecDeque::new())),
        seen: Arc::new(Mutex::new(HashSet::new())),
    };

    let shield = Shield::with_defaults();
    let router = Router::new()
        .route("/webhook", axum::routing::post(webhook_handler))
        .route("/events", get(events_handler))
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

    fn signature(secret: &str, body: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("valid key");
        mac.update(body);
        format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
    }

    #[test]
    fn valid_signature_is_accepted() {
        let body = br#"{"event":"ping"}"#;
        let header = signature("secret", body);
        assert!(verify_signature("secret", body, &header));
    }

    #[test]
    fn tampered_body_is_rejected() {
        let header = signature("secret", br#"{"event":"ping"}"#);
        assert!(!verify_signature("secret", br#"{"event":"pong"}"#, &header));
    }

    #[test]
    fn malformed_and_wrong_length_headers_are_rejected() {
        assert!(!verify_signature("secret", b"body", "not-hex"));
        assert!(!verify_signature("secret", b"body", "sha256=00"));
    }
}
