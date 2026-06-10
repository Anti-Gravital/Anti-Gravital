//! Durable PostgreSQL queue backend (feature `postgres`).
//!
//! Built on `ag-data`'s pool and `sqlx`'s embedded migrations. Leasing uses
//! `FOR UPDATE SKIP LOCKED` so concurrent workers never block each other. Queries use
//! the runtime `sqlx` API (not the compile-time macros) so the crate builds without a
//! live `DATABASE_URL`. Integration tests are `#[ignore]` and require a Postgres
//! instance, per the repository convention.

use ag_data::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{Postgres, Row, Transaction};

use crate::ids::{JobId, JobKind, QueueName, TenantId, WorkerId};
use crate::job::{JobEnvelope, JobPriority, JobStatus, NewJob};
use crate::outcome::DeadLetterReason;
use crate::payload::PayloadBytes;
use crate::queue::{QueueBackend, QueueDepth, QueueError, RetryDecision};

/// Embedded migrations for the worker tables (`./migrations`).
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

const INSERT_SQL: &str = "INSERT INTO ag_worker_jobs \
    (id, kind, queue, payload, payload_version, priority, status, attempts, max_attempts, \
     timeout_ms, scheduled_at, available_at, created_at, updated_at, locked_by, lock_until, \
     dedup_key, tenant_id, trace_id, last_error) \
    VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)";

const SELECT_COLUMNS: &str = "id, kind, queue, payload, payload_version, priority, status, \
    attempts, max_attempts, timeout_ms, scheduled_at, available_at, created_at, updated_at, \
    locked_by, lock_until, dedup_key, tenant_id, trace_id, last_error";

fn backend_err(e: sqlx::Error) -> QueueError {
    QueueError::Backend(e.to_string())
}

/// Durable queue backend over PostgreSQL.
pub struct PostgresQueue {
    pool: DbPool,
    lease_timeout_ms: i64,
}

impl PostgresQueue {
    /// Builds the backend over an existing `ag-data` pool with the default lease timeout.
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            lease_timeout_ms: 60_000,
        }
    }

    /// Sets the lease duration in milliseconds.
    pub fn with_lease_timeout_ms(mut self, lease_timeout_ms: i64) -> Self {
        self.lease_timeout_ms = lease_timeout_ms;
        self
    }

    /// Applies the embedded worker-table migrations through `ag-data`'s migration
    /// machinery (reusing its pool, not a parallel sqlx surface).
    pub async fn run_migrations(&self) -> Result<(), QueueError> {
        ag_data::run_migrations(&self.pool, &MIGRATOR)
            .await
            .map_err(|e| QueueError::Backend(e.to_string()))
    }

    /// Transactional enqueue: the job insert participates in the caller's `ag-data`
    /// transaction, so the job and the caller's writes commit atomically (the
    /// transactional-outbox property without a separate outbox table). The `ag-data`
    /// crate exposes no canonical transaction handle today, so this takes a raw
    /// `sqlx::Transaction`. This gap is to be tracked as a GitHub Issue (label
    /// `tech-debt`) per CLAUDE.md rule 29; the decision is recorded in ADR-0013 §6.
    pub async fn enqueue_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job: NewJob,
    ) -> Result<JobId, QueueError> {
        let env = job.into_envelope(Utc::now());
        bind_insert(&env)
            .execute(&mut **tx)
            .await
            .map_err(backend_err)?;
        Ok(env.id)
    }

    /// Extends the lease on a job the worker still holds (long-running jobs).
    pub async fn heartbeat(&self, job_id: JobId, worker: &WorkerId) -> Result<(), QueueError> {
        let affected = sqlx::query(
            "UPDATE ag_worker_jobs \
             SET lock_until = now() + ($2::bigint * interval '1 millisecond'), updated_at = now() \
             WHERE id = $1 AND locked_by = $3 AND status = 'leased'",
        )
        .bind(job_id.as_uuid())
        .bind(self.lease_timeout_ms)
        .bind(worker.as_str())
        .execute(&self.pool)
        .await
        .map_err(backend_err)?
        .rows_affected();
        if affected == 0 {
            return Err(QueueError::NotFound(job_id));
        }
        Ok(())
    }

    /// Returns abandoned (lease-expired) jobs to `queued`. Returns the count recovered.
    /// This is the durable reaper; run it periodically by a single elected task.
    pub async fn release_expired_leases(&self) -> Result<u64, QueueError> {
        let affected = sqlx::query(
            "UPDATE ag_worker_jobs \
             SET status = 'queued', locked_by = NULL, lock_until = NULL, updated_at = now() \
             WHERE status = 'leased' AND lock_until IS NOT NULL AND lock_until < now()",
        )
        .execute(&self.pool)
        .await
        .map_err(backend_err)?
        .rows_affected();
        Ok(affected)
    }
}

