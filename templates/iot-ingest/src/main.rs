//! Batched IoT event ingestion with bounded-channel backpressure.
//!
//! Routes:
//! - POST /events         accept a JSON batch of device events
//! - GET  /stats          per-device count and last value seen
//! - GET  /events/stream  SSE stream of batch summaries (subject `iot.batch`)
//! - GET  /health         health check
//!
//! The HTTP handler awaits `mpsc::Sender::send` on a bounded channel. When the
//! channel is full the producer is slowed (backpressure) and, past a bounded
//! request size, rejected with 503. A background task drains the channel and
//! flushes batches by size or interval, publishing a summary via `ag-realtime`.

use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ag_realtime::{AgRealtime, RealtimeConfig};
use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event as SseEvent, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};

const CHANNEL_CAPACITY: usize = 128;
const BATCH_SIZE: usize = 32;
const MAX_BATCH_EVENTS: usize = 1000;
const BATCH_SUBJECT: &str = "iot.batch";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceEvent {
    device_id: String,
    ts: u64,
    value: f64,
}

#[derive(Debug, Clone, Serialize)]
struct BatchSummary {
    count: usize,
    devices: usize,
    max_value: f64,
    flushed_at: u64,
}

#[derive(Debug, Serialize)]
struct DeviceStat {
    device_id: String,
    count: u64,
    last_value: f64,
}

#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<DeviceEvent>,
    stats: Arc<RwLock<HashMap<String, (u64, f64)>>>,
    realtime: Arc<AgRealtime>,
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

    let (tx, rx) = mpsc::channel::<DeviceEvent>(CHANNEL_CAPACITY);
    let realtime = Arc::new(AgRealtime::new(RealtimeConfig::default()).await?);
    let stats: Arc<RwLock<HashMap<String, (u64, f64)>>> =
        Arc::new(RwLock::new(HashMap::new()));

    tokio::spawn(ingest_loop(
        rx,
        stats.clone(),
        realtime.clone(),
        BATCH_SIZE,
        Duration::from_millis(200),
    ));

    let state = AppState {
        tx,
        stats,
        realtime,
    };

    let app = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/events", post(post_events))
        .route("/events/stream", get(stream_batches))
        .route("/stats", get(get_stats))
        .with_state(state);

    let bind = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "iot-ingest listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ingest_loop(
    mut rx: mpsc::Receiver<DeviceEvent>,
    stats: Arc<RwLock<HashMap<String, (u64, f64)>>>,
    realtime: Arc<AgRealtime>,
    batch_size: usize,
    flush_interval: Duration,
) {
    let mut batch = Vec::with_capacity(batch_size);
    let mut tick = tokio::time::interval(flush_interval);
    // The first tick completes immediately; skip it so it does not flush an
    // empty batch at startup.
    tick.tick().await;

    loop {
        tokio::select! {
            incoming = rx.recv() => {
                match incoming {
                    Some(event) => {
                        batch.push(event);
                        if batch.len() >= batch_size {
                            flush_batch(&mut batch, &stats, &realtime).await;
                        }
                    }
                    None => {
                        flush_batch(&mut batch, &stats, &realtime).await;
                        break;
                    }
                }
            }
            _ = tick.tick() => {
                flush_batch(&mut batch, &stats, &realtime).await;
            }
        }
    }
}

async fn flush_batch(
    batch: &mut Vec<DeviceEvent>,
    stats: &Arc<RwLock<HashMap<String, (u64, f64)>>>,
    realtime: &Arc<AgRealtime>,
) {
    if batch.is_empty() {
        return;
    }
    let count = batch.len();
    let mut devices = HashSet::new();
    let mut max_value = f64::NEG_INFINITY;

    {
        let mut map = stats.write().await;
        for event in batch.iter() {
            devices.insert(event.device_id.clone());
            max_value = max_value.max(event.value);
            let entry = map.entry(event.device_id.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 = event.value;
        }
    }

    let summary = BatchSummary {
        count,
        devices: devices.len(),
        max_value,
        flushed_at: now_secs(),
    };
    if let Err(e) = realtime.broadcast_json(BATCH_SUBJECT, &summary) {
        tracing::warn!(error = %e, "batch summary broadcast failed");
    }
    batch.clear();
}

async fn post_events(
    State(state): State<AppState>,
    Json(events): Json<Vec<DeviceEvent>>,
) -> impl IntoResponse {
    if events.is_empty() || events.len() > MAX_BATCH_EVENTS {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    }
    for event in events {
        // `send` awaits when the bounded channel is full: that is the
        // backpressure signal propagated to the HTTP client.
        if state.tx.send(event).await.is_err() {
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    }
    StatusCode::ACCEPTED.into_response()
}

async fn get_stats(State(state): State<AppState>) -> Json<Vec<DeviceStat>> {
    let map = state.stats.read().await;
    let mut rows: Vec<DeviceStat> = map
        .iter()
        .map(|(device_id, (count, last_value))| DeviceStat {
            device_id: device_id.clone(),
            count: *count,
            last_value: *last_value,
        })
        .collect();
    rows.sort_by(|a, b| a.device_id.cmp(&b.device_id));
    Json(rows)
}

async fn stream_batches(
    State(state): State<AppState>,
) -> Sse<impl futures_util::Stream<Item = Result<SseEvent, Infallible>>> {
    let rx = state
        .realtime
        .bus()
        .expect("in-process event bus is the default")
        .subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(ev) if ev.subject == BATCH_SUBJECT => {
            let data = String::from_utf8_lossy(&ev.payload).into_owned();
            Some(Ok(SseEvent::default().data(data)))
        }
        _ => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
