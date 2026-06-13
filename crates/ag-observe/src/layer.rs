//! Construction of the composed tracing subscriber.

use crate::config::{LogFormat, ObserveConfig};
use crate::metrics::install_prometheus_recorder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialization error for the observability subsystem.
#[derive(Debug)]
#[non_exhaustive]
pub enum ObserveError {
    /// The global subscriber was already initialized.
    AlreadyInitialized,
    /// Error configuring the OTLP exporter.
    OtlpSetup(String),
}

impl std::fmt::Display for ObserveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObserveError::AlreadyInitialized => {
                write!(f, "the tracing subscriber was already initialized")
            }
            ObserveError::OtlpSetup(msg) => write!(f, "error configuring OTLP: {msg}"),
        }
    }
}

impl std::error::Error for ObserveError {}

/// Initializes the observability subsystem.
///
/// Configures the global `tracing` subscriber with layers for structured
/// logging and Prometheus metrics.
///
/// OTLP trace export is reserved for a future version. If
/// `ObserveConfig::otlp_endpoint` is configured, initialization returns
/// [`ObserveError::OtlpSetup`] instead of silently dropping traces.
///
/// Must be called only once at process startup, before any
/// invocation of tracing macros.
///
/// # Errors
///
/// Returns [`ObserveError::AlreadyInitialized`] if the global subscriber
/// was already configured by a previous call.
pub fn init(config: &ObserveConfig) -> Result<(), ObserveError> {
    if let Some(endpoint) = &config.otlp_endpoint {
        return Err(ObserveError::OtlpSetup(format!(
            "OTEL_EXPORTER_OTLP_ENDPOINT is configured as {endpoint:?}, but OTLP trace export is not available in this version"
        )));
    }

    install_prometheus_recorder();

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let registry = tracing_subscriber::registry().with(env_filter);

    match config.log_format {
        LogFormat::Json => {
            registry
                .with(tracing_subscriber::fmt::layer().json())
                .try_init()
                .map_err(|_| ObserveError::AlreadyInitialized)?;
        }
        LogFormat::Pretty => {
            registry
                .with(tracing_subscriber::fmt::layer())
                .try_init()
                .map_err(|_| ObserveError::AlreadyInitialized)?;
        }
    }

    tracing::info!(
        prometheus_port = config.prometheus_port,
        otlp = false,
        "observability initialized"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ObserveConfig;

    #[test]
    fn init_does_not_panic_with_default_config() {
        // init() may fail with AlreadyInitialized if another test called it first
        // but should never panic
        let result = init(&ObserveConfig::default());
        assert!(result.is_ok() || matches!(result, Err(ObserveError::AlreadyInitialized)));
    }

    #[test]
    fn init_json_format_does_not_panic() {
        use crate::config::{LogFormat, ObserveConfig};
        let config = ObserveConfig {
            log_format: LogFormat::Json,
            ..ObserveConfig::default()
        };
        let result = init(&config);
        assert!(result.is_ok() || matches!(result, Err(ObserveError::AlreadyInitialized)));
    }

    #[test]
    fn init_with_otlp_endpoint_returns_error() {
        use crate::config::ObserveConfig;
        let config = ObserveConfig {
            otlp_endpoint: Some("http://localhost:4317".to_string()),
            ..ObserveConfig::default()
        };
        let result = init(&config);
        assert!(
            matches!(result, Err(ObserveError::OtlpSetup(message)) if message.contains("OTEL_EXPORTER_OTLP_ENDPOINT"))
        );
    }

    #[test]
    fn already_initialized_error_displays_correctly() {
        let e = ObserveError::AlreadyInitialized;
        assert!(e.to_string().contains("initialized"));
    }

    #[test]
    fn otlp_setup_error_displays_correctly() {
        let e = ObserveError::OtlpSetup("failed".to_string());
        assert!(e.to_string().contains("failed"));
    }
}
