//! Bounded CPU-bound execution.
//!
//! CPU-bound handlers must not starve the HTTP/async path. They run on
//! `tokio::task::spawn_blocking` gated by a fixed-size semaphore (not `rayon`: an
//! unjustified dependency for independent per-job work, per ADR-0013). The semaphore is
//! the admission/backpressure mechanism — only `max_workers` CPU jobs run at once.

use std::sync::Arc;

use tokio::sync::Semaphore;

/// A bounded pool for CPU-bound work backed by `spawn_blocking` and a semaphore.
#[derive(Clone)]
pub struct CpuPool {
    semaphore: Arc<Semaphore>,
}

impl CpuPool {
    /// Creates a pool admitting at most `max_workers` concurrent CPU jobs.
    pub fn new(max_workers: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_workers.max(1))),
        }
    }

    /// Number of permits currently available (free CPU slots).
    pub fn available(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Runs a blocking closure on the blocking pool once a permit is acquired.
    ///
    /// The permit (backpressure) is held for the duration of the closure and released
    /// when it returns. Panics inside `f` propagate as a join error.
    pub async fn run<F, R>(&self, f: F) -> Result<R, tokio::task::JoinError>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        // Acquire an owned permit so it can move into the spawned task and release on
        // completion.
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("cpu pool semaphore never closed");
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            f()
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn runs_blocking_work() {
        let pool = CpuPool::new(2);
        let r = pool.run(|| 21 * 2).await.unwrap();
        assert_eq!(r, 42);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn bounds_concurrency() {
        let pool = CpuPool::new(2);
        let live = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..8 {
            let pool = pool.clone();
            let live = live.clone();
            let peak = peak.clone();
            handles.push(tokio::spawn(async move {
                pool.run(move || {
                    let now = live.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(now, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    live.fetch_sub(1, Ordering::SeqCst);
                })
                .await
                .unwrap();
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
        assert!(
            peak.load(Ordering::SeqCst) <= 2,
            "never more than max_workers CPU jobs at once"
        );
    }
}
