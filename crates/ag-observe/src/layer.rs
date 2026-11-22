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

/// Builds the OTLP trace layer using the HTTP/protobuf transport.
///
/// The transport uses pre-generated protocol buffers, so no `protoc` compiler
/// is needed at build time. The blocking HTTP client works with a plain `main`
/// as well as with `#[tokio::main]`; only the gRPC transport needs a Tokio
/// runtime at provider construction. See ADR-0019.
#[cfg(feature = "otlp")]
fn build_otlp_layer<S>(
    endpoint: &str,
) -> Result<Box<dyn tracing_subscriber::Layer<S> + Send + Sync>, ObserveError>
where
    S: tracing::Subscriber
        + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>
        + Send
        + Sync
        + 'static,
{
    use opentelemetry_otlp::{Protocol, SpanExporter, WithExportConfig};

    // reqwest 0.13 selects a TLS backend through Cargo features that are
    // unified across the dependency graph. When another crate enables
    // `rustls-no-provider`, building any reqwest client panics unless a
    // process-wide provider was installed first. Install ring idempotently,
    // matching the workspace convention (ag-mail native MTA, issue #181).
    let _ =
        rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());

    let exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(endpoint.to_string())
        .build()
        .map_err(|err| {
            ObserveError::OtlpSetup(format!("failed to build the OTLP HTTP exporter: {err}"))
        })?;

    let service_name = std::env::var("SERVICE_NAME").unwrap_or_else(|_| "ag-service".to_string());
    let resource = opentelemetry_sdk::Resource::builder()
        .with_service_name(service_name)
        .build();

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();
    opentelemetry::global::set_tracer_provider(provider);

    Ok(Box::new(
        tracing_opentelemetry::layer::<S>()
            .with_tracer(opentelemetry::global::tracer("ag-observe")),
    ))
}

/// Initializes the observability subsystem.
///
/// Configures the global `tracing` subscriber with layers for structured
/// logging and Prometheus metrics. When the `otlp` Cargo feature is enabled and
/// `ObserveConfig::otlp_endpoint` is set, an OpenTelemetry layer exporting
/// traces over OTLP/HTTP is added as well. Without the feature, configuring an
/// endpoint returns [`ObserveError::OtlpSetup`] instead of silently dropping
/// traces.
///
/// The native logging and Prometheus exporters are always active and never
/// require an external collector (ADR-0009).
///
/// Must be called only once at process startup, before any
/// invocation of tracing macros.
///
/// # Errors
///
/// Returns [`ObserveError::AlreadyInitialized`] if the global subscriber
/// was already configured by a previous call, and
/// [`ObserveError::OtlpSetup`] if an endpoint was configured but the OTLP
/// layer could not be built.
pub fn init(config: &ObserveConfig) -> Result<(), ObserveError> {
    #[cfg(not(feature = "otlp"))]
    if let Some(endpoint) = &config.otlp_endpoint {
        return Err(ObserveError::OtlpSetup(format!(
            "OTEL_EXPORTER_OTLP_ENDPOINT is set to {endpoint:?}, but OTLP export requires the `otlp` Cargo feature"
        )));
    }

    install_prometheus_recorder();

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match config.log_format {
        LogFormat::Json => {
            let registry = tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json());
            #[cfg(feature = "otlp")]
            let registry = {
                let otlp_layer = match &config.otlp_endpoint {
                    Some(endpoint) => Some(build_otlp_layer(endpoint)?),
                    None => None,
                };
                registry.with(otlp_layer)
            };
            registry
                .try_init()
                .map_err(|_| ObserveError::AlreadyInitialized)?;
        }
        LogFormat::Pretty => {
            let registry = tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer());
            #[cfg(feature = "otlp")]
            let registry = {
                let otlp_layer = match &config.otlp_endpoint {
                    Some(endpoint) => Some(build_otlp_layer(endpoint)?),
                    None => None,
                };
                registry.with(otlp_layer)
            };
            registry
                .try_init()
                .map_err(|_| ObserveError::AlreadyInitialized)?;
        }
    }

    tracing::info!(
        prometheus_port = config.prometheus_port,
        otlp = cfg!(feature = "otlp") && config.otlp_endpoint.is_some(),
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

    #[cfg(not(feature = "otlp"))]
    #[test]
    fn init_with_otlp_endpoint_returns_error_without_feature() {
        let config = ObserveConfig {
            otlp_endpoint: Some("http://localhost:4318/v1/traces".to_string()),
            ..ObserveConfig::default()
        };
        let result = init(&config);
        assert!(
            matches!(result, Err(ObserveError::OtlpSetup(message)) if message.contains("OTEL_EXPORTER_OTLP_ENDPOINT"))
        );
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn build_otlp_layer_accepts_http_endpoint() {
        // Building the exporter creates an HTTP client but does not contact the
        // collector, so no external service is needed for this test.
        let layer =
            build_otlp_layer::<tracing_subscriber::Registry>("http://127.0.0.1:4318/v1/traces");
        assert!(layer.is_ok());
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn init_with_otlp_endpoint_succeeds_with_feature() {
        let config = ObserveConfig {
            otlp_endpoint: Some("http://127.0.0.1:4318/v1/traces".to_string()),
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