#[async_trait]
impl QueueBackend for PostgresQueue {
    async fn enqueue(&self, job: NewJob) -> Result<JobId, QueueError> {
        let env = job.into_envelope(Utc::now());
        bind_insert(&env)
            .execute(&self.pool)
            .await
            .map_err(backend_err)?;
        Ok(env.id)
    }

    async fn lease(
        &self,
        queue: &QueueName,
        worker: &WorkerId,
        limit: usize,
    ) -> Result<Vec<JobEnvelope>, QueueError> {
        // Claim eligible rows with FOR UPDATE SKIP LOCKED, mark them leased and increment
        // the attempt count at lease time (this is what makes the poison guard correct
        // under panic = "abort"), and return the updated rows.
        let sql = format!(
            "WITH due AS ( \
                 SELECT id FROM ag_worker_jobs \
                 WHERE queue = $1 \
                   AND status IN ('queued', 'retry_scheduled', 'scheduled') \
                   AND available_at <= now() \
                   AND (lock_until IS NULL OR lock_until < now()) \
                 ORDER BY priority DESC, available_at ASC, created_at ASC \
                 FOR UPDATE SKIP LOCKED \
                 LIMIT $2 \
             ) \
             UPDATE ag_worker_jobs j \
             SET status = 'leased', attempts = j.attempts + 1, locked_by = $3, \
                 lock_until = now() + ($4::bigint * interval '1 millisecond'), updated_at = now() \
             FROM due WHERE j.id = due.id \
             RETURNING {SELECT_COLUMNS}"
        );
        let rows = sqlx::query(&sql)
            .bind(queue.as_str())
            .bind(limit as i64)
            .bind(worker.as_str())
            .bind(self.lease_timeout_ms)
            .fetch_all(&self.pool)
            .await
            .map_err(backend_err)?;
        rows.iter().map(row_to_envelope).collect()
    }

