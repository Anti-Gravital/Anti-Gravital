//! Construction of the composed tracing subscriber.

use crate::config::{is_valid_project_id, LogFormat, ObserveConfig};
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
    /// The configured project scope is invalid. The value is never printed.
    InvalidScope,
}

impl std::fmt::Display for ObserveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObserveError::AlreadyInitialized => {
                write!(f, "the tracing subscriber was already initialized")
            }
            ObserveError::OtlpSetup(msg) => write!(f, "error configuring OTLP: {msg}"),
            ObserveError::InvalidScope => write!(
                f,
                "AG_PROJECT_ID is invalid: expected 1-64 characters from [A-Za-z0-9_.-]; value withheld"
            ),
        }
    }
}

impl std::error::Error for ObserveError {}

/// Builds the OpenTelemetry resource with the optional project scope.
#[cfg(feature = "otlp")]
fn build_resource(project_id: Option<&str>) -> opentelemetry_sdk::Resource {
    // RFC-0020 PR 4: this is the only direct env read left in ag-observe and
    // it is intentional. SERVICE_NAME is consumed here, while building the
    // OpenTelemetry resource inside init(), and not on the ObserveConfig::
    // from_env path (the `otlp` feature gates this whole function). Routing
    // it through the declared schema would require a new public field on the
    // public ObserveConfig struct, i.e. a public API change forbidden by this
    // PR. The key and its fallback are declared in
    // config::observe_env_declarations() for auditability; this fallback is
    // kept byte-identical to that declared default ("ag-service").
    let service_name = std::env::var("SERVICE_NAME").unwrap_or_else(|_| "ag-service".to_string());
    let mut builder = opentelemetry_sdk::Resource::builder().with_service_name(service_name);
    if let Some(project) = project_id {
        builder = builder.with_attribute(opentelemetry::KeyValue::new(
            "project.id",
            project.to_string(),
        ));
    }
    builder.build()
}

/// Builds the OTLP trace layer using the HTTP/protobuf transport.
///
/// The transport uses pre-generated protocol buffers, so no `protoc` compiler
/// is needed at build time. The blocking HTTP client works with a plain `main`
/// as well as with `#[tokio::main]`; only the gRPC transport needs a Tokio
/// runtime at provider construction. See ADR-0019.
#[cfg(feature = "otlp")]
fn build_otlp_layer<S>(
    endpoint: &str,
    project_id: Option<&str>,
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

    let resource = build_resource(project_id);

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

    if let Some(project) = &config.project_id {
        if !is_valid_project_id(project) {
            return Err(ObserveError::InvalidScope);
        }
    }

    install_prometheus_recorder(config.project_id.as_deref());

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match config.log_format {
        LogFormat::Json => {
            let registry = tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json());
            #[cfg(feature = "otlp")]
            let registry = {
                let otlp_layer = match &config.otlp_endpoint {
                    Some(endpoint) => {
                        Some(build_otlp_layer(endpoint, config.project_id.as_deref())?)
                    }
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
                    Some(endpoint) => {
                        Some(build_otlp_layer(endpoint, config.project_id.as_deref())?)
                    }
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
        project = config.project_id.as_deref().unwrap_or(""),
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
    fn otlp_resource_carries_the_project_attribute() {
        let resource = build_resource(Some("demo"));
        let value = resource
            .get(&opentelemetry::Key::new("project.id"))
            .expect("project.id attribute must be present");
        assert_eq!(value.to_string(), "demo");
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn build_otlp_layer_accepts_http_endpoint() {
        // Building the exporter creates an HTTP client but does not contact the
        // collector, so no external service is needed for this test.
        let layer = build_otlp_layer::<tracing_subscriber::Registry>(
            "http://127.0.0.1:4318/v1/traces",
            Some("test-project"),
        );
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
    fn init_rejects_invalid_project_scope_without_printing_it() {
        let config = ObserveConfig {
            project_id: Some("bad/scope".to_string()),
            ..ObserveConfig::default()
        };
        let result = init(&config);
        assert!(matches!(result, Err(ObserveError::InvalidScope)));
        let message = ObserveError::InvalidScope.to_string();
        assert!(!message.contains("bad/scope"));
        assert!(message.contains("AG_PROJECT_ID"));
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
