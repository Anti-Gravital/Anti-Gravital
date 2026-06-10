//! Producer-only (edge / serverless) usage of `ag-workers` (RFC-0012 section 17.4).
//!
//! Some deployment targets (the edge data plane, serverless functions) cannot host
//! long-lived workers. In producer-only mode a process links the enqueue API only:
//! it writes jobs to the shared backend and hosts no leasing loop, no scheduler, no
//! worker runtime. A separate standalone/distributed worker fleet consumes them.
//!
//! This example shows both sides against one in-memory backend so it runs end to end;
//! in production the two sides are different processes (often different machines), the
//! producer linking only `enqueue` / `enqueue_in_tx`.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ag_workers::{
    JobContext, JobHandler, JobOutcome, JobPayload, MemoryQueue, NewJob, QueueBackend, QueueName,
    RuntimeConfig, Shutdown, WorkerError, WorkerMode, WorkerPool, WorkerRegistry, WorkersConfig,
};
use serde::{Deserialize, Serialize};

/// Work produced at the edge (for example, a webhook delivery request).
#[derive(Debug, Serialize, Deserialize)]
struct DeliverWebhook {
    url: String,
    event: String,
}

impl JobPayload for DeliverWebhook {
    const PAYLOAD_VERSION: u16 = 1;
}

struct WebhookHandler {
    delivered: Arc<AtomicU32>,
}

#[async_trait::async_trait]
impl JobHandler for WebhookHandler {
    type Payload = DeliverWebhook;

    fn kind(&self) -> &'static str {
        "deliver_webhook"
    }

    async fn handle(
        &self,
        _ctx: JobContext,
        payload: DeliverWebhook,
    ) -> Result<JobOutcome, WorkerError> {
        println!("delivering '{}' to {}", payload.event, payload.url);
        self.delivered.fetch_add(1, Ordering::SeqCst);
        Ok(JobOutcome::Complete)
    }
}

#[tokio::main]
async fn main() {
    let queue = QueueName::new("webhooks");

    // The shared durable backend. In production this is the PostgreSQL backend that
    // both the edge producer and the worker fleet point at; here it is in-memory.
    let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());

    // --- Edge / serverless process: producer only ---------------------------------
    // It links the enqueue API and nothing else. The deployment is configured as
    // `Producer`, which hosts no worker loop.
    let producer_config = WorkersConfig {
        mode: WorkerMode::Producer,
        ..WorkersConfig::default()
    };
    assert_eq!(producer_config.mode, WorkerMode::Producer);

    const N: usize = 4;
    for i in 0..N {
        let job = NewJob::encode::<DeliverWebhook>(
            "deliver_webhook",
            "webhooks",
            &DeliverWebhook {
                url: format!("https://example.com/hook/{i}"),
                event: "order.created".to_owned(),
            },
        )
        .expect("encode payload");
        backend.enqueue(job).await.expect("enqueue");
    }
    let queued = backend.depth(&queue).await.expect("depth");
    println!(
        "producer (edge): enqueued {} jobs, hosts no workers",
        queued.total()
    );

    // --- Worker fleet process: a separate deployment that consumes -----------------
    let delivered = Arc::new(AtomicU32::new(0));
    let registry = Arc::new(
        WorkerRegistry::builder()
            .register(WebhookHandler {
                delivered: delivered.clone(),
            })
            .expect("unique kind")
            .build(),
    );
    let shutdown = Shutdown::new();
    let pool = WorkerPool::start(
        backend.clone(),
        registry,
        Arc::new(RuntimeConfig::default()),
        shutdown.clone(),
        queue.clone(),
        2,
    );

    for _ in 0..200 {
        if delivered.load(Ordering::SeqCst) >= N as u32 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    shutdown.trigger();
    pool.join_with_timeout(Duration::from_secs(2)).await;
    println!("fleet: delivered {} jobs", delivered.load(Ordering::SeqCst));
}
