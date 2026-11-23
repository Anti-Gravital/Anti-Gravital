//! Real-time subsystem configuration.
//!
//! Environment variables are read through the declared schema below (RFC-0020
//! PR 4), not through scattered direct reads: `realtime_env_declarations` is the
//! single auditable list and [`EnvLoader`] reads only those keys.

use ag_core::{EnvDeclaration, EnvLoader, EnvType, LoadedEnv};

/// Connection mode to the NATS event bus.
#[derive(Debug, Clone, Default)]
pub enum NatsMode {
    /// In-memory bus via tokio::sync::broadcast (no external NATS server).
    #[default]
    InProcess,
    /// Connection to an external NATS server.
    External,
}

/// Real-time subsystem configuration.
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    /// Event bus mode. Default: InProcess.
    pub nats_mode: NatsMode,
    /// External NATS server URL (only relevant in External mode).
    /// Environment variable: `NATS_URL`.
    pub nats_url: String,
    /// Capacity of the internal broadcast channel (InProcess mode only).
    pub broadcast_capacity: usize,
    // TLS
    /// If `true`, enables TLS with the system CA.
    /// Environment variable: `NATS_TLS`.
    pub nats_tls: bool,
    /// Path to a custom CA (level 2: custom CA).
    /// Environment variable: `NATS_TLS_CA`.
    pub nats_tls_ca_path: Option<String>,
    /// Path to the client certificate (level 3: mTLS).
    /// Environment variable: `NATS_TLS_CERT`.
    pub nats_tls_cert_path: Option<String>,
    /// Path to the client private key (level 3: mTLS).
    /// Environment variable: `NATS_TLS_KEY`.
    pub nats_tls_key_path: Option<String>,
    // JetStream
    /// If `true`, publishes and consumes via JetStream with ACK.
    /// Environment variable: `NATS_JETSTREAM`.
    pub jetstream_enabled: bool,
    /// JetStream stream name. Default: `AG_EVENTS`.
    /// Environment variable: `NATS_JS_STREAM`.
    pub jetstream_stream_name: String,
    /// Stream message limit. Default: 1_000_000.
    /// Environment variable: `NATS_JS_MAX_MSGS`.
    pub jetstream_max_msgs: i64,
    /// Stream byte limit. Default: 1 GiB.
    /// Environment variable: `NATS_JS_MAX_BYTES`.
    pub jetstream_max_bytes: i64,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            nats_mode: NatsMode::InProcess,
            nats_url: "nats://localhost:4222".to_string(),
            broadcast_capacity: 1024,
            nats_tls: false,
            nats_tls_ca_path: None,
            nats_tls_cert_path: None,
            nats_tls_key_path: None,
            jetstream_enabled: false,
            jetstream_stream_name: "AG_EVENTS".to_string(),
            jetstream_max_msgs: 1_000_000,
            jetstream_max_bytes: 1_073_741_824,
        }
    }
}

/// Declared environment schema for `ag-realtime` (RFC-0020 PR 4, audit A2).
///
/// Every key `config.rs` reads is listed once here: name, type and, where the
/// raw variable has one, its declared default. This is the single auditable
/// answer to "which variables does this service need"; [`EnvLoader`] reads only
/// these keys.
///
/// None of these keys is a secret. `NATS_TLS_CERT` and `NATS_TLS_KEY` hold
/// *paths* to a client certificate and private key, not the credential bytes,
/// so they are not marked secret; the loader never prints a value in an error
/// even for a non-secret declaration (S-1/S-8).
///
/// All keys are declared as [`EnvType::String`] on purpose, matching the proven
/// `ag-storage`, `ag-observe` and `ag-workers` migrations. The previous
/// `from_env` used expressions such as
/// `var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(DEFAULT)` *per field*:
/// an unset or malformed value fell back independently of every other field.
/// [`EnvLoader`] validates the whole set at once and rejects the load if a
/// single value fails its declared type, so a typed declaration (`Int`, `Bool`,
/// `Path`, ...) would silently change that per-field fallback (an empty TLS
/// path, for example, would become a load error instead of `Some("")`). Parsing
/// therefore stays in `RealtimeConfig::from_loaded_env`, mirroring the original
/// expressions byte for byte; the loader still provides the declared list,
/// per-field defaults for the string-valued keys, process-environment isolation
/// (only declared keys are read) and value-free errors.
pub(crate) fn realtime_env_declarations() -> Vec<EnvDeclaration> {
    vec![
        EnvDeclaration::new("NATS_MODE", EnvType::String).required(false),
        EnvDeclaration::new("NATS_URL", EnvType::String)
            .required(false)
            .with_default("nats://localhost:4222"),
        EnvDeclaration::new("RT_BROADCAST_CAPACITY", EnvType::String).required(false),
        EnvDeclaration::new("NATS_TLS", EnvType::String).required(false),
        EnvDeclaration::new("NATS_TLS_CA", EnvType::String).required(false),
        EnvDeclaration::new("NATS_TLS_CERT", EnvType::String).required(false),
        EnvDeclaration::new("NATS_TLS_KEY", EnvType::String).required(false),
        EnvDeclaration::new("NATS_JETSTREAM", EnvType::String).required(false),
        EnvDeclaration::new("NATS_JS_STREAM", EnvType::String)
            .required(false)
            .with_default("AG_EVENTS"),
        EnvDeclaration::new("NATS_JS_MAX_MSGS", EnvType::String).required(false),
        EnvDeclaration::new("NATS_JS_MAX_BYTES", EnvType::String).required(false),
    ]
}

