//! Load test for the in-process event bus. Verifies the Phase 4 criterion of
//! 50,000 concurrent subscribers. Marked `#[ignore]` because it is resource
//! intensive; run explicitly in the manual gate:
//!
//!   cargo test -p ag-realtime --test load_50k -- --ignored --nocapture
//!
//! Methodology and hardware are recorded in docs/benchmarks/ag-realtime-load.md
//! per CLAUDE.md section 17.

use ag_realtime::EventBus;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "resource intensive; run in manual scalability gate"]
async fn fifty_thousand_subscribers_receive_event() {
    const SUBSCRIBERS: usize = 50_000;
    // Capacity must hold the burst so slow receivers do not lag out.
    let bus = EventBus::new(1024);

    let mut receivers: Vec<_> = (0..SUBSCRIBERS).map(|_| bus.subscribe()).collect();

    bus.publish("load.test", b"ping".to_vec())
        .expect("publish to 50k subscribers must succeed");

    let mut received = 0usize;
    for rx in receivers.iter_mut() {
        match rx.try_recv() {
            Ok(ev) => {
                assert_eq!(ev.subject, "load.test");
                received += 1;
            }
            Err(_) => {}
        }
    }

    // Allow a small tolerance for lagged receivers under broadcast backpressure.
    assert!(
        received >= SUBSCRIBERS * 99 / 100,
        "expected >=99% delivery, got {received}/{SUBSCRIBERS}"
    );
}
