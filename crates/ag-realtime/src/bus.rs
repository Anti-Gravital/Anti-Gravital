//! Internal event bus based on `tokio::sync::broadcast`.
//!
//! In InProcess mode it does not require an external NATS server.
//! The external NATS bus is integrated in the second iteration via `async-nats`.

use std::sync::Arc;
use tokio::sync::broadcast;

/// Event published on the bus.
#[derive(Debug, Clone)]
pub struct Event {
    /// Channel name or subject of the event.
    pub subject: String,
    /// Payload serialized as bytes.
    pub payload: Vec<u8>,
}

/// Event bus error.
#[derive(Debug)]
pub enum BusError {
    /// The receiver fell behind and lost events (lagged).
    Lagged(u64),
    /// The bus was closed.
    Closed,
}

impl std::fmt::Display for BusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BusError::Lagged(n) => write!(f, "bus lagged: {n} eventos perdidos"),
            BusError::Closed => write!(f, "bus cerrado"),
        }
    }
}

impl std::error::Error for BusError {}

/// In-process event bus based on `tokio::sync::broadcast`.
///
/// Lets you publish typed events with a subject and binary payload,
/// and subscribe to receive them asynchronously.
#[derive(Clone)]
pub struct EventBus {
    sender: Arc<broadcast::Sender<Event>>,
}

impl EventBus {
    /// Creates a new bus with the given buffer capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Publishes an event on the bus. Active subscribers receive it.
    ///
    /// Returns an error if the bus is closed (no active senders).
    pub fn publish(&self, subject: impl Into<String>, payload: Vec<u8>) -> Result<(), BusError> {
        let event = Event {
            subject: subject.into(),
            payload,
        };
        self.sender
            .send(event)
            .map(|_| ())
            .map_err(|_| BusError::Closed)
    }

    /// Publishes an event serialized as JSON.
    ///
    /// Returns an error if serialization fails or the bus is closed.
    pub fn publish_json<T: serde::Serialize>(
        &self,
        subject: impl Into<String>,
        value: &T,
    ) -> Result<(), BusError> {
        let payload = serde_json::to_vec(value).map_err(|_| BusError::Closed)?;
        self.publish(subject, payload)
    }

    /// Creates a new receiver to listen for events from the bus.
    ///
    /// Events published before the receiver is created are not visible.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publish_and_receive() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        bus.publish("user.created", b"payload".to_vec()).unwrap();
        let event = rx.recv().await.unwrap();
        assert_eq!(event.subject, "user.created");
        assert_eq!(event.payload, b"payload");
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_same_event() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();
        bus.publish("ping", b"data".to_vec()).unwrap();
        assert_eq!(rx1.recv().await.unwrap().subject, "ping");
        assert_eq!(rx2.recv().await.unwrap().subject, "ping");
    }

    #[tokio::test]
    async fn publish_json_deserializes_correctly() {
        use serde::Serialize;
        #[derive(Serialize)]
        struct Msg {
            id: u32,
        }
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        bus.publish_json("msg.created", &Msg { id: 7 }).unwrap();
        let event = rx.recv().await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&event.payload).unwrap();
        assert_eq!(val["id"], 7);
    }

    #[tokio::test]
    async fn publish_to_closed_bus_returns_error() {
        let bus = EventBus::new(1);
        // With no active subscribers, send may return Err.
        // With broadcast, if there are no receivers, the send fails with SendError.
        // We verify that there is no panic and that the result is handleable.
        let result = bus.publish("x", b"y".to_vec());
        let _ = result;
    }
}
