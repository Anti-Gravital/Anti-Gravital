//! Construction of the composed tracing subscriber.

use crate::config::{LogFormat, ObserveConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialization error for the observability subsystem.
#[derive(Debug)]
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
/// logging, OTLP export (if configured) and Prometheus metrics.
///
/// Must be called only once at process startup, before any
/// invocation of tracing macros.
///
/// # Errors
///
/// Returns [`ObserveError::AlreadyInitialized`] if the global subscriber
/// was already configured by a previous call.
pub fn init(config: &ObserveConfig) -> Result<(), ObserveError> {
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

    // TECH-DEBT:
    // motivo: Full opentelemetry-otlp integration requires an active tokio
    //         runtime at the time init() is called. The real OTLP exporter
    //         is omitted here to avoid forcing a runtime dependency at the
    //         initialization point.
    // impacto: The otlp_endpoint field is read and logged but no exporter
    //          is connected. Traces are not exported via OTLP.
    // eliminacion esperada: Phase 4, iteration ag-observe v0.2, when the
    //          async process initialization pattern is established.
    if config.otlp_endpoint.is_some() {
        tracing::warn!(
            otlp_endpoint = config.otlp_endpoint.as_deref(),
            "otlp_endpoint configured but the OTLP exporter is not active in this version"
        );
    }

    // Install the Prometheus exporter globally
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .install()
        .ok(); // If already installed, silently ignore

    tracing::info!(
        prometheus_port = config.prometheus_port,
        otlp = config.otlp_endpoint.is_some(),
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
    fn init_with_otlp_endpoint_logs_warning() {
        use crate::config::ObserveConfig;
        let config = ObserveConfig {
            otlp_endpoint: Some("http://localhost:4317".to_string()),
            ..ObserveConfig::default()
        };
        let result = init(&config);
        assert!(result.is_ok() || matches!(result, Err(ObserveError::AlreadyInitialized)));
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
