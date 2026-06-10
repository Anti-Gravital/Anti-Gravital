# ag-realtime

> Capitulo de arquitectura: `docs/architecture/08-modulos-batteries-included.md`.
> README del crate: `crates/ag-realtime/README.md`.
> Criticidad: Estandar.
> Fase de implementacion: Fase 4. Estado: implementado.

## Dominio

Bus de eventos pub/sub en proceso, cliente NATS externo con TLS y JetStream,
helpers Axum para WebSocket y SSE (EventSource-compatible).

## Stack implementado

| Componente | Libreria | Version |
|---|---|---|
| Broadcast bus InProcess | `tokio::sync::broadcast` | (tokio 1.x) |
| NATS cliente externo | `async-nats` | 0.48 |
| SSE stream adapter | `tokio-stream` | 0.1 |
| HTTP server helpers | `axum` | 0.7 |
| Serializacion payload | `serde_json` | 1.x |

## Capacidades implementadas (Fase 4)

### Bus InProcess

- `EventBus` con `broadcast(subject, payload)` y `subscribe() -> Receiver<Event>`.
- `AgRealtime::new(config) -> Result<Self>` — asincrono.
- `AgRealtime::bus() -> Option<Arc<EventBus>>` — solo en modo InProcess.
- `AgRealtime::broadcast(subject, payload)` / `broadcast_json(subject, &T)`.

### NATS externo (feature `nats-external`)

- `NatsExternalClient` — connect, publish, subscribe como `BoxStream`.
- TLS nivel 1: CAs del sistema operativo (`NATS_TLS=system`).
- TLS nivel 2: CA custom PEM (`NATS_TLS=custom`, `NATS_CA_PATH`).
- TLS nivel 3: mTLS (`NATS_TLS=mtls`, cert+key del cliente).
- JetStream: stream `AG_EVENTS`, publish con ACK, consumer efimero.

### Helpers Axum

- `ws_handler` — WebSocket <-> EventBus. Bridge bidireccional con spawn.
- `sse_handler` — EventBus -> SSE stream compatible con `EventSource`.
- `bus_to_sse_stream(bus)` — funcion publica para composicion manual.

## RealtimeBus enum

```rust
enum RealtimeBus {
    InProcess(Arc<EventBus>),
    #[cfg(feature = "nats-external")]
    External(Arc<NatsExternalClient>),
}
```

La variante se selecciona en runtime por `NATS_MODE` (`inprocess` | `external`).

## Dependencias internas permitidas

Depende de `ag-core`. Puede depender de `ag-auth` para suscripciones autenticadas.

### Event persistence (feature `event-persistence`)

Critical events can be written to an append-only NDJSON file before publishing,
so a restart does not drop them. Usage:

```rust
use ag_realtime::persistence::{EventBuffer, replay_into_bus};

// On startup: replay persisted events into the bus.
let buf = EventBuffer::open("events.ndjson")?;
replay_into_bus(&buf, &bus)?;

// On each critical publish: append first, then publish.
buf.append("user.created", payload)?;
bus.publish("user.created", payload.to_vec())?;
```

For async producers, use `append_async`. It reuses the persistent handle,
runs filesystem work on Tokio's blocking pool, and permits 64 pending appends
by default. `open_with_max_pending_appends` configures that bound; calls wait
for capacity rather than submitting unbounded blocking work.

During shutdown, stop producers, await `flush_async`, and then drop all
`EventBuffer` clones. There is no detached writer task. Malformed or
truncated replay records fail with `InvalidData` and line context;
publication failures stop replay with `BrokenPipe`.

The buffer is intentionally minimal (file-only). A database-backed store
requires an RFC per CLAUDE.md section 22.

### Fallback: External -> InProcess

When `NATS_MODE=external` but the `nats-external` feature is not compiled in,
`AgRealtime::new()` falls back to `InProcess` with a warning log. This means
services always start, even without a NATS server.

## Scalability criterion (Phase 4)

50,000 concurrent subscribers received 100% of events in < 35 ms (debug
profile, AMD Ryzen 5 2500U, 4 worker threads). Full methodology and hardware:
`docs/benchmarks/ag-realtime-load.md`.

Run the load test manually:

```bash
cargo test -p ag-realtime --test load_50k -- --ignored --nocapture
```

## Tests

Coverage >= 80%. Race condition in env-var tests resolved with
`static ENV_LOCK: Mutex<()>` in `config.rs`. Working example:
`examples/realtime-chat` (InProcess, port 3000).

## Status

Phases 2-4 complete. All criteria for Phase 4 satisfied.
