//! Durable spool backing for the native MTA scheduled queue (RFC-0009 section 4.2).
//!
//! The in-memory [`super::queue::MtaQueue`] is the runtime source of truth; a
//! [`Spool`] is an opt-in write-through mirror so that scheduled
//! (not-yet-delivered) jobs survive a process restart. Two backends ship:
//!
//! - [`InMemorySpool`]: a process-local map. It is the reference implementation
//!   and is used by the queue's own tests; it does not outlive the process, so
//!   it documents the [`Spool`] contract rather than providing real durability.
//! - [`PostgresSpool`] (feature `queue-postgres`): a durable PostgreSQL mirror.
//!
//! Keeping the in-memory queue as the native default honours ADR-0009 rule 2:
//! the durable backend is opt-in and never required to use the MTA.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

/// Error from a spool backend.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SpoolError {
    /// The backing store rejected or failed an operation.
    #[error("spool backend error: {0}")]
    Backend(String),
}

/// A delivery job as persisted in a spool.
///
/// Identical to a [`super::queue::DeliveryJob`] except that the two timing
/// fields are wall-clock UNIX-epoch milliseconds instead of monotonic
/// `Instant`s, which are meaningless across a restart. The queue converts
/// between the two representations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedJob {
    /// Stable job id (primary key).
    pub id: String,
    /// Owning tenant.
    pub tenant: String,
    /// Campaign / stream.
    pub campaign: String,
    /// Recipient domain.
    pub domain: String,
    /// Destination `site_name` (MX rollup) used for shaping.
    pub site_name: String,
    /// Envelope recipients at this domain.
    pub recipients: Vec<String>,
    /// Envelope sender.
    pub from: String,
    /// Rendered RFC 5322 message bytes.
    pub content: Vec<u8>,
    /// Attempts made so far.
    pub attempts: u32,
    /// Wall-clock time the job first entered the queue (UNIX millis).
    pub enqueued_at_ms: i64,
    /// Wall-clock earliest next attempt (UNIX millis).
    pub next_attempt_at_ms: i64,
}

/// A durable backing store for the MTA scheduled queue.
///
/// Implementations mirror the in-memory scheduled set: [`upsert`](Spool::upsert)
/// on enqueue and on every reschedule, [`remove`](Spool::remove) when a job
/// leaves the queue (delivered, suppressed or expired), and
/// [`load_all`](Spool::load_all) to repopulate the queue on startup. After the
/// initial enqueue, mirroring is best-effort: a failed mirror write is logged
/// and does not fail delivery, since the in-memory queue remains the runtime
/// source of truth.
#[async_trait]
pub trait Spool: Send + Sync {
    /// Persists or updates a job, keyed by [`PersistedJob::id`].
    async fn upsert(&self, job: &PersistedJob) -> Result<(), SpoolError>;
    /// Removes a job that has left the queue.
    async fn remove(&self, id: &str) -> Result<(), SpoolError>;
    /// Loads every persisted job, for recovery on startup.
    async fn load_all(&self) -> Result<Vec<PersistedJob>, SpoolError>;
}

/// Process-local reference spool.
///
/// Useful as a default and in tests; it does not survive the process, so it
/// documents the [`Spool`] contract rather than providing real cross-restart
/// durability.
#[derive(Debug, Default)]
pub struct InMemorySpool {
    jobs: Mutex<HashMap<String, PersistedJob>>,
}

impl InMemorySpool {
    /// Creates an empty spool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of jobs currently held (inspection/test helper).
    pub fn len(&self) -> usize {
        self.jobs.lock().expect("spool lock poisoned").len()
    }

    /// Whether the spool is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl Spool for InMemorySpool {
    async fn upsert(&self, job: &PersistedJob) -> Result<(), SpoolError> {
        self.jobs
            .lock()
            .expect("spool lock poisoned")
            .insert(job.id.clone(), job.clone());
        Ok(())
    }

    async fn remove(&self, id: &str) -> Result<(), SpoolError> {
        self.jobs.lock().expect("spool lock poisoned").remove(id);
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<PersistedJob>, SpoolError> {
        Ok(self
            .jobs
            .lock()
            .expect("spool lock poisoned")
            .values()
            .cloned()
            .collect())
    }
}

#[cfg(feature = "queue-postgres")]
pub use postgres::PostgresSpool;

#[cfg(feature = "queue-postgres")]
mod postgres {
    use super::{PersistedJob, Spool, SpoolError};
    use async_trait::async_trait;
    use sqlx::postgres::PgPoolOptions;
    use sqlx::{PgPool, Row};

