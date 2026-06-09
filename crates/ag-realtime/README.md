# ag-realtime

Comunicacion en tiempo real para Anti-Gravital: bus de eventos en proceso,
cliente NATS externo con TLS y JetStream, helpers WebSocket y SSE para Axum.

> Estado: Fase 4 -- implementado.

## Uso minimo (InProcess)

```rust
use ag_realtime::{AgRealtime, RealtimeConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RealtimeConfig::default();  // NATS_MODE no definido => InProcess
    let realtime = AgRealtime::new(config).await?;

    // Publicar
    realtime.broadcast("chat.sala1", b"hola".to_vec())?;

    // Suscribir
    if let Some(bus) = realtime.bus() {
        let mut rx = bus.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                println!("{}: {:?}", event.subject, event.payload);
            }
        });
    }
    Ok(())
}
```

## Uso con NATS externo

```bash
NATS_MODE=external NATS_URL=nats://localhost:4222 cargo run
```

Con TLS:

```bash
NATS_TLS=system          # usa CAs del sistema operativo
NATS_TLS=custom          # NATS_CA_PATH=/etc/certs/ca.pem
NATS_TLS=mtls            # NATS_CA_PATH=... NATS_CERT_PATH=... NATS_KEY_PATH=...
```

Con JetStream:

```bash
NATS_JETSTREAM=true      # crea stream AG_EVENTS automaticamente
```

## Helpers Axum

### WebSocket

```rust
use ag_realtime::ws::ws_handler;
use std::sync::Arc;

let bus = realtime.bus().unwrap();
let app = Router::new()
    .route("/ws", get(ws_handler))
    .with_state(bus);
```

### SSE (EventSource-compatible)

```rust
use ag_realtime::sse::sse_handler;

let app = Router::new()
    .route("/events", get(sse_handler))
    .with_state(bus);
```

## Variables de entorno

| Variable | Default | Descripcion |
|---|---|---|
| `NATS_MODE` | `inprocess` | `inprocess` o `external` |
| `NATS_URL` | `nats://localhost:4222` | URL del servidor NATS |
| `NATS_TLS` | `""` | `system`, `custom` o `mtls` |
| `NATS_CA_PATH` | `""` | Ruta al certificado CA (custom/mtls) |
| `NATS_CERT_PATH` | `""` | Ruta al certificado cliente (mtls) |
| `NATS_KEY_PATH` | `""` | Ruta a la clave cliente (mtls) |
| `NATS_JETSTREAM` | `false` | Activar JetStream |

## Persistencia opcional de eventos

La feature `event-persistence` habilita un buffer NDJSON append-only para
eventos criticos. `EventBuffer::append_async` reutiliza un handle persistente,
ejecuta el I/O en el pool bloqueante y limita a 64 los appends pendientes por
defecto. `open_with_max_pending_appends` permite ajustar ese limite; al
alcanzarlo, las llamadas esperan capacidad y aplican backpressure. En shutdown,
se dejan de enviar appends, se espera `flush_async` y se descartan todos los
clones del buffer; no queda un writer task en segundo plano.

Durante replay, registros truncados, campos ausentes o tipos invalidos detienen
el proceso con `InvalidData` y numero de linea; un fallo de publicacion se
propaga como `BrokenPipe`.

## Features

- `nats-external` -- cliente NATS externo via `async-nats 0.48`.
  Habilitado por defecto. Requerir `NATS_MODE=external` en runtime.
- `event-persistence` -- buffer NDJSON local para eventos criticos.

## Referencias

- Spec de diseno: `docs/superpowers/specs/2026-05-23-fase4-completion-design.md`
- Arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 8.4.
- RFC-0005 (propuesto): `docs/rfc/RFC-0005-ag-cache-native-l2.md`.
- Constitucion tecnica: `CLAUDE.md`.
