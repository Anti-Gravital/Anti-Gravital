//! Authentication service template built on ag-auth and ag-core.
//!
//! Demonstrates Ed25519 JWT issuance/verification through `JwtSigner` and
//! BLAKE3 API keys through the `ag_auth` helpers. All routes are wrapped by
//! the `ag-core` Shield pipeline.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use ag_auth::{generate_api_key, verify_api_key, Claims, JwtSigner};
use ag_core::Shield;
use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

const DEFAULT_TTL_SECS: u64 = 3600;

#[derive(Clone)]
struct AppState {
    signer: Arc<JwtSigner>,
    ttl_secs: u64,
    mode: &'static str,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    sub: String,
    #[serde(default = "default_role")]
    role: String,
}

fn default_role() -> String {
    "user".into()
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: &'static str,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct ApiKeyInput {
    key: String,
    hash: String,
}

#[derive(Debug, Serialize)]
struct ApiKeyCreated {
    /// Shown once; the caller stores only `hash`.
    api_key: String,
    key_hash: String,
}

#[derive(Debug, Serialize)]
struct ApiKeyCheck {
    valid: bool,
}

#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
    mode: &'static str,
}

struct ApiError {
    status: StatusCode,
    message: String,
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

/// Builds a signer from persistent PEM keys when both are provided, otherwise
/// generates an ephemeral Ed25519 keypair suitable for local development.
fn build_signer() -> Result<(JwtSigner, &'static str), Box<dyn std::error::Error>> {
    if let (Ok(private_pem), Ok(public_pem)) =
        (std::env::var("JWT_PRIVATE_KEY"), std::env::var("JWT_PUBLIC_KEY"))
    {
        return Ok((JwtSigner::new(private_pem, public_pem), "persistent"));
    }

    tracing::warn!("JWT_PRIVATE_KEY/JWT_PUBLIC_KEY absent; generating ephemeral demo keypair");
    use ed25519_dalek::{
        pkcs8::{spki::der::pem::LineEnding, EncodePrivateKey, EncodePublicKey},
        SigningKey,
    };
    let mut secret = [0u8; 32];
    getrandom::getrandom(&mut secret)?;
    let signing = SigningKey::from_bytes(&secret);
    let verifying = signing.verifying_key();
    let private_pem = signing.to_pkcs8_pem(LineEnding::LF)?.to_string();
    let public_pem = verifying.to_public_key_pem(LineEnding::LF)?.to_string();
    Ok((JwtSigner::new(private_pem, public_pem), "ephemeral"))
}

fn bearer_token(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|token| !token.is_empty())
        .ok_or_else(|| ApiError {
            status: StatusCode::UNAUTHORIZED,
            message: "missing or malformed Bearer token".into(),
        })
}

async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    if payload.sub.trim().is_empty() {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            message: "sub must not be empty".into(),
        });
    }
    let now = now_secs();
    let claims = Claims {
        sub: payload.sub,
        exp: now + state.ttl_secs,
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
        role: payload.role,
    };
    let access_token = state.signer.sign(&claims).map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: e.to_string(),
    })?;
    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer",
        expires_in: state.ttl_secs,
    }))
}

async fn me_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Claims>, ApiError> {
    let token = bearer_token(&headers)?;
    let claims = state.signer.verify(token).map_err(|e| ApiError {
        status: StatusCode::UNAUTHORIZED,
        message: e.to_string(),
    })?;
    Ok(Json(claims))
}

async fn create_api_key_handler() -> Json<ApiKeyCreated> {
    let (api_key, key_hash) = generate_api_key("sk");
    Json(ApiKeyCreated { api_key, key_hash })
}

async fn verify_api_key_handler(Json(payload): Json<ApiKeyInput>) -> Json<ApiKeyCheck> {
    Json(ApiKeyCheck {
        valid: verify_api_key(&payload.key, &payload.hash),
    })
}

async fn health_handler(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        mode: state.mode,
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

    let (signer, mode) = build_signer()?;
    let ttl_secs = std::env::var("JWT_TTL_SECS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_TTL_SECS);
    tracing::info!(mode, ttl_secs, "auth signer ready");

    let state = AppState {
        signer: Arc::new(signer),
        ttl_secs,
        mode,
    };

    let shield = Shield::with_defaults();
    let router = Router::new()
        .route("/login", post(login_handler))
        .route("/me", get(me_handler))
        .route("/api-keys", post(create_api_key_handler))
        .route("/api-keys/verify", post(verify_api_key_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    let app = shield.apply(router);
    let bind = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "listening");
    shield.serve(listener, app).await?;
    Ok(())
}
