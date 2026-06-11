//! PostgreSQL backend integration tests.
//!
//! Require a live PostgreSQL instance via `DATABASE_URL`. Ignored by default so CI
//! without a database stays green (repository convention; see ag-domains/ag-mail).
//!
//! They share one database and reset the schema in `fresh_backend`, so run them
//! serially with `--test-threads=1`:
//!
//! ```text
//! DATABASE_URL=postgres://user:pass@localhost/ag_workers_test \
//!   cargo test -p ag-workers --features postgres --test postgres_queue \
//!   -- --ignored --test-threads=1
//! ```
#![cfg(feature = "postgres")]

mod common;

use std::time::Duration;

use ag_workers::queue::postgres::PostgresQueue;
use ag_workers::{DeadLetterReason, QueueBackend, QueueName, WorkerId};

use common::unit_job;

async fn fresh_backend() -> PostgresQueue {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for postgres tests");
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    // Clean slate so repeated runs are deterministic. Dropping `_sqlx_migrations`
    // too is required: otherwise sqlx sees the migrations as already applied and
    // skips recreating the worker tables on the next run.
    let _ = sqlx::query(
        "DROP TABLE IF EXISTS ag_worker_jobs, ag_worker_dead_letters, ag_worker_schedules, _sqlx_migrations CASCADE",
    )
    .execute(&pool)
    .await;
    let backend = PostgresQueue::new(pool).with_lease_timeout_ms(200);
    backend.run_migrations().await.unwrap();
    backend
}

#[tokio::test]
#[ignore = "requires DATABASE_URL to a Postgres instance"]
async fn enqueue_lease_ack_roundtrip() {
    let backend = fresh_backend().await;
    let queue = QueueName::new("mail");
    let worker = WorkerId::generate();
    backend.enqueue(unit_job("t", "mail")).await.unwrap();

    let leased = backend.lease(&queue, &worker, 10).await.unwrap();
    assert_eq!(leased.len(), 1);
    assert_eq!(leased[0].attempts, 1);

    // Locked: a second lease finds nothing until the lease expires.
    assert!(backend.lease(&queue, &worker, 10).await.unwrap().is_empty());

    backend.ack(leased[0].id).await.unwrap();
    let depth = backend.depth(&queue).await.unwrap();
    assert_eq!(depth.total(), 0);
}

#[tokio::test]
#[ignore = "requires DATABASE_URL to a Postgres instance"]
async fn skip_locked_no_double_lease() {
    let backend = fresh_backend().await;
    let queue = QueueName::new("mail");
    for _ in 0..5 {
        backend.enqueue(unit_job("t", "mail")).await.unwrap();
    }
    let w1 = WorkerId::generate();
    let w2 = WorkerId::generate();
    let (a, b) = tokio::join!(backend.lease(&queue, &w1, 5), backend.lease(&queue, &w2, 5));
    let mut ids: Vec<_> = a.unwrap().into_iter().map(|j| j.id).collect();
    ids.extend(b.unwrap().into_iter().map(|j| j.id));
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), 5, "all five leased exactly once across workers");
    assert_eq!(unique.len(), ids.len(), "no job leased twice");
}

#[tokio::test]
#[ignore = "requires DATABASE_URL to a Postgres instance"]
async fn reaper_recovers_expired_lease() {
    let backend = fresh_backend().await;
    let queue = QueueName::new("mail");
    let worker = WorkerId::generate();
    backend.enqueue(unit_job("t", "mail")).await.unwrap();
    backend.lease(&queue, &worker, 1).await.unwrap();

    // Lease timeout is 200ms; wait it out, then the reaper returns the job to queued.
    tokio::time::sleep(Duration::from_millis(300)).await;
    let recovered = backend.release_expired_leases().await.unwrap();
    assert_eq!(recovered, 1);
    let leased = backend.lease(&queue, &worker, 1).await.unwrap();
    assert_eq!(leased.len(), 1);
    assert_eq!(leased[0].attempts, 2, "re-lease increments the attempt");
}

