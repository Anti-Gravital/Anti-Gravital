//! Handlers HTTP del example ai-backend.

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

/// Request body para POST /chat.
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    /// Texto del prompt.
    pub prompt: String,
    /// Proveedor a usar: "claude", "gemini", "openai".
    /// Si se omite, usa el proveedor por defecto del registry.
    pub provider: Option<String>,
    /// Modelo especifico. Si se omite, usa el modelo por defecto del proveedor.
    pub model: Option<String>,
}

/// Cada evento SSE enviado al cliente.
#[derive(Serialize)]
struct TokenEvent {
    token: String,
    done: bool,
}

/// Evento de error enviado como ultimo evento SSE del stream.
#[derive(Serialize)]
struct ErrorEvent {
    error: String,
    done: bool,
}

/// POST /chat — inicia un stream SSE de tokens de IA.
///
/// Respuestas:
/// - `200` + SSE stream si hay proveedor disponible
/// - `422` si el prompt esta vacio
/// - `503` si el proveedor pedido no esta configurado
pub async fn chat_stream(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    let prompt = req.prompt.trim();
    if prompt.is_empty() {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    }

    // Seleccionar proveedor
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

    // Iniciar stream del proveedor
    let token_stream: BoxStream<'static, Result<String, AiError>> =
        match provider.stream_completion(prompt, model).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "error iniciando stream del proveedor");
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
        };

    // Evento final "done: true"
    let done_event = futures_util::stream::once(async {
        let data = serde_json::to_string(&TokenEvent {
            token: String::new(),
            done: true,
        })
        .unwrap_or_default();
        Ok::<SseEvent, Infallible>(SseEvent::default().data(data))
    });

    // Convertir stream de tokens a stream SSE
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

/// Response body para GET /providers.
#[derive(Serialize)]
struct ProvidersResponse {
    providers: Vec<crate::provider::ProviderInfo>,
    default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<&'static str>,
}

/// GET /providers — lista los proveedores disponibles segun las keys configuradas.
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

/// GET /health — siempre 200.
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let count = state.registry.available().len();
    Json(serde_json::json!({
        "status": "ok",
        "providers_available": count
    }))
}
