//! Observabilidad nativa: tracing, OpenTelemetry exporter, metricas Prometheus y tokio-console.
//!
//! Estado: Fase 4 (Modulos Estandar). Implementa configuracion, inicializacion
//! del subscriber de tracing y metricas Prometheus estandar.

pub mod config;

pub use config::{LogFormat, ObserveConfig};
