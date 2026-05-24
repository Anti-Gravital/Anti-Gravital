# ag-observe

Observabilidad nativa para Anti-Gravital: tracing estructurado (JSON/text),
exporter OpenTelemetry (OTLP gRPC), metricas Prometheus con handler HTTP.

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

### OpenTelemetry OTLP

- Exporter gRPC hacia un colector OTLP (Jaeger, Grafana Tempo, etc.).
- Activado cuando `OTLP_ENDPOINT` esta definida.
- Propagacion de contexto W3C TraceContext.

### Metricas Prometheus

- Registro global de metricas via `metrics` + `metrics-exporter-prometheus`.
- `metrics_handler` para Axum expone el texto Prometheus en `/metrics`.

## Variables de entorno

| Variable | Default | Descripcion |
|---|---|---|
| `SERVICE_NAME` | `ag-service` | Nombre del servicio en trazas |
| `LOG_LEVEL` | `info` | Nivel de log (`trace`, `debug`, `info`, `warn`, `error`) |
| `LOG_FORMAT` | `text` | `text` o `json` |
| `OTLP_ENDPOINT` | `""` | gRPC OTLP (ej: `http://localhost:4317`). Vacio = deshabilitado |

## Referencias

- Spec de diseno: `docs/superpowers/specs/2026-05-23-fase4-completion-design.md`
- Arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 14.
- Capitulo navegable: `docs/architecture/14-observabilidad-ag-observe.md`.
- Constitucion tecnica: `CLAUDE.md`.
