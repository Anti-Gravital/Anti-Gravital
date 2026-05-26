//! HTTP handlers for the ai-backend example.

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event as SseEvent, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures_util::{stream::BoxStream, StreamExt as _};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

use crate::{provider::AiError, AppState};

/// Request body for POST /chat.
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    /// Prompt text.
    pub prompt: String,
    /// Provider to use: "claude", "gemini", "openai".
    /// If omitted, uses the registry default provider.
    pub provider: Option<String>,
    /// Specific model. If omitted, uses the provider default model.
    pub model: Option<String>,
}

/// Each SSE event sent to the client.
#[derive(Serialize)]
struct TokenEvent {
    token: String,
    done: bool,
}

/// Error event sent as the last SSE event of the stream.
#[derive(Serialize)]
struct ErrorEvent {
    error: String,
    done: bool,
}

/// POST /chat — starts an SSE stream of AI tokens.
///
/// Responses:
/// - `200` + SSE stream if a provider is available
/// - `422` if the prompt is empty
/// - `503` if the requested provider is not configured
pub async fn chat_stream(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    let prompt = req.prompt.trim();
    if prompt.is_empty() {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    }

    // Select provider
    let provider = match req.provider.as_deref() {
        Some(name) => state.registry.get(name),
        None => state.registry.default_provider(),
    };

    let Some(provider) = provider else {
        let hint = if state.registry.is_empty() {
            "Configura ANTHROPIC_API_KEY, GEMINI_API_KEY o OPENAI_API_KEY"
        } else {
            "Proveedor no disponible. Consulta GET /providers para ver los disponibles."
        };
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": hint})),
        )
            .into_response();
    };

    let model = req
        .model
        .as_deref()
        .unwrap_or_else(|| provider.default_model());

    // Start the provider stream
    let token_stream: BoxStream<'static, Result<String, AiError>> =
        match provider.stream_completion(prompt, model).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "error iniciando stream del proveedor");
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
        };

    // Final "done: true" event
    let done_event = futures_util::stream::once(async {
        let data = serde_json::to_string(&TokenEvent {
            token: String::new(),
            done: true,
        })
        .unwrap_or_default();
        Ok::<SseEvent, Infallible>(SseEvent::default().data(data))
    });

    // Convert the token stream into an SSE stream
    let sse_stream = token_stream
        .map(|result| {
            let data = match result {
                Ok(token) => {
                    serde_json::to_string(&TokenEvent { token, done: false }).unwrap_or_default()
                }
                Err(e) => serde_json::to_string(&ErrorEvent {
                    error: e.to_string(),
                    done: true,
                })
                .unwrap_or_default(),
            };
            Ok::<SseEvent, Infallible>(SseEvent::default().data(data))
        })
        .chain(done_event);

    Sse::new(sse_stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

/// Response body for GET /providers.
#[derive(Serialize)]
struct ProvidersResponse {
    providers: Vec<crate::provider::ProviderInfo>,
    default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<&'static str>,
}

/// GET /providers — lists the available providers based on the configured keys.
pub async fn list_providers(State(state): State<AppState>) -> impl IntoResponse {
    let providers = state.registry.available();
    let default = state.registry.default_name().map(String::from);
    let hint = if providers.is_empty() {
        Some("Configura ANTHROPIC_API_KEY, GEMINI_API_KEY o OPENAI_API_KEY")
    } else {
        None
    };
    Json(ProvidersResponse {
        providers,
        default,
        hint,
    })
}

/// GET /health — always 200.
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let count = state.registry.available().len();
    Json(serde_json::json!({
        "status": "ok",
        "providers_available": count
    }))
}
