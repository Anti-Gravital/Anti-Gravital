//! Native observability for the Anti-Gravital ecosystem.
//!
//! Configures the global `tracing` subscriber with structured logging and
//! Prometheus metrics. Includes pre-configured Grafana dashboards in
//! `dashboards/`.
//!
//! Trace export over OTLP/HTTP is available behind the opt-in `otlp` Cargo
//! feature. It is off by default so the native exporters never require an
//! external collector (ADR-0009, ADR-0019). The transport uses pre-generated
//! protocol buffers and needs no `protoc` compiler at build time.
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
//! | `OTEL_EXPORTER_OTLP_ENDPOINT` | none | With the `otlp` feature, full traces URL (e.g. `http://localhost:4318/v1/traces`); without it, `init()` returns an error |
//! | `SERVICE_NAME` | `ag-service` | Value of the `service.name` resource attribute when OTLP export is enabled |
//! | `AG_DEV_CONSOLE` | `false` | Enables tokio-console (feature `dev-console`) |

pub mod config;
pub mod layer;
pub mod metrics;

pub use config::{LogFormat, ObserveConfig};
pub use layer::{init, ObserveError};
pub use metrics::{
    dec_active_connections, inc_active_connections, metrics_handler, record_request, set_db_pool,
};
