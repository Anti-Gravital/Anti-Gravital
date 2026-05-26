//! SSE (Server-Sent Events) helper for Axum that turns the bus into an EventSource-compatible stream.

use crate::bus::EventBus;
use axum::{
    extract::State,
    response::{
        sse::{Event as SseEvent, KeepAlive, Sse},
        IntoResponse,
    },
};
use futures_util::stream::Stream;
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio_stream::StreamExt as _;

/// Axum SSE handler connected to the event bus.
///
/// The Axum state must be `Arc<EventBus>`. Compatible with the browser EventSource.
pub async fn sse_handler(State(bus): State<Arc<EventBus>>) -> impl IntoResponse {
    let stream = bus_to_sse_stream(bus);
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    )
}

/// Converts an `EventBus` into a `Stream<Item = Result<SseEvent, Infallible>>`.
pub fn bus_to_sse_stream(
    bus: Arc<EventBus>,
) -> impl Stream<Item = Result<SseEvent, Infallible>> + Send + 'static {
    let rx = bus.subscribe();
    tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => {
            let data = match serde_json::to_string(&serde_json::json!({
                "subject": event.subject,
                "payload": String::from_utf8_lossy(&event.payload),
            })) {
                Ok(s) => s,
                Err(_) => return None,
            };
            Some(Ok(SseEvent::default().event(&event.subject).data(data)))
        }
        Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
            tracing::warn!("SSE client lagged: {n} eventos perdidos");
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_module_compiles() {
        let _bus = Arc::new(EventBus::new(16));
        let _stream = bus_to_sse_stream(Arc::new(EventBus::new(16)));
    }
}
