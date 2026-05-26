//! Asynchronous mail queue with retries and exponential backoff.
//!
//! `MailQueue` is the trait that every queue implementation must satisfy.
//! `InMemoryQueue` is the default backend: a Tokio channel with a worker
//! that processes messages applying exponential backoff on failure.
//!
//! The queue does not guarantee durability across process restarts. For
//! durability, use `queue-persistent` (optional feature, future Stage)
//! that persists the jobs in `ag-data`.

#[cfg(feature = "queue-persistent")]
pub mod store;

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::{error::AgMailError, message::Email, metrics, sender::MailSender};

/// Configuration of the retry behavior.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retries (not counting the initial attempt).
    pub max_retries: u32,
    /// Base delay for exponential backoff.
    pub base_delay: Duration,
    /// Backoff multiplier (raised to the power of the retry number).
    pub backoff_factor: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(2),
            backoff_factor: 2,
        }
    }
}

impl RetryPolicy {
    /// Computes the delay for retry `n` (0-based).
    pub fn delay_for(&self, attempt: u32) -> Duration {
        let multiplier = self.backoff_factor.saturating_pow(attempt) as u64;
        self.base_delay.saturating_mul(multiplier as u32)
    }
}

/// Abstraction over a mail queue.
#[async_trait]
pub trait MailQueue: Send + Sync {
    /// Enqueues an email for asynchronous sending.
    ///
    /// Returns `Ok(())` when the email has been accepted into the queue.
    /// The actual send happens in the background.
    async fn enqueue(&self, email: Email) -> Result<(), AgMailError>;
}

// Internal channel message.
struct QueueItem {
    email: Email,
    attempt: u32,
}

/// In-memory queue backed by a Tokio channel and an asynchronous worker.
///
/// The worker is spawned as a Tokio task when the queue is built and
/// finishes when all the channel senders disconnect (i.e., when the queue
/// is dropped).
pub struct InMemoryQueue {
    tx: mpsc::Sender<QueueItem>,
}

impl InMemoryQueue {
    /// Creates the queue with a sender and the given retry policy.
    ///
    /// `capacity` is the maximum number of pending messages in the channel
    /// before `enqueue` blocks the caller (backpressure).
    pub fn new(sender: Arc<dyn MailSender>, policy: RetryPolicy, capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        tokio::spawn(worker(rx, sender, policy));
        Self { tx }
    }
}

#[async_trait]
impl MailQueue for InMemoryQueue {
    async fn enqueue(&self, email: Email) -> Result<(), AgMailError> {
        self.tx
            .send(QueueItem { email, attempt: 0 })
            .await
            .map_err(|_| AgMailError::SendExhausted("cola cerrada".to_owned()))
    }
}

// Worker that consumes the channel and re-enqueues with backoff on failure.
async fn worker(
    mut rx: mpsc::Receiver<QueueItem>,
    sender: Arc<dyn MailSender>,
    policy: RetryPolicy,
) {
    // We need to re-enqueue without the consumed rx, so we use a secondary
    // channel for retries.
    let (retry_tx, mut retry_rx) = mpsc::channel::<QueueItem>(64);

    loop {
        // Selects the next item, prioritizing retries over new ones.
        let item = tokio::select! {
            biased;
            Some(item) = retry_rx.recv() => item,
            Some(item) = rx.recv() => item,
            else => break,
        };

        let provider = sender.name();
        let start = std::time::Instant::now();

        match sender.send(&item.email).await {
            Ok(_) => {
                let elapsed = start.elapsed().as_secs_f64();
                info!(provider, "correo enviado");
                metrics::record_send(provider, true);
                metrics::record_send_latency(provider, elapsed);
            }
            Err(e) => {
                metrics::record_send(provider, false);

                if item.attempt < policy.max_retries {
                    let delay = policy.delay_for(item.attempt);
                    warn!(
                        provider,
                        attempt = item.attempt + 1,
                        max = policy.max_retries,
                        delay_ms = delay.as_millis(),
                        error = %e,
                        "fallo de envio, reintentando"
                    );
                    metrics::record_retry(provider);

                    let retry_tx = retry_tx.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(delay).await;
                        let _ = retry_tx
                            .send(QueueItem {
                                email: item.email,
                                attempt: item.attempt + 1,
                            })
                            .await;
                    });
                } else {
                    error!(
                        provider,
                        attempts = item.attempt + 1,
                        error = %e,
                        "correo descartado tras agotar reintentos"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{Address, EmailBuilder};
    use crate::sender::test_utils::NullSender;

    fn test_email() -> Email {
        EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .to(Address::new("t@t.com"))
            .subject("test")
            .text_body("body")
            .build()
            .unwrap()
    }

    #[test]
    fn retry_policy_delay_grows_exponentially() {
        let policy = RetryPolicy {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            backoff_factor: 2,
        };
        assert_eq!(policy.delay_for(0), Duration::from_millis(100));
        assert_eq!(policy.delay_for(1), Duration::from_millis(200));
        assert_eq!(policy.delay_for(2), Duration::from_millis(400));
    }

    #[tokio::test]
    async fn enqueue_accepts_email() {
        let sender = Arc::new(NullSender::new());
        let queue = InMemoryQueue::new(sender.clone(), RetryPolicy::default(), 16);
        queue.enqueue(test_email()).await.unwrap();
        // Small wait so the worker processes it.
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(sender.emails_sent(), 1);
    }

    #[tokio::test]
    async fn enqueue_multiple_emails() {
        let sender = Arc::new(NullSender::new());
        let queue = InMemoryQueue::new(sender.clone(), RetryPolicy::default(), 16);
        for _ in 0..5 {
            queue.enqueue(test_email()).await.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(sender.emails_sent(), 5);
    }
}
