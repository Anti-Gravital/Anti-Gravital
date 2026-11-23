//! Runtime configuration.
//!
//! Two layers exist:
//!
//! - [`RuntimeConfig`] — the low-level knobs the dispatch loop and worker pools read
//!   (poison guard, poll interval, lease batch, retry policy).
//! - [`WorkersConfig`] — the deployment-level `[workers]` configuration (RFC-0012
//!   §28): mode, backend, shutdown timeout, payload limit and per-queue settings,
//!   loadable from a TOML document and overridable through `AG_WORKERS_*` environment
//!   variables. The CLI (`ag workers run`/`doctor`) reads this layer and derives a
//!   [`RuntimeConfig`] from it.
//!
//! Environment variables are read through the declared schema below (RFC-0020
//! PR 4), not through scattered direct reads: [`workers_env_declarations`] is the
//! single auditable list and [`EnvLoader`] reads only those keys.

use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::Duration;

use ag_core::{EnvDeclaration, EnvLoader, EnvType, LoadedEnv};
use serde::Deserialize;
use thiserror::Error;

use crate::backoff::RetryPolicy;

/// Default poison-guard threshold: a job leased more than this many times without an
/// outcome is dead-lettered before execution (crash-loop protection).
pub const DEFAULT_POISON_GUARD_ATTEMPTS: u32 = 3;

/// Default maximum encoded payload size accepted at enqueue and decode (256 KiB).
pub const DEFAULT_MAX_PAYLOAD_BYTES: usize = 262_144;

/// Default graceful-shutdown budget.
pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration shared by the dispatch loop and worker pools.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// A job whose attempt count exceeds this is dead-lettered before executing. Keep
    /// this at or below the retry policy's `max_attempts`; it is the closed loop that
    /// turns an infinite crash loop into a bounded dead-letter entry under
    /// `panic = "abort"`.
    pub poison_guard_attempts: u32,
    /// How long a worker waits before polling again when a queue is empty.
    pub poll_interval: Duration,
    /// Maximum number of jobs a worker leases per poll.
    pub lease_batch: usize,
    /// Retry/backoff policy applied to retriable failures.
    pub retry_policy: RetryPolicy,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            poison_guard_attempts: DEFAULT_POISON_GUARD_ATTEMPTS,
            poll_interval: Duration::from_millis(100),
            lease_batch: 10,
            retry_policy: RetryPolicy::default(),
        }
    }
}

/// Errors produced while loading [`WorkersConfig`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// The TOML document could not be parsed.
    #[error("invalid workers config TOML: {0}")]
    Toml(String),

    /// A duration string could not be parsed (e.g. `"30s"`, `"5m"`, `"250ms"`).
    #[error("invalid duration {value:?}: {reason}")]
    Duration {
        /// The offending value.
        value: String,
        /// Why it could not be parsed.
        reason: String,
    },

    /// An enum-valued field held an unknown variant.
    #[error("invalid {field} value {value:?} (expected one of: {expected})")]
    Variant {
        /// The field name.
        field: &'static str,
        /// The offending value.
        value: String,
        /// The accepted variants.
        expected: &'static str,
    },
}

/// Where workers execute relative to the application process (RFC-0012 §17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkerMode {
    /// Workers run inside the app process. Default for development.
    #[default]
    Embedded,
    /// Workers run as a separate process from the same binary/config.
    Standalone,
    /// Multiple worker processes lease from the durable backend.
    Distributed,
    /// Enqueue-only: links the enqueue API, hosts no worker runtime (edge/serverless).
    Producer,
}

impl FromStr for WorkerMode {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "embedded" => Ok(Self::Embedded),
            "standalone" => Ok(Self::Standalone),
            "distributed" => Ok(Self::Distributed),
            "producer" => Ok(Self::Producer),
            other => Err(ConfigError::Variant {
                field: "mode",
                value: other.to_owned(),
                expected: "embedded | standalone | distributed | producer",
            }),
        }
    }
}

/// Which queue backend the deployment uses (RFC-0012 §11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendKind {
    /// In-memory backend (development, tests, single process). Default.
    #[default]
    Memory,
    /// Durable PostgreSQL backend through `ag-data`.
    Postgres,
}

impl FromStr for BackendKind {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "memory" => Ok(Self::Memory),
            "postgres" | "postgresql" | "pg" => Ok(Self::Postgres),
            other => Err(ConfigError::Variant {
                field: "backend",
                value: other.to_owned(),
                expected: "memory | postgres",
            }),
        }
    }
}