/// Reusable loader for the declared realtime schema.
pub(crate) fn realtime_loader() -> EnvLoader {
    EnvLoader::new(realtime_env_declarations())
}

/// Resolves the declared schema from the process environment.
///
/// All declarations are optional strings with valid defaults, so the only
/// possible load error is a non-UTF-8 process value. The old
/// `std::env::var(k).ok()` skipped such a value and kept that field's previous
/// fallback; replaying the declared keys leniently preserves that byte-identical
/// behavior.
fn resolve_process_env(loader: &EnvLoader) -> LoadedEnv {
    match loader.load_process() {
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
    }
}

impl RealtimeConfig {
    /// Reads the configuration from environment variables.
    ///
    /// Recognized variables:
    /// - `NATS_MODE`: `"external"` enables External mode; any other value uses InProcess.
    /// - `NATS_URL`: NATS server URL (default: `nats://localhost:4222`).
    /// - `RT_BROADCAST_CAPACITY`: broadcast channel capacity (default: 1024).
    /// - `NATS_TLS`: `"true"` enables TLS with the system CA.
    /// - `NATS_TLS_CA`: path to a custom CA.
    /// - `NATS_TLS_CERT`: path to the client certificate (mTLS).
    /// - `NATS_TLS_KEY`: path to the client private key (mTLS).
    /// - `NATS_JETSTREAM`: `"true"` enables JetStream with ACK.
    /// - `NATS_JS_STREAM`: JetStream stream name (default: `AG_EVENTS`).
    /// - `NATS_JS_MAX_MSGS`: message limit (default: 1_000_000).
    /// - `NATS_JS_MAX_BYTES`: byte limit (default: 1_073_741_824).
    ///
    /// Production path: resolves the declared schema through
    /// [`EnvLoader::load_process`], which reads only declared keys and never
    /// prints a value. A non-UTF-8 process value is the only possible load
    /// error for this all-string schema; it is handled below by replaying the
    /// declared keys leniently so that field keeps its previous fallback.
    pub fn from_env() -> Self {
        Self::from_loaded_env(&resolve_process_env(&realtime_loader()))
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
        let env = realtime_loader()
            .load_from(vars)
            .expect("all declarations are optional strings and always resolve");
        Self::from_loaded_env(&env)
    }

    /// Builds a config from a resolved environment. Parsing and fallbacks are
    /// identical to the pre-migration direct reads, field by field.
    fn from_loaded_env(env: &LoadedEnv) -> Self {
        let nats_mode = match env.string("NATS_MODE") {
            Some("external") => NatsMode::External,
            _ => NatsMode::InProcess,
        };
        Self {
            nats_mode,
            nats_url: env
                .string("NATS_URL")
                .map(str::to_owned)
                .unwrap_or_else(|| "nats://localhost:4222".to_string()),
            broadcast_capacity: env
                .string("RT_BROADCAST_CAPACITY")
                .and_then(|value| value.parse().ok())
                .unwrap_or(1024),
            nats_tls: env
                .string("NATS_TLS")
                .map(|value| value.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            nats_tls_ca_path: env.string("NATS_TLS_CA").map(str::to_owned),
            nats_tls_cert_path: env.string("NATS_TLS_CERT").map(str::to_owned),
            nats_tls_key_path: env.string("NATS_TLS_KEY").map(str::to_owned),
            jetstream_enabled: env
                .string("NATS_JETSTREAM")
                .map(|value| value.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            jetstream_stream_name: env
                .string("NATS_JS_STREAM")
                .map(str::to_owned)
                .unwrap_or_else(|| "AG_EVENTS".to_string()),
            jetstream_max_msgs: env
                .string("NATS_JS_MAX_MSGS")
                .and_then(|value| value.parse().ok())
                .unwrap_or(1_000_000),
            jetstream_max_bytes: env
                .string("NATS_JS_MAX_BYTES")
                .and_then(|value| value.parse().ok())
                .unwrap_or(1_073_741_824),
        }
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
