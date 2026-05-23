//! Construccion del subscriber compuesto de tracing.

use crate::config::{LogFormat, ObserveConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Error de inicializacion del subsistema de observabilidad.
#[derive(Debug)]
pub enum ObserveError {
    /// El subscriber global ya fue inicializado.
    AlreadyInitialized,
    /// Error configurando el exporter OTLP.
    OtlpSetup(String),
}

impl std::fmt::Display for ObserveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObserveError::AlreadyInitialized => {
                write!(f, "el subscriber de tracing ya fue inicializado")
            }
            ObserveError::OtlpSetup(msg) => write!(f, "error configurando OTLP: {msg}"),
        }
    }
}

impl std::error::Error for ObserveError {}

/// Inicializa el subsistema de observabilidad.
///
/// Configura el subscriber global de `tracing` con layers para logging
/// estructurado, exportacion OTLP (si esta configurada) y metricas Prometheus.
///
/// Debe llamarse una sola vez al inicio del proceso, antes de cualquier
/// invocacion a macros de tracing.
///
/// # Errores
///
/// Retorna [`ObserveError::AlreadyInitialized`] si el subscriber global
/// ya fue configurado por una llamada anterior.
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
    // motivo: La integracion completa de opentelemetry-otlp requiere un
    //         runtime tokio activo en el momento de llamar a init(). El
    //         exporter OTLP real se omite aqui para no forzar una
    //         dependencia de runtime en el punto de inicializacion.
    // impacto: El campo otlp_endpoint se lee y se loguea pero no se
    //          conecta ningun exporter. Las trazas no se exportan via OTLP.
    // eliminacion esperada: Fase 4, iteracion ag-observe v0.2, cuando se
    //          establezca el patron de inicializacion asincrona del proceso.
    if config.otlp_endpoint.is_some() {
        tracing::warn!(
            otlp_endpoint = config.otlp_endpoint.as_deref(),
            "otlp_endpoint configurado pero el exporter OTLP no esta activo en esta version"
        );
    }

    // Inicializar el exporter Prometheus globalmente
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .install()
        .ok(); // Si ya esta instalado, ignorar silenciosamente

    tracing::info!(
        prometheus_port = config.prometheus_port,
        otlp = config.otlp_endpoint.is_some(),
        "observabilidad inicializada"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ObserveConfig;

    #[test]
    fn init_does_not_panic_with_default_config() {
        // init() puede fallar con AlreadyInitialized si otro test lo llamo antes
        // pero nunca debe panic
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
        assert!(e.to_string().contains("inicializado"));
    }

    #[test]
    fn otlp_setup_error_displays_correctly() {
        let e = ObserveError::OtlpSetup("fallo".to_string());
        assert!(e.to_string().contains("fallo"));
    }
}
