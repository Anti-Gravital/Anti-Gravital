//! Cron-like recurring jobs with the `ag-workers` singleton scheduler.
//!
//! Three schedules enqueue typed jobs into one queue at different intervals.
//! The scheduler claims each due tick as a singleton and a worker pool executes
//! the enqueued jobs. The in-memory store keeps a single process trivially
//! correct; no external service is required.

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

/// Every tick carries the same marker payload; the schedule's kind identifies
/// which handler runs it.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Tick;

impl JobPayload for Tick {
    const PAYLOAD_VERSION: u16 = 1;
}

struct CountHandler {
    name: &'static str,
    count: Arc<AtomicU32>,
}

#[async_trait::async_trait]
impl JobHandler for CountHandler {
    type Payload = Tick;

    fn kind(&self) -> &'static str {
        self.name
    }

    async fn handle(
        &self,
        _ctx: JobContext,
        _payload: Tick,
    ) -> Result<JobOutcome, WorkerError> {
        let n = self.count.fetch_add(1, Ordering::SeqCst) + 1;
        println!("{} ran ({n})", self.name);
        Ok(JobOutcome::Complete)
    }
}

fn schedule<P: JobPayload + Serialize>(
    kind: &'static str,
    queue: &str,
    payload: &P,
    interval: Duration,
) -> Schedule {
    Schedule::every(
        kind,
        queue,
        encode(payload).expect("encode payload"),
        P::PAYLOAD_VERSION,
        interval,
        Utc::now(),
    )
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cleanups = Arc::new(AtomicU32::new(0));
    let aggregates = Arc::new(AtomicU32::new(0));
    let pings = Arc::new(AtomicU32::new(0));

    let registry = Arc::new(
        ag_workers::WorkerRegistry::builder()
            .register(CountHandler {
                name: "cleanup_sessions",
                count: cleanups.clone(),
            })
            .expect("unique kind")
            .register(CountHandler {
                name: "aggregate_metrics",
                count: aggregates.clone(),
            })
            .expect("unique kind")
            .register(CountHandler {
                name: "health_ping",
                count: pings.clone(),
            })
            .expect("unique kind")
            .build(),
    );

    let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());
    let queue = QueueName::new("maintenance");

    let store = MemoryScheduleStore::new();
    store.register(schedule(
        "cleanup_sessions",
        "maintenance",
        &Tick,
        Duration::from_millis(100),
    ));
    store.register(schedule(
        "aggregate_metrics",
        "maintenance",
        &Tick,
        Duration::from_millis(250),
    ));
    store.register(schedule(
        "health_ping",
        "maintenance",
        &Tick,
        Duration::from_millis(500),
    ));

    let shutdown = Shutdown::new();

    let scheduler = tokio::spawn(run_scheduler(
        store,
        backend.clone(),
        Duration::from_millis(20),
        shutdown.clone(),
    ));

    let pool = WorkerPool::start(
        backend.clone(),
        registry,
        Arc::new(RuntimeConfig::default()),
        shutdown.clone(),
        queue,
        1,
    );

    tokio::time::sleep(Duration::from_millis(1100)).await;

    shutdown.trigger();
    let _ = scheduler.await;
    pool.join_with_timeout(Duration::from_secs(2)).await;

    println!(
        "done: cleanup={}, aggregate={}, ping={}",
        cleanups.load(Ordering::SeqCst),
        aggregates.load(Ordering::SeqCst),
        pings.load(Ordering::SeqCst)
    );
}