    async fn ack(&self, job_id: JobId) -> Result<(), QueueError> {
        let affected = sqlx::query("DELETE FROM ag_worker_jobs WHERE id = $1")
            .bind(job_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(backend_err)?
            .rows_affected();
        if affected == 0 {
            return Err(QueueError::NotFound(job_id));
        }
        Ok(())
    }

    async fn retry(&self, job_id: JobId, decision: RetryDecision) -> Result<(), QueueError> {
        let affected = sqlx::query(
            "UPDATE ag_worker_jobs \
             SET status = 'retry_scheduled', available_at = $2, last_error = $3, \
                 locked_by = NULL, lock_until = NULL, updated_at = now() \
             WHERE id = $1",
        )
        .bind(job_id.as_uuid())
        .bind(decision.available_at)
        .bind(decision.last_error)
        .execute(&self.pool)
        .await
        .map_err(backend_err)?
        .rows_affected();
        if affected == 0 {
            return Err(QueueError::NotFound(job_id));
        }
        Ok(())
    }

    async fn dead_letter(&self, job_id: JobId, reason: DeadLetterReason) -> Result<(), QueueError> {
        let reason_str = dead_letter_reason_text(&reason);
        let mut tx = self.pool.begin().await.map_err(backend_err)?;
        let inserted = sqlx::query(
            "INSERT INTO ag_worker_dead_letters \
                 (id, kind, queue, payload, payload_version, priority, attempts, max_attempts, \
                  timeout_ms, created_at, dedup_key, tenant_id, trace_id, last_error, reason, \
                  dead_lettered_at) \
             SELECT id, kind, queue, payload, payload_version, priority, attempts, max_attempts, \
                  timeout_ms, created_at, dedup_key, tenant_id, trace_id, last_error, $2, now() \
             FROM ag_worker_jobs WHERE id = $1",
        )
        .bind(job_id.as_uuid())
        .bind(&reason_str)
        .execute(&mut *tx)
        .await
        .map_err(backend_err)?
        .rows_affected();
        if inserted == 0 {
            tx.rollback().await.map_err(backend_err)?;
            return Err(QueueError::NotFound(job_id));
        }
        sqlx::query("DELETE FROM ag_worker_jobs WHERE id = $1")
            .bind(job_id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(backend_err)?;
        tx.commit().await.map_err(backend_err)?;
        Ok(())
    }

    async fn depth(&self, queue: &QueueName) -> Result<QueueDepth, QueueError> {
        let row = sqlx::query(
            "SELECT \
                 count(*) FILTER (WHERE status = 'queued' \
                     OR (status IN ('scheduled', 'retry_scheduled') AND available_at <= now())) AS ready, \
                 count(*) FILTER (WHERE status IN ('scheduled', 'retry_scheduled') AND available_at > now()) AS scheduled, \
                 count(*) FILTER (WHERE status IN ('leased', 'running')) AS leased \
             FROM ag_worker_jobs WHERE queue = $1",
        )
        .bind(queue.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(backend_err)?;
        Ok(QueueDepth {
            ready: row.try_get::<i64, _>("ready").map_err(backend_err)? as u64,
            scheduled: row.try_get::<i64, _>("scheduled").map_err(backend_err)? as u64,
            leased: row.try_get::<i64, _>("leased").map_err(backend_err)? as u64,
        })
    }
}

/// Builds a parameter-bound INSERT query for an envelope.
fn bind_insert(env: &JobEnvelope) -> sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments> {
    sqlx::query(INSERT_SQL)
        .bind(env.id.as_uuid())
        .bind(env.kind.as_str())
        .bind(env.queue.as_str())
        .bind(env.payload.as_slice())
        .bind(env.payload_version as i32)
        .bind(env.priority.as_i16())
        .bind(env.status.as_str())
        .bind(env.attempts as i32)
        .bind(env.max_attempts as i32)
        .bind(env.timeout.as_millis() as i64)
        .bind(env.scheduled_at)
        .bind(env.available_at)
        .bind(env.created_at)
        .bind(env.updated_at)
        .bind(env.locked_by.as_ref().map(|w| w.as_str().to_string()))
        .bind(env.lock_until)
        .bind(env.dedup_key.clone())
        .bind(env.tenant_id.as_ref().map(|t| t.as_str().to_string()))
        .bind(env.trace_id.clone())
        .bind(env.last_error.clone())
}

/// Maps a database row to a [`JobEnvelope`].
fn row_to_envelope(row: &PgRow) -> Result<JobEnvelope, QueueError> {
    let status_str: String = row.try_get("status").map_err(backend_err)?;
    let status = JobStatus::from_db_str(&status_str)
        .ok_or_else(|| QueueError::Backend(format!("unknown status in row: {status_str}")))?;
    let timeout_ms: i64 = row.try_get("timeout_ms").map_err(backend_err)?;
    let priority: i16 = row.try_get("priority").map_err(backend_err)?;
    let payload: Vec<u8> = row.try_get("payload").map_err(backend_err)?;
    let payload_version: i32 = row.try_get("payload_version").map_err(backend_err)?;
    let attempts: i32 = row.try_get("attempts").map_err(backend_err)?;
    let max_attempts: i32 = row.try_get("max_attempts").map_err(backend_err)?;
    let locked_by: Option<String> = row.try_get("locked_by").map_err(backend_err)?;
    let tenant_id: Option<String> = row.try_get("tenant_id").map_err(backend_err)?;

    Ok(JobEnvelope {
        id: JobId::from_uuid(row.try_get("id").map_err(backend_err)?),
        kind: JobKind::new(row.try_get::<String, _>("kind").map_err(backend_err)?),
        queue: QueueName::new(row.try_get::<String, _>("queue").map_err(backend_err)?),
        payload: PayloadBytes::from_vec(payload),
        payload_version: payload_version as u16,
        priority: JobPriority::from_i16(priority),
        status,
        attempts: attempts.max(0) as u32,
        max_attempts: max_attempts.max(0) as u32,
        timeout: std::time::Duration::from_millis(timeout_ms.max(0) as u64),
        scheduled_at: row.try_get("scheduled_at").map_err(backend_err)?,
        available_at: row
            .try_get::<DateTime<Utc>, _>("available_at")
            .map_err(backend_err)?,
        created_at: row
            .try_get::<DateTime<Utc>, _>("created_at")
            .map_err(backend_err)?,
        updated_at: row
            .try_get::<DateTime<Utc>, _>("updated_at")
            .map_err(backend_err)?,
        locked_by: locked_by.map(WorkerId::from_string),
        lock_until: row.try_get("lock_until").map_err(backend_err)?,
        dedup_key: row.try_get("dedup_key").map_err(backend_err)?,
        tenant_id: tenant_id.map(TenantId::new),
        trace_id: row.try_get("trace_id").map_err(backend_err)?,
        last_error: row.try_get("last_error").map_err(backend_err)?,
    })
}

/// Builds the stored reason text, including the detail message when present.
fn dead_letter_reason_text(reason: &DeadLetterReason) -> String {
    match reason {
        DeadLetterReason::Permanent(m)
        | DeadLetterReason::InvalidPayload(m)
        | DeadLetterReason::Requested(m) => format!("{}: {}", reason.as_str(), m),
        _ => reason.as_str().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_text_includes_detail() {
        assert_eq!(
            dead_letter_reason_text(&DeadLetterReason::Permanent("boom".into())),
            "permanent: boom"
        );
        assert_eq!(
            dead_letter_reason_text(&DeadLetterReason::PoisonGuard),
            "poison_guard"
        );
    }
}
