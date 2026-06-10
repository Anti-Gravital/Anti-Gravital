//! Observability configuration read from environment variables.

/// Structured log format.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable format for local development.
    #[default]
    Pretty,
    /// Structured JSON for production.
    Json,
}

/// Observability subsystem configuration.
#[derive(Debug, Clone)]
pub struct ObserveConfig {
    /// Reserved OTLP endpoint for exporting traces.
    ///
    /// None keeps OTLP disabled. Some returns an explicit initialization error
    /// until a real exporter is implemented.
    /// Variable: OTEL_EXPORTER_OTLP_ENDPOINT
    pub otlp_endpoint: Option<String>,
    /// Port where /metrics is exposed in Prometheus format.
    /// Variable: PROMETHEUS_PORT (default 9090)
    pub prometheus_port: u16,
    /// Log format written to stdout.
    /// Variable: LOG_FORMAT=pretty|json (default pretty)
    pub log_format: LogFormat,
    /// Enable tokio-console in dev mode (feature dev-console).
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
    /// Reads the configuration from environment variables.
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
        Self {
            otlp_endpoint,
            prometheus_port,
            log_format,
            dev_console,
        }
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
        // Save and restore the environment variable to avoid polluting other tests
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
