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

## Tests

Cobertura >= 80%. Race condition en tests de env vars resuelta con
`static ENV_LOCK: Mutex<()>` en `config.rs` (equivalente al patron de `ag-auth`).
Ejemplo operativo: `examples/realtime-chat` (InProcess, puerto 3000).

## Pendiente (criterios externos)

- Publicacion en crates.io con version 0.1.0.
- Benchmark 50K conexiones WebSocket en 2 vCPU.
- Presence y replay de eventos (planificados para Fase 5).
