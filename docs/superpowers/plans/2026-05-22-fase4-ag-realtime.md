# ag-realtime — Plan de Implementacion

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Comunicacion en tiempo real: WebSocket binario (msgpack), SSE fallback automatico, y bus pub/sub via NATS embebido o externo. Objetivo: 50K conexiones WebSocket en 2 vCPU.

**Architecture:** `AgRealtime` orquesta los tres subsistemas. El bus interno usa NATS. WebSocket y SSE son handlers Axum que se conectan al bus. Rama: `fase-4/ag-realtime`.

**Precondicion:** `fase-4/ag-observe` mergeado a `fase-4`.

**Tech Stack:** `async-nats` (NATS client + embedded), `axum` (WebSocket + SSE ya en workspace), `rmp-serde` (msgpack), `testcontainers` (NATS para tests distribuidos).

**Spec:** `docs/superpowers/specs/2026-05-22-fase4-modulos-estandar-design.md` seccion ag-realtime.
**Arquitectura:** `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 8.3.

---

## Mapa de archivos

### Crear
- `crates/ag-realtime/Cargo.toml`
- `crates/ag-realtime/src/lib.rs` — `AgRealtime`, `RealtimeConfig`, re-exports
- `crates/ag-realtime/src/config.rs` — `RealtimeConfig::from_env()`, `NatsMode`
- `crates/ag-realtime/src/bus.rs` — abstraccion sobre NATS: publish/subscribe
- `crates/ag-realtime/src/ws.rs` — handler WebSocket Axum con msgpack
- `crates/ag-realtime/src/sse.rs` — handler SSE Axum

### Modificar
- `Cargo.toml` (root) — dependencias nuevas + miembro `crates/ag-realtime`

---

## Task 1: Cargo.toml y dependencias

- [ ] Agregar a `[workspace.members]`: `"crates/ag-realtime"`
- [ ] Agregar a `[workspace.dependencies]`:
  ```toml
  async-nats = "0.38"
  rmp-serde  = "1"
  ```
- [ ] Crear `crates/ag-realtime/Cargo.toml` con deps: `async-nats`, `rmp-serde`, `axum` (con feature `ws`), `tokio`, `serde`, `tracing`, `ag-core`, `futures-util`; dev-deps: `testcontainers`, `testcontainers-modules` (con feature `nats`)
- [ ] Nota: agregar `ws` a los features de `axum` en `Cargo.toml` root si no esta
- [ ] Verificar: `cargo check -p ag-realtime 2>&1 | grep "^error" | head -5`
- [ ] Commit: `chore(realtime): Cargo.toml y deps workspace`

---

## Task 2: RealtimeConfig

**Files:** `crates/ag-realtime/src/config.rs`

- [ ] TDD: `realtime_config_embedded_by_default` — sin `NATS_URL`, `config.nats_mode` es `NatsMode::Embedded`
- [ ] Implementar:
  ```rust
  pub enum NatsMode { Embedded, External }
  pub struct RealtimeConfig {
      pub nats_mode: NatsMode,        // NATS_MODE=embedded|external
      pub nats_url: String,           // NATS_URL, default "nats://localhost:4222"
      pub max_connections: usize,     // RT_MAX_CONNECTIONS, default 100_000
  }
  ```
- [ ] Correr test, commit: `feat(realtime): RealtimeConfig from_env`

---

## Task 3: Bus NATS (embedded y externo)

**Files:** `crates/ag-realtime/src/bus.rs`

- [ ] TDD (unitarios, servidor NATS en memoria via `async-nats` test utilities):
  - `bus_publish_and_subscribe_roundtrip` — publicar mensaje en subject, suscriptor lo recibe
  - `bus_multiple_subscribers_receive_same_message` — dos suscriptores en el mismo subject
- [ ] TDD (integracion con testcontainers NATS, modo externo):
  - `bus_external_nats_connect_and_publish`
- [ ] Correr para verificar que fallan
- [ ] Implementar `EventBus`:
  ```rust
  pub struct EventBus { client: async_nats::Client }
  impl EventBus {
      pub async fn connect_embedded() -> Result<Self, RealtimeError>
      pub async fn connect_external(url: &str) -> Result<Self, RealtimeError>
      pub async fn publish<T: Serialize>(&self, subject: &str, payload: &T) -> Result<()>
      pub async fn subscribe(&self, subject: &str) -> Result<impl Stream<Item = Message>>
  }
  ```
- [ ] `Message` tiene: `subject: String`, `payload: Vec<u8>`
- [ ] Integrar con ag-observe: counter `ag_realtime_messages_total{subject, direction}`
- [ ] Correr todos los tests
- [ ] Commit: `feat(realtime): EventBus NATS — embedded y externo`

---

## Task 4: WebSocket binario con msgpack

**Files:** `crates/ag-realtime/src/ws.rs`

- [ ] TDD (unitarios con servidor en memoria):
  - `ws_handler_upgrades_connection` — cliente se conecta, recibe mensaje de bienvenida
  - `ws_message_deserialized_from_msgpack` — enviar bytes msgpack, handler los deserializa correctamente
  - `ws_broadcast_reaches_all_connected_clients` — conectar 3 clientes, broadcast llega a los tres
- [ ] Correr para verificar que fallan
- [ ] Implementar:
  - `WsHandler { bus: Arc<EventBus>, connections: Arc<RwLock<HashMap<Uuid, WsSender>>> }`
  - Handler Axum `ws_upgrade(ws: WebSocketUpgrade, ...) -> Response` que: acepta el upgrade, registra la conexion, loop de lectura/escritura con msgpack
  - `WsHandler::broadcast(subject, payload)` — publica en bus y envia a todos los clientes conectados
- [ ] Gauge `ag_realtime_active_connections` incrementado/decrementado en connect/disconnect
- [ ] Correr tests
- [ ] Commit: `feat(realtime): WebSocket handler binario con msgpack`

---

## Task 5: SSE fallback

**Files:** `crates/ag-realtime/src/sse.rs`

- [ ] TDD (unitarios):
  - `sse_stream_sends_events` — suscribir a subject, publicar evento, verificar que aparece en el stream SSE
  - `sse_stream_filters_by_predicate` — solo eventos que pasen el filtro llegan al cliente
- [ ] Correr para verificar que fallan
- [ ] Implementar:
  ```rust
  pub fn sse_stream(
      bus: Arc<EventBus>,
      subject: &str,
      filter: impl Fn(&SseEvent) -> bool + Send + 'static,
  ) -> impl IntoResponse
  ```
  Usando `axum::response::sse::Sse` con `futures_util::StreamExt`
- [ ] Cada mensaje del bus se convierte en `Event::default().data(json_payload)`
- [ ] Correr tests
- [ ] Commit: `feat(realtime): SSE handler con filtro`

---

## Task 6: AgRealtime facade y verificacion final

**Files:** `crates/ag-realtime/src/lib.rs`

- [ ] Implementar `AgRealtime { bus: Arc<EventBus>, ws: WsHandler }` con:
  - `new(config: RealtimeConfig) -> Result<Self>`
  - `broadcast(subject, payload)` — delega al bus
  - `ws_handler()` — retorna el handler Axum para montar en el router
  - `sse_handler(subject, filter)` — retorna el handler SSE
- [ ] `cargo fmt --all && cargo clippy -p ag-realtime -- -D warnings`
- [ ] `cargo test -p ag-realtime 2>&1 | tail -10`
- [ ] `cargo doc -p ag-realtime --no-deps 2>&1 | grep "^error" | head -5`
- [ ] Commit: `feat(realtime): AgRealtime facade`
- [ ] Merge: `git checkout fase-4 && git merge --no-ff fase-4/ag-realtime -m "feat(realtime): ag-realtime completo — WebSocket/SSE/NATS"`
