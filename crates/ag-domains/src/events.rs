//! Domain lifecycle events.
//!
//! Control-plane operations emit [`DomainEvent`]s to an [`EventSink`]
//! (RFC-0009 / blueprint section 16.2). The default sink is a no-op so events
//! are opt-in; a native [`InMemoryEventSink`] is provided for tests and
//! single-node use, and [`TracingEventSink`] forwards to `tracing`. No external
//! broker is required (ADR-0009 native-first).
//!
//! Event type strings follow the blueprint vocabulary (`domain.attachment.created`,
//! `domain.ownership.verified`, `domain.detached`). Only events the control
//! plane currently produces are modelled; more are added as their operations
//! land, rather than emitting events that never fire.

use std::sync::Mutex;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A control-plane domain event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum DomainEvent {
    /// An attachment was created (awaiting ownership proof).
    AttachmentCreated {
        /// Attachment id.
        id: String,
        /// Canonical hostname.
        hostname: String,
    },
    /// Ownership was verified for an attachment.
    OwnershipVerified {
        /// Attachment id.
        id: String,
        /// Canonical hostname.
        hostname: String,
    },
    /// An attachment was detached and tombstoned.
    Detached {
        /// Attachment id.
        id: String,
        /// Canonical hostname.
        hostname: String,
    },
}

impl DomainEvent {
    /// The canonical event-type string (blueprint section 16.2 vocabulary).
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::AttachmentCreated { .. } => "domain.attachment.created",
            DomainEvent::OwnershipVerified { .. } => "domain.ownership.verified",
            DomainEvent::Detached { .. } => "domain.detached",
        }
    }

    /// The canonical hostname the event concerns.
    pub fn hostname(&self) -> &str {
        match self {
            DomainEvent::AttachmentCreated { hostname, .. }
            | DomainEvent::OwnershipVerified { hostname, .. }
            | DomainEvent::Detached { hostname, .. } => hostname,
        }
    }
}

/// A sink that receives domain events.
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Receives one event. Implementations must not panic; sinks are
    /// best-effort and never block a control-plane operation's result.
    async fn emit(&self, event: DomainEvent);
}

/// A no-op sink (default): events are discarded.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullEventSink;

#[async_trait]
impl EventSink for NullEventSink {
    async fn emit(&self, _event: DomainEvent) {}
}

/// An in-memory sink that records events in order (native; tests/single-node).
#[derive(Debug, Default)]
pub struct InMemoryEventSink {
    events: Mutex<Vec<DomainEvent>>,
}

impl InMemoryEventSink {
    /// Creates an empty sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a snapshot of recorded events.
    pub fn events(&self) -> Vec<DomainEvent> {
        self.events
            .lock()
            .expect("event sink lock poisoned")
            .clone()
    }

    /// Number of recorded events.
    pub fn len(&self) -> usize {
        self.events.lock().expect("event sink lock poisoned").len()
    }

    /// Whether no events were recorded.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl EventSink for InMemoryEventSink {
    async fn emit(&self, event: DomainEvent) {
        self.events
            .lock()
            .expect("event sink lock poisoned")
            .push(event);
    }
}

/// A sink that forwards events to `tracing` at info level.
#[derive(Debug, Default, Clone, Copy)]
pub struct TracingEventSink;

#[async_trait]
impl EventSink for TracingEventSink {
    async fn emit(&self, event: DomainEvent) {
        tracing::info!(
            event = event.event_type(),
            hostname = event.hostname(),
            "domain event"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_strings() {
        let e = DomainEvent::AttachmentCreated {
            id: "dom_x".to_owned(),
            hostname: "example.com".to_owned(),
        };
        assert_eq!(e.event_type(), "domain.attachment.created");
        assert_eq!(e.hostname(), "example.com");
    }

    #[test]
    fn event_serializes_with_tag() {
        let e = DomainEvent::Detached {
            id: "dom_x".to_owned(),
            hostname: "example.com".to_owned(),
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"event\":\"detached\""));
        assert!(json.contains("\"hostname\":\"example.com\""));
    }

    #[tokio::test]
    async fn in_memory_sink_records_in_order() {
        let sink = InMemoryEventSink::new();
        assert!(sink.is_empty());
        sink.emit(DomainEvent::AttachmentCreated {
            id: "dom_1".to_owned(),
            hostname: "a.example.com".to_owned(),
        })
        .await;
        sink.emit(DomainEvent::Detached {
            id: "dom_1".to_owned(),
            hostname: "a.example.com".to_owned(),
        })
        .await;
        let events = sink.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type(), "domain.attachment.created");
        assert_eq!(events[1].event_type(), "domain.detached");
    }

    #[tokio::test]
    async fn null_sink_discards() {
        let sink = NullEventSink;
        sink.emit(DomainEvent::OwnershipVerified {
            id: "dom_1".to_owned(),
            hostname: "a.example.com".to_owned(),
        })
        .await;
    }
}