#[tokio::test]
#[ignore = "requires DATABASE_URL to a Postgres instance"]
async fn dead_letter_preserves_and_removes() {
    let backend = fresh_backend().await;
    let queue = QueueName::new("mail");
    let worker = WorkerId::generate();
    let id = backend.enqueue(unit_job("t", "mail")).await.unwrap();
    backend.lease(&queue, &worker, 1).await.unwrap();
    backend
        .dead_letter(id, DeadLetterReason::Permanent("boom".into()))
        .await
        .unwrap();
    assert_eq!(backend.depth(&queue).await.unwrap().total(), 0);
}

#[tokio::test]
#[ignore = "requires DATABASE_URL to a Postgres instance"]
async fn bulk_dlq_filtered_redrive_and_purge() {
    let backend = fresh_backend().await;
    let mail = QueueName::new("mail");
    let worker = WorkerId::generate();

    // Dead-letter 3 mail jobs (2 of kind "send_receipt", 1 "send_welcome") and 1 sms job.
    for (kind, queue) in [
        ("send_receipt", "mail"),
        ("send_receipt", "mail"),
        ("send_welcome", "mail"),
        ("notify", "sms"),
    ] {
        let id = backend.enqueue(unit_job(kind, queue)).await.unwrap();
        backend
            .lease(&QueueName::new(queue), &worker, 10)
            .await
            .unwrap();
        backend
            .dead_letter(id, DeadLetterReason::Permanent("boom".into()))
            .await
            .unwrap();
    }

    // Dry-run: preview the send_receipt entries without mutating.
    let filter = ag_workers::DlqFilter::new("mail", 100).with_kind(Some("send_receipt".to_owned()));
    let preview = backend.preview_dead_letters(&filter).await.unwrap();
    assert_eq!(preview.affected, 2);
    assert_eq!(
        backend
            .list_dead_letters(Some("mail"), 100)
            .await
            .unwrap()
            .len(),
        3,
        "preview must not remove anything"
    );

    // Bulk re-drive the send_receipt entries back onto the mail queue.
    let outcome = backend.redrive_dead_letters(&filter).await.unwrap();
    assert_eq!(outcome.affected, 2);
    assert_eq!(
        backend.depth(&mail).await.unwrap().total(),
        2,
        "re-driven jobs are back"
    );
    assert_eq!(
        backend
            .list_dead_letters(Some("mail"), 100)
            .await
            .unwrap()
            .len(),
        1,
        "only send_welcome remains dead on mail"
    );

    // Filtered purge of the remaining mail dead-letters leaves sms untouched.
    let purged = backend
        .purge_dead_letters_filtered(&ag_workers::DlqFilter::new("mail", 100))
        .await
        .unwrap();
    assert_eq!(purged.affected, 1);
    assert!(backend
        .list_dead_letters(Some("mail"), 100)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(
        backend
            .list_dead_letters(Some("sms"), 100)
            .await
            .unwrap()
            .len(),
        1,
        "sms dead-letter is untouched"
    );
}

#[tokio::test]
#[ignore = "requires DATABASE_URL to a Postgres instance"]
async fn enqueue_in_tx_rolls_back_with_caller() {
    let url = std::env::var("DATABASE_URL").unwrap();
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    let _ = sqlx::query(
        "DROP TABLE IF EXISTS ag_worker_jobs, ag_worker_dead_letters, ag_worker_schedules, _sqlx_migrations CASCADE",
    )
    .execute(&pool)
    .await;
    let backend = PostgresQueue::new(pool.clone());
    backend.run_migrations().await.unwrap();
    let queue = QueueName::new("mail");

    // Rollback: the job must not survive.
    let mut tx = ag_data::AgTx::begin(&pool).await.unwrap();
    backend
        .enqueue_in_tx(&mut tx, unit_job("t", "mail"))
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(backend.depth(&queue).await.unwrap().total(), 0);

    // Commit: the job survives.
    let mut tx = ag_data::AgTx::begin(&pool).await.unwrap();
    backend
        .enqueue_in_tx(&mut tx, unit_job("t", "mail"))
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(backend.depth(&queue).await.unwrap().total(), 1);
}