/// Per-queue worker settings (RFC-0012 §28 `[workers.queue.<name>]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueConfig {
    /// Minimum (and, for static pools, fixed) worker concurrency.
    pub min_workers: usize,
    /// Maximum worker concurrency for dynamic pools.
    pub max_workers: usize,
    /// Admission cap: enqueues beyond this depth are rejected.
    pub max_depth: usize,
    /// How long a lease is held before the reaper may reclaim it.
    pub lease_timeout: Duration,
    /// How often a long-running job refreshes its lease.
    pub heartbeat_interval: Duration,
    /// Whether this queue runs CPU-bound jobs on the blocking pool (`kind = "cpu"`).
    pub cpu_bound: bool,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            min_workers: 1,
            max_workers: 4,
            max_depth: 10_000,
            lease_timeout: Duration::from_secs(60),
            heartbeat_interval: Duration::from_secs(15),
            cpu_bound: false,
        }
    }
}

/// Deployment-level workers configuration (RFC-0012 §28).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkersConfig {
    /// Whether the workers subsystem is enabled.
    pub enabled: bool,
    /// Execution placement / capability.
    pub mode: WorkerMode,
    /// Queue backend.
    pub backend: BackendKind,
    /// Graceful-shutdown budget.
    pub shutdown_timeout: Duration,
    /// Maximum encoded payload size accepted at enqueue and decode.
    pub max_payload_bytes: usize,
    /// Poison-guard threshold (see [`RuntimeConfig::poison_guard_attempts`]).
    pub poison_guard_attempts: u32,
    /// Per-queue settings, keyed by queue name.
    pub queues: BTreeMap<String, QueueConfig>,
}

impl Default for WorkersConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: WorkerMode::default(),
            backend: BackendKind::default(),
            shutdown_timeout: DEFAULT_SHUTDOWN_TIMEOUT,
            max_payload_bytes: DEFAULT_MAX_PAYLOAD_BYTES,
            poison_guard_attempts: DEFAULT_POISON_GUARD_ATTEMPTS,
            queues: BTreeMap::new(),
        }
    }
}

/// Declared environment schema for `ag-workers` (RFC-0020 PR 4, audit A2).
///
/// Every fixed key the crate reads is listed once here: name, type and the
/// fact that it is optional. This is the single auditable answer to "which
/// variables does this service need"; [`EnvLoader`] reads only these keys.
/// Per-queue `AG_WORKERS_QUEUE_<NAME>_MIN` / `_MAX` keys are derived from the
/// queue names present in a config by [`workers_loader`].
///
/// No key is a secret: they are placement knobs, ports-equivalent sizes and
/// timeouts, and the loader never prints a value in an error even for a
/// non-secret declaration (S-1/S-8).
///
/// All keys are declared as [`EnvType::String`] on purpose, matching the
/// proven `ag-storage` and `ag-observe` migrations. The previous code applied
/// `std::env::var(k)` and parsed *per field*: an unset or malformed value
/// either kept the TOML/default value (`env_parse`) or propagated a typed
/// parse error (`mode`, `backend`, `shutdown timeout`). [`EnvLoader`] validates
/// the whole set at once and rejects the load if one value fails its declared
/// type, so a typed declaration (`Int`/`Url`/`Duration`) would silently change
/// that per-field fallback. Parsing therefore stays in
/// [`WorkersConfig::apply_from_loaded_env`], mirroring the original expressions
/// byte for byte; the loader still provides the declared list,
/// process-environment isolation (only declared keys are read) and value-free
/// errors.
pub(crate) fn workers_env_declarations() -> Vec<EnvDeclaration> {
    vec![
        EnvDeclaration::new("AG_WORKERS_ENABLED", EnvType::String).required(false),
        EnvDeclaration::new("AG_WORKERS_MODE", EnvType::String).required(false),
        EnvDeclaration::new("AG_WORKERS_BACKEND", EnvType::String).required(false),
        EnvDeclaration::new("AG_WORKERS_SHUTDOWN_TIMEOUT", EnvType::String).required(false),
        EnvDeclaration::new("AG_WORKERS_MAX_PAYLOAD_BYTES", EnvType::String).required(false),
        EnvDeclaration::new("AG_WORKERS_POISON_GUARD_ATTEMPTS", EnvType::String).required(false),
    ]
}

/// Builds the loader for the declared schema plus the per-queue keys that
/// correspond to the queues present in a config.
fn workers_loader<'a>(queue_names: impl Iterator<Item = &'a str>) -> EnvLoader {
    let mut declarations = workers_env_declarations();
    for name in queue_names {
        let upper = name.to_ascii_uppercase();
        declarations.push(
            EnvDeclaration::new(format!("AG_WORKERS_QUEUE_{upper}_MIN"), EnvType::String)
                .required(false),
        );
        declarations.push(
            EnvDeclaration::new(format!("AG_WORKERS_QUEUE_{upper}_MAX"), EnvType::String)
                .required(false),
        );
    }
    EnvLoader::new(declarations)
}

