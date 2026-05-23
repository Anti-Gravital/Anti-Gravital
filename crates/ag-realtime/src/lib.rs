//! Tiempo real para el ecosistema Anti-Gravital.
//!
//! Ofrece un bus de eventos pub/sub en memoria (InProcess) y soporte
//! futuro para WebSocket y SSE. La integracion con un servidor NATS
//! externo esta documentada como TECH-DEBT y se aborda en la segunda
//! iteracion de Fase 4.
//!
//! # Uso minimo
//!
//! ```no_run
//! use ag_realtime::{AgRealtime, RealtimeConfig};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let rt = AgRealtime::new(RealtimeConfig::default());
//! rt.broadcast("user.created", b"payload".to_vec())?;
//! # Ok(())
//! # }
//! ```

pub mod bus;
pub mod config;

pub use bus::{BusError, Event, EventBus};
pub use config::{NatsMode, RealtimeConfig};

use std::sync::Arc;

/// Subsistema de tiempo real de Anti-Gravital.
///
/// Gestiona el bus de eventos y expone helpers para publicar
/// y suscribirse a eventos tipados.
pub struct AgRealtime {
    bus: Arc<EventBus>,
}

impl AgRealtime {
    /// Crea una nueva instancia con la configuracion dada.
    ///
    /// En modo `NatsMode::External` se registra un warning y se continua
    /// con el bus in-process hasta que la segunda iteracion integre `async-nats`.
    pub fn new(config: RealtimeConfig) -> Self {
        // TECH-DEBT:
        // motivo: el modo External (NATS real) requiere async-nats con conexion
        //         a un servidor externo. Se implementa en la segunda iteracion
        //         cuando la infra NATS este disponible en CI/CD.
        // impacto: en modo External, se usa igualmente el bus in-process.
        // eliminacion esperada: segunda iteracion ag-realtime en Fase 4.
        if matches!(config.nats_mode, NatsMode::External) {
            tracing::warn!(
                nats_url = %config.nats_url,
                "modo NATS External configurado pero no activo en esta version (TECH-DEBT)"
            );
        }
        let bus = Arc::new(EventBus::new(config.broadcast_capacity));
        Self { bus }
    }

    /// Publica un evento en el bus.
    pub fn broadcast(&self, subject: impl Into<String>, payload: Vec<u8>) -> Result<(), BusError> {
        self.bus.publish(subject, payload)
    }

    /// Publica un evento serializado como JSON.
    pub fn broadcast_json<T: serde::Serialize>(
        &self,
        subject: impl Into<String>,
        value: &T,
    ) -> Result<(), BusError> {
        self.bus.publish_json(subject, value)
    }

    /// Retorna una referencia al bus para suscripcion directa.
    pub fn bus(&self) -> Arc<EventBus> {
        Arc::clone(&self.bus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_and_broadcast() {
        let rt = AgRealtime::new(RealtimeConfig::default());
        let mut rx = rt.bus().subscribe();
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
        let rt = AgRealtime::new(RealtimeConfig::default());
        let mut rx = rt.bus().subscribe();
        rt.broadcast_json("ev", &Payload { id: 42 }).unwrap();
        let ev = rx.recv().await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&ev.payload).unwrap();
        assert_eq!(val["id"], 42);
    }
}
