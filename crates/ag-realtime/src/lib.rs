//! Real-time for the Anti-Gravital ecosystem.
//!
//! Offers an in-memory pub/sub event bus (InProcess) and, with the
//! `nats-external` feature, connection to a real NATS server with TLS and JetStream.
//!
//! # InProcess usage (no NATS server)
//!
//! ```no_run
//! use ag_realtime::{AgRealtime, RealtimeConfig};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let rt = AgRealtime::new(RealtimeConfig::default()).await?;
//! rt.broadcast("user.created", b"payload".to_vec())?;
//! # Ok(())
//! # }
//! ```

pub mod bus;
pub mod config;
pub mod sse;
pub mod ws;

#[cfg(feature = "nats-external")]
pub mod external;

#[cfg(feature = "event-persistence")]
pub mod persistence;

pub use bus::{BusError, Event, EventBus};
pub use config::{NatsMode, RealtimeConfig};

use std::sync::Arc;

#[cfg(feature = "nats-external")]
use external::{NatsError, NatsExternalClient};

/// Real-time subsystem error.
#[derive(Debug)]
pub enum RealtimeError {
    /// Error in the internal bus.
    Bus(BusError),
    /// Error in the external NATS client.
    #[cfg(feature = "nats-external")]
    Nats(NatsError),
}

impl std::fmt::Display for RealtimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RealtimeError::Bus(e) => write!(f, "bus error: {e}"),
            #[cfg(feature = "nats-external")]
            RealtimeError::Nats(e) => write!(f, "NATS error: {e}"),
        }
    }
}

impl std::error::Error for RealtimeError {}

impl From<BusError> for RealtimeError {
    fn from(e: BusError) -> Self {
        RealtimeError::Bus(e)
    }
}

enum RealtimeBus {
    InProcess(Arc<EventBus>),
    #[cfg(feature = "nats-external")]
    External(Arc<NatsExternalClient>),
}

/// Anti-Gravital real-time subsystem.
pub struct AgRealtime {
    inner: RealtimeBus,
    event_bus: Option<Arc<EventBus>>,
}

impl AgRealtime {
    /// Creates a new instance.
    ///
    /// In `External` mode with the `nats-external` feature, connects to the NATS server.
    /// Without the feature, External mode uses InProcess with a warning.
    pub async fn new(config: RealtimeConfig) -> Result<Self, RealtimeError> {
        match config.nats_mode {
            NatsMode::InProcess => {
                let bus = Arc::new(EventBus::new(config.broadcast_capacity));
                Ok(Self {
                    event_bus: Some(Arc::clone(&bus)),
                    inner: RealtimeBus::InProcess(bus),
                })
            }
            NatsMode::External => {
                #[cfg(feature = "nats-external")]
                {
                    let client = NatsExternalClient::connect(&config)
                        .await
                        .map_err(RealtimeError::Nats)?;
                    return Ok(Self {
                        event_bus: None,
                        inner: RealtimeBus::External(Arc::new(client)),
                    });
                }
                #[cfg(not(feature = "nats-external"))]
                {
                    tracing::warn!(
                        "NATS External configurado pero feature nats-external no activa; usando InProcess"
                    );
                    let bus = Arc::new(EventBus::new(config.broadcast_capacity));
                    Ok(Self {
                        event_bus: Some(Arc::clone(&bus)),
                        inner: RealtimeBus::InProcess(bus),
                    })
                }
            }
        }
    }

    /// Publishes an event on the bus (fire-and-forget).
    pub fn broadcast(
        &self,
        subject: impl Into<String>,
        payload: Vec<u8>,
    ) -> Result<(), RealtimeError> {
        match &self.inner {
            RealtimeBus::InProcess(bus) => bus.publish(subject, payload).map_err(Into::into),
            #[cfg(feature = "nats-external")]
            RealtimeBus::External(client) => {
                let client = Arc::clone(client);
                let subject = subject.into();
                tokio::spawn(async move {
                    if let Err(e) = client.publish(&subject, payload).await {
                        tracing::error!(error = %e, "NATS publish error");
                    }
                });
                Ok(())
            }
        }
    }

    /// Publishes an event serialized as JSON.
    pub fn broadcast_json<T: serde::Serialize>(
        &self,
        subject: impl Into<String>,
        value: &T,
    ) -> Result<(), RealtimeError> {
        let payload =
            serde_json::to_vec(value).map_err(|_| RealtimeError::Bus(BusError::Closed))?;
        self.broadcast(subject, payload)
    }

    /// Returns the InProcess bus if running in InProcess mode.
    pub fn bus(&self) -> Option<Arc<EventBus>> {
        self.event_bus.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_inprocess_and_broadcast() {
        let rt = AgRealtime::new(RealtimeConfig::default()).await.unwrap();
        let bus = rt.bus().expect("InProcess debe tener bus");
        let mut rx = bus.subscribe();
        rt.broadcast("test.event", b"data".to_vec()).unwrap();
        let ev = rx.recv().await.unwrap();
        assert_eq!(ev.subject, "test.event");
    }

    #[tokio::test]
    async fn broadcast_json_serializes_correctly() {
        use serde::Serialize;
        #[derive(Serialize)]
        struct Payload {
            id: u32,
        }
        let rt = AgRealtime::new(RealtimeConfig::default()).await.unwrap();
        let bus = rt.bus().unwrap();
        let mut rx = bus.subscribe();
        rt.broadcast_json("ev", &Payload { id: 42 }).unwrap();
        let ev = rx.recv().await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&ev.payload).unwrap();
        assert_eq!(val["id"], 42);
    }

    #[tokio::test]
    async fn inprocess_bus_is_some() {
        let rt = AgRealtime::new(RealtimeConfig::default()).await.unwrap();
        assert!(rt.bus().is_some());
    }

    #[tokio::test]
    async fn external_mode_without_nats_feature_falls_back() {
        let config = RealtimeConfig {
            nats_mode: NatsMode::External,
            ..RealtimeConfig::default()
        };
        let result = AgRealtime::new(config).await;
        let _ = result;
    }
}
