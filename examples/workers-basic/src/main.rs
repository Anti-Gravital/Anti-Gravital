//! Background jobs with `ag-workers` on the in-memory backend.
//!
//! Demonstrates the smallest useful setup: a typed payload, a `JobHandler`, a registry,
//! the in-memory queue, a static worker pool, enqueueing jobs and a graceful shutdown.
//! No external service is required.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ag_workers::{
    JobContext, JobHandler, JobOutcome, JobPayload, MemoryQueue, NewJob, QueueBackend, QueueName,
    RuntimeConfig, Shutdown, WorkerError, WorkerRegistry,
};
use serde::{Deserialize, Serialize};

/// The typed payload for a "send welcome email" job.
#[derive(Debug, Serialize, Deserialize)]
struct SendWelcome {
    user_id: u64,
    email: String,
}

impl JobPayload for SendWelcome {
    const PAYLOAD_VERSION: u16 = 1;
}

/// Handler that "sends" a welcome email (here, just prints and counts).
struct SendWelcomeHandler {
    sent: Arc<AtomicU32>,
}

#[async_trait::async_trait]
impl JobHandler for SendWelcomeHandler {
    type Payload = SendWelcome;

    fn kind(&self) -> &'static str {
        "send_welcome"
    }

    async fn handle(
        &self,
        ctx: JobContext,
        payload: SendWelcome,
    ) -> Result<JobOutcome, WorkerError> {
        println!(
            "[attempt {}] sending welcome to user {} <{}>",
            ctx.attempt, payload.user_id, payload.email
        );
        self.sent.fetch_add(1, Ordering::SeqCst);
        Ok(JobOutcome::Complete)
    }
}

#[tokio::main]
async fn main() {
    let sent = Arc::new(AtomicU32::new(0));
    let registry = Arc::new(
        WorkerRegistry::builder()
            .register(SendWelcomeHandler { sent: sent.clone() })
            .expect("unique kind")
            .build(),
    );

    let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());
    let config = Arc::new(RuntimeConfig::default());
    let queue = QueueName::new("mail");

    // Enqueue a handful of jobs.
    const N: u64 = 5;
    for user_id in 1..=N {
        let job = NewJob::encode::<SendWelcome>(
            "send_welcome",
            "mail",
            &SendWelcome {
                user_id,
                email: format!("user{user_id}@example.com"),
            },
        )
        .expect("encode payload");
        backend.enqueue(job).await.expect("enqueue");
    }

    // Run a static pool of 2 workers (embedded mode).
    let shutdown = Shutdown::new();
    let pool = ag_workers::WorkerPool::start(
        backend.clone(),
        registry,
        config,
        shutdown.clone(),
        queue.clone(),
        2,
    );

    // Wait until all jobs are processed (bounded).
    for _ in 0..200 {
        if sent.load(Ordering::SeqCst) >= N as u32 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Graceful shutdown.
    shutdown.trigger();
    pool.join_with_timeout(Duration::from_secs(2)).await;

    let depth = backend.depth(&queue).await.expect("depth");
    println!(
        "done: {} jobs sent, {} remaining in queue",
        sent.load(Ordering::SeqCst),
        depth.total()
    );
}
