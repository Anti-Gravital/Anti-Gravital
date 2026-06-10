//! Adapter that routes generic mail-delivery scheduling through `ag-workers`.
//!
//! Enabled by the `workers` feature (RFC-0012 S7/M2). Only the *generic* queue
//! mechanics — enqueue, retry/backoff, dead-lettering and (with the shared
//! PostgreSQL backend) durability — move to the `ag-workers` substrate. The
//! *mail-specific* delivery path (SMTP/MTA/DKIM/bounce) stays behind
//! [`MailSender`]; this module merely binds an [`Email`] to the worker payload
//! contract and adapts a [`MailSender`] to the [`JobHandler`] contract.
//!
//! This is the migration that retires the duplication between `ag-mail`'s own
//! persistent queue (`crate::queue::store`, feature `queue-persistent`) and
//! `ag-workers`' PostgreSQL backend (see RFC-0012 S7/M3-M4).
//!
//! ```no_run
//! # #[cfg(feature = "workers")]
//! # async fn demo() -> Result<(), Box<dyn std::error::Error>> {
//! use std::sync::Arc;
//! use ag_mail::queue::MailQueue;
//! use ag_mail::workers::{mail_delivery_registry, WorkersMailQueue};
//! use ag_workers::{MemoryQueue, QueueBackend, RuntimeConfig, Shutdown, QueueName, WorkerPool};
//!
//! // Build the shared backend and wire the mail-delivery handler.
//! let sender: Arc<dyn ag_mail::sender::MailSender> = unimplemented!();
//! let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());
//! let registry = Arc::new(mail_delivery_registry(sender)?);
//!
//! // Producers enqueue through the MailQueue adapter; a worker pool delivers.
//! let queue = WorkersMailQueue::new(backend.clone());
//! queue.enqueue(unimplemented!()).await?;
//!
//! let shutdown = Shutdown::new();
//! let _pool = WorkerPool::start(
//!     backend,
//!     registry,
//!     Arc::new(RuntimeConfig::default()),
//!     shutdown,
//!     QueueName::new("mail"),
//!     2,
//! );
//! # Ok(()) }
//! ```

use std::sync::Arc;

use ag_workers::{
    JobContext, JobHandler, JobOutcome, JobPayload, NewJob, QueueBackend, QueueName, RegistryError,
    WorkerError, WorkerRegistry,
};
use async_trait::async_trait;

use crate::{error::AgMailError, message::Email, queue::MailQueue, sender::MailSender};

/// Stable job kind under which a generic mail-delivery job is registered and
/// stored on the shared `ag-workers` queue. Mail payloads become this `kind` in
/// `ag_worker_jobs` (RFC-0012 S7/M3).
pub const MAIL_DELIVERY_KIND: &str = "mail.delivery";

/// Default queue name mail-delivery jobs are enqueued onto.
pub const MAIL_QUEUE: &str = "mail";

/// [`Email`] is the typed payload of a mail-delivery job.
///
/// `Email` already derives `Serialize`/`Deserialize`; this binds it to the
/// `ag-workers` payload contract with a stable version, so a job enqueued before
/// a redeploy can still be decoded — or safely dead-lettered — after it.
impl JobPayload for Email {
    const PAYLOAD_VERSION: u16 = 1;
}

/// A [`JobHandler`] that delivers one queued [`Email`] via the configured
/// [`MailSender`].
///
/// The handler holds an injected sender (never a global) and classifies failures
/// for the worker retry taxonomy: transient transport/infrastructure faults are
/// retriable; caller/content/configuration faults are permanent and dead-lettered.
pub struct MailDeliveryHandler {
    sender: Arc<dyn MailSender>,
}

impl MailDeliveryHandler {
    /// Builds the handler over the given sender.
    pub fn new(sender: Arc<dyn MailSender>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl JobHandler for MailDeliveryHandler {
    type Payload = Email;

    fn kind(&self) -> &'static str {
        MAIL_DELIVERY_KIND
    }

    async fn handle(&self, _ctx: JobContext, payload: Email) -> Result<JobOutcome, WorkerError> {
        match self.sender.send(&payload).await {
            Ok(_) => Ok(JobOutcome::Complete),
            Err(err) => Err(classify(&err)),
        }
    }
}

/// Maps an [`AgMailError`] onto the worker retry taxonomy.
///
/// Transient transport/infrastructure failures (`Provider`, `Dns`, `Queue`) obey
/// the backoff policy. Caller/content/configuration faults never succeed on
/// retry and are dead-lettered: a bad address, a broken template, a missing
/// destination mail host or a DKIM key fault will not heal by retrying.
fn classify(err: &AgMailError) -> WorkerError {
    match err {
        AgMailError::Provider { .. } | AgMailError::Dns(_) | AgMailError::Queue(_) => {
            WorkerError::retriable(err.to_string())
        }
        AgMailError::InvalidAddress(_)
        | AgMailError::Template(_)
        | AgMailError::VarMismatch(_)
        | AgMailError::Config(_)
        | AgMailError::NoMailHost(_)
        | AgMailError::Dkim(_)
        | AgMailError::SendExhausted(_) => WorkerError::permanent(err.to_string()),
    }
}

/// A [`MailQueue`] that enqueues delivery jobs onto an `ag-workers` backend
/// instead of `ag-mail`'s own in-process channel.
///
/// Pair it with a worker pool running a [`MailDeliveryHandler`] (see
/// [`mail_delivery_registry`]) to actually deliver. With the in-memory backend
/// this is a drop-in for [`crate::queue::InMemoryQueue`]; with the shared
/// PostgreSQL backend it gives durable, restart-surviving delivery without
/// `ag-mail` owning a second persistent queue.
pub struct WorkersMailQueue {
    backend: Arc<dyn QueueBackend>,
    queue: QueueName,
}

impl WorkersMailQueue {
    /// Builds the adapter over `backend`, enqueuing onto [`MAIL_QUEUE`].
    pub fn new(backend: Arc<dyn QueueBackend>) -> Self {
        Self::with_queue(backend, QueueName::new(MAIL_QUEUE))
    }

