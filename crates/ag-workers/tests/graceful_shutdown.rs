//! A static worker pool drains queued jobs and exits cleanly on shutdown.

mod common;

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use ag_workers::{
    MemoryQueue, QueueBackend, QueueName, RuntimeConfig, Shutdown, WorkerPool, WorkerRegistry,
};

use common::{unit_job, CountingHandler};

#[tokio::test]
async fn pool_processes_jobs_then_shuts_down() {
    let handler = CountingHandler::new("work");
    let calls = handler.counter();
    let registry = Arc::new(WorkerRegistry::builder().register(handler).unwrap().build());

    let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());
    let config = Arc::new(RuntimeConfig {
        poll_interval: Duration::from_millis(2),
        ..RuntimeConfig::default()
    });
    let queue = QueueName::new("default");

    const N: u32 = 20;
    for _ in 0..N {
        backend.enqueue(unit_job("work", "default")).await.unwrap();
    }

    let shutdown = Shutdown::new();
    let pool = WorkerPool::start(
        backend.clone(),
        registry,
        config,
        shutdown.clone(),
        queue.clone(),
        4,
    );

    // Wait until all jobs are processed (bounded).
    for _ in 0..200 {
        if calls.load(Ordering::SeqCst) >= N {
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    shutdown.trigger();
    let joined = pool.join_with_timeout(Duration::from_secs(2)).await;

    assert!(joined, "workers must exit promptly after shutdown");
    assert_eq!(calls.load(Ordering::SeqCst), N, "all jobs processed");
    let depth = backend.depth(&queue).await.unwrap();
    assert_eq!(depth.ready, 0);
    assert_eq!(depth.leased, 0);
}

#[tokio::test]
async fn shutdown_stops_idle_workers() {
    let registry = Arc::new(
        WorkerRegistry::builder()
            .register(CountingHandler::new("noop"))
            .unwrap()
            .build(),
    );
    let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());
    let config = Arc::new(RuntimeConfig::default());
    let shutdown = Shutdown::new();
    let pool = WorkerPool::start(
        backend,
        registry,
        config,
        shutdown.clone(),
        QueueName::new("default"),
        2,
    );
    // Idle workers (no jobs) must still exit promptly on shutdown.
    shutdown.trigger();
    assert!(pool.join_with_timeout(Duration::from_secs(2)).await);
}
