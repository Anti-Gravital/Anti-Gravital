//! Multilevel cache configuration.
//!
//! Environment variables are read through the declared schema below (RFC-0020
//! PR 4), not through scattered direct reads: `cache_env_declarations` is the
//! single auditable list and [`EnvLoader`] reads only those keys.

use ag_core::{EnvDeclaration, EnvLoader, EnvType, LoadedEnv};

/// Configuration for the L1 cache (moka, in memory) and optional native RESP2 server.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in L1. Default: 10_000.
    pub l1_max_capacity: u64,
    /// Default TTL for L1 entries in seconds. Default: 300.
    pub l1_ttl_secs: u64,
    /// Redis connection URL for L2. None disables L2.
    /// Environment variable: REDIS_URL
    pub redis_url: Option<String>,
    /// Number of connections in the Redis pool. Default: 10.
    pub redis_pool_size: u32,
    /// Enable the native RESP2 server. Requires feature `native-server`. Default: false.
    pub native_server_enabled: bool,
    /// TCP port for the native RESP2 server. Default: 6379.
    pub native_server_port: u16,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_max_capacity: 10_000,
            l1_ttl_secs: 300,
            redis_url: None,
            redis_pool_size: 10,
            native_server_enabled: false,
            native_server_port: 6379,
        }
    }
}

/// Declared environment schema for `ag-cache` (RFC-0020 PR 4, audit A2).
///
/// Every key the crate reads is listed once here: name, type and secret
/// classification. This is the single auditable answer to "which variables
/// does this service need"; [`EnvLoader`] reads only these keys.
///
/// `REDIS_URL` is declared `secret(true)`: a Redis URL can embed
/// `user:pass@host` credentials, so its bytes are delivered as
/// `Secret<String>`, whose `Debug`/`Display` redact and which require an
/// explicit `expose()` call. The loader never prints a value in an error, and
/// the declared secret is never left in a `Debug`-printable loader state
/// (S-1/S-8). The public [`CacheConfig::redis_url`] keeps its original
/// `Option<String>` type (no public API change); the value is copied out only
/// in [`CacheConfig::from_loaded_env`] via `expose()`. The other five keys are
/// operational knobs (sizes, a flag and a port) and are not secrets.
///
/// All keys are declared as [`EnvType::String`] on purpose. The previous
/// `from_env` used `std::env::var(k)` with a *per-field* fallback: an unset,
/// non-UTF-8 or (where parsed) malformed value fell back independently for
/// that field. [`EnvLoader`] validates the whole set at once and rejects the
/// load if a single value fails its declared type, so a typed declaration
/// would silently change that per-field behavior. Parsing therefore stays in
/// [`CacheConfig::from_loaded_env`], mirroring the original expressions byte
/// for byte; the loader still provides the declared list, secret redaction and
/// process-environment isolation (only declared keys are read).
pub(crate) fn cache_env_declarations() -> Vec<EnvDeclaration> {
    vec![
        EnvDeclaration::new("CACHE_L1_MAX_CAPACITY", EnvType::String).required(false),
        EnvDeclaration::new("CACHE_L1_TTL_SECS", EnvType::String).required(false),
        EnvDeclaration::new("REDIS_URL", EnvType::String)
            .required(false)
            .secret(true)
            .description_text("Redis connection URL for L2 (may embed credentials)"),
        EnvDeclaration::new("REDIS_POOL_SIZE", EnvType::String).required(false),
        EnvDeclaration::new("CACHE_NATIVE_SERVER", EnvType::String).required(false),
        EnvDeclaration::new("CACHE_NATIVE_PORT", EnvType::String).required(false),
    ]
}

/// Reusable loader for the declared cache schema.
pub(crate) fn cache_loader() -> EnvLoader {
    EnvLoader::new(cache_env_declarations())
}

impl CacheConfig {
    /// Reads the configuration from environment variables.
    ///
    /// Production path: resolves the declared schema through
    /// [`EnvLoader::load_process`], which reads only declared keys and never
    /// prints a value. A non-UTF-8 process value is the only possible load
    /// error for this all-string schema; it is handled below by replaying the
    /// declared keys leniently so that field keeps its previous fallback.
    pub fn from_env() -> Self {
        let loader = cache_loader();
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
        let env = cache_loader()
            .load_from(vars)
            .expect("all declarations are optional strings and always resolve");
        Self::from_loaded_env(&env)
    }

    /// Builds a config from a resolved environment. Parsing and fallbacks are
    /// identical to the pre-migration direct reads, field by field.
    fn from_loaded_env(env: &LoadedEnv) -> Self {
        Self {
            l1_max_capacity: env
                .string("CACHE_L1_MAX_CAPACITY")
                .and_then(|v| v.parse().ok())
                .unwrap_or(10_000),
            l1_ttl_secs: env
                .string("CACHE_L1_TTL_SECS")
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            redis_url: env
                .secret("REDIS_URL")
                .map(|value| value.expose().to_owned()),
            redis_pool_size: env
                .string("REDIS_POOL_SIZE")
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            native_server_enabled: env
                .string("CACHE_NATIVE_SERVER")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
            native_server_port: env
                .string("CACHE_NATIVE_PORT")
                .and_then(|v| v.parse().ok())
                .unwrap_or(6379),
        }
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