/// Resolves the declared schema from the process environment.
///
/// All declarations are optional strings with no default, so the only possible
/// load error is a non-UTF-8 process value. The old `std::env::var(k).ok()`
/// skipped such a value and kept that field's TOML/default fallback; replaying
/// the declared keys leniently preserves that byte-identical behavior.
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

impl WorkersConfig {
    /// Parses a `[workers]` section from a TOML document.
    ///
    /// A document without a `[workers]` table yields [`WorkersConfig::default`].
    pub fn from_toml_str(toml_src: &str) -> Result<Self, ConfigError> {
        let raw: RawDocument =
            toml::from_str(toml_src).map_err(|e| ConfigError::Toml(e.to_string()))?;
        let Some(w) = raw.workers else {
            return Ok(Self::default());
        };
        let default = Self::default();

        let mut queues = BTreeMap::new();
        for (name, rq) in w.queue.unwrap_or_default() {
            let dq = QueueConfig::default();
            queues.insert(
                name,
                QueueConfig {
                    min_workers: rq.min_workers.unwrap_or(dq.min_workers),
                    max_workers: rq.max_workers.unwrap_or(dq.max_workers),
                    max_depth: rq.max_depth.unwrap_or(dq.max_depth),
                    lease_timeout: opt_duration(rq.lease_timeout, dq.lease_timeout)?,
                    heartbeat_interval: opt_duration(rq.heartbeat_interval, dq.heartbeat_interval)?,
                    cpu_bound: rq
                        .kind
                        .map(|k| k.eq_ignore_ascii_case("cpu"))
                        .unwrap_or(dq.cpu_bound),
                },
            );
        }

        Ok(Self {
            enabled: w.enabled.unwrap_or(default.enabled),
            mode: match w.mode {
                Some(m) => m.parse()?,
                None => default.mode,
            },
            backend: match w.backend {
                Some(b) => b.parse()?,
                None => default.backend,
            },
            shutdown_timeout: opt_duration(w.shutdown_timeout, default.shutdown_timeout)?,
            max_payload_bytes: w.max_payload_bytes.unwrap_or(default.max_payload_bytes),
            poison_guard_attempts: w
                .poison_guard_attempts
                .unwrap_or(default.poison_guard_attempts),
            queues,
        })
    }

    /// Applies `AG_WORKERS_*` environment overrides on top of `self`.
    ///
    /// Recognized: `AG_WORKERS_ENABLED`, `AG_WORKERS_MODE`, `AG_WORKERS_BACKEND`,
    /// `AG_WORKERS_SHUTDOWN_TIMEOUT`, `AG_WORKERS_MAX_PAYLOAD_BYTES`,
    /// `AG_WORKERS_POISON_GUARD_ATTEMPTS`, and per-queue
    /// `AG_WORKERS_QUEUE_<NAME>_MIN` / `_MAX` (name upper-cased).
    pub fn apply_env(self) -> Result<Self, ConfigError> {
        let loader = workers_loader(self.queues.keys().map(String::as_str));
        let env = resolve_process_env(&loader);
        self.apply_from_loaded_env(&env)
    }

    /// Test seam: applies the same declared schema from an injected in-memory
    /// source instead of the process environment. Using this in tests removes
    /// the `set_var`/`remove_var` global races (audit A1) without a global
    /// mutex. Scoped to tests so the public API is unchanged.
    #[cfg(test)]
    pub(crate) fn apply_env_source<I, K, V>(self, vars: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let loader = workers_loader(self.queues.keys().map(String::as_str));
        let env = loader
            .load_from(vars)
            .expect("all declarations are optional strings and always resolve");
        self.apply_from_loaded_env(&env)
    }

