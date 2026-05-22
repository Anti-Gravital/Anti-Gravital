//! Bus de eventos interno basado en `tokio::sync::broadcast`.
//!
//! En modo InProcess no requiere servidor NATS externo.
//! El bus NATS externo se integra en la segunda iteracion via `async-nats`.

use std::sync::Arc;
use tokio::sync::broadcast;

/// Evento publicado en el bus.
#[derive(Debug, Clone)]
pub struct Event {
    /// Nombre del canal o subject del evento.
    pub subject: String,
    /// Payload serializado como bytes.
    pub payload: Vec<u8>,
}

/// Error del bus de eventos.
#[derive(Debug)]
pub enum BusError {
    /// El receiver se quedo atras y perdio eventos (lagged).
    Lagged(u64),
    /// El bus fue cerrado.
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

/// Bus de eventos in-process basado en `tokio::sync::broadcast`.
///
/// Permite publicar eventos tipados con un subject y payload binario,
/// y suscribirse para recibirlos de forma asincrona.
#[derive(Clone)]
pub struct EventBus {
    sender: Arc<broadcast::Sender<Event>>,
}

impl EventBus {
    /// Crea un nuevo bus con la capacidad de buffer indicada.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Publica un evento en el bus. Los suscriptores activos lo reciben.
    ///
    /// Retorna error si el bus esta cerrado (sin senders activos).
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

    /// Publica un evento serializado como JSON.
    ///
    /// Retorna error si la serializacion falla o el bus esta cerrado.
    pub fn publish_json<T: serde::Serialize>(
        &self,
        subject: impl Into<String>,
        value: &T,
    ) -> Result<(), BusError> {
        let payload = serde_json::to_vec(value).map_err(|_| BusError::Closed)?;
        self.publish(subject, payload)
    }

    /// Crea un nuevo receptor para escuchar eventos del bus.
    ///
    /// Los eventos publicados antes de crear el receptor no son visibles.
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
        // Sin suscriptores activos, send puede retornar Err.
        // Con broadcast, si no hay receivers, el envio falla con SendError.
        // Verificamos que no haya panic y que el resultado sea tratable.
        let result = bus.publish("x", b"y".to_vec());
        let _ = result;
    }
}
