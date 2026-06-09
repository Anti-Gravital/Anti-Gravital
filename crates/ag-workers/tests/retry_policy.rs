//! A retriable handler is retried with backoff up to the cap, then dead-lettered as
//! `AttemptsExhausted` — never retried forever.

mod common;

use std::sync::atomic::Ordering;
use std::time::Duration;

use ag_workers::{
    execute_leased_job, DeadLetterReason, ExecResult, MemoryQueue, QueueBackend, QueueName,
    RetryPolicy, RuntimeConfig, Shutdown, WorkerId, WorkerRegistry,
};

use common::{unit_job, AlwaysRetriable};

#[tokio::test]
async fn retriable_job_is_capped_then_dead_lettered() {
    let handler = AlwaysRetriable::new("flaky");
    let calls = handler.counter();
    let registry = WorkerRegistry::builder().register(handler).unwrap().build();

    let config = RuntimeConfig {
        poison_guard_attempts: 3,
        poll_interval: Duration::from_millis(1),
        lease_batch: 5,
        retry_policy: RetryPolicy {
            max_attempts: 10,
            base_delay: Duration::from_millis(1),
            backoff_factor: 2,
            max_delay: Duration::from_millis(8),
            jitter: false,
            max_age: None,
        },
    };

    let queue = QueueName::new("default");
    let backend = MemoryQueue::new();
    let worker = WorkerId::generate();
    backend.enqueue(unit_job("flaky", "default")).await.unwrap();

    // Drive the job to a terminal state with a bounded manual loop.
    let shutdown = Shutdown::new();
    let mut last = ExecResult::Retried;
    for _ in 0..100 {
        let leased = backend.lease(&queue, &worker, 5).await.unwrap();
        if leased.is_empty() {
            if !backend.dead_letters().is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
            continue;
        }
        for env in leased {
            last = execute_leased_job(&backend, &registry, &config, &shutdown, env)
                .await
                .unwrap();
        }
    }

    // Effective cap = min(poison_guard_attempts=3, max_attempts=10) = 3 executions.
    assert_eq!(calls.load(Ordering::SeqCst), 3, "exactly 3 attempts ran");
    assert_eq!(
        last,
        ExecResult::DeadLettered(DeadLetterReason::AttemptsExhausted)
    );
    let dead = backend.dead_letters();
    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].reason, DeadLetterReason::AttemptsExhausted);
    assert_eq!(dead[0].envelope.attempts, 3);
}
