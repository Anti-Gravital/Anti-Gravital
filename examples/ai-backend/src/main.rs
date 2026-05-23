//! ai-backend: streaming SSE multi-proveedor (Claude, Gemini, OpenAI-compatible).
//!
//! Ejecutar con al menos una API key:
//! ```sh
//! ANTHROPIC_API_KEY=sk-ant-... cargo run -p ai-backend
//! GEMINI_API_KEY=AIza...       cargo run -p ai-backend
//! OPENAI_API_KEY=sk-...        cargo run -p ai-backend
//! # OpenAI-compatible (Ollama):
//! OPENAI_API_KEY=ollama OPENAI_BASE_URL=http://localhost:11434 cargo run -p ai-backend
//! ```

#![allow(missing_docs)]

mod handlers;
mod provider;

use ag_observe::{init as observe_init, ObserveConfig};
use axum::{
    routing::{get, post},
    Router,
};
use provider::ProviderRegistry;
use std::sync::Arc;

/// Estado compartido entre handlers.
#[derive(Clone)]
pub struct AppState {
    /// Registro dinamico de proveedores de IA disponibles.
    pub registry: Arc<ProviderRegistry>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observe_init(&ObserveConfig::from_env()).ok();

    let registry = Arc::new(ProviderRegistry::from_env());

    if registry.is_empty() {
        tracing::warn!(
            "Ningun proveedor de IA configurado. \
             Configura ANTHROPIC_API_KEY, GEMINI_API_KEY o OPENAI_API_KEY. \
             El endpoint /chat devolvera 503."
        );
    } else {
        let names: Vec<_> = registry.available().into_iter().map(|p| p.name).collect();
        tracing::info!(providers = ?names, "proveedores de IA disponibles");
    }

    let state = AppState { registry };

    let app = Router::new()
        .route("/chat", post(handlers::chat_stream))
        .route("/providers", get(handlers::list_providers))
        .route("/health", get(handlers::health))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{port}");
    tracing::info!(addr = %addr, "ai-backend iniciando");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
