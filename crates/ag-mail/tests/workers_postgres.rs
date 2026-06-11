//! Parity integration tests for mail delivery routed through `ag-workers`'
//! PostgreSQL backend (RFC-0012 S7/M3).
//!
//! These prove that a mail-delivery job enqueued via [`WorkersMailQueue`] becomes
//! a durable row in `ag_worker_jobs` under `kind = 'mail.delivery'`, is leased and
//! delivered by a [`MailDeliveryHandler`] with the same observable outcome as
//! `ag-mail`'s own persistent queue (`tests/persistent_queue.rs`), and survives a
//! simulated process restart. This is the evidence required before the duplicated
//! generic queue (`queue-persistent`) is retired (S7/M4).
//!
//! Require a live PostgreSQL instance via `TEST_DATABASE_URL`; `#[ignore]`d by
//! default so CI without a database stays green (repository convention, mirrors
//! `tests/persistent_queue.rs` and the `ag-workers` Postgres suite).
//!
//! ```text
//! TEST_DATABASE_URL=postgres://user:pass@localhost/ag_mail_test \
//!   cargo test -p ag-mail --features workers-postgres -- --ignored
//! ```

#![cfg(feature = "workers-postgres")]

use std::sync::Arc;
use std::time::Duration;

use ag_mail::message::{Address, Email, EmailBuilder};
use ag_mail::queue::MailQueue;
use ag_mail::sender::test_utils::NullSender;
use ag_mail::workers::{mail_delivery_registry, WorkersMailQueue, MAIL_DELIVERY_KIND, MAIL_QUEUE};
use ag_workers::queue::postgres::PostgresQueue;
use ag_workers::{QueueBackend, QueueName, RuntimeConfig, Shutdown, WorkerPool};

async fn fresh_backend() -> (sqlx::PgPool, PostgresQueue) {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL for postgres tests");
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    // Clean slate so repeated runs are deterministic. Dropping `_sqlx_migrations`
    // too is required: otherwise sqlx sees the migrations as already applied and
    // skips recreating the worker tables on the next run.
    let _ = sqlx::query(
        "DROP TABLE IF EXISTS ag_worker_jobs, ag_worker_dead_letters, ag_worker_schedules, _sqlx_migrations CASCADE",
    )
    .execute(&pool)
    .await;
    let backend = PostgresQueue::new(pool.clone()).with_lease_timeout_ms(500);
    backend.run_migrations().await.unwrap();
    (pool, backend)
}

fn test_email() -> Email {
    EmailBuilder::new()
        .from(Address::new("a@x.com"))
        .to(Address::new("b@y.com"))
        .subject("s")
        .text_body("t")
        .build()
        .unwrap()
}

async fn count_jobs(pool: &sqlx::PgPool, kind: &str, status: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM ag_worker_jobs WHERE kind = $1 AND status = $2")
        .bind(kind)
        .bind(status)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn count_jobs_any_status(pool: &sqlx::PgPool, kind: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM ag_worker_jobs WHERE kind = $1")
        .bind(kind)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Enqueue persists as `kind = 'mail.delivery'`, then a pool leases and delivers
/// it, reaching `succeeded` with the same NullSender outcome as the legacy queue.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL PostgreSQL"]
async fn mail_job_persists_as_kind_and_is_delivered() {
    let (pool, backend) = fresh_backend().await;
    let backend: Arc<dyn QueueBackend> = Arc::new(backend);

    // Producer side: enqueue through the MailQueue adapter (no worker running yet).
    let producer = WorkersMailQueue::new(backend.clone());
    producer.enqueue(test_email()).await.unwrap();

    // Durability + "mail payloads become a kind": the row exists, queued, before
    // any worker touches it.
    assert_eq!(
        count_jobs(&pool, MAIL_DELIVERY_KIND, "queued").await,
        1,
        "mail job must be a durable 'mail.delivery' row"
    );

    // Consumer side: a static pool leases and delivers via the MailSender.
    let sender = Arc::new(NullSender::new());
    let registry = Arc::new(mail_delivery_registry(sender.clone()).unwrap());
    let shutdown = Shutdown::new();
    let pool_handle = WorkerPool::start(
        backend.clone(),
        registry,
        Arc::new(RuntimeConfig::default()),
        shutdown.clone(),
        QueueName::new(MAIL_QUEUE),
        2,
    );

    for _ in 0..40 {
        if sender.emails_sent() >= 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    shutdown.trigger();
    pool_handle.join_with_timeout(Duration::from_secs(2)).await;

    assert_eq!(sender.emails_sent(), 1, "NullSender must capture one email");
    // The `ag-workers` backend acks a completed job by deleting its row (there is no
    // persisted `succeeded` status; see `PostgresQueue::ack`). Parity with the legacy
    // queue is therefore "delivered exactly once" plus "the durable job is gone": the
    // `mail.delivery` row must no longer exist in any state.
    assert_eq!(
        count_jobs_any_status(&pool, MAIL_DELIVERY_KIND).await,
        0,
        "delivered mail job must be acked (row removed) from ag_worker_jobs"
    );
}

/// A mail job enqueued before a "restart" is still durable and gets delivered by
/// a freshly constructed backend over the same database — proving the work lives
/// in PostgreSQL, not in process memory.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL PostgreSQL"]
async fn mail_job_survives_simulated_restart() {
    let (pool, backend_a) = fresh_backend().await;
    let backend_a: Arc<dyn QueueBackend> = Arc::new(backend_a);

    // Enqueue, then drop the producer/backend handle (simulated process exit).
    WorkersMailQueue::new(backend_a.clone())
        .enqueue(test_email())
        .await
        .unwrap();
    drop(backend_a);
    assert_eq!(count_jobs(&pool, MAIL_DELIVERY_KIND, "queued").await, 1);

    // "Restart": a new backend over the same pool leases the surviving job.
    let backend_b: Arc<dyn QueueBackend> =
        Arc::new(PostgresQueue::new(pool.clone()).with_lease_timeout_ms(500));
    let sender = Arc::new(NullSender::new());
    let registry = Arc::new(mail_delivery_registry(sender.clone()).unwrap());
    let shutdown = Shutdown::new();
    let pool_handle = WorkerPool::start(
        backend_b.clone(),
        registry,
        Arc::new(RuntimeConfig::default()),
        shutdown.clone(),
        QueueName::new(MAIL_QUEUE),
        1,
    );

    for _ in 0..40 {
        if sender.emails_sent() >= 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    shutdown.trigger();
    pool_handle.join_with_timeout(Duration::from_secs(2)).await;

    assert_eq!(
        sender.emails_sent(),
        1,
        "job enqueued before restart must be delivered after it"
    );
}
