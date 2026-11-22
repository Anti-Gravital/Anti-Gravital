//! Typed background jobs with retries and a dead-letter queue, using the
//! `ag-workers` in-memory backend. No external service is required.
//!
//! The example enqueues three kinds of work:
//! - normal `send_email` jobs that complete,
//! - a poison `send_email` job the handler sends straight to the DLQ,
//! - a `generate_report` job that fails once and then succeeds via retry.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ag_workers::{
    JobContext, JobHandler, JobOutcome, JobPayload, MemoryQueue, NewJob, QueueBackend, QueueName,
    RetryPolicy, RuntimeConfig, Shutdown, WorkerError, WorkerPool, WorkerRegistry,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SendEmail {
    to: String,
    subject: String,
    /// When true the handler treats the payload as poison and dead-letters it.
    poison: bool,
}

impl JobPayload for SendEmail {
    const PAYLOAD_VERSION: u16 = 1;
}

struct EmailHandler {
    sent: Arc<AtomicU32>,
    dead: Arc<AtomicU32>,
}

#[async_trait::async_trait]
impl JobHandler for EmailHandler {
    type Payload = SendEmail;

    fn kind(&self) -> &'static str {
        "send_email"
    }

    async fn handle(
        &self,
        ctx: JobContext,
        payload: SendEmail,
    ) -> Result<JobOutcome, WorkerError> {
        if payload.poison {
            self.dead.fetch_add(1, Ordering::SeqCst);
            return Ok(JobOutcome::DeadLetter {
                reason: format!("invalid recipient: {}", payload.to),
            });
        }
        println!(
            "[attempt {}] email to {}: {}",
            ctx.attempt, payload.to, payload.subject
        );
        self.sent.fetch_add(1, Ordering::SeqCst);
        Ok(JobOutcome::Complete)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GenerateReport {
    report_id: u64,
}

impl JobPayload for GenerateReport {
    const PAYLOAD_VERSION: u16 = 1;
}

struct ReportHandler {
    tries: Arc<AtomicU32>,
}

#[async_trait::async_trait]
impl JobHandler for ReportHandler {
    type Payload = GenerateReport;

    fn kind(&self) -> &'static str {
        "generate_report"
    }

    async fn handle(
        &self,
        ctx: JobContext,
        payload: GenerateReport,
    ) -> Result<JobOutcome, WorkerError> {
        let attempt = self.tries.fetch_add(1, Ordering::SeqCst) + 1;
        if attempt < 2 {
            return Err(WorkerError::retriable("report store not ready yet"));
        }
        println!("[attempt {}] report {} ready", ctx.attempt, payload.report_id);
        Ok(JobOutcome::Complete)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let sent = Arc::new(AtomicU32::new(0));
    let dead = Arc::new(AtomicU32::new(0));
    let tries = Arc::new(AtomicU32::new(0));

    let registry = Arc::new(
        WorkerRegistry::builder()
            .register(EmailHandler {
                sent: sent.clone(),
                dead: dead.clone(),
            })
            .expect("unique kind")
            .register(ReportHandler {
                tries: tries.clone(),
            })
            .expect("unique kind")
            .build(),
    );

    // Keep the concrete handle so the DLQ can be inspected after shutdown.
    let memory = Arc::new(MemoryQueue::new());
    let backend: Arc<dyn QueueBackend> = memory.clone();
    let queue = QueueName::new("default");

    for id in 1..=3 {
        let job = NewJob::encode::<SendEmail>(
            "send_email",
            "default",
            &SendEmail {
                to: format!("user{id}@example.com"),
                subject: "Welcome".into(),
                poison: false,
            },
        )
        .expect("encode payload");
        backend.enqueue(job).await.expect("enqueue");
    }
    let poison = NewJob::encode::<SendEmail>(
        "send_email",
        "default",
        &SendEmail {
            to: "bad@@address".into(),
            subject: "Welcome".into(),
            poison: true,
        },
    )
    .expect("encode payload");
    backend.enqueue(poison).await.expect("enqueue");

    let report = NewJob::encode::<GenerateReport>(
        "generate_report",
        "default",
        &GenerateReport { report_id: 42 },
    )
    .expect("encode payload");
    backend.enqueue(report).await.expect("enqueue");

    let config = Arc::new(RuntimeConfig {
        poll_interval: Duration::from_millis(10),
        retry_policy: RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            backoff_factor: 2,
            max_delay: Duration::from_millis(100),
            jitter: false,
            max_age: Some(Duration::from_secs(60)),
        },
        ..RuntimeConfig::default()
    });

    let shutdown = Shutdown::new();
    let pool = WorkerPool::start(
        backend.clone(),
        registry,
        config,
        shutdown.clone(),
        queue,
        2,
    );

    for _ in 0..500 {
        if sent.load(Ordering::SeqCst) >= 3 && dead.load(Ordering::SeqCst) >= 1 && tries.load(Ordering::SeqCst) >= 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    shutdown.trigger();
    pool.join_with_timeout(Duration::from_secs(2)).await;

    let letters = memory.dead_letters();
    println!(
        "done: {} emails sent, {} report attempts, {} dead-lettered",
        sent.load(Ordering::SeqCst),
        tries.load(Ordering::SeqCst),
        letters.len()
    );
    for letter in &letters {
        println!(
            "  dlq: kind={} reason={}",
            letter.envelope.kind,
            letter.reason.as_str()
        );
    }
}
