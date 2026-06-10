//! Static worker pools.
//!
//! A [`WorkerPool`] runs a fixed number of long-lived workers that lease from one queue
//! and dispatch jobs through [`execute_leased_job`].
//! Workers stop leasing as soon as shutdown is triggered and exit once their in-flight
//! job finishes (graceful shutdown never loses a leased durable job: it becomes
//! leasable again after its lease expires).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::config::RuntimeConfig;
use crate::ids::{QueueName, WorkerId};
use crate::queue::QueueBackend;
use crate::registry::WorkerRegistry;
use crate::runtime::execute_leased_job;
use crate::scaling::ScaleConfig;
use crate::shutdown::Shutdown;

/// A fixed-size set of workers leasing from a single queue.
pub struct WorkerPool {
    handles: Vec<JoinHandle<()>>,
}

impl WorkerPool {
    /// Starts `concurrency` workers leasing from `queue`.
    pub fn start(
        backend: Arc<dyn QueueBackend>,
        registry: Arc<WorkerRegistry>,
        config: Arc<RuntimeConfig>,
        shutdown: Shutdown,
        queue: QueueName,
        concurrency: usize,
    ) -> Self {
        let mut handles = Vec::with_capacity(concurrency);
        for _ in 0..concurrency {
            let backend = backend.clone();
            let registry = registry.clone();
            let config = config.clone();
            let shutdown = shutdown.clone();
            let queue = queue.clone();
            handles.push(tokio::spawn(worker_loop(
                backend, registry, config, shutdown, queue,
            )));
        }
        Self { handles }
    }

    /// Waits for all workers to exit.
    pub async fn join(self) {
        for handle in self.handles {
            let _ = handle.await;
        }
    }

    /// Waits for all workers to exit, up to `timeout`. Returns `true` if they all
    /// finished in time.
    pub async fn join_with_timeout(self, timeout: std::time::Duration) -> bool {
        let join_all = async {
            for handle in self.handles {
                let _ = handle.await;
            }
        };
        tokio::time::timeout(timeout, join_all).await.is_ok()
    }
}

/// A dynamic worker pool that scales the number of *actively leasing* workers between
/// `min_workers` and `max_workers` based on observed queue depth.
///
/// All `max_workers` tasks are spawned up front; a shared `desired` count (recomputed by
/// a controller from [`ScaleConfig::desired_workers`]) gates how many actually lease — a
/// worker whose index is at or above `desired` parks instead of leasing. This scales both
/// up and down without spawning/killing tasks.
pub struct DynamicPool {
    handles: Vec<JoinHandle<()>>,
    controller: JoinHandle<()>,
}

impl DynamicPool {
    /// Starts a dynamic pool for `queue`.
    pub fn start(
        backend: Arc<dyn QueueBackend>,
        registry: Arc<WorkerRegistry>,
        config: Arc<RuntimeConfig>,
        scale: ScaleConfig,
        shutdown: Shutdown,
        queue: QueueName,
    ) -> Self {
        let desired = Arc::new(AtomicUsize::new(scale.min_workers.max(1)));
        let max = scale.max_workers.max(scale.min_workers).max(1);

        let mut handles = Vec::with_capacity(max);
        for index in 0..max {
            handles.push(tokio::spawn(dynamic_worker_loop(
                backend.clone(),
                registry.clone(),
                config.clone(),
                shutdown.clone(),
                queue.clone(),
                index,
                desired.clone(),
            )));
        }

        let controller = tokio::spawn(controller_loop(
            backend.clone(),
            config.clone(),
            scale,
            shutdown.clone(),
            queue,
            desired,
        ));

        Self {
            handles,
            controller,
        }
    }

    /// Waits for all workers and the controller to exit, up to `timeout`.
    pub async fn join_with_timeout(self, timeout: std::time::Duration) -> bool {
        let join_all = async {
            for handle in self.handles {
                let _ = handle.await;
            }
            let _ = self.controller.await;
        };
        tokio::time::timeout(timeout, join_all).await.is_ok()
    }
}

