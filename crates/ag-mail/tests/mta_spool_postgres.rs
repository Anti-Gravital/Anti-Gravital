//! Live PostgreSQL durable-spool test for the native MTA (RFC-0009 section 4.2).
//!
//! Ignored by default: it needs a reachable PostgreSQL through `DATABASE_URL`.
//! Run it explicitly, e.g.:
//!
//! ```sh
//! DATABASE_URL=postgres://user:pass@localhost/db \
//!   cargo test -p ag-mail --features queue-postgres --test mta_spool_postgres -- --ignored
//! ```
//!
//! The test is self-isolating: it asserts on the ids it inserts (so leftover
//! rows from other runs do not affect it) and removes them at the end.
#![cfg(feature = "queue-postgres")]

use std::sync::Arc;
use std::time::{Duration, Instant};

use ag_mail::sender::mta::{
    DeliveryJob, MtaQueue, PostgresSpool, QueueConfig, Shaper, ShapingConfig, ShapingLimits, Spool,
};

fn shaper() -> Shaper {
    Shaper::new(ShapingConfig::new(ShapingLimits::UNLIMITED))
}

fn job(now: Instant) -> DeliveryJob {
    DeliveryJob::new(
        "tenant-a",
        "welcome",
        "example.com",
        "mx.example.com",
        "from@send.example",
        vec!["a@example.com".to_owned()],
        b"From: from@send.example\r\nSubject: hi\r\n\r\nbody\r\n".to_vec(),
        now,
    )
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing at a live PostgreSQL"]
async fn jobs_survive_restart_against_postgres() {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must point at a live PostgreSQL for this test");

    // First process lifetime: enqueue a future-dated job through the queue,
    // mirrored to PostgreSQL, then drop everything (simulating a crash).
    let id = {
        let spool = Arc::new(PostgresSpool::connect(&url).await.unwrap());
        let queue = MtaQueue::new(QueueConfig::default(), shaper()).with_spool(spool.clone());
        let now = Instant::now();
        let mut j = job(now);
        j.next_attempt_at = now + Duration::from_secs(3600);
        let id = j.id.clone();
        queue.enqueue_persistent(j).await.unwrap();
        id
    };

    // Second process lifetime: a fresh connection over the same table sees it.
    let spool2 = Arc::new(PostgresSpool::connect(&url).await.unwrap());
    let all = spool2.load_all().await.unwrap();
    assert!(
        all.iter().any(|p| p.id == id),
        "the scheduled job must survive a restart in PostgreSQL"
    );

    // A fresh queue recovers it from the durable backend.
    let recovered_queue =
        MtaQueue::new(QueueConfig::default(), shaper()).with_spool(spool2.clone());
    let recovered = recovered_queue.recover().await.unwrap();
    assert!(recovered >= 1, "recover() must repopulate the queue");
    assert!(!recovered_queue.is_empty());

    // Clean up this test's row so reruns stay isolated.
    spool2.remove(&id).await.unwrap();
}
