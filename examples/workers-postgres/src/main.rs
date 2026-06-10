//! Durable `ag-workers` backend with transactional enqueue (RFC-0012 sections 13.2-13.3).
//!
//! Shows the PostgreSQL backend and `enqueue_in_tx`: the job insert participates in the
//! caller's transaction, so a domain write and its follow-up job commit atomically (the
//! transactional-outbox property without a separate outbox table). A worker pool then
//! leases and runs the job.
//!
//! Requires a reachable PostgreSQL instance via `DATABASE_URL`. Without it the example
//! prints how to run and exits cleanly.
//!
//! ```text
//! DATABASE_URL=postgres://user:pass@localhost/ag_workers_example \
//!   cargo run -p workers-postgres
//! ```

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ag_workers::{
    JobContext, JobHandler, JobOutcome, JobPayload, NewJob, PostgresQueue, QueueBackend, QueueName,
    RuntimeConfig, Shutdown, WorkerError, WorkerRegistry,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SendReceipt {
    order_id: u64,
}

impl JobPayload for SendReceipt {
    const PAYLOAD_VERSION: u16 = 1;
}

struct ReceiptHandler {
    sent: Arc<AtomicU32>,
}

#[async_trait::async_trait]
impl JobHandler for ReceiptHandler {
    type Payload = SendReceipt;

    fn kind(&self) -> &'static str {
        "send_receipt"
    }

    async fn handle(
        &self,
        _ctx: JobContext,
        payload: SendReceipt,
    ) -> Result<JobOutcome, WorkerError> {
        println!("sending receipt for order {}", payload.order_id);
        self.sent.fetch_add(1, Ordering::SeqCst);
        Ok(JobOutcome::Complete)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        println!(
            "DATABASE_URL not set. Run with:\n  \
             DATABASE_URL=postgres://user:pass@localhost/ag_workers_example \\\n    \
             cargo run -p workers-postgres"
        );
        return Ok(());
    };

    let pool = sqlx::PgPool::connect(&url).await?;
    let queue = PostgresQueue::new(pool.clone());
    queue.run_migrations().await?;

    // Transactional enqueue: in a real app the domain writes (e.g. inserting the order)
    // happen on the same `tx`, so the order and its receipt job commit atomically -- or
    // neither does. Here we show only the job insert participating in the transaction.
    let mut tx = pool.begin().await?;
    let job = NewJob::encode::<SendReceipt>("send_receipt", "mail", &SendReceipt { order_id: 42 })?;
    let job_id = queue.enqueue_in_tx(&mut tx, job).await?;
    tx.commit().await?;
    println!("committed order + job atomically (job {job_id})");

    // A worker pool leases and runs the durably-enqueued job.
    let sent = Arc::new(AtomicU32::new(0));
    let registry = Arc::new(
        WorkerRegistry::builder()
            .register(ReceiptHandler { sent: sent.clone() })?
            .build(),
    );
    let backend: Arc<dyn QueueBackend> = Arc::new(queue);
    let mail = QueueName::new("mail");
    let shutdown = Shutdown::new();
    let pool_handle = ag_workers::WorkerPool::start(
        backend.clone(),
        registry,
        Arc::new(RuntimeConfig::default()),
        shutdown.clone(),
        mail.clone(),
        1,
    );

    for _ in 0..200 {
        if sent.load(Ordering::SeqCst) >= 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    shutdown.trigger();
    pool_handle.join_with_timeout(Duration::from_secs(2)).await;
    println!(
        "done: {} receipt(s) delivered durably",
        sent.load(Ordering::SeqCst)
    );
    Ok(())
}
