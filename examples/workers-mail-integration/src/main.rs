//! Routing `ag-mail` delivery through `ag-workers` (RFC-0012 S7/M2 adapter).
//!
//! Only the generic queue mechanics (enqueue, retry/backoff, dead-lettering, and with
//! the shared PostgreSQL backend, durability) move to the `ag-workers` substrate. The
//! mail-specific delivery path stays behind `MailSender`; the adapter binds an `Email`
//! to the worker payload contract and a `MailSender` to the `JobHandler` contract.
//!
//! Producers enqueue through the `WorkersMailQueue` adapter; a worker pool delivers.
//! This example uses a no-op sender so it runs without an SMTP server.

use std::sync::Arc;
use std::time::Duration;

use ag_mail::error::AgMailError;
use ag_mail::message::{Address, Email, EmailBuilder};
use ag_mail::queue::MailQueue;
use ag_mail::sender::{MailSender, SendResult};
use ag_mail::workers::{mail_delivery_registry, WorkersMailQueue};
use ag_workers::{MemoryQueue, QueueBackend, RuntimeConfig, Shutdown, WorkerPool};

/// A sender that does not actually send: it stands in for SMTP/MTA in the example.
struct NoopSender;

#[async_trait::async_trait]
impl MailSender for NoopSender {
    fn name(&self) -> &'static str {
        "noop"
    }

    async fn send(&self, email: &Email) -> Result<SendResult, AgMailError> {
        let to = email
            .to
            .first()
            .map(Address::display)
            .unwrap_or_else(|| "<no recipient>".to_owned());
        println!("delivering mail to {to}");
        Ok(SendResult {
            message_id: None,
            provider: "noop",
        })
    }
}

fn sample_email(to: &str) -> Email {
    EmailBuilder::new()
        .from(Address::new("noreply@example.com"))
        .to(Address::new(to))
        .subject("Welcome")
        .text_body("Thanks for signing up.")
        .build()
        .expect("valid email")
}

#[tokio::main]
async fn main() {
    let sender: Arc<dyn MailSender> = Arc::new(NoopSender);
    let registry = Arc::new(mail_delivery_registry(sender).expect("registry"));

    let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());

    // Producer side: enqueue mail through the adapter (the generic queue mechanics).
    let producer = WorkersMailQueue::new(backend.clone());
    let queue = producer.queue_name().clone();
    for i in 0..3 {
        producer
            .enqueue(sample_email(&format!("user{i}@example.com")))
            .await
            .expect("enqueue");
    }

    // Consumer side: a worker pool delivers via the bound MailSender.
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
        if backend.depth(&queue).await.expect("depth").total() == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    shutdown.trigger();
    pool.join_with_timeout(Duration::from_secs(2)).await;
    println!("done: mail drained through the ag-workers substrate");
}
