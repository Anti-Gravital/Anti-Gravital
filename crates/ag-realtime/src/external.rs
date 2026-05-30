//! External NATS client with TLS and JetStream.
//!
//! Only available with the `nats-external` feature.
//!
//! Supports three TLS levels:
//! - Level 1: `nats_tls = true` — TLS with the system CA.
//! - Level 2: `nats_tls_ca_path` — TLS with a custom CA.
//! - Level 3: `nats_tls_cert_path` + `nats_tls_key_path` — mTLS.

use crate::{bus::Event, config::RealtimeConfig};
use async_nats::{jetstream, Client, ConnectOptions};
use futures_util::StreamExt;
use std::{path::PathBuf, pin::Pin};
use tokio_stream::wrappers::ReceiverStream;

/// External NATS client error.
#[derive(Debug, thiserror::Error)]
pub enum NatsError {
    /// Connection error to the NATS server.
    #[error("error de conexion NATS: {0}")]
    Connect(String),
    /// Error publishing a message.
    #[error("error de publicacion: {0}")]
    Publish(String),
    /// Error subscribing to a subject.
    #[error("error de suscripcion: {0}")]
    Subscribe(String),
    /// TLS configuration error.
    #[error("error de TLS: {0}")]
    Tls(String),
    /// JetStream error (stream or publish with ACK).
    #[error("error de JetStream: {0}")]
    JetStream(String),
}

/// Event stream from the external bus.
pub type EventStream = Pin<Box<dyn futures_util::Stream<Item = Event> + Send + 'static>>;

/// External NATS client with optional TLS and JetStream support.
pub struct NatsExternalClient {
    client: Client,
    js: Option<jetstream::Context>,
}

impl NatsExternalClient {
    /// Connects to the NATS server and, if JetStream is enabled, ensures the stream.
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

    /// Publishes an event. If JetStream is enabled, waits for the server ACK.
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

    /// Subscribes to a subject and returns a stream of events.
    ///
    /// Uses an internal channel with capacity 512 to decouple the NATS
    /// reception from the stream consumer, avoiding back-pressure blocking.
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

/// Builds the NATS connection options according to the configured TLS level.
///
/// - Level 1: `nats_tls = true` — enables `require_tls`.
/// - Level 2: `nats_tls_ca_path` — adds a custom CA via `add_root_certificates`.
/// - Level 3: `nats_tls_cert_path` + `nats_tls_key_path` — mTLS via `add_client_certificate`.
fn build_connect_options(config: &RealtimeConfig) -> Result<ConnectOptions, NatsError> {
    let mut opts = ConnectOptions::new();

    if config.nats_tls || config.nats_tls_ca_path.is_some() || config.nats_tls_cert_path.is_some() {
        opts = opts.require_tls(true);
    }

    // Level 2: custom CA
    if let Some(ca_path) = &config.nats_tls_ca_path {
        opts = opts.add_root_certificates(PathBuf::from(ca_path));
    }

    // Level 3: mTLS (client cert + key)
    if let (Some(cert_path), Some(key_path)) =
        (&config.nats_tls_cert_path, &config.nats_tls_key_path)
    {
        opts = opts.add_client_certificate(PathBuf::from(cert_path), PathBuf::from(key_path));
    }

    Ok(opts)
}

/// Creates the JetStream stream if it does not exist, using the crate config.
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

    /// Returns true if NATS_URL is defined in the environment (real server available).
    fn nats_available() -> bool {
        std::env::var("NATS_URL").is_ok()
    }

    #[test]
    fn connect_options_no_tls_by_default() {
        let config = RealtimeConfig::default();
        assert!(!config.nats_tls);
        // We verify that build_connect_options does not fail with the default config.
        let result = build_connect_options(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn connect_options_tls_level1() {
        let config = RealtimeConfig {
            nats_tls: true,
            ..Default::default()
        };
        let result = build_connect_options(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn connect_options_tls_level2_ca_path() {
        let config = RealtimeConfig {
            nats_tls_ca_path: Some("/tmp/ca.pem".to_string()),
            ..Default::default()
        };
        let result = build_connect_options(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn connect_options_tls_level3_mtls() {
        let config = RealtimeConfig {
            nats_tls_cert_path: Some("/tmp/client.crt".to_string()),
            nats_tls_key_path: Some("/tmp/client.key".to_string()),
            ..Default::default()
        };
        let result = build_connect_options(&config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn connect_fails_without_server() {
        if nats_available() {
            return; // skip if a real server is available
        }
        let config = RealtimeConfig {
            nats_mode: NatsMode::External,
            nats_url: "nats://127.0.0.1:14222".to_string(), // nonexistent port
            ..Default::default()
        };
        let result = NatsExternalClient::connect(&config).await;
        assert!(result.is_err(), "debe fallar sin servidor NATS");
    }

    #[tokio::test]
    async fn publish_and_subscribe_with_real_nats() {
        if !nats_available() {
            return; // skip in CI without NATS
        }
        let config = RealtimeConfig {
            nats_mode: NatsMode::External,
            nats_url: std::env::var("NATS_URL").unwrap(),
            ..Default::default()
        };

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
        let config = RealtimeConfig {
            nats_mode: NatsMode::External,
            nats_url: std::env::var("NATS_URL").unwrap(),
            jetstream_enabled: true,
            jetstream_stream_name: "AG_EVENTS_TEST".to_string(),
            ..Default::default()
        };

        let client = NatsExternalClient::connect(&config)
            .await
            .expect("debe conectar con JetStream");

        client
            .publish("ag.test.event", b"payload".to_vec())
            .await
            .expect("publish JetStream debe tener exito con ACK");
    }
}
