//! Observabilidad nativa: tracing, OpenTelemetry exporter, metricas Prometheus y tokio-console.
//!
//! Estado: Fase 4 (Modulos Estandar). Implementa configuracion, inicializacion
//! del subscriber de tracing y metricas Prometheus estandar.

pub mod config;
pub mod layer;
pub mod metrics;

pub use config::{LogFormat, ObserveConfig};
pub use layer::{init, ObserveError};
pub use metrics::{
    dec_active_connections, inc_active_connections, metrics_handler, record_request, set_db_pool,
};
