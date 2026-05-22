//! Configuracion de observabilidad leida desde variables de entorno.

/// Formato de los logs estructurados.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Formato legible para desarrollo local.
    #[default]
    Pretty,
    /// JSON estructurado para produccion.
    Json,
}

/// Configuracion del subsistema de observabilidad.
#[derive(Debug, Clone)]
pub struct ObserveConfig {
    /// Endpoint OTLP para exportar trazas. None deshabilita el exporter.
    /// Variable: OTEL_EXPORTER_OTLP_ENDPOINT
    pub otlp_endpoint: Option<String>,
    /// Puerto donde se expone /metrics en formato Prometheus.
    /// Variable: PROMETHEUS_PORT (default 9090)
    pub prometheus_port: u16,
    /// Formato de los logs a stdout.
    /// Variable: LOG_FORMAT=pretty|json (default pretty)
    pub log_format: LogFormat,
    /// Habilitar tokio-console en modo dev (feature dev-console).
    /// Variable: AG_DEV_CONSOLE=true|false (default false)
    pub dev_console: bool,
}

impl Default for ObserveConfig {
    fn default() -> Self {
        Self {
            otlp_endpoint: None,
            prometheus_port: 9090,
            log_format: LogFormat::Pretty,
            dev_console: false,
        }
    }
}

impl ObserveConfig {
    /// Lee la configuracion desde variables de entorno.
    pub fn from_env() -> Self {
        let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
        let prometheus_port = std::env::var("PROMETHEUS_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(9090);
        let log_format = match std::env::var("LOG_FORMAT").as_deref() {
            Ok("json") => LogFormat::Json,
            _ => LogFormat::Pretty,
        };
        let dev_console = std::env::var("AG_DEV_CONSOLE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        Self { otlp_endpoint, prometheus_port, log_format, dev_console }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_are_sensible() {
        let cfg = ObserveConfig::default();
        assert!(cfg.otlp_endpoint.is_none());
        assert_eq!(cfg.prometheus_port, 9090);
        assert!(!cfg.dev_console);
    }

    #[test]
    fn config_reads_log_format_json() {
        // Guarda y restaura la variable de entorno para no contaminar otros tests
        let prev = std::env::var("LOG_FORMAT").ok();
        std::env::set_var("LOG_FORMAT", "json");
        let cfg = ObserveConfig::from_env();
        assert!(matches!(cfg.log_format, LogFormat::Json));
        match prev {
            Some(v) => std::env::set_var("LOG_FORMAT", v),
            None => std::env::remove_var("LOG_FORMAT"),
        }
    }
}
