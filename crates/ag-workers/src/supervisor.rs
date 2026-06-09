//! Static worker pools.
//!
//! A [`WorkerPool`] runs a fixed number of long-lived workers that lease from one queue
//! and dispatch jobs through [`execute_leased_job`](crate::runtime::execute_leased_job).
//! Workers stop leasing as soon as shutdown is triggered and exit once their in-flight
//! job finishes (graceful shutdown never loses a leased durable job: it becomes
//! leasable again after its lease expires).

use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::config::RuntimeConfig;
use crate::ids::{QueueName, WorkerId};
use crate::queue::QueueBackend;
use crate::registry::WorkerRegistry;
use crate::runtime::execute_leased_job;
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
