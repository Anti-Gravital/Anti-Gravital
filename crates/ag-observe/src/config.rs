//! Observability configuration read from declared environment variables.

use ag_core::{EnvDeclaration, EnvLoader, EnvType, LoadedEnv};

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

/// Declared environment schema for `ag-observe` (RFC-0020 PR 4, audit A2).
///
/// Every key the crate reads is listed once here: name, type, required flag
/// and declared default. This is the single auditable answer to "which
/// variables does this service need". No key is a secret: `AG_PROJECT_ID` is
/// an opaque, bounded label (not a credential), and the loader never prints a
/// value in an error even for non-secret declarations, so a malformed or
/// out-of-range label is never leaked through diagnostics (S-1/S-8).
///
/// All keys are declared as [`EnvType::String`] on purpose, matching the
/// proven `ag-storage` migration. The previous `from_env` used
/// `var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(DEFAULT)` *per field*:
/// an unset or malformed value fell back independently for that field.
/// [`EnvLoader`] validates the whole set at once and rejects the load if one
/// value fails its declared type, so a typed declaration (`Int`/`Url`) would
/// silently change that per-field fallback. Parsing therefore stays in
/// [`ObserveConfig::from_loaded_env`], mirroring the original expressions
/// byte for byte; the loader still provides the declared list, per-field
/// defaults, process-environment isolation (only declared keys are read) and
/// value-free errors.
///
/// `SERVICE_NAME` is listed for auditability with its documented `ag-service`
/// default, but its value is still read directly in
/// `crate::layer::build_resource`. That read is `#[cfg(feature = "otlp")]`
/// and happens while building the OpenTelemetry resource inside `init()`, not
/// on the `from_env` config path. Routing it through `ObserveConfig` would
/// require adding a new public field to a public struct, i.e. a public API
/// change forbidden by this PR. The fallback there is left byte-identical to
/// the declared default.
pub(crate) fn observe_env_declarations() -> Vec<EnvDeclaration> {
    vec![
        EnvDeclaration::new("OTEL_EXPORTER_OTLP_ENDPOINT", EnvType::String).required(false),
        EnvDeclaration::new("PROMETHEUS_PORT", EnvType::String).required(false),
        EnvDeclaration::new("LOG_FORMAT", EnvType::String).required(false),
        EnvDeclaration::new("AG_DEV_CONSOLE", EnvType::String).required(false),
        EnvDeclaration::new("AG_PROJECT_ID", EnvType::String).required(false),
        EnvDeclaration::new("SERVICE_NAME", EnvType::String)
            .required(false)
            .with_default("ag-service"),
    ]
}

/// Reusable loader for the declared observability schema.
pub(crate) fn observe_loader() -> EnvLoader {
    EnvLoader::new(observe_env_declarations())
}

impl ObserveConfig {
    /// Reads the configuration from environment variables.
    /// Undefined values use the defaults from [`ObserveConfig::default`].
    ///
    /// Production path: resolves the declared schema through
    /// [`EnvLoader::load_process`], which reads only declared keys and never
    /// prints a value. A non-UTF-8 process value is the only possible load
    /// error for this all-string schema; it is handled below by replaying the
    /// declared keys leniently so that field keeps its previous fallback.
    pub fn from_env() -> Self {
        let loader = observe_loader();
        let env = match loader.load_process() {
            Ok(env) => env,
            Err(_) => {
                let vars: Vec<(String, String)> = loader
                    .declarations()
                    .iter()
                    .filter_map(|declaration| {
                        std::env::var(declaration.name())
                            .ok()
                            .map(|value| (declaration.name().to_owned(), value))
                    })
                    .collect();
                loader
                    .load_from(vars)
                    .expect("all declarations are optional strings and always resolve")
            }
        };
        Self::from_loaded_env(&env)
    }

    /// Test seam: resolves the same declared schema from an injected in-memory
    /// source instead of the process environment. Using this in tests removes
    /// the `set_var`/`remove_var` global races (audit A1) without a global
    /// mutex. Scoped to tests so the public API is unchanged.
    #[cfg(test)]
    pub(crate) fn from_env_source<I, K, V>(vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let env = observe_loader()
            .load_from(vars)
            .expect("all declarations are optional strings and always resolve");
        Self::from_loaded_env(&env)
    }

    /// Builds a config from a resolved environment. Parsing and fallbacks are
    /// identical to the pre-migration direct reads, field by field.
    fn from_loaded_env(env: &LoadedEnv) -> Self {
        let otlp_endpoint = env.string("OTEL_EXPORTER_OTLP_ENDPOINT").map(str::to_owned);
        let prometheus_port = env
            .string("PROMETHEUS_PORT")
            .and_then(|value| value.parse().ok())
            .unwrap_or(9090);
        let log_format = match env.string("LOG_FORMAT") {
            Some("json") => LogFormat::Json,
            _ => LogFormat::Pretty,
        };
        let dev_console = env
            .string("AG_DEV_CONSOLE")
            .map(|value| value == "true" || value == "1")
            .unwrap_or(false);
        let project_id = env.string("AG_PROJECT_ID").map(str::to_owned);
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
#[path = "config_tests.rs"]
mod tests;
