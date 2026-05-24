//! Cliente NATS externo con TLS y JetStream.
//!
//! Solo disponible con la feature `nats-external`.
//!
//! Soporta tres niveles de TLS:
//! - Nivel 1: `nats_tls = true` — TLS con CA del sistema.
//! - Nivel 2: `nats_tls_ca_path` — TLS con CA personalizada.
//! - Nivel 3: `nats_tls_cert_path` + `nats_tls_key_path` — mTLS.

use crate::{bus::Event, config::RealtimeConfig};
use async_nats::{jetstream, Client, ConnectOptions};
use futures_util::StreamExt;
use std::{path::PathBuf, pin::Pin};
use tokio_stream::wrappers::ReceiverStream;

/// Error del cliente NATS externo.
#[derive(Debug, thiserror::Error)]
pub enum NatsError {
    /// Error de conexion al servidor NATS.
    #[error("error de conexion NATS: {0}")]
    Connect(String),
    /// Error al publicar un mensaje.
    #[error("error de publicacion: {0}")]
    Publish(String),
    /// Error al suscribirse a un subject.
    #[error("error de suscripcion: {0}")]
    Subscribe(String),
    /// Error de configuracion TLS.
    #[error("error de TLS: {0}")]
    Tls(String),
    /// Error de JetStream (stream o publicacion con ACK).
    #[error("error de JetStream: {0}")]
    JetStream(String),
}

/// Stream de eventos del bus externo.
pub type EventStream = Pin<Box<dyn futures_util::Stream<Item = Event> + Send + 'static>>;

/// Cliente NATS externo con TLS opcional y soporte JetStream.
pub struct NatsExternalClient {
    client: Client,
    js: Option<jetstream::Context>,
}

impl NatsExternalClient {
    /// Conecta al servidor NATS y, si JetStream esta habilitado, asegura el stream.
    pub async fn connect(config: &RealtimeConfig) -> Result<Self, NatsError> {
        let options = build_connect_options(config)?;
        let client = options
            .connect(&config.nats_url)
            .await
            .map_err(|e| NatsError::Connect(e.to_string()))?;

        tracing::info!(url = %config.nats_url, "NATS external client conectado");

        let js = if config.jetstream_enabled {
            let ctx = jetstream::new(client.clone());
            ensure_stream(&ctx, config).await?;
            Some(ctx)
        } else {
            None
        };

        Ok(Self { client, js })
    }

    /// Publica un evento. Si JetStream esta habilitado, espera ACK del servidor.
    pub async fn publish(&self, subject: &str, payload: Vec<u8>) -> Result<(), NatsError> {
        if let Some(js) = &self.js {
            js.publish(subject.to_string(), payload.into())
                .await
                .map_err(|e| NatsError::Publish(e.to_string()))?
                .await
                .map_err(|e| NatsError::Publish(format!("ACK: {e}")))?;
        } else {
            self.client
                .publish(subject.to_string(), payload.into())
                .await
                .map_err(|e| NatsError::Publish(e.to_string()))?;
        }
        Ok(())
    }

