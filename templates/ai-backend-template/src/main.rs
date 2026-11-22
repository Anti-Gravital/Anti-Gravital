//! Provider-agnostic chat API scaffold.
//!
//! The default provider is a local echo provider: it needs no network access
//! and no API key. Remote model providers can be introduced by implementing
//! the [`ChatProvider`] trait. Credentials for a remote provider must come
//! from the environment, never from source or config files.

use std::sync::Arc;

use ag_core::Shield;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

/// One message in a chat conversation.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Message {
    role: String,
    content: String,
}

/// Request body for `POST /chat`.
#[derive(Debug, Deserialize)]
struct ChatRequest {
    messages: Vec<Message>,
}

/// Response body for `POST /chat`.
#[derive(Debug, Serialize)]
struct ChatResponse {
    provider: &'static str,
    reply: String,
}

/// JSON error body returned together with a non-2xx status code.
#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

/// Health payload.
#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
    service: &'static str,
    provider: &'static str,
}

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ErrorBody>)>;

/// Transport-independent chat provider boundary.
trait ChatProvider: Send + Sync {
    /// Stable identifier reported to callers.
    fn name(&self) -> &'static str;

    /// Produce a reply for the given conversation.
    fn complete(&self, messages: &[Message]) -> Result<String, String>;
}

/// Local provider that echoes the most recent user message.
struct EchoProvider;

impl ChatProvider for EchoProvider {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn complete(&self, messages: &[Message]) -> Result<String, String> {
        let last = messages
            .iter()
            .rev()
            .find(|message| message.role == "user")
            .ok_or_else(|| "no user message in the conversation".to_owned())?;
        Ok(format!("echo: {}", last.content))
    }
}

#[derive(Clone)]
struct AppState {
    provider: Arc<dyn ChatProvider>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state = AppState {
        provider: Arc::new(EchoProvider),
    };
    tracing::info!(provider = state.provider.name(), "model provider selected");

    let shield = Shield::with_defaults();
    let router = Router::new()
        .route("/health", get(health_handler))
        .route("/providers", get(providers_handler))
        .route("/chat", post(chat_handler))
        .with_state(state);
    let app = shield.apply(router);

    // Bind to loopback by default; `ag dev` sets BIND to its --bind value.
    let bind = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("listening on http://{bind}");
    shield.serve(listener, app).await?;
    Ok(())
}

async fn health_handler(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        service: "{{name}}",
        provider: state.provider.name(),
    })
}

/// Lists the providers compiled into this scaffold.
async fn providers_handler() -> Json<Vec<&'static str>> {
    Json(vec!["echo"])
}

async fn chat_handler(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> ApiResult<ChatResponse> {
    let reply = state
        .provider
        .complete(&request.messages)
        .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, Json(ErrorBody { error })))?;
    Ok(Json(ChatResponse {
        provider: state.provider.name(),
        reply,
    }))
}
