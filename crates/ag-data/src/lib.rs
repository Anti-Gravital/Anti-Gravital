//! Anti-Gravital data layer.
//!
//! Provides the PostgreSQL connection pool, declarative configuration
//! and the helper to run embedded migrations. In Phase 3+ the DSL
//! generates typed query builders that use this pool as backend.
//!
//! # Minimal example
//!
//! ```no_run
//! use ag_data::{DataConfig, connect};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = DataConfig {
//!     url: "postgresql://localhost/mi_app".into(),
//!     ..DataConfig::default()
//! };
//!
//! let pool = connect(&config).await?;
//! # let _ = pool;
//! # Ok(())
//! # }
//! ```
//!
//! # Migrations
//!
//! Migrations are embedded in the binary with the `sqlx::migrate!` macro
//! and applied at startup with [`run_migrations`]:
//!
//! ```ignore
//! use ag_data::{DataConfig, DbPool, connect, run_migrations};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! # let pool = connect(&DataConfig::default()).await?;
//! // The macro reads the SQL files from the consuming crate, not from ag-data.
//! // The path is relative to the Cargo.toml of the project calling migrate!.
//! static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
//! run_migrations(&pool, &MIGRATOR).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Live integration tests
//!
//! The `#[ignore]` tests exercise [`connect`] and [`run_migrations`] against a
//! real PostgreSQL. They are skipped by default and only run when a database is
//! available:
//!
//! ```sh
//! export DATABASE_URL="postgresql://user:password@localhost:5432/ag_data_test"
//! cargo test -p ag-data -- --ignored
//! ```
//!
//! `DATABASE_URL` is the only required variable; the migration applied by the
//! live test is the fixture under `tests/migrations/`.

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use thiserror::Error;

/// PostgreSQL connection pool shared across handlers.
///
/// It is an alias of [`sqlx::PgPool`], which already implements [`Clone`]
/// at O(1) cost (it shares the internal pool through `Arc`).
pub type DbPool = sqlx::PgPool;

/// Canonical transaction handle (ADR-0013 section 6).
///
/// Wraps a `sqlx` transaction so consumers (e.g. `ag-workers`'
/// `enqueue_in_tx`) depend on this `ag-data` abstraction instead of leaking the
/// raw `sqlx::Transaction` type into every call site. `ag-data` is the project's
/// sanctioned `sqlx` boundary, so it is the one place that exposes an executor
/// (`as_executor`) for query participation; the caller's writes and the queue
/// insert then commit atomically (the transactional-outbox property).
///
/// ```ignore
/// let mut tx = AgTx::begin(&pool).await?;
/// // ... the caller's own writes via tx.as_executor() ...
/// queue.enqueue_in_tx(&mut tx, job).await?;
/// tx.commit().await?; // or tx.rollback().await? to discard both
/// ```
pub struct AgTx {
    inner: sqlx::Transaction<'static, sqlx::Postgres>,
}

impl AgTx {
    /// Begins a transaction on the pool.
    pub async fn begin(pool: &DbPool) -> Result<Self, DataError> {
        Ok(Self {
            inner: pool.begin().await?,
        })
    }

    /// Commits the transaction, persisting every write made through it.
    pub async fn commit(self) -> Result<(), DataError> {
        self.inner.commit().await?;
        Ok(())
    }

    /// Rolls the transaction back, discarding every write made through it.
    pub async fn rollback(self) -> Result<(), DataError> {
        self.inner.rollback().await?;
        Ok(())
    }

    /// Borrows the underlying connection so a statement can run inside this
    /// transaction. `ag-data` is the sanctioned `sqlx` boundary, so this is the
    /// single place that surface is exposed.
    pub fn as_executor(&mut self) -> &mut sqlx::PgConnection {
        &mut self.inner
    }
}

/// Data layer error.
#[derive(Debug, Error)]
pub enum DataError {
    /// Error originating in sqlx (connection, query, protocol).
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Error during migration execution.
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

impl From<DataError> for ag_core::AgError {
    fn from(err: DataError) -> Self {
        ag_core::AgError::Database(err.to_string())
    }
}

/// Connection pool configuration.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DataConfig {
    /// PostgreSQL connection URL.
    ///
    /// Format: `postgresql://user:password@host:port/database`
    pub url: String,

    /// Maximum number of simultaneous connections in the pool.
    #[serde(default = "DataConfig::default_max_connections")]
    pub max_connections: u32,

    /// Maximum seconds to acquire a connection from the pool.
    #[serde(default = "DataConfig::default_acquire_timeout_secs")]
    pub acquire_timeout_secs: u64,
}

impl DataConfig {
    fn default_max_connections() -> u32 {
        10
    }
    fn default_acquire_timeout_secs() -> u64 {
        30
    }
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost/app".into(),
            max_connections: Self::default_max_connections(),
            acquire_timeout_secs: Self::default_acquire_timeout_secs(),
        }
    }
}

