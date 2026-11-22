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

- Formato `json` (produccion) o `pretty` (desarrollo) via `LOG_FORMAT`.
- Filtro de nivel via `RUST_LOG` del `tracing-subscriber` (default: `info`).
  No se lee una variable `LOG_LEVEL`.
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

## Alcance por proyecto (RFC-0038)

`AG_PROJECT_ID` es una etiqueta opcional y acotada (1-64 caracteres de
`[A-Za-z0-9_.-]`) que identifica el proyecto de forma logica. No es un limite
de seguridad ni un identificador multi-tenant. Se valida al arrancar: un valor
invalido hace que `init()` devuelva `ObserveError::InvalidScope` **sin imprimir
el valor**. Cuando esta definida:

- se anade como etiqueta constante `project` a todas las metricas Prometheus;
- se anade como atributo `project.id` al recurso OpenTelemetry cuando la
  feature `otlp` esta activa.

Los valores no acotados se rechazan para evitar crecimiento ilimitado de series.

## Salud de componentes (RFC-0038)

`HealthRegistry` registra compruebas sincronicas por nombre y expone dos
handlers para Axum:

```rust
use ag_observe::{HealthRegistry, ComponentHealth, readiness_handler, liveness_handler};
use axum::{Router, routing::get, extract::State};

let health = HealthRegistry::new(std::env::var("AG_PROJECT_ID").ok());
health.register("database", || ComponentHealth::new("database", true));

let app = Router::new()
    .route("/health", get(liveness_handler))
    .route("/ready", get(readiness_handler))
    .with_state(health);
```

`/health` devuelve 200 sin evaluar dependencias. `/ready` devuelve 200 si todos
los componentes estan sanos y 503 si alguno falla; el cuerpo JSON incluye
`status`, `project` y la lista de componentes. No hay collector, poller ni
servicio externo: las comprobaciones se evaluan cuando se consulta el endpoint
(ADR-0009). Las comprobaciones no deben devolver secretos ni datos personales.

## Variables de entorno

| Variable | Default | Descripcion |
|---|---|---|
| `SERVICE_NAME` | `ag-service` | Valor del atributo `service.name` cuando se activa OTLP |
| `RUST_LOG` | `info` | Filtro de nivel de `tracing-subscriber` |
| `LOG_FORMAT` | `pretty` | `pretty` o `json` |
| `PROMETHEUS_PORT` | `9090` | Puerto del endpoint `/metrics` |
| `AG_PROJECT_ID` | none | Etiqueta de proyecto acotada (1-64 chars, `[A-Za-z0-9_.-]`) |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | none | Con la feature `otlp`, activa la exportacion OTLP/HTTP. Sin ella, `init()` falla explicitamente |
| `AG_DEV_CONSOLE` | `false` | Activa tokio-console con la feature `dev-console` |

## Referencias

- Spec de diseno: `docs/superpowers/specs/2026-05-23-fase4-completion-design.md`
- Arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 14.
- Capitulo navegable: `docs/architecture/14-observabilidad-ag-observe.md`.
- Decision de transporte OTLP: `docs/adr/0019-ag-observe-otlp-transport.md`.
- Alcance y salud de componentes: `docs/rfc/RFC-0038-observability-scopes-and-health.md`.
- Constitucion tecnica: `CLAUDE.md`.