    /// Suscribe a un subject y retorna un stream de eventos.
    ///
    /// Usa un canal interno de capacidad 512 para desacoplar la recepcion
    /// NATS del consumidor del stream, evitando bloqueos de back-pressure.
    pub async fn subscribe(&self, subject: &str) -> Result<EventStream, NatsError> {
        let mut sub = self
            .client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| NatsError::Subscribe(e.to_string()))?;

        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(512);
        tokio::spawn(async move {
            while let Some(msg) = sub.next().await {
                let ev = Event {
                    subject: msg.subject.to_string(),
                    payload: msg.payload.to_vec(),
                };
                if tx.send(ev).await.is_err() {
                    break;
                }
            }
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}

/// Construye las opciones de conexion NATS segun el nivel de TLS configurado.
///
/// - Nivel 1: `nats_tls = true` — activa `require_tls`.
/// - Nivel 2: `nats_tls_ca_path` — anade CA personalizada via `add_root_certificates`.
/// - Nivel 3: `nats_tls_cert_path` + `nats_tls_key_path` — mTLS via `add_client_certificate`.
fn build_connect_options(config: &RealtimeConfig) -> Result<ConnectOptions, NatsError> {
    let mut opts = ConnectOptions::new();

    if config.nats_tls
        || config.nats_tls_ca_path.is_some()
        || config.nats_tls_cert_path.is_some()
    {
        opts = opts.require_tls(true);
    }

    // Nivel 2: CA personalizada
    if let Some(ca_path) = &config.nats_tls_ca_path {
        opts = opts.add_root_certificates(PathBuf::from(ca_path));
    }

    // Nivel 3: mTLS (cert + key del cliente)
    if let (Some(cert_path), Some(key_path)) =
        (&config.nats_tls_cert_path, &config.nats_tls_key_path)
    {
        opts = opts.add_client_certificate(PathBuf::from(cert_path), PathBuf::from(key_path));
    }

    Ok(opts)
}

/// Crea el stream JetStream si no existe, usando la configuracion del crate.
async fn ensure_stream(ctx: &jetstream::Context, config: &RealtimeConfig) -> Result<(), NatsError> {
    let stream_config = jetstream::stream::Config {
        name: config.jetstream_stream_name.clone(),
        subjects: vec![">".to_string()],
        max_messages: config.jetstream_max_msgs,
        max_bytes: config.jetstream_max_bytes,
        retention: jetstream::stream::RetentionPolicy::Limits,
        ..Default::default()
    };

    ctx.get_or_create_stream(stream_config)
        .await
        .map(|_| ())
        .map_err(|e| NatsError::JetStream(format!("ensure_stream: {e}")))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NatsMode;

    /// Retorna true si NATS_URL esta definida en el entorno (servidor real disponible).
    fn nats_available() -> bool {
        std::env::var("NATS_URL").is_ok()
    }

    #[test]
    fn connect_options_no_tls_by_default() {
        let config = RealtimeConfig::default();
        assert!(!config.nats_tls);
        // Verificamos que build_connect_options no falla con config por defecto.
        let result = build_connect_options(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn connect_options_tls_level1() {
        let mut config = RealtimeConfig::default();
        config.nats_tls = true;
        let result = build_connect_options(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn connect_options_tls_level2_ca_path() {
        let mut config = RealtimeConfig::default();
        config.nats_tls_ca_path = Some("/tmp/ca.pem".to_string());
        let result = build_connect_options(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn connect_options_tls_level3_mtls() {
        let mut config = RealtimeConfig::default();
        config.nats_tls_cert_path = Some("/tmp/client.crt".to_string());
        config.nats_tls_key_path = Some("/tmp/client.key".to_string());
        let result = build_connect_options(&config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn connect_fails_without_server() {
        if nats_available() {
            return; // skip si hay servidor real disponible
        }
        let mut config = RealtimeConfig::default();
        config.nats_mode = NatsMode::External;
        config.nats_url = "nats://127.0.0.1:14222".to_string(); // puerto inexistente
        let result = NatsExternalClient::connect(&config).await;
        assert!(result.is_err(), "debe fallar sin servidor NATS");
    }

    #[tokio::test]
    async fn publish_and_subscribe_with_real_nats() {
        if !nats_available() {
            return; // skip en CI sin NATS
        }
        let mut config = RealtimeConfig::default();
        config.nats_mode = NatsMode::External;
        config.nats_url = std::env::var("NATS_URL").unwrap();

        let client = NatsExternalClient::connect(&config)
            .await
            .expect("debe conectar al NATS real");

        let mut stream = client
            .subscribe("test.ag.realtime")
            .await
            .expect("subscribe debe tener exito");

        client
            .publish("test.ag.realtime", b"hola".to_vec())
            .await
            .expect("publish debe tener exito");

        let event = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            futures_util::StreamExt::next(&mut stream),
        )
        .await
        .expect("timeout")
        .expect("debe recibir evento");

        assert_eq!(event.payload, b"hola");
    }

    #[tokio::test]
    async fn jetstream_stream_ensured_with_real_nats() {
        if !nats_available() {
            return;
        }
        let mut config = RealtimeConfig::default();
        config.nats_mode = NatsMode::External;
        config.nats_url = std::env::var("NATS_URL").unwrap();
        config.jetstream_enabled = true;
        config.jetstream_stream_name = "AG_EVENTS_TEST".to_string();

        let client = NatsExternalClient::connect(&config)
            .await
            .expect("debe conectar con JetStream");

        client
            .publish("ag.test.event", b"payload".to_vec())
            .await
            .expect("publish JetStream debe tener exito con ACK");
    }
}