/// Connects to PostgreSQL and returns a pool ready to use.
///
/// Applies the pool configuration declared in [`DataConfig`].
/// The pool is initialized lazily: real connections are opened
/// when needed, up to the declared maximum.
///
/// # Errors
///
/// Returns [`DataError::Sqlx`] if the URL is invalid or if the
/// initial test connection cannot be established.
pub async fn connect(config: &DataConfig) -> Result<DbPool, DataError> {
    tracing::debug!(url = %sanitize_url(&config.url), "connecting to the PostgreSQL pool");

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_secs))
        .connect(&config.url)
        .await?;

    tracing::info!(
        max_connections = config.max_connections,
        "PostgreSQL pool ready"
    );
    Ok(pool)
}

/// Runs the embedded migrations against the pool.
///
/// The `migrator` is built with the `sqlx::migrate!("./migrations")` macro
/// in the crate that contains the SQL files. This helper applies all
/// pending migrations in order and is idempotent.
///
/// # Errors
///
/// Returns [`DataError::Migration`] if a migration fails.
pub async fn run_migrations(
    pool: &DbPool,
    migrator: &sqlx::migrate::Migrator,
) -> Result<(), DataError> {
    tracing::info!("applying pending migrations");
    migrator.run(pool).await?;
    tracing::info!("migrations completed");
    Ok(())
}

/// Strips credentials from the URL for safe logging.
fn sanitize_url(url: &str) -> String {
    // Hides user:password@ to avoid leaking credentials in logs.
    if let Some(at) = url.find('@') {
        if let Some(proto) = url.find("://") {
            return format!("{}://<redacted>@{}", &url[..proto], &url[at + 1..]);
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_config_default_is_sensible() {
        let cfg = DataConfig::default();
        assert_eq!(cfg.max_connections, 10);
        assert_eq!(cfg.acquire_timeout_secs, 30);
        assert!(cfg.url.starts_with("postgresql://"));
    }

    #[test]
    fn data_config_deserializes_with_serde_defaults() {
        // Only `url` is provided; the two pool knobs fall back to their
        // `#[serde(default = ...)]` functions.
        let cfg: DataConfig =
            serde_json::from_str(r#"{"url":"postgresql://localhost/only_url"}"#).unwrap();
        assert_eq!(cfg.url, "postgresql://localhost/only_url");
        assert_eq!(cfg.max_connections, 10);
        assert_eq!(cfg.acquire_timeout_secs, 30);
    }

    #[test]
    fn data_config_deserializes_all_fields() {
        let cfg: DataConfig = serde_json::from_str(
            r#"{"url":"postgresql://h/db","max_connections":5,"acquire_timeout_secs":2}"#,
        )
        .unwrap();
        assert_eq!(cfg.max_connections, 5);
        assert_eq!(cfg.acquire_timeout_secs, 2);
    }

    #[test]
    fn data_config_requires_url() {
        // `url` has no default, so an empty object must fail to deserialize.
        let parsed: Result<DataConfig, _> = serde_json::from_str("{}");
        assert!(parsed.is_err());
    }

    #[tokio::test]
    async fn connect_with_invalid_url_returns_sqlx_error() {
        // A malformed connection string fails at parse time, before any
        // network I/O, so this stays a fast unit-level check (no timeout wait).
        let config = DataConfig {
            url: "not-a-valid-url".into(),
            acquire_timeout_secs: 1,
            ..DataConfig::default()
        };
        let err = connect(&config).await.expect_err("invalid URL must error");
        assert!(matches!(err, DataError::Sqlx(_)));
    }

    #[test]
    fn sanitize_url_removes_credentials() {
        let url = "postgresql://user:secret@localhost:5432/db";
        let safe = sanitize_url(url);
        assert!(!safe.contains("secret"));
        assert!(safe.contains("localhost:5432/db"));
    }

    #[test]
    fn sanitize_url_without_credentials_is_unchanged() {
        let url = "postgresql://localhost/db";
        let safe = sanitize_url(url);
        assert_eq!(safe, url);
    }

    #[test]
    fn data_error_converts_to_ag_error() {
        use ag_core::AgError;
        let data_err = DataError::Sqlx(sqlx::Error::RowNotFound);
        let ag_err = AgError::from(data_err);
        assert_eq!(ag_err.code(), "database_error");
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL pointing at a reachable PostgreSQL"]
    async fn connect_to_real_db() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for this test");
        let config = DataConfig {
            url,
            ..DataConfig::default()
        };
        let pool = connect(&config).await.expect("connection failed");
        let row: (i64,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL pointing at a reachable PostgreSQL"]
    async fn connect_and_run_migrations() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for this test");
        let config = DataConfig {
            url,
            ..DataConfig::default()
        };
        let pool = connect(&config).await.expect("connection failed");

        // Fixture migration lives under `tests/migrations/`.
        static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("tests/migrations");
        run_migrations(&pool, &MIGRATOR)
            .await
            .expect("migrations must apply");
        // Re-running is idempotent: already-applied versions are skipped.
        run_migrations(&pool, &MIGRATOR)
            .await
            .expect("re-running migrations must be idempotent");

        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS (SELECT FROM information_schema.tables \
             WHERE table_name = 'ag_data_smoke')",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(exists.0, "smoke migration must create ag_data_smoke");
    }
}
