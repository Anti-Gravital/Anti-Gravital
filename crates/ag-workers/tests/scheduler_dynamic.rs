//! Scheduler (memory, singleton) enqueues on tick; the dynamic pool drains a burst.

mod common;

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use ag_workers::{
    run_scheduler, DynamicPool, MemoryQueue, MemoryScheduleStore, QueueBackend, QueueName,
    RuntimeConfig, ScaleConfig, Schedule, Shutdown, WorkerRegistry,
};

use common::{unit_job, CountingHandler, Unit};

#[tokio::test]
async fn scheduler_enqueues_interval_jobs() {
    let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());
    let store = MemoryScheduleStore::new();
    store.register(Schedule::every(
        "tick",
        "default",
        ag_workers::encode::<Unit>(&Unit {}).unwrap(),
        1,
        Duration::from_millis(10),
        chrono::Utc::now(),
    ));

    let shutdown = Shutdown::new();
    let runner = tokio::spawn(run_scheduler(
        store,
        backend.clone(),
        Duration::from_millis(5),
        shutdown.clone(),
    ));

    // Let several intervals elapse, then stop the scheduler.
    tokio::time::sleep(Duration::from_millis(60)).await;
    shutdown.trigger();
    let _ = runner.await;

    let depth = backend.depth(&QueueName::new("default")).await.unwrap();
    assert!(
        depth.ready >= 1,
        "scheduler enqueued at least one interval job"
    );
}

#[tokio::test]
async fn dynamic_pool_drains_a_burst() {
    let handler = CountingHandler::new("burst");
    let calls = handler.counter();
    let registry = Arc::new(WorkerRegistry::builder().register(handler).unwrap().build());
    let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());
    let config = Arc::new(RuntimeConfig {
        poll_interval: Duration::from_millis(2),
        ..RuntimeConfig::default()
    });
    let scale = ScaleConfig {
        min_workers: 1,
        max_workers: 4,
        queue_depth_per_worker: 5,
        cooldown: Duration::from_millis(5),
    };
    let queue = QueueName::new("default");

    const N: u32 = 40;
    for _ in 0..N {
        backend.enqueue(unit_job("burst", "default")).await.unwrap();
    }

    let shutdown = Shutdown::new();
    let pool = DynamicPool::start(
        backend.clone(),
        registry,
        config,
        scale,
        shutdown.clone(),
        queue.clone(),
    );

    for _ in 0..200 {
        if calls.load(Ordering::SeqCst) >= N {
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    shutdown.trigger();
    assert!(pool.join_with_timeout(Duration::from_secs(2)).await);
    assert_eq!(calls.load(Ordering::SeqCst), N, "burst fully drained");
}
