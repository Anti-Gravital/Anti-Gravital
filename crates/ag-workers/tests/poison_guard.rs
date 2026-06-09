//! The poison guard must convert a crash-looping job into a bounded DLQ entry without
//! ever executing it again once the lease count exceeds the guard threshold.

mod common;

use std::sync::atomic::Ordering;
use std::time::Duration;

use ag_workers::{
    execute_leased_job, DeadLetterReason, ExecResult, MemoryQueue, QueueBackend, QueueName,
    RuntimeConfig, Shutdown, WorkerId, WorkerRegistry,
};

use common::{unit_job, CountingHandler};

#[tokio::test]
async fn poison_job_is_guarded_into_dlq_without_executing() {
    // A job that has been leased more than `poison_guard_attempts` times (as if the
    // process aborted each time before producing an outcome) must be dead-lettered
    // before the handler runs.
    let handler = CountingHandler::new("poison");
    let calls = handler.counter();
    let registry = WorkerRegistry::builder().register(handler).unwrap().build();

    let config = RuntimeConfig {
        poison_guard_attempts: 2,
        ..RuntimeConfig::default()
    };

    // Tiny lease timeout so re-leasing reclaims the job (simulating crash + restart).
    let queue = QueueName::new("default");
    let backend = MemoryQueue::new().with_lease_timeout(Duration::from_millis(1));
    let worker = WorkerId::generate();
    backend
        .enqueue(unit_job("poison", "default"))
        .await
        .unwrap();

    // Lease three times without ever producing an outcome: attempts climbs to 3 (> 2).
    let mut last = None;
    for _ in 0..3 {
        tokio::time::sleep(Duration::from_millis(3)).await;
        let mut leased = backend.lease(&queue, &worker, 1).await.unwrap();
        assert_eq!(leased.len(), 1, "expired lease should be reclaimable");
        last = leased.pop();
    }
    let env = last.unwrap();
    assert_eq!(env.attempts, 3);

    let shutdown = Shutdown::new();
    let result = execute_leased_job(&backend, &registry, &config, &shutdown, env)
        .await
        .unwrap();

    assert_eq!(
        result,
        ExecResult::DeadLettered(DeadLetterReason::PoisonGuard)
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "the poison job must never execute"
    );
    let dead = backend.dead_letters();
    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].reason, DeadLetterReason::PoisonGuard);
}
