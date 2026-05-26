//! Native observability for the Anti-Gravital ecosystem.
//!
//! Configures the global `tracing` subscriber with structured logging,
//! OpenTelemetry (OTLP) export and Prometheus metrics. Includes
//! pre-configured Grafana dashboards in `dashboards/`.
//!
//! # Minimal usage
//!
//! ```no_run
//! use ag_observe::{ObserveConfig, init};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! init(&ObserveConfig::from_env())?;
//! tracing::info!("service started");
//! # Ok(())
//! # }
//! ```
//!
//! # Environment variables
//!
//! | Variable | Default | Description |
//! |---|---|---|
//! | `LOG_FORMAT` | `pretty` | Log format: `pretty` or `json` |
//! | `PROMETHEUS_PORT` | `9090` | Port where `/metrics` is exposed |
//! | `OTEL_EXPORTER_OTLP_ENDPOINT` | none | OTLP endpoint for traces |
//! | `AG_DEV_CONSOLE` | `false` | Enables tokio-console (feature `dev-console`) |

pub mod config;
pub mod layer;
pub mod metrics;

pub use config::{LogFormat, ObserveConfig};
pub use layer::{init, ObserveError};
pub use metrics::{
    dec_active_connections, inc_active_connections, metrics_handler, record_request, set_db_pool,
};
