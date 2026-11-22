//! Multi-room chat with per-room presence, built on `ag-realtime`.
//!
//! Routes:
//! - POST /rooms/:room/join       join a room (body: {"user": "..."})
//! - POST /rooms/:room/leave      leave a room (body: {"user": "..."})
//! - POST /rooms/:room/messages   publish a message (body: {"user": "...", "text": "..."})
//! - GET  /rooms/:room/events     SSE stream of that room's messages
//! - GET  /rooms/:room/presence   sorted list of users currently in the room
//! - GET  /health                 health check

use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use ag_realtime::{AgRealtime, RealtimeConfig};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event as SseEvent, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};

#[derive(Clone)]
struct AppState {
    realtime: Arc<AgRealtime>,
    presence: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    user: String,
    text: String,
    ts: u64,
}

#[derive(Debug, Deserialize)]
struct PostMessage {
    user: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct PresenceChange {
    user: String,
}

fn message_subject(room: &str) -> String {
    format!("chat.room.{room}.message")
}

fn presence_subject(room: &str) -> String {
    format!("chat.room.{room}.presence")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let realtime = Arc::new(AgRealtime::new(RealtimeConfig::default()).await?);
    let state = AppState {
        realtime,
        presence: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/rooms/:room/join", post(join))
        .route("/rooms/:room/leave", post(leave))
        .route("/rooms/:room/messages", post(post_message))
        .route("/rooms/:room/events", get(get_events))
        .route("/rooms/:room/presence", get(get_presence))
        .with_state(state);

    let bind = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "realtime-chat-v2 listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn post_message(
    State(state): State<AppState>,
    Path(room): Path<String>,
    Json(req): Json<PostMessage>,
) -> impl IntoResponse {
    let user = req.user.trim();
    let text = req.text.trim();
    if user.is_empty()
        || user.chars().count() > 32
        || text.is_empty()
        || text.chars().count() > 512
    {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    }

    let msg = ChatMessage {
        user: user.to_owned(),
        text: text.to_owned(),
        ts: now_secs(),
    };
    match state.realtime.broadcast_json(message_subject(&room), &msg) {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "broadcast failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn join(
    State(state): State<AppState>,
    Path(room): Path<String>,
    Json(req): Json<PresenceChange>,
) -> StatusCode {
    let user = req.user.trim().to_owned();
    if user.is_empty() || user.chars().count() > 32 {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }
    state
        .presence
        .write()
        .await
        .entry(room.clone())
        .or_default()
        .insert(user.clone());
    let event = serde_json::json!({ "room": room, "user": user, "event": "join" });
    let _ = state.realtime.broadcast_json(presence_subject(&room), &event);
    StatusCode::NO_CONTENT
}

async fn leave(
    State(state): State<AppState>,
    Path(room): Path<String>,
    Json(req): Json<PresenceChange>,
) -> StatusCode {
    let user = req.user.trim().to_owned();
    if user.is_empty() || user.chars().count() > 32 {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }
    if let Some(users) = state.presence.write().await.get_mut(&room) {
        users.remove(&user);
    }
    let event = serde_json::json!({ "room": room, "user": user, "event": "leave" });
    let _ = state.realtime.broadcast_json(presence_subject(&room), &event);
    StatusCode::NO_CONTENT
}

async fn get_presence(
    State(state): State<AppState>,
    Path(room): Path<String>,
) -> Json<Vec<String>> {
    let map = state.presence.read().await;
    let mut users: Vec<String> = map
        .get(&room)
        .map(|set| set.iter().cloned().collect())
        .unwrap_or_default();
    users.sort();
    Json(users)
}

async fn get_events(
    State(state): State<AppState>,
    Path(room): Path<String>,
) -> Sse<impl futures_util::Stream<Item = Result<SseEvent, Infallible>>> {
    let subject = message_subject(&room);
    let rx = state
        .realtime
        .bus()
        .expect("in-process event bus is the default")
        .subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |result| match result {
        Ok(ev) if ev.subject == subject => {
            let data = String::from_utf8_lossy(&ev.payload).into_owned();
            Some(Ok(SseEvent::default().data(data)))
        }
        // Lagged receiver or a different subject: ignore without dropping the client.
        _ => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
