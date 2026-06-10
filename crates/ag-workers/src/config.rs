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

use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::Duration;

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
    pub fn apply_env(mut self) -> Result<Self, ConfigError> {
        if let Some(v) = env_bool("AG_WORKERS_ENABLED") {
            self.enabled = v;
        }
        if let Ok(v) = std::env::var("AG_WORKERS_MODE") {
            self.mode = v.parse()?;
        }
        if let Ok(v) = std::env::var("AG_WORKERS_BACKEND") {
            self.backend = v.parse()?;
        }
        if let Ok(v) = std::env::var("AG_WORKERS_SHUTDOWN_TIMEOUT") {
            self.shutdown_timeout = parse_duration(&v)?;
        }
        if let Some(v) = env_parse::<usize>("AG_WORKERS_MAX_PAYLOAD_BYTES") {
            self.max_payload_bytes = v;
        }
        if let Some(v) = env_parse::<u32>("AG_WORKERS_POISON_GUARD_ATTEMPTS") {
            self.poison_guard_attempts = v;
        }
        // Per-queue MIN/MAX overrides for queues already declared in the config.
        let names: Vec<String> = self.queues.keys().cloned().collect();
        for name in names {
            let upper = name.to_ascii_uppercase();
            if let Some(v) = env_parse::<usize>(&format!("AG_WORKERS_QUEUE_{upper}_MIN")) {
                self.queues.get_mut(&name).unwrap().min_workers = v;
            }
            if let Some(v) = env_parse::<usize>(&format!("AG_WORKERS_QUEUE_{upper}_MAX")) {
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

fn env_bool(key: &str) -> Option<bool> {
    std::env::var(key)
        .ok()
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
}

fn env_parse<T: FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok().and_then(|v| v.trim().parse().ok())
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
mod tests {
    use super::*;

    #[test]
    fn default_workers_config_is_embedded_memory() {
        let cfg = WorkersConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.mode, WorkerMode::Embedded);
        assert_eq!(cfg.backend, BackendKind::Memory);
        assert_eq!(cfg.shutdown_timeout, Duration::from_secs(30));
        assert_eq!(cfg.max_payload_bytes, 262_144);
        assert!(cfg.queues.is_empty());
    }

    #[test]
    fn empty_toml_yields_default() {
        let cfg = WorkersConfig::from_toml_str("# nothing here\n").unwrap();
        assert_eq!(cfg, WorkersConfig::default());
    }

    #[test]
    fn full_toml_parses_all_fields() {
        let src = r#"
            [workers]
            enabled = true
            mode = "distributed"
            backend = "postgres"
            shutdown_timeout = "45s"
            max_payload_bytes = 131072
            poison_guard_attempts = 5

            [workers.queue.mail]
            min_workers = 2
            max_workers = 8
            max_depth = 5000
            lease_timeout = "90s"
            heartbeat_interval = "10s"

            [workers.queue.media]
            kind = "cpu"
            max_workers = 4
        "#;
        let cfg = WorkersConfig::from_toml_str(src).unwrap();
        assert_eq!(cfg.mode, WorkerMode::Distributed);
        assert_eq!(cfg.backend, BackendKind::Postgres);
        assert_eq!(cfg.shutdown_timeout, Duration::from_secs(45));
        assert_eq!(cfg.max_payload_bytes, 131_072);
        assert_eq!(cfg.poison_guard_attempts, 5);

        let mail = &cfg.queues["mail"];
        assert_eq!(mail.min_workers, 2);
        assert_eq!(mail.max_workers, 8);
        assert_eq!(mail.lease_timeout, Duration::from_secs(90));
        assert_eq!(mail.heartbeat_interval, Duration::from_secs(10));
        assert!(!mail.cpu_bound);

        let media = &cfg.queues["media"];
        assert!(media.cpu_bound);
        assert_eq!(media.max_workers, 4);
        // Unspecified fields fall back to defaults.
        assert_eq!(media.max_depth, QueueConfig::default().max_depth);
    }

    #[test]
    fn unknown_mode_is_rejected() {
        let src = "[workers]\nmode = \"galactic\"\n";
        assert!(matches!(
            WorkersConfig::from_toml_str(src),
            Err(ConfigError::Variant { field: "mode", .. })
        ));
    }

    #[test]
    fn parse_duration_units() {
        assert_eq!(parse_duration("250ms").unwrap(), Duration::from_millis(250));
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("12").unwrap(), Duration::from_secs(12));
        assert!(parse_duration("soon").is_err());
    }

    #[test]
    fn runtime_config_carries_poison_guard() {
        let cfg = WorkersConfig {
            poison_guard_attempts: 7,
            ..WorkersConfig::default()
        };
        assert_eq!(cfg.runtime_config().poison_guard_attempts, 7);
    }

    #[test]
    fn env_overrides_win_over_toml() {
        std::env::set_var("AG_WORKERS_MODE", "standalone");
        std::env::set_var("AG_WORKERS_MAX_PAYLOAD_BYTES", "999");
        std::env::set_var("AG_WORKERS_QUEUE_MAIL_MAX", "16");
        let src = "[workers]\nmode = \"embedded\"\nmax_payload_bytes = 1\n[workers.queue.mail]\nmax_workers = 8\n";
        let cfg = WorkersConfig::load(src).unwrap();
        assert_eq!(cfg.mode, WorkerMode::Standalone);
        assert_eq!(cfg.max_payload_bytes, 999);
        assert_eq!(cfg.queues["mail"].max_workers, 16);
        std::env::remove_var("AG_WORKERS_MODE");
        std::env::remove_var("AG_WORKERS_MAX_PAYLOAD_BYTES");
        std::env::remove_var("AG_WORKERS_QUEUE_MAIL_MAX");
    }

    #[test]
    fn worker_mode_from_str_all_variants() {
        assert_eq!(
            "embedded".parse::<WorkerMode>().unwrap(),
            WorkerMode::Embedded
        );
        assert_eq!(
            "  Standalone ".parse::<WorkerMode>().unwrap(),
            WorkerMode::Standalone
        );
        assert_eq!(
            "DISTRIBUTED".parse::<WorkerMode>().unwrap(),
            WorkerMode::Distributed
        );
        assert_eq!(
            "producer".parse::<WorkerMode>().unwrap(),
            WorkerMode::Producer
        );
        assert!("galactic".parse::<WorkerMode>().is_err());
    }

    #[test]
    fn backend_kind_from_str_all_variants() {
        assert_eq!(
            "memory".parse::<BackendKind>().unwrap(),
            BackendKind::Memory
        );
        for pg in ["postgres", "postgresql", "PG"] {
            assert_eq!(pg.parse::<BackendKind>().unwrap(), BackendKind::Postgres);
        }
        assert!("sqlite".parse::<BackendKind>().is_err());
    }

    #[test]
    fn parse_duration_days_and_weeks() {
        assert_eq!(
            parse_duration("2d").unwrap(),
            Duration::from_secs(2 * 86_400)
        );
        assert_eq!(parse_duration("1w").unwrap(), Duration::from_secs(604_800));
        assert!(parse_duration("5x").is_err());
    }
}
