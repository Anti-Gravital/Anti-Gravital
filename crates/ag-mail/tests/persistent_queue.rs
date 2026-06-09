//! Integration test for the persistent mail queue. Requires a PostgreSQL
//! instance via TEST_DATABASE_URL. Ignored by default so CI without a DB stays green.

#![cfg(feature = "queue-persistent")]
// PersistentQueue is deprecated in favor of the `workers` feature (RFC-0012 S7/M4);
// this legacy parity test keeps exercising it until it is removed.
#![allow(deprecated)]

use std::sync::Arc;

use ag_mail::message::{Address, EmailBuilder};
use ag_mail::queue::store::PersistentQueue;
use ag_mail::queue::{MailQueue, RetryPolicy};
use ag_mail::sender::test_utils::NullSender;

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL PostgreSQL"]
async fn message_survives_and_is_sent() {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL");
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let sender = Arc::new(NullSender::new());
    let queue = PersistentQueue::new(pool.clone(), sender.clone(), RetryPolicy::default());

    let email = EmailBuilder::new()
        .from(Address::new("a@x.com"))
        .to(Address::new("b@y.com"))
        .subject("s")
        .text_body("t")
        .build()
        .unwrap();
    queue.enqueue(email).await.unwrap();

    // Wait for the worker to drain it.
    for _ in 0..10 {
        let sent: i64 =
            sqlx::query_scalar("SELECT count(*) FROM ag_mail_queue WHERE status = 'sent'")
                .fetch_one(&pool)
                .await
                .unwrap();
        if sent == 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    assert_eq!(
        sender.emails_sent(),
        1,
        "NullSender must have captured one email"
    );
}
