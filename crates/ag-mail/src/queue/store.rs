//! Persistent mail queue backend backed by `ag-data` (PostgreSQL).
//!
//! Enabled by the `queue-persistent` feature. Each queued email is one row in
//! `ag_mail_queue`. A background worker polls due `pending` rows, sends them via
//! the `MailSender`, and updates the row state with persisted exponential backoff.
//!
//! # Deprecated: superseded by the shared `ag-workers` queue (RFC-0012 S7)
//!
//! This is `ag-mail`'s own generic persistent queue. The shared background
//! execution engine `ag-workers` provides the same durable queue/retry/DLQ
//! mechanics for every module, so maintaining a second persistent queue here is
//! duplication. New code should route delivery through the `workers` feature
//! ([`crate::workers::WorkersMailQueue`] over `ag-workers`' PostgreSQL backend);
//! mail-specific transport (SMTP/MTA/DKIM/bounce) stays in [`crate::sender`].
//!
//! Parity is proven by `tests/workers_postgres.rs`; this module is retained until
//! that parity is verified against a live database, then removed (S7/M4 tracked as
//! GitHub Issue #103, not in `docs/DEBT.md`).

// The whole module IS the deprecated queue; suppress the lint internally so its own
// impls/tests build, while external users still see the `#[deprecated]` on the
// public `PersistentQueue`.
#![allow(deprecated)]

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use sqlx::types::Json;
use uuid::Uuid;

use crate::{
    error::AgMailError,
    message::Email,
    queue::{MailQueue, RetryPolicy},
    sender::MailSender,
};

/// Lifecycle state of a queued email.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    /// Waiting to be sent or retried.
    Pending,
    /// Currently being sent by the worker.
    Sending,
    /// Successfully delivered.
    Sent,
    /// Exhausted all retries without success.
    Failed,
}

impl JobStatus {
    /// Returns the string representation stored in the database column.
    pub fn as_str(self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Sending => "sending",
            JobStatus::Sent => "sent",
            JobStatus::Failed => "failed",
        }
    }
}

/// Computes the next retry timestamp for a given attempt using the policy.
pub(crate) fn next_retry_at(
    policy: &RetryPolicy,
    attempt: u32,
    now: DateTime<Utc>,
) -> DateTime<Utc> {
    let delay = policy.delay_for(attempt);
    now + ChronoDuration::from_std(delay).unwrap_or_else(|_| ChronoDuration::seconds(2))
}

/// Persistent queue: enqueues into PostgreSQL and a worker drains due rows.
///
/// Deprecated in favor of the shared `ag-workers` queue; see the module note.
#[deprecated(
    note = "use the `workers` feature (ag_mail::workers::WorkersMailQueue over ag-workers' \
            PostgreSQL backend); ag-mail's own generic persistent queue is being retired (RFC-0012 S7)"
)]
pub struct PersistentQueue {
    pool: ag_data::DbPool,
    policy: RetryPolicy,
}

impl PersistentQueue {
    /// Builds the queue over an existing pool and spawns the worker.
    pub fn new<S>(pool: ag_data::DbPool, sender: Arc<S>, policy: RetryPolicy) -> Self
    where
        S: MailSender + 'static,
    {
        let worker_pool = pool.clone();
        let worker_policy = policy.clone();
        tokio::spawn(async move {
            run_worker(worker_pool, sender, worker_policy).await;
        });
        Self { pool, policy }
    }
}

#[async_trait]
impl MailQueue for PersistentQueue {
    async fn enqueue(&self, email: Email) -> Result<(), AgMailError> {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO ag_mail_queue (id, payload, status, attempts, max_retries) \
             VALUES ($1, $2, 'pending', 0, $3)",
        )
        .bind(id)
        .bind(Json(&email))
        .bind(self.policy.max_retries as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| AgMailError::Queue(e.to_string()))?;

        crate::metrics::queue_depth_inc();
        Ok(())
    }
}

/// Polls due rows and sends them. One pass per second.
async fn run_worker<S>(pool: ag_data::DbPool, sender: Arc<S>, policy: RetryPolicy)
where
    S: MailSender + 'static,
{
    use tokio::time::{sleep, Duration};
    loop {
        if let Err(e) = drain_once(&pool, sender.as_ref(), &policy).await {
            tracing::error!(error = %e, "ag-mail persistent worker pass failed");
        }
        sleep(Duration::from_secs(1)).await;
    }
}

async fn drain_once<S>(
    pool: &ag_data::DbPool,
    sender: &S,
    policy: &RetryPolicy,
) -> Result<(), AgMailError>
where
    S: MailSender,
{
    // Claim one due row atomically (SKIP LOCKED for safe concurrency).
    let row: Option<(Uuid, Json<Email>, i32)> = sqlx::query_as(
        "UPDATE ag_mail_queue SET status = 'sending', updated_at = now() \
         WHERE id = ( \
            SELECT id FROM ag_mail_queue \
            WHERE status = 'pending' AND next_retry_at <= now() \
            ORDER BY next_retry_at FOR UPDATE SKIP LOCKED LIMIT 1 \
         ) RETURNING id, payload, attempts",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AgMailError::Queue(e.to_string()))?;

    let Some((id, Json(email), attempts)) = row else {
        return Ok(());
    };

    match sender.send(&email).await {
        Ok(_) => {
            sqlx::query(
                "UPDATE ag_mail_queue SET status = 'sent', updated_at = now() WHERE id = $1",
            )
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| AgMailError::Queue(e.to_string()))?;
            crate::metrics::queue_depth_dec();
        }
        Err(e) => {
            let attempt = attempts as u32 + 1;
            let (status, next) = if attempt > policy.max_retries {
                ("failed", Utc::now())
            } else {
                ("pending", next_retry_at(policy, attempt, Utc::now()))
            };
            sqlx::query(
                "UPDATE ag_mail_queue SET status = $1, attempts = $2, next_retry_at = $3, \
                 last_error = $4, updated_at = now() WHERE id = $5",
            )
            .bind(status)
            .bind(attempt as i32)
            .bind(next)
            .bind(e.to_string())
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| AgMailError::Queue(e.to_string()))?;
            if status == "failed" {
                crate::metrics::queue_depth_dec();
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use crate::message::{Address, EmailBuilder};

    #[test]
    fn next_retry_grows_with_attempt() {
        let policy = RetryPolicy {
            max_retries: 3,
            base_delay: Duration::from_secs(2),
            backoff_factor: 2,
        };
        let now = Utc::now();
        let r0 = next_retry_at(&policy, 0, now);
        let r1 = next_retry_at(&policy, 1, now);
        assert!(r1 > r0, "retry for attempt 1 must be later than attempt 0");
    }

    #[test]
    fn email_roundtrips_through_json() {
        let email = EmailBuilder::new()
            .from(Address::new("a@x.com"))
            .to(Address::new("b@y.com"))
            .subject("hi")
            .text_body("body")
            .build()
            .unwrap();
        let json = serde_json::to_value(&email).unwrap();
        let back: Email = serde_json::from_value(json).unwrap();
        assert_eq!(email, back);
    }
}
