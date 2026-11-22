# ag-observe

Observabilidad nativa para Anti-Gravital: tracing estructurado (JSON/text)
y metricas Prometheus con handler HTTP.

> Estado: Fase 4 -- implementado.

## Uso minimo

```rust
use ag_observe::{init, ObserveConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ObserveConfig::from_env();
    init(config)?;  // llama una sola vez al arranque

    tracing::info!(user_id = 42, action = "login", "usuario autenticado");
    Ok(())
}
```

## Endpoint de metricas Prometheus

```rust
use ag_observe::metrics_handler;
use axum::{Router, routing::get};

let app = Router::new()
    .route("/metrics", get(metrics_handler));
```

Expone metricas en formato Prometheus text en `/metrics`.

## Capacidades

### Tracing estructurado

- Formato `json` (produccion) o `text` (desarrollo) via `LOG_FORMAT`.
- Nivel de log configurable via `LOG_LEVEL` (default: `info`).
- Integra con `tracing` y sus macros (`info!`, `warn!`, `error!`, etc.).

### OpenTelemetry OTLP (feature opt-in `otlp`)

El exportador de trazas OTLP se habilita con la feature de Cargo `otlp`, que
esta DESACTIVADA por defecto. El transporte es OTLP/HTTP con protobuf
binario (`http-proto`) y cliente HTTP bloqueante:

```toml
[dependencies]
ag-observe = { path = "...", features = ["otlp"] }
```

```rust
std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4318/v1/traces");
std::env::set_var("SERVICE_NAME", "mi-servicio");
ag_observe::init(&ag_observe::ObserveConfig::from_env())?;
```

- No requiere `protoc`: el crate usa codigo protobuf pregenerado. El
  transporte `grpc-tonic`, que en versiones antiguas si compilaba `.proto`
  en tiempo de build, NO se habilita (ADR-0019).
- No requiere un runtime Tokio activo: el `BatchSpanProcessor` usa un hilo
  propio y el cliente HTTP es bloqueante. Funciona con `main` o
  `#[tokio::main]`.
- Sin la feature `otlp`, si `OTEL_EXPORTER_OTLP_ENDPOINT` esta definida,
  `init()` devuelve un error explicito en vez de aceptar una configuracion
  que no exporta trazas.
- El valor de `OTEL_EXPORTER_OTLP_ENDPOINT` se usa literalmente como URL de
  exportacion de trazas; debe incluir la ruta del senal, por ejemplo
  `http://localhost:4318/v1/traces`. No se anade `/v1/traces` automaticamente.
- El colector OTLP es opcional: sin la feature o sin el endpoint, el
  logging estructurado y las metricas Prometheus siguen funcionando (ADR-0009).

### Metricas Prometheus

- Registro global de metricas via `metrics` + `metrics-exporter-prometheus`.
- `metrics_handler` para Axum expone el texto Prometheus en `/metrics`.

## Variables de entorno

| Variable | Default | Descripcion |
|---|---|---|
| `SERVICE_NAME` | `ag-service` | Nombre del servicio en trazas |
| `LOG_LEVEL` | `info` | Nivel de log (`trace`, `debug`, `info`, `warn`, `error`) |
| `LOG_FORMAT` | `text` | `text` o `json` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `""` | Con la feature `otlp`, activa la exportacion OTLP/HTTP. Sin ella, `init()` falla explicitamente |
| `SERVICE_NAME` | `ag-service` | Valor del atributo `service.name` cuando se activa OTLP |

## Referencias

- Spec de diseno: `docs/superpowers/specs/2026-05-23-fase4-completion-design.md`
- Arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 14.
- Capitulo navegable: `docs/architecture/14-observabilidad-ag-observe.md`.
- Decision de transporte OTLP: `docs/adr/0019-ag-observe-otlp-transport.md`.
- Constitucion tecnica: `CLAUDE.md`.
