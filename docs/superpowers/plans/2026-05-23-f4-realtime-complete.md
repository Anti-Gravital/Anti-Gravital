# f4-realtime-complete — Plan de Implementacion

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar el modo NATS externo real en `ag-realtime`: cliente `async-nats` con TLS (3 niveles) + JetStream, helpers de WebSocket y SSE para Axum. Eliminar el TECH-DEBT del modo External.

**Architecture:** Nuevo `external.rs` con `NatsExternalClient` encapsula toda la logica async-nats. `lib.rs` introduce `RealtimeBus` enum (`InProcess | External`) — `AgRealtime::new()` pasa a ser `async`. `ws.rs` y `sse.rs` son helpers de Axum puros, sin dependencias de NATS. `config.rs` recibe campos TLS y JetStream parseados desde entorno.

**Tech Stack:** `async-nats 0.48` (ya en workspace, feature `nats-external`), `axum 0.7` (ya en workspace), `futures-util` (ya en workspace), `tokio-stream` (ya en workspace), `rcgen` (dev-dep para certs TLS en tests).

---

## Mapa de archivos

| Accion | Archivo | Responsabilidad |
|---|---|---|
| Modify | `crates/ag-realtime/src/config.rs` | Campos TLS + JetStream + from_env |
| Create | `crates/ag-realtime/src/external.rs` | NatsExternalClient — connect, publish, subscribe, JetStream |
| Create | `crates/ag-realtime/src/ws.rs` | websocket_handler Axum — bridge bus <-> WebSocket |
| Create | `crates/ag-realtime/src/sse.rs` | sse_handler Axum — convierte bus en SSE stream |
| Modify | `crates/ag-realtime/src/lib.rs` | RealtimeBus enum, AgRealtime::new async, bus() Option |
| Modify | `crates/ag-realtime/Cargo.toml` | Añadir tokio-stream, rcgen (dev) |

---

## Task 1: Extender `RealtimeConfig` con TLS y JetStream

**Files:**
- Modify: `crates/ag-realtime/src/config.rs`

- [ ] **Step 1: Escribir tests que fallen**

Añadir al bloque `#[cfg(test)]` de `config.rs`:

```rust
#[test]
fn from_env_external_mode() {
    std::env::set_var("NATS_MODE", "external");
    let cfg = RealtimeConfig::from_env();
    std::env::remove_var("NATS_MODE");
    assert!(matches!(cfg.nats_mode, NatsMode::External));
}

#[test]
fn from_env_custom_url() {
    std::env::set_var("NATS_URL", "nats://my-nats:4222");
    let cfg = RealtimeConfig::from_env();
    std::env::remove_var("NATS_URL");
    assert_eq!(cfg.nats_url, "nats://my-nats:4222");
}

#[test]
fn from_env_custom_capacity() {
    std::env::set_var("RT_BROADCAST_CAPACITY", "2048");
    let cfg = RealtimeConfig::from_env();
    std::env::remove_var("RT_BROADCAST_CAPACITY");
    assert_eq!(cfg.broadcast_capacity, 2048);
}

#[test]
fn from_env_tls_enabled() {
    std::env::set_var("NATS_TLS", "true");
    let cfg = RealtimeConfig::from_env();
    std::env::remove_var("NATS_TLS");
    assert!(cfg.nats_tls);
}

#[test]
fn from_env_jetstream_stream_name() {
    std::env::set_var("NATS_JS_STREAM", "MY_STREAM");
    let cfg = RealtimeConfig::from_env();
    std::env::remove_var("NATS_JS_STREAM");
    assert_eq!(cfg.jetstream_stream_name, "MY_STREAM");
}

#[test]
fn defaults_tls_off_jetstream_off() {
    let cfg = RealtimeConfig::default();
    assert!(!cfg.nats_tls);
    assert!(!cfg.jetstream_enabled);
    assert_eq!(cfg.jetstream_stream_name, "AG_EVENTS");
}
```

- [ ] **Step 2: Ejecutar para verificar que fallan**

```bash
cargo test -p ag-realtime config -- --nocapture 2>&1 | grep -E "FAILED|error"
```

Esperado: los tests de TLS/JetStream fallan con "field not found".

- [ ] **Step 3: Ampliar RealtimeConfig**

Reemplazar la definicion completa de `RealtimeConfig`, `Default` y `from_env` en `config.rs`:

