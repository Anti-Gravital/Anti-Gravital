//! Exercises every dispatch outcome/error arm of `execute_leased_job` on the memory
//! backend, so each classification path is covered.

mod common;

use std::time::Duration;

use ag_workers::{
    execute_leased_job, DeadLetterReason, ExecResult, JobContext, JobHandler, JobOutcome,
    MemoryQueue, NewJob, QueueBackend, QueueName, RetryPolicy, RuntimeConfig, Shutdown,
    WorkerError, WorkerRegistry,
};

use common::{unit_job, Unit};

// A handler whose behavior is chosen by kind, so one registry covers every arm.
struct Behaving(&'static str);

#[async_trait::async_trait]
impl JobHandler for Behaving {
    type Payload = Unit;
    fn kind(&self) -> &'static str {
        self.0
    }
    async fn handle(&self, _ctx: JobContext, _p: Unit) -> Result<JobOutcome, WorkerError> {
        match self.0 {
            "discard" => Ok(JobOutcome::Discard {
                reason: "nothing to do".into(),
            }),
            "deadletter" => Ok(JobOutcome::DeadLetter {
                reason: "by request".into(),
            }),
            "permanent" => Err(WorkerError::permanent("nope")),
            "cancelled" => Err(WorkerError::cancelled("shutting down")),
            "invalid" => Err(WorkerError::invalid_payload("bad shape")),
            "dependency" => Err(WorkerError::dependency_unavailable("db down")),
            "slow" => {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok(JobOutcome::Complete)
            }
            _ => Ok(JobOutcome::Complete),
        }
    }
}

fn registry() -> WorkerRegistry {
    let mut b = WorkerRegistry::builder();
    for kind in [
        "discard",
        "deadletter",
        "permanent",
        "cancelled",
        "invalid",
        "dependency",
        "slow",
        "expire",
    ] {
        b = b.register(Behaving(kind)).unwrap();
    }
    b.build()
}

async fn run(kind: &str, job: NewJob, config: &RuntimeConfig) -> (ExecResult, MemoryQueue) {
    let backend = MemoryQueue::new();
    let registry = registry();
    let worker = ag_workers::WorkerId::generate();
    let queue = QueueName::new("default");
    backend.enqueue(job).await.unwrap();
    let mut leased = backend.lease(&queue, &worker, 1).await.unwrap();
    assert_eq!(leased.len(), 1, "kind {kind} should lease");
    let shutdown = Shutdown::new();
    let res = execute_leased_job(
        &backend,
        &registry,
        config,
        &shutdown,
        leased.pop().unwrap(),
    )
    .await
    .unwrap();
    (res, backend)
}

#[tokio::test]
async fn discard_is_acked() {
    let (res, backend) = run(
        "discard",
        unit_job("discard", "default"),
        &RuntimeConfig::default(),
    )
    .await;
    assert_eq!(res, ExecResult::Discarded);
    assert_eq!(backend.job_count(), 0);
}

#[tokio::test]
async fn requested_dead_letter() {
    let (res, backend) = run(
        "deadletter",
        unit_job("deadletter", "default"),
        &RuntimeConfig::default(),
    )
    .await;
    assert!(matches!(
        res,
        ExecResult::DeadLettered(DeadLetterReason::Requested(_))
    ));
    assert_eq!(backend.dead_letters().len(), 1);
}

#[tokio::test]
async fn permanent_error_dead_letters() {
    let (res, backend) = run(
        "permanent",
        unit_job("permanent", "default"),
        &RuntimeConfig::default(),
    )
    .await;
    assert!(matches!(
        res,
        ExecResult::DeadLettered(DeadLetterReason::Permanent(_))
    ));
    assert_eq!(backend.dead_letters().len(), 1);
}

#[tokio::test]
async fn invalid_payload_from_handler_dead_letters() {
    let (res, _b) = run(
        "invalid",
        unit_job("invalid", "default"),
        &RuntimeConfig::default(),
    )
    .await;
    assert!(matches!(
        res,
        ExecResult::DeadLettered(DeadLetterReason::InvalidPayload(_))
    ));
}

#[tokio::test]
async fn cancelled_is_released() {
    let (res, _b) = run(
        "cancelled",
        unit_job("cancelled", "default"),
        &RuntimeConfig::default(),
    )
    .await;
    assert_eq!(res, ExecResult::Released);
}

#[tokio::test]
async fn dependency_unavailable_retries() {
    let (res, _b) = run(
        "dependency",
        unit_job("dependency", "default"),
        &RuntimeConfig::default(),
    )
    .await;
    assert_eq!(res, ExecResult::Retried);
}

#[tokio::test]
async fn timeout_is_retried() {
    // Per-job timeout of 1ms; the handler sleeps 50ms, so it times out and is retried.
    let job = unit_job("slow", "default").with_timeout(Duration::from_millis(1));
    let (res, _b) = run("slow", job, &RuntimeConfig::default()).await;
    assert_eq!(res, ExecResult::Retried);
}

#[tokio::test]
async fn expired_job_dead_letters() {
    // max_age of 1ms; wait past it before executing so the job is expired.
    let config = RuntimeConfig {
        retry_policy: RetryPolicy {
            max_age: Some(Duration::from_millis(1)),
            ..RetryPolicy::default()
        },
        ..RuntimeConfig::default()
    };
    let backend = MemoryQueue::new();
    let registry = registry();
    let worker = ag_workers::WorkerId::generate();
    let queue = QueueName::new("default");
    backend
        .enqueue(unit_job("expire", "default"))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    let mut leased = backend.lease(&queue, &worker, 1).await.unwrap();
    let res = execute_leased_job(
        &backend,
        &registry,
        &config,
        &Shutdown::new(),
        leased.pop().unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(res, ExecResult::DeadLettered(DeadLetterReason::Expired));
}

#[tokio::test]
async fn explicit_retry_after_delay() {
    // A handler returning Retry { After } reschedules rather than acking.
    struct RetryAfter;
    #[async_trait::async_trait]
    impl JobHandler for RetryAfter {
        type Payload = Unit;
        fn kind(&self) -> &'static str {
            "retry_after"
        }
        async fn handle(&self, _c: JobContext, _p: Unit) -> Result<JobOutcome, WorkerError> {
            Ok(JobOutcome::retry_after(Duration::from_millis(5)))
        }
    }
    let backend = MemoryQueue::new();
    let registry = WorkerRegistry::builder()
        .register(RetryAfter)
        .unwrap()
        .build();
    let worker = ag_workers::WorkerId::generate();
    let queue = QueueName::new("default");
    backend
        .enqueue(unit_job("retry_after", "default"))
        .await
        .unwrap();
    let mut leased = backend.lease(&queue, &worker, 1).await.unwrap();
    let res = execute_leased_job(
        &backend,
        &registry,
        &RuntimeConfig::default(),
        &Shutdown::new(),
        leased.pop().unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(res, ExecResult::Retried);
}
