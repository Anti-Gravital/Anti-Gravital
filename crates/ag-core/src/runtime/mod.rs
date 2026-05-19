//! Construccion del runtime Tokio para Anti-Gravital.
//!
//! Lee la configuracion declarativa de `crate::config::RuntimeConfig`
//! y construye un `tokio::runtime::Runtime` multi-thread con los
//! parametros indicados. Los valores por defecto vienen del maestro
//! de arquitectura seccion 6.5.

use crate::config::RuntimeConfig;
use crate::error::{AgError, AgResult};

/// Construye un runtime Tokio multi-thread segun la configuracion.
///
/// # Errores
///
/// Devuelve `AgError::Other` si la construccion del runtime falla.
pub fn build(config: &RuntimeConfig) -> AgResult<tokio::runtime::Runtime> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    if let Some(workers) = config.workers {
        builder.worker_threads(workers);
    }
    builder.max_blocking_threads(config.blocking_threads);
    builder
        .build()
        .map_err(|e| AgError::Other(format!("failed to build tokio runtime: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_runtime_with_defaults() {
        let rt = build(&RuntimeConfig::default()).unwrap();
        let result = rt.block_on(async { 1 + 1 });
        assert_eq!(result, 2);
    }

    #[test]
    fn builds_runtime_with_two_workers() {
        let cfg = RuntimeConfig {
            workers: Some(2),
            blocking_threads: 16,
        };
        let rt = build(&cfg).unwrap();
        let result = rt.block_on(async { "ok" });
        assert_eq!(result, "ok");
    }
}
