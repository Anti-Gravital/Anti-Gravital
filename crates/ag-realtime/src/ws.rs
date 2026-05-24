//! Helper WebSocket para Axum que conecta al bus de eventos.

use crate::bus::EventBus;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;

/// Handler Axum para upgrade WebSocket conectado al bus de eventos.
///
/// El estado Axum debe ser `Arc<EventBus>`. Cada cliente WS recibe todos
/// los eventos publicados en el bus mientras la conexion esta abierta.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(bus): State<Arc<EventBus>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, bus))
}

async fn handle_socket(socket: WebSocket, bus: Arc<EventBus>) {
    let mut rx = bus.subscribe();
    let (mut sender, mut receiver) = socket.split();

    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let json = match serde_json::to_string(&serde_json::json!({
                        "subject": event.subject,
                        "payload": String::from_utf8_lossy(&event.payload),
                    })) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("ws client lagged: {n} eventos perdidos");
                }
            }
        }
    });

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    send_task.abort();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_module_compiles() {
        let _bus = Arc::new(EventBus::new(16));
    }
}
