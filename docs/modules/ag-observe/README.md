# ag-observe

> Capitulo de arquitectura: `docs/architecture/14-observabilidad-ag-observe.md`.
> README del crate: `crates/ag-observe/README.md`.
> Criticidad: Estandar.
> Fase de implementacion: Fase 4. Estado: implementado.

## Dominio

Observabilidad nativa: tracing estructurado (JSON/text), exporter OpenTelemetry OTLP,
metricas Prometheus con handler HTTP Axum.

## Stack implementado

| Componente | Libreria | Version |
|---|---|---|
| Tracing estructurado | `tracing` + `tracing-subscriber` | 0.1 / 0.3 |
| Formato JSON | `tracing-subscriber` EnvFilter + fmt | 0.3 |
| Exporter OTLP | `opentelemetry-otlp` | 0.27 |
| Metricas Prometheus | `metrics` + `metrics-exporter-prometheus` | 0.24 |

## Capacidades implementadas (Fase 4)

- `ObserveConfig::from_env()` / `ObserveConfig::default()`.
- `ag_observe::init(config: &ObserveConfig) -> Result<(), ObserveError>` — idempotente.
  Retorna `ObserveError::AlreadyInitialized` en llamadas subsiguientes (no panic).
- `ag_observe::metrics_handler` — handler Axum para `/metrics` Prometheus.
- `LogFormat::Json` / `LogFormat::Text` — seleccionado via `LOG_FORMAT`.
- Propagacion de contexto W3C TraceContext cuando `OTLP_ENDPOINT` esta definida.

## Uso en Axum

```rust
use ag_observe::{init, metrics_handler, ObserveConfig};
use axum::{Router, routing::get};

ag_observe::init(&ObserveConfig::from_env())?;

let app = Router::new()
    .route("/metrics", get(metrics_handler));
```

## Dependencias internas permitidas

Depende de `ag-core`.

## Tests

Cobertura >= 80%. Tests para init idempotente, AlreadyInitialized, metrics handler,
layer personalizado y lectura de ObserveConfig desde variables de entorno.

## Pendiente (criterios externos)

- Publicacion en crates.io con version 0.1.0.
- Dashboards Grafana JSON incluidos en el crate (planificado Fase 5).
- Integracion con tokio-console en modo dev (planificado Fase 5).
