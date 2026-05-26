//! realtime-chat: real-time chat with ag-realtime and SSE.
//!
//! Run:
//! ```sh
//! cargo run -p realtime-chat
//! # Open http://localhost:3000 in two browser windows
//! ```
//!
//! | Route            | Description                    |
//! |------------------|--------------------------------|
//! | GET  /           | Embedded chat UI               |
//! | GET  /events     | SSE message stream             |
//! | POST /messages   | Publish a message              |
//! | GET  /health     | Health check                   |

#![allow(missing_docs)]

mod handlers;

use ag_observe::{init as observe_init, ObserveConfig};
use ag_realtime::{AgRealtime, RealtimeConfig};
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

/// Shared state across all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Real-time subsystem for message broadcast.
    pub realtime: Arc<AgRealtime>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observe_init(&ObserveConfig::from_env()).ok();

    let realtime = Arc::new(AgRealtime::new(RealtimeConfig::default()).await?);
    let state = AppState { realtime };

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/events", get(handlers::get_events))
        .route("/messages", post(handlers::post_message))
        .route("/health", get(|| async { StatusCode::OK }))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    tracing::info!(addr = %addr, "realtime-chat starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
