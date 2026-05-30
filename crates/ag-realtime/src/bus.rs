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
            BusError::Lagged(n) => write!(f, "bus lagged: {n} events lost"),
            BusError::Closed => write!(f, "bus closed"),
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

    /// Publishes an event on the bus (fire-and-forget). Active subscribers
    /// receive it; if there are none, the event is dropped silently.
    ///
    /// Publishing with zero subscribers is a successful no-op, not an error:
    /// a broadcast bus owns its [`broadcast::Sender`] for its whole lifetime,
    /// so it is never actually closed, and `send` only reports the transient
    /// "no current receivers" condition. Returning `Ok(())` here keeps the
    /// InProcess path consistent with the External (NATS) path, which is also
    /// fire-and-forget.
    pub fn publish(&self, subject: impl Into<String>, payload: Vec<u8>) -> Result<(), BusError> {
        let event = Event {
            subject: subject.into(),
            payload,
        };
        // `send` returns `Err` only when there are no active receivers; treat
        // that as a delivered-to-nobody no-op rather than a failure.
        let _ = self.sender.send(event);
        Ok(())
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

    #[test]
    fn bus_error_display_is_english() {
        assert_eq!(BusError::Lagged(7).to_string(), "bus lagged: 7 events lost");
        assert_eq!(BusError::Closed.to_string(), "bus closed");
        // Exercise the std::error::Error impl as well.
        let err: &dyn std::error::Error = &BusError::Closed;
        assert!(err.source().is_none());
    }

    #[tokio::test]
    async fn publish_with_no_subscribers_is_ok() {
        // Regression for the realtime-chat example: POST /messages must not
        // fail (HTTP 500) just because no SSE client is currently connected.
        // Publishing into an empty bus is a fire-and-forget no-op.
        let bus = EventBus::new(1);
        assert!(bus.publish("x", b"y".to_vec()).is_ok());
        assert!(bus.publish_json("x", &7u32).is_ok());
    }

    #[tokio::test]
    async fn late_subscriber_only_sees_events_after_subscribing() {
        let bus = EventBus::new(16);
        // Published before anyone subscribed: dropped, no error.
        bus.publish("early", b"lost".to_vec()).unwrap();
        let mut rx = bus.subscribe();
        bus.publish("late", b"seen".to_vec()).unwrap();
        assert_eq!(rx.recv().await.unwrap().subject, "late");
    }
}
