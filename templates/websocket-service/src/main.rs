//! WebSocket echo and broadcast service with per-connection rate limits.
//!
//! Routes:
//! - GET /ws/echo       echoes each text frame back to the sender
//! - GET /ws/broadcast  relays each text frame to every broadcast subscriber
//! - GET /health        health check
//!
//! Both socket routes enforce a per-connection token-bucket rate limit on
//! inbound client messages. The broadcast route fans out through the
//! `ag-realtime` in-process bus, so no external broker is required.

use std::sync::Arc;
use std::time::Instant;

use ag_realtime::{AgRealtime, RealtimeConfig};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::sync::broadcast;

const BROADCAST_SUBJECT: &str = "ws.broadcast";

#[derive(Clone)]
struct AppState {
    realtime: Arc<AgRealtime>,
    rate: RateLimit,
}

#[derive(Clone, Copy)]
struct RateLimit {
    capacity: u32,
    refill_per_second: u32,
}

/// Simple token bucket. Each connection owns one instance, so limits are local
/// to a socket and cannot be bypassed by sharing a global counter.
struct Bucket {
    tokens: f64,
    last: Instant,
    limit: RateLimit,
}

impl Bucket {
    fn new(limit: RateLimit) -> Self {
        Self {
            tokens: limit.capacity as f64,
            last: Instant::now(),
            limit,
        }
    }

    fn allow(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f64();
        self.last = now;
        self.tokens = (self.tokens + elapsed * self.limit.refill_per_second as f64)
            .min(self.limit.capacity as f64);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
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
        rate: RateLimit {
            capacity: 20,
            refill_per_second: 10,
        },
    };

    let app = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/ws/echo", get(ws_echo))
        .route("/ws/broadcast", get(ws_broadcast))
        .with_state(state);

    let bind = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "websocket-service listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_echo(
    State(state): State<AppState>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| handle_echo(socket, state.rate))
}

async fn handle_echo(mut socket: WebSocket, limit: RateLimit) {
    let mut bucket = Bucket::new(limit);
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                if !bucket.allow() {
                    let _ = socket
                        .send(Message::Text("rate limit exceeded".into()))
                        .await;
                    continue;
                }
                let reply = format!("echo: {text}");
                if socket.send(Message::Text(reply)).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

async fn ws_broadcast(
    State(state): State<AppState>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| handle_broadcast(socket, state.realtime, state.rate))
}

async fn handle_broadcast(mut socket: WebSocket, realtime: Arc<AgRealtime>, limit: RateLimit) {
    let bus = realtime
        .bus()
        .expect("in-process event bus is the default");
    let mut rx = bus.subscribe();
    let mut bucket = Bucket::new(limit);

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if !bucket.allow() {
                            let _ = socket
                                .send(Message::Text("rate limit exceeded".into()))
                                .await;
                            continue;
                        }
                        if let Err(e) = realtime.broadcast(BROADCAST_SUBJECT, text.into_bytes()) {
                            tracing::error!(error = %e, "broadcast failed");
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            received = rx.recv() => {
                match received {
                    Ok(ev) if ev.subject == BROADCAST_SUBJECT => {
                        let text = String::from_utf8_lossy(&ev.payload).into_owned();
                        if socket.send(Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Skip dropped frames rather than tearing down the socket.
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}