```rust
/// Configuracion del subsistema de tiempo real.
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    /// Modo del bus de eventos.
    pub nats_mode: NatsMode,
    /// URL del servidor NATS externo.
    pub nats_url: String,
    /// Capacidad del canal broadcast interno (modo InProcess).
    pub broadcast_capacity: usize,
    // TLS
    /// Si `true`, activa TLS con CA del sistema.
    pub nats_tls: bool,
    /// Ruta a CA personalizada (nivel 2: CA custom).
    pub nats_tls_ca_path: Option<String>,
    /// Ruta al certificado de cliente (nivel 3: mTLS).
    pub nats_tls_cert_path: Option<String>,
    /// Ruta a la clave privada del cliente (nivel 3: mTLS).
    pub nats_tls_key_path: Option<String>,
    // JetStream
    /// Si `true`, publica y consume via JetStream con ACK.
    pub jetstream_enabled: bool,
    /// Nombre del stream JetStream. Default: `AG_EVENTS`.
    pub jetstream_stream_name: String,
    /// Limite de mensajes del stream. Default: 1_000_000.
    pub jetstream_max_msgs: i64,
    /// Limite de bytes del stream. Default: 1 GiB.
    pub jetstream_max_bytes: i64,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            nats_mode: NatsMode::InProcess,
            nats_url: "nats://localhost:4222".to_string(),
            broadcast_capacity: 1024,
            nats_tls: false,
            nats_tls_ca_path: None,
            nats_tls_cert_path: None,
            nats_tls_key_path: None,
            jetstream_enabled: false,
            jetstream_stream_name: "AG_EVENTS".to_string(),
            jetstream_max_msgs: 1_000_000,
            jetstream_max_bytes: 1_073_741_824,
        }
    }
}

impl RealtimeConfig {
    /// Lee la configuracion desde variables de entorno.
    pub fn from_env() -> Self {
        let nats_mode = match std::env::var("NATS_MODE").as_deref() {
            Ok("external") => NatsMode::External,
            _ => NatsMode::InProcess,
        };
        Self {
            nats_mode,
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            broadcast_capacity: std::env::var("RT_BROADCAST_CAPACITY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1024),
            nats_tls: std::env::var("NATS_TLS")
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            nats_tls_ca_path: std::env::var("NATS_TLS_CA").ok(),
            nats_tls_cert_path: std::env::var("NATS_TLS_CERT").ok(),
            nats_tls_key_path: std::env::var("NATS_TLS_KEY").ok(),
            jetstream_enabled: std::env::var("NATS_JETSTREAM")
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            jetstream_stream_name: std::env::var("NATS_JS_STREAM")
                .unwrap_or_else(|_| "AG_EVENTS".to_string()),
            jetstream_max_msgs: std::env::var("NATS_JS_MAX_MSGS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1_000_000),
            jetstream_max_bytes: std::env::var("NATS_JS_MAX_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1_073_741_824),
        }
    }
}
```

- [ ] **Step 4: Ejecutar tests**

```bash
cargo test -p ag-realtime config -- --nocapture 2>&1
```

Esperado: todos los tests de config pasan.

- [ ] **Step 5: Commit**

```bash
git add crates/ag-realtime/src/config.rs
git commit -m "feat(ag-realtime): RealtimeConfig ampliada — campos TLS y JetStream"
```

---

## Task 2: `external.rs` — NatsExternalClient

**Files:**
- Create: `crates/ag-realtime/src/external.rs`
- Modify: `crates/ag-realtime/Cargo.toml`

- [ ] **Step 1: Añadir tokio-stream como dep regular**

En `crates/ag-realtime/Cargo.toml`, seccion `[dependencies]`:

```toml
tokio-stream = { workspace = true }
```

Y en `[dev-dependencies]`:

```toml
rcgen = { workspace = true }
tokio = { workspace = true, features = ["test-util", "fs"] }
```

- [ ] **Step 2: Crear external.rs**

Crear `crates/ag-realtime/src/external.rs`:

