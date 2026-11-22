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
    /// OTLP traces endpoint for exporting traces.
    ///
    /// None keeps OTLP disabled. With the `otlp` Cargo feature, Some builds
    /// an OTLP/HTTP exporter; the value is used as the full traces URL and
    /// must include the signal path (e.g. `http://localhost:4318/v1/traces`).
    /// Without the feature, Some returns an explicit initialization error.
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
    /// Logical project scope attached to metrics and traces.
    ///
    /// This is an opaque, bounded label, not a security boundary and not a
    /// multi-tenant identifier. It lets a shared collector or dashboard group
    /// series by project. Invalid values are rejected at initialization by
    /// `init`, which never prints the value itself (RFC-0038).
    /// Variable: AG_PROJECT_ID
    pub project_id: Option<String>,
}

/// Validates a project scope label.
///
/// The label is bounded to avoid unbounded metric cardinality: 1 to 64
/// characters from `[A-Za-z0-9_.-]`. This is deliberately not a free-form
/// field and does not accept identifiers of people or secrets.
pub fn is_valid_project_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}

impl Default for ObserveConfig {
    fn default() -> Self {
        Self {
            otlp_endpoint: None,
            prometheus_port: 9090,
            log_format: LogFormat::Pretty,
            dev_console: false,
            project_id: None,
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
        let project_id = std::env::var("AG_PROJECT_ID").ok();
        Self {
            otlp_endpoint,
            prometheus_port,
            log_format,
            dev_console,
            project_id,
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
    fn project_id_validation_accepts_bounded_labels() {
        assert!(is_valid_project_id("proj-1"));
        assert!(is_valid_project_id("a.b_c-9"));
        assert!(is_valid_project_id(&"x".repeat(64)));
    }

    #[test]
    fn project_id_validation_rejects_unbounded_or_unsafe_labels() {
        assert!(!is_valid_project_id(""));
        assert!(!is_valid_project_id(&"x".repeat(65)));
        assert!(!is_valid_project_id("with space"));
        assert!(!is_valid_project_id("a/b"));
        assert!(!is_valid_project_id("a=b"));
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