    /// Builds the adapter enqueuing onto a custom queue name.
    pub fn with_queue(backend: Arc<dyn QueueBackend>, queue: QueueName) -> Self {
        Self { backend, queue }
    }

    /// The queue name jobs are enqueued onto.
    pub fn queue_name(&self) -> &QueueName {
        &self.queue
    }
}

#[async_trait]
impl MailQueue for WorkersMailQueue {
    async fn enqueue(&self, email: Email) -> Result<(), AgMailError> {
        let job = NewJob::encode::<Email>(MAIL_DELIVERY_KIND, self.queue.clone(), &email)
            .map_err(|e| AgMailError::Queue(e.to_string()))?;
        self.backend
            .enqueue(job)
            .await
            .map(|_| ())
            .map_err(|e| AgMailError::Queue(e.to_string()))
    }
}

/// Builds a single-handler [`WorkerRegistry`] that delivers mail via `sender`.
///
/// Convenience for the consumer side: the returned registry resolves the
/// [`MAIL_DELIVERY_KIND`] job kind to a [`MailDeliveryHandler`].
pub fn mail_delivery_registry(
    sender: Arc<dyn MailSender>,
) -> Result<WorkerRegistry, RegistryError> {
    Ok(WorkerRegistry::builder()
        .register(MailDeliveryHandler::new(sender))?
        .build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use ag_workers::{payload, MemoryQueue, RuntimeConfig, Shutdown, WorkerPool};

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
    fn email_payload_roundtrips_through_rmp() {
        let email = test_email();
        let bytes = payload::encode(&email).unwrap();
        let back: Email = payload::decode(&bytes).unwrap();
        assert_eq!(email, back);
        assert_eq!(Email::PAYLOAD_VERSION, 1);
    }

    #[test]
    fn transient_failures_are_retriable() {
        for err in [
            AgMailError::Provider {
                provider: "smtp",
                message: "502".into(),
            },
            AgMailError::Dns("mx".into()),
            AgMailError::Queue("locked".into()),
        ] {
            assert!(
                classify(&err).kind().is_retriable(),
                "{err:?} should be retriable"
            );
        }
    }

    #[test]
    fn permanent_failures_are_not_retriable() {
        for err in [
            AgMailError::InvalidAddress("nope".into()),
            AgMailError::Template("missing".into()),
            AgMailError::VarMismatch("x".into()),
            AgMailError::Config("no key".into()),
            AgMailError::NoMailHost("d".into()),
            AgMailError::Dkim("bad key".into()),
            AgMailError::SendExhausted("gave up".into()),
        ] {
            assert!(
                !classify(&err).kind().is_retriable(),
                "{err:?} should be permanent"
            );
        }
    }

    #[tokio::test]
    async fn enqueue_through_adapter_is_delivered_by_the_pool() {
        let sender = Arc::new(NullSender::new());
        let registry = Arc::new(mail_delivery_registry(sender.clone()).unwrap());
        let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());
        let queue = QueueName::new(MAIL_QUEUE);

        // Producer side: enqueue through the MailQueue adapter.
        let producer = WorkersMailQueue::new(backend.clone());
        for _ in 0..3 {
            producer.enqueue(test_email()).await.unwrap();
        }

        // Consumer side: a static worker pool drains and delivers.
        let shutdown = Shutdown::new();
        let pool = WorkerPool::start(
            backend.clone(),
            registry,
            Arc::new(RuntimeConfig::default()),
            shutdown.clone(),
            queue.clone(),
            2,
        );

        for _ in 0..200 {
            if sender.emails_sent() >= 3 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        shutdown.trigger();
        pool.join_with_timeout(Duration::from_secs(2)).await;

        assert_eq!(sender.emails_sent(), 3);
        assert_eq!(backend.depth(&queue).await.unwrap().total(), 0);
    }
}