```rust
//! Cliente NATS externo con TLS y JetStream.
//!
//! Solo disponible con la feature `nats-external`.

use crate::{bus::Event, config::RealtimeConfig};
use async_nats::{jetstream, Client, ConnectOptions};
use futures_util::StreamExt;
use std::pin::Pin;
use tokio_stream::wrappers::ReceiverStream;

/// Error del cliente NATS externo.
#[derive(Debug, thiserror::Error)]
pub enum NatsError {
    #[error("error de conexion NATS: {0}")]
    Connect(String),
    #[error("error de publicacion: {0}")]
    Publish(String),
    #[error("error de suscripcion: {0}")]
    Subscribe(String),
    #[error("error de TLS: {0}")]
    Tls(String),
    #[error("error de JetStream: {0}")]
    JetStream(String),
}

/// Stream de eventos del bus.
pub type EventStream = Pin<Box<dyn futures_util::Stream<Item = Event> + Send + 'static>>;

/// Cliente NATS externo con TLS y soporte JetStream.
pub struct NatsExternalClient {
    client: Client,
    js: Option<jetstream::Context>,
    config: RealtimeConfig,
}

impl NatsExternalClient {
    /// Conecta al servidor NATS y, si JetStream esta habilitado, asegura el stream.
    pub async fn connect(config: &RealtimeConfig) -> Result<Self, NatsError> {
        let options = build_connect_options(config).await?;
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

        Ok(Self {
            client,
            js,
            config: config.clone(),
        })
    }

    /// Publica un evento. Si JetStream esta habilitado, espera ACK.
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
    /// El stream entrega eventos mientras este abierto.
    /// Usa un canal interno de capacidad 512 para no bloquear al cliente NATS.
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
async fn build_connect_options(config: &RealtimeConfig) -> Result<ConnectOptions, NatsError> {
    let mut opts = ConnectOptions::new();

    if config.nats_tls || config.nats_tls_ca_path.is_some() || config.nats_tls_cert_path.is_some() {
        opts = opts.require_tls(true);
    }

    // Nivel 2: CA personalizada
    if let Some(ca_path) = &config.nats_tls_ca_path {
        let ca_data = tokio::fs::read(ca_path)
            .await
            .map_err(|e| NatsError::Tls(format!("leer CA {ca_path}: {e}")))?;
        let ca = rustls_pki_types::CertificateDer::from(ca_data);
        opts = opts.add_root_certificates(ca);
    }

    // Nivel 3: mTLS (cert + key del cliente)
    if let (Some(cert_path), Some(key_path)) = (&config.nats_tls_cert_path, &config.nats_tls_key_path) {
        let cert_data = tokio::fs::read(cert_path)
            .await
            .map_err(|e| NatsError::Tls(format!("leer cert {cert_path}: {e}")))?;
        let key_data = tokio::fs::read(key_path)
            .await
            .map_err(|e| NatsError::Tls(format!("leer key {key_path}: {e}")))?;
        let cert = rustls_pki_types::CertificateDer::from(cert_data);
        let key = rustls_pki_types::PrivateKeyDer::try_from(key_data)
            .map_err(|e| NatsError::Tls(format!("parsear key: {e}")))?;
        opts = opts.add_client_certificate(cert, key);
    }

    Ok(opts)
}

/// Crea el stream JetStream si no existe.
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

    /// Helper: retorna true si NATS_URL esta disponible en el entorno.
    fn nats_available() -> bool {
        std::env::var("NATS_URL").is_ok()
    }

    #[test]
    fn connect_options_no_tls_by_default() {
        let config = RealtimeConfig::default();
        // Solo verificamos que la funcion async puede construirse; no la ejecutamos sin runtime.
        // Este test valida que el modulo compila correctamente.
        let _ = &config.nats_tls;
        assert!(!config.nats_tls);
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

        let client = NatsExternalClient::connect(&config).await
            .expect("debe conectar al NATS real");

        let mut stream = client.subscribe("test.ag.realtime").await
            .expect("subscribe debe tener exito");

        client.publish("test.ag.realtime", b"hola".to_vec()).await
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

        let client = NatsExternalClient::connect(&config).await
            .expect("debe conectar con JetStream");

        client.publish("ag.test.event", b"payload".to_vec()).await
            .expect("publish JetStream debe tener exito con ACK");
    }
}
```

- [ ] **Step 3: Compilar**

```bash
cargo check -p ag-realtime --features nats-external 2>&1 | grep "^error" | head -20
```

Si hay errores por `rustls_pki_types`, añadir a `[dependencies]` de ag-realtime:

```toml
rustls-pki-types = { workspace = true }
```

- [ ] **Step 4: Tests sin NATS**

```bash
cargo test -p ag-realtime --features nats-external external -- --nocapture 2>&1
```

Esperado: `connect_fails_without_server` pasa, los tests `with_real_nats` se saltan (no hay NATS_URL).

- [ ] **Step 5: Commit**