async fn controller_loop(
    backend: Arc<dyn QueueBackend>,
    config: Arc<RuntimeConfig>,
    scale: ScaleConfig,
    shutdown: Shutdown,
    queue: QueueName,
    desired: Arc<AtomicUsize>,
) {
    // Sample at the poll interval for responsiveness; `cooldown` bounds how often the
    // desired count is actually changed.
    let sample = config.poll_interval;
    let mut last_change = tokio::time::Instant::now()
        .checked_sub(scale.cooldown)
        .unwrap_or_else(tokio::time::Instant::now);
    while !shutdown.is_triggered() {
        if let Ok(depth) = backend.depth(&queue).await {
            let want = scale.desired_workers(depth.ready);
            if want != desired.load(Ordering::SeqCst) && last_change.elapsed() >= scale.cooldown {
                desired.store(want, Ordering::SeqCst);
                last_change = tokio::time::Instant::now();
            }
        }
        tokio::select! {
            biased;
            _ = shutdown.wait() => break,
            _ = tokio::time::sleep(sample) => {}
        }
    }
}

async fn dynamic_worker_loop(
    backend: Arc<dyn QueueBackend>,
    registry: Arc<WorkerRegistry>,
    config: Arc<RuntimeConfig>,
    shutdown: Shutdown,
    queue: QueueName,
    index: usize,
    desired: Arc<AtomicUsize>,
) {
    let worker_id = WorkerId::generate();
    while !shutdown.is_triggered() {
        // Park if this worker is above the desired active count.
        if index >= desired.load(Ordering::SeqCst) {
            tokio::select! {
                biased;
                _ = shutdown.wait() => break,
                _ = tokio::time::sleep(config.poll_interval) => continue,
            }
        }

        let leased = match backend.lease(&queue, &worker_id, config.lease_batch).await {
            Ok(jobs) => jobs,
            Err(e) => {
                tracing::error!(queue = %queue, error = %e, "lease failed");
                Vec::new()
            }
        };
        if leased.is_empty() {
            tokio::select! {
                biased;
                _ = shutdown.wait() => break,
                _ = tokio::time::sleep(config.poll_interval) => {}
            }
            continue;
        }
        for env in leased {
            let job_id = env.id;
            if let Err(e) =
                execute_leased_job(backend.as_ref(), &registry, &config, &shutdown, env).await
            {
                tracing::error!(queue = %queue, job_id = %job_id, error = %e, "dispatch failed");
            }
            if shutdown.is_triggered() {
                break;
            }
        }
    }
}

async fn worker_loop(
    backend: Arc<dyn QueueBackend>,
    registry: Arc<WorkerRegistry>,
    config: Arc<RuntimeConfig>,
    shutdown: Shutdown,
    queue: QueueName,
) {
    let worker_id = WorkerId::generate();
    while !shutdown.is_triggered() {
        let leased = match backend.lease(&queue, &worker_id, config.lease_batch).await {
            Ok(jobs) => jobs,
            Err(e) => {
                tracing::error!(queue = %queue, error = %e, "lease failed");
                Vec::new()
            }
        };

        if leased.is_empty() {
            // Nothing ready: wait for the poll interval or an earlier shutdown.
            tokio::select! {
                biased;
                _ = shutdown.wait() => break,
                _ = tokio::time::sleep(config.poll_interval) => {}
            }
            continue;
        }

        for env in leased {
            let job_id = env.id;
            if let Err(e) =
                execute_leased_job(backend.as_ref(), &registry, &config, &shutdown, env).await
            {
                tracing::error!(queue = %queue, job_id = %job_id, error = %e, "dispatch failed");
            }
            // Stop pulling more work once shutdown is requested; the current batch is
            // allowed to finish.
            if shutdown.is_triggered() {
                break;
            }
        }
    }
}