    /// Applies overrides from a resolved environment. Parsing and fallbacks are
    /// identical to the pre-migration direct reads, field by field.
    fn apply_from_loaded_env(mut self, env: &LoadedEnv) -> Result<Self, ConfigError> {
        if let Some(v) = loaded_bool(env, "AG_WORKERS_ENABLED") {
            self.enabled = v;
        }
        if let Some(v) = env.string("AG_WORKERS_MODE") {
            self.mode = v.parse()?;
        }
        if let Some(v) = env.string("AG_WORKERS_BACKEND") {
            self.backend = v.parse()?;
        }
        if let Some(v) = env.string("AG_WORKERS_SHUTDOWN_TIMEOUT") {
            self.shutdown_timeout = parse_duration(v)?;
        }
        if let Some(v) = loaded_parse::<usize>(env, "AG_WORKERS_MAX_PAYLOAD_BYTES") {
            self.max_payload_bytes = v;
        }
        if let Some(v) = loaded_parse::<u32>(env, "AG_WORKERS_POISON_GUARD_ATTEMPTS") {
            self.poison_guard_attempts = v;
        }
        // Per-queue MIN/MAX overrides for queues already declared in the config.
        let names: Vec<String> = self.queues.keys().cloned().collect();
        for name in names {
            let upper = name.to_ascii_uppercase();
            if let Some(v) = loaded_parse::<usize>(env, &format!("AG_WORKERS_QUEUE_{upper}_MIN")) {
                self.queues.get_mut(&name).unwrap().min_workers = v;
            }
            if let Some(v) = loaded_parse::<usize>(env, &format!("AG_WORKERS_QUEUE_{upper}_MAX")) {
                self.queues.get_mut(&name).unwrap().max_workers = v;
            }
        }
        Ok(self)
    }

    /// Loads from a TOML document then applies `AG_WORKERS_*` environment overrides
    /// (environment wins over TOML wins over defaults).
    pub fn load(toml_src: &str) -> Result<Self, ConfigError> {
        Self::from_toml_str(toml_src)?.apply_env()
    }

    /// Derives the low-level [`RuntimeConfig`] for the dispatch loop.
    pub fn runtime_config(&self) -> RuntimeConfig {
        RuntimeConfig {
            poison_guard_attempts: self.poison_guard_attempts,
            ..RuntimeConfig::default()
        }
    }
}

/// Parses a human duration: a bare integer is seconds; suffixes `ms`, `s`, `m`, `h`,
/// `d`, `w` are supported (e.g. `"250ms"`, `"30s"`, `"5m"`, `"1h"`, `"30d"`, `"2w"`).
pub fn parse_duration(value: &str) -> Result<Duration, ConfigError> {
    let v = value.trim();
    let err = |reason: &str| ConfigError::Duration {
        value: value.to_owned(),
        reason: reason.to_owned(),
    };
    let (num, unit) = if let Some(n) = v.strip_suffix("ms") {
        (n, "ms")
    } else if let Some(n) = v.strip_suffix('s') {
        (n, "s")
    } else if let Some(n) = v.strip_suffix('m') {
        (n, "m")
    } else if let Some(n) = v.strip_suffix('h') {
        (n, "h")
    } else if let Some(n) = v.strip_suffix('d') {
        (n, "d")
    } else if let Some(n) = v.strip_suffix('w') {
        (n, "w")
    } else {
        (v, "s")
    };
    let n: u64 = num
        .trim()
        .parse()
        .map_err(|_| err("expected a non-negative integer magnitude"))?;
    Ok(match unit {
        "ms" => Duration::from_millis(n),
        "s" => Duration::from_secs(n),
        "m" => Duration::from_secs(n * 60),
        "h" => Duration::from_secs(n * 3600),
        "d" => Duration::from_secs(n * 86_400),
        "w" => Duration::from_secs(n * 604_800),
        _ => unreachable!(),
    })
}

fn opt_duration(value: Option<String>, default: Duration) -> Result<Duration, ConfigError> {
    match value {
        Some(s) => parse_duration(&s),
        None => Ok(default),
    }
}

/// Mirrors the old `env_bool`, sourcing the raw (untrimmed) value from the
/// declared loader instead of a direct process read.
fn loaded_bool(env: &LoadedEnv, key: &str) -> Option<bool> {
    env.string(key)
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
}

/// Mirrors the old `env_parse`, sourcing the raw (untrimmed) value from the
/// declared loader instead of a direct process read.
fn loaded_parse<T: FromStr>(env: &LoadedEnv, key: &str) -> Option<T> {
    env.string(key).and_then(|v| v.trim().parse().ok())
}

// ---------------------------------------------------------------------------
// Raw TOML shapes (string durations / enums, all optional).
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawDocument {
    workers: Option<RawWorkers>,
}

#[derive(Debug, Deserialize)]
struct RawWorkers {
    enabled: Option<bool>,
    mode: Option<String>,
    backend: Option<String>,
    shutdown_timeout: Option<String>,
    max_payload_bytes: Option<usize>,
    poison_guard_attempts: Option<u32>,
    queue: Option<BTreeMap<String, RawQueue>>,
}

#[derive(Debug, Default, Deserialize)]
struct RawQueue {
    min_workers: Option<usize>,
    max_workers: Option<usize>,
    max_depth: Option<usize>,
    lease_timeout: Option<String>,
    heartbeat_interval: Option<String>,
    kind: Option<String>,
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