    fn backend_err(e: sqlx::Error) -> SpoolError {
        SpoolError::Backend(e.to_string())
    }

    const CREATE_SQL: &str = "CREATE TABLE IF NOT EXISTS ag_mail_mta_spool (\
        id TEXT PRIMARY KEY, tenant TEXT NOT NULL, campaign TEXT NOT NULL, \
        domain TEXT NOT NULL, site_name TEXT NOT NULL, recipients TEXT[] NOT NULL, \
        sender TEXT NOT NULL, content BYTEA NOT NULL, attempts INTEGER NOT NULL, \
        enqueued_at_ms BIGINT NOT NULL, next_attempt_at_ms BIGINT NOT NULL)";

    /// Durable PostgreSQL mirror of the MTA scheduled queue (RFC-0009 section 4.2).
    ///
    /// Uses the runtime `sqlx` API (not the compile-time macros) so the crate
    /// builds without a live `DATABASE_URL`. The backing table is created on
    /// demand. Recipients map to a `TEXT[]` column and the message bytes to
    /// `BYTEA`, so no extra serialization is needed.
    pub struct PostgresSpool {
        pool: PgPool,
    }

    impl PostgresSpool {
        /// Connects to `database_url`, opening a small pool, and ensures the
        /// spool table exists.
        pub async fn connect(database_url: &str) -> Result<Self, SpoolError> {
            let pool = PgPoolOptions::new()
                .max_connections(4)
                .connect(database_url)
                .await
                .map_err(backend_err)?;
            Self::from_pool(pool).await
        }

        /// Builds the spool over an existing pool, ensuring the table exists.
        pub async fn from_pool(pool: PgPool) -> Result<Self, SpoolError> {
            sqlx::query(CREATE_SQL)
                .execute(&pool)
                .await
                .map_err(backend_err)?;
            Ok(Self { pool })
        }
    }

    #[async_trait]
    impl Spool for PostgresSpool {
        async fn upsert(&self, job: &PersistedJob) -> Result<(), SpoolError> {
            // Only `attempts` and `next_attempt_at_ms` change across reschedules;
            // the rest of the row is immutable for a given job id.
            sqlx::query(
                "INSERT INTO ag_mail_mta_spool \
                 (id, tenant, campaign, domain, site_name, recipients, sender, content, \
                  attempts, enqueued_at_ms, next_attempt_at_ms) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) \
                 ON CONFLICT (id) DO UPDATE SET \
                  attempts = EXCLUDED.attempts, \
                  next_attempt_at_ms = EXCLUDED.next_attempt_at_ms",
            )
            .bind(&job.id)
            .bind(&job.tenant)
            .bind(&job.campaign)
            .bind(&job.domain)
            .bind(&job.site_name)
            .bind(&job.recipients)
            .bind(&job.from)
            .bind(&job.content)
            .bind(i32::try_from(job.attempts).unwrap_or(i32::MAX))
            .bind(job.enqueued_at_ms)
            .bind(job.next_attempt_at_ms)
            .execute(&self.pool)
            .await
            .map_err(backend_err)?;
            Ok(())
        }

        async fn remove(&self, id: &str) -> Result<(), SpoolError> {
            sqlx::query("DELETE FROM ag_mail_mta_spool WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(backend_err)?;
            Ok(())
        }

        async fn load_all(&self) -> Result<Vec<PersistedJob>, SpoolError> {
            let rows = sqlx::query(
                "SELECT id, tenant, campaign, domain, site_name, recipients, sender, \
                 content, attempts, enqueued_at_ms, next_attempt_at_ms \
                 FROM ag_mail_mta_spool",
            )
            .fetch_all(&self.pool)
            .await
            .map_err(backend_err)?;

            Ok(rows
                .into_iter()
                .map(|row| PersistedJob {
                    id: row.get("id"),
                    tenant: row.get("tenant"),
                    campaign: row.get("campaign"),
                    domain: row.get("domain"),
                    site_name: row.get("site_name"),
                    recipients: row.get("recipients"),
                    from: row.get("sender"),
                    content: row.get("content"),
                    attempts: u32::try_from(row.get::<i32, _>("attempts")).unwrap_or(0),
                    enqueued_at_ms: row.get("enqueued_at_ms"),
                    next_attempt_at_ms: row.get("next_attempt_at_ms"),
                })
                .collect())
        }
    }
}