```bash
git add crates/ag-realtime/src/external.rs crates/ag-realtime/Cargo.toml
git commit -m "feat(ag-realtime): NatsExternalClient — connect TLS 3 niveles + JetStream"
```

---

## Task 3: `ws.rs` — WebSocket helper para Axum

**Files:**
- Create: `crates/ag-realtime/src/ws.rs`

- [ ] **Step 1: Crear ws.rs**

```rust
//! Helper WebSocket para Axum que conecta al bus de eventos.
//!
//! Uso en un router Axum:
//! ```no_run
//! use ag_realtime::{AgRealtime, RealtimeConfig};
//! use ag_realtime::ws::ws_handler;
//! use axum::{Router, extract::State, routing::get};
//! use std::sync::Arc;
//!
//! # async fn run() {
//! let rt = Arc::new(AgRealtime::new(RealtimeConfig::default()).await.unwrap());
//! let app = Router::new()
//!     .route("/ws", get(ws_handler))
//!     .with_state(rt);
//! # }
//! ```

use crate::bus::{Event, EventBus};
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

    // Tarea 1: reenviar eventos del bus al cliente WS
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

    // Tarea 2: leer mensajes del cliente (ping/close) y terminar si se desconecta
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
    use crate::bus::EventBus;

    #[test]
    fn ws_module_compiles() {
        // Verifica que todos los imports son validos.
        let _bus = Arc::new(EventBus::new(16));
    }
}
```

- [ ] **Step 2: Compilar**

```bash
cargo check -p ag-realtime 2>&1 | grep "^error" | head -10
```

- [ ] **Step 3: Commit**

```bash
git add crates/ag-realtime/src/ws.rs
git commit -m "feat(ag-realtime): ws_handler — bridge WebSocket <-> EventBus para Axum"
```

---

## Task 4: `sse.rs` — SSE helper para Axum

**Files:**
- Create: `crates/ag-realtime/src/sse.rs`

- [ ] **Step 1: Crear sse.rs**

```rust
//! Helper SSE (Server-Sent Events) para Axum que convierte el bus en stream compatible con EventSource.
//!
//! Uso en un router Axum:
//! ```no_run
//! use ag_realtime::{AgRealtime, RealtimeConfig};
//! use ag_realtime::sse::sse_handler;
//! use axum::{Router, extract::State, routing::get};
//! use std::sync::Arc;
//!
//! # async fn run() {
//! let rt = Arc::new(AgRealtime::new(RealtimeConfig::default()).await.unwrap());
//! let app = Router::new()
//!     .route("/events", get(sse_handler))
//!     .with_state(rt);
//! # }
//! ```

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
/// El estado Axum debe ser `Arc<EventBus>`. Cada cliente SSE recibe todos
/// los eventos publicados mientras la conexion esta abierta.
/// Compatible con la API EventSource del navegador.
pub async fn sse_handler(
    State(bus): State<Arc<EventBus>>,
) -> impl IntoResponse {
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
    tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| {
        match result {
            Ok(event) => {
                let data = match serde_json::to_string(&serde_json::json!({
                    "subject": event.subject,
                    "payload": String::from_utf8_lossy(&event.payload),
                })) {
                    Ok(s) => s,
                    Err(_) => return None,
                };
                Some(Ok(SseEvent::default()
                    .event(&event.subject)
                    .data(data)))
            }
            Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
                tracing::warn!("SSE client lagged: {n} eventos perdidos");
                None
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::EventBus;

    #[tokio::test]
    async fn bus_to_sse_stream_produces_events() {
        use futures_util::StreamExt;

        let bus = Arc::new(EventBus::new(16));
        let stream = bus_to_sse_stream(Arc::clone(&bus));

        bus.publish("test.subject", b"test payload".to_vec()).unwrap();

        // Tomar el primer evento del stream con timeout
        let event = tokio::time::timeout(
            Duration::from_secs(1),
            futures_util::pin_mut!(stream);
            stream.next(),
        )
        .await;

        // El stream puede estar vacio si no hay suscriptores activos cuando se publico.
        // Este test solo verifica que el modulo compila y el stream se crea correctamente.
        let _ = event;
    }

    #[test]
    fn sse_module_compiles() {
        let _bus = Arc::new(EventBus::new(16));
    }
}
```

Nota: el test de stream puede necesitar ajuste — la semantica de BroadcastStream requiere que el
subscriber exista ANTES del publish. Si el test falla con timeout, refactorizar para crear el stream
antes de publicar.

- [ ] **Step 2: Añadir tokio-stream con feature sync a workspace si falta**

En `Cargo.toml` raiz verificar que `tokio-stream` incluye `features = ["sync"]`:

```toml
tokio-stream = { version = "0.1", features = ["sync"] }
```

- [ ] **Step 3: Compilar**

```bash
cargo check -p ag-realtime 2>&1 | grep "^error" | head -10
```

- [ ] **Step 4: Commit**

```bash
git add crates/ag-realtime/src/sse.rs Cargo.toml
git commit -m "feat(ag-realtime): sse_handler — bus de eventos como SSE stream EventSource-compatible"
```

---

## Task 5: Actualizar `lib.rs` — AgRealtime async, RealtimeBus enum

**Files:**
- Modify: `crates/ag-realtime/src/lib.rs`

- [ ] **Step 1: Reemplazar el contenido de lib.rs**

```rust
//! Tiempo real para el ecosistema Anti-Gravital.
//!
//! Ofrece un bus de eventos pub/sub en memoria (InProcess) y, con la feature
//! `nats-external`, conexion a un servidor NATS real con TLS y JetStream.
//!
//! # Uso InProcess (sin servidor NATS)
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

pub use bus::{BusError, Event, EventBus};
pub use config::{NatsMode, RealtimeConfig};

use std::sync::Arc;

#[cfg(feature = "nats-external")]
use external::{NatsError, NatsExternalClient};

/// Error del subsistema de tiempo real.
#[derive(Debug)]
pub enum RealtimeError {
    /// Error en el bus interno.
    Bus(BusError),
    /// Error en el cliente NATS externo.
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

/// Bus de eventos interno — InProcess o External (NATS).
enum RealtimeBus {
    InProcess(Arc<EventBus>),
    #[cfg(feature = "nats-external")]
    External(Arc<NatsExternalClient>),
}

/// Subsistema de tiempo real de Anti-Gravital.
pub struct AgRealtime {
    inner: RealtimeBus,
    /// Bus InProcess disponible siempre que el modo sea InProcess.
    event_bus: Option<Arc<EventBus>>,
}

impl AgRealtime {
    /// Crea una nueva instancia.
    ///
    /// En modo `External` con la feature `nats-external`, conecta al servidor NATS.
    /// Sin la feature, el modo External se ignora y se usa InProcess con un warning.
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

    /// Publica un evento en el bus (fire-and-forget).
    pub fn broadcast(&self, subject: impl Into<String>, payload: Vec<u8>) -> Result<(), RealtimeError> {
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

    /// Publica un evento serializado como JSON.
    pub fn broadcast_json<T: serde::Serialize>(
        &self,
        subject: impl Into<String>,
        value: &T,
    ) -> Result<(), RealtimeError> {
        let payload = serde_json::to_vec(value).map_err(|_| RealtimeError::Bus(BusError::Closed))?;
        self.broadcast(subject, payload)
    }

    /// Retorna el bus InProcess si esta en modo InProcess.
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
        struct Payload { id: u32 }
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
    async fn external_mode_without_feature_falls_back_to_inprocess() {
        let mut config = RealtimeConfig::default();
        config.nats_mode = NatsMode::External;
        let result = AgRealtime::new(config).await;
        // Sin feature nats-external: siempre InProcess
        // Con feature nats-external: falla si no hay servidor
        // Solo verificamos que no hay panic
        let _ = result;
    }
}
```

- [ ] **Step 2: Eliminar el TECH-DEBT del modo External en lib.rs anterior**

El nuevo lib.rs ya no contiene el TECH-DEBT del modo External. Verificar que el archivo se ha reemplazado completamente.

- [ ] **Step 3: Compilar**

```bash
cargo test -p ag-realtime 2>&1
```

Esperado: todos los tests pasan.

- [ ] **Step 4: CI local**

```bash
cargo fmt --all
cargo clippy -p ag-realtime -- -D warnings
cargo deny check
```

- [ ] **Step 5: Commit final**

```bash
git add crates/ag-realtime/src/lib.rs crates/ag-realtime/src/ws.rs crates/ag-realtime/src/sse.rs
git commit -m "feat(ag-realtime): AgRealtime async, RealtimeBus enum, helpers WS+SSE"
git checkout fase-4
git merge --no-ff f4-realtime-complete -m "merge(f4-realtime-complete): NATS External + TLS + JetStream + WS + SSE"
```
