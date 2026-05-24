//! Helper SSE (Server-Sent Events) para Axum que convierte el bus en stream compatible con EventSource.

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

/// Handler Axum SSE conectado al bus de eventos.
///
/// El estado Axum debe ser `Arc<EventBus>`. Compatible con EventSource del navegador.
pub async fn sse_handler(State(bus): State<Arc<EventBus>>) -> impl IntoResponse {
    let stream = bus_to_sse_stream(bus);
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    )
}

/// Convierte un `EventBus` en un `Stream<Item = Result<SseEvent, Infallible>>`.
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
