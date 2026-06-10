//! Interval (scheduled) jobs with `ag-workers` (RFC-0012 section 21).
//!
//! Demonstrates a recurring job that the scheduler enqueues every interval, and a
//! worker pool that executes it. The scheduler claims each due tick as a singleton,
//! so with the durable backend the same interval fires once across N processes; here
//! the in-memory store keeps a single process trivially correct. No external service
//! is required.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ag_workers::{
    encode, run_scheduler, JobContext, JobHandler, JobOutcome, JobPayload, MemoryQueue,
    MemoryScheduleStore, QueueBackend, QueueName, RuntimeConfig, Schedule, Shutdown, WorkerError,
    WorkerPool,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Payload for the recurring cleanup job. Interval jobs carry the same payload on
/// every tick, so an empty marker is enough here.
#[derive(Debug, Default, Serialize, Deserialize)]
struct CleanupExpiredSessions;

impl JobPayload for CleanupExpiredSessions {
    const PAYLOAD_VERSION: u16 = 1;
}

/// Counts how many times the scheduled job ran.
struct CleanupHandler {
    runs: Arc<AtomicU32>,
}

#[async_trait::async_trait]
impl JobHandler for CleanupHandler {
    type Payload = CleanupExpiredSessions;

    fn kind(&self) -> &'static str {
        "cleanup_expired_sessions"
    }

    async fn handle(
        &self,
        ctx: JobContext,
        _payload: CleanupExpiredSessions,
    ) -> Result<JobOutcome, WorkerError> {
        let n = self.runs.fetch_add(1, Ordering::SeqCst) + 1;
        println!("[tick {n}] cleanup ran (job {})", ctx.job_id);
        Ok(JobOutcome::Complete)
    }
}

#[tokio::main]
async fn main() {
    let runs = Arc::new(AtomicU32::new(0));
    let registry = Arc::new(
        ag_workers::WorkerRegistry::builder()
            .register(CleanupHandler { runs: runs.clone() })
            .expect("unique kind")
            .build(),
    );

    let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());
    let config = Arc::new(RuntimeConfig::default());
    let queue = QueueName::new("maintenance");

    // Register a recurring job: every 100ms enqueue a cleanup job.
    let store = MemoryScheduleStore::new();
    store.register(Schedule::every(
        "cleanup_expired_sessions",
        "maintenance",
        encode(&CleanupExpiredSessions).expect("encode payload"),
        CleanupExpiredSessions::PAYLOAD_VERSION,
        Duration::from_millis(100),
        Utc::now(),
    ));

    let shutdown = Shutdown::new();

    // The scheduler loop: claims due ticks and enqueues them (singleton).
    let scheduler = tokio::spawn(run_scheduler(
        store,
        backend.clone(),
        Duration::from_millis(20),
        shutdown.clone(),
    ));

    // The worker pool: drains and executes the enqueued cleanup jobs.
    let pool = WorkerPool::start(
        backend.clone(),
        registry,
        config,
        shutdown.clone(),
        queue.clone(),
        1,
    );

    // Let a few intervals elapse.
    tokio::time::sleep(Duration::from_millis(550)).await;

    // Graceful shutdown of both the scheduler and the worker pool.
    shutdown.trigger();
    let _ = scheduler.await;
    pool.join_with_timeout(Duration::from_secs(2)).await;

    println!(
        "done: {} scheduled runs executed",
        runs.load(Ordering::SeqCst)
    );
}
