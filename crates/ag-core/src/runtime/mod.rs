//! Tokio runtime construction for Anti-Gravital.
//!
//! Reads the declarative configuration from `crate::config::RuntimeConfig`
//! and builds a multi-thread `tokio::runtime::Runtime` with the given
//! parameters. The default values come from the architecture master
//! document, section 6.5.

use crate::config::RuntimeConfig;
use crate::error::{AgError, AgResult};

/// Builds a multi-thread Tokio runtime according to the configuration.
///
/// # Errors
///
/// Returns `AgError::Other` if building the runtime fails.
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
