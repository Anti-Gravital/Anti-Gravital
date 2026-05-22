//! Observabilidad nativa para el ecosistema Anti-Gravital.
//!
//! Configura el subscriber global de `tracing` con logging estructurado,
//! exportacion OpenTelemetry (OTLP) y metricas Prometheus. Incluye dashboards
//! Grafana pre-configurados en `dashboards/`.
//!
//! # Uso minimo
//!
//! ```no_run
//! use ag_observe::{ObserveConfig, init};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! init(&ObserveConfig::from_env())?;
//! tracing::info!("servicio iniciado");
//! # Ok(())
//! # }
//! ```
//!
//! # Variables de entorno
//!
//! | Variable | Default | Descripcion |
//! |---|---|---|
//! | `LOG_FORMAT` | `pretty` | Formato de logs: `pretty` o `json` |
//! | `PROMETHEUS_PORT` | `9090` | Puerto donde se expone `/metrics` |
//! | `OTEL_EXPORTER_OTLP_ENDPOINT` | ninguno | Endpoint OTLP para trazas |
//! | `AG_DEV_CONSOLE` | `false` | Habilita tokio-console (feature `dev-console`) |

pub mod config;
pub mod layer;
pub mod metrics;

pub use config::{LogFormat, ObserveConfig};
pub use layer::{init, ObserveError};
pub use metrics::{
    dec_active_connections, inc_active_connections, metrics_handler, record_request, set_db_pool,
};
