//! Memory-backend throughput benchmarks for `ag-workers`.
//!
//! Measures the hot paths a request handler and a worker loop exercise, with no
//! external infrastructure (in-memory backend only, per ADR-0009 native-first):
//!
//! 1. `payload_encode` / `payload_decode`: the `rmp-serde` codec around a typed
//!    payload (the per-enqueue and per-dispatch serialization cost, §9.3).
//! 2. `enqueue`: admission + envelope build + insert into `MemoryQueue` (§12, §18).
//! 3. `lease`: the `ORDER BY priority` selection a worker poll performs (§13.2's
//!    in-memory equivalent), measured against a pre-filled queue.
//!
//! These establish a regression baseline for the in-memory path. The PostgreSQL
//! backend's throughput (network/DB bound) is measured separately against a live
//! database and is tracked in the durable-backend verification issue (#108), not
//! here, because the default benchmark environment has no PostgreSQL.
//!
//! Methodology, hardware-recording duties and how to run: see
//! `crates/ag-workers/benches/README.md` and the workspace `docs/benchmarks/`
//! discipline (RFC-0012 §34, §17 benchmark rule).

// `criterion_group!` generates undocumented items; suppress `missing_docs` only
// here so the global workspace rule is not weakened.
#![allow(missing_docs)]

use ag_workers::{
    decode, encode, JobPayload, MemoryQueue, NewJob, QueueBackend, QueueName, WorkerId,
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

/// A representative typed payload (a couple of scalar fields, as most jobs carry).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SendReceipt {
    user_id: u64,
    amount_cents: u64,
    currency: String,
}

impl JobPayload for SendReceipt {
    const PAYLOAD_VERSION: u16 = 1;
}

fn sample_payload() -> SendReceipt {
    SendReceipt {
        user_id: 42,
        amount_cents: 1_999,
        currency: "USD".to_string(),
    }
}

fn bench_payload_codec(c: &mut Criterion) {
    let payload = sample_payload();
    let encoded = encode(&payload).expect("encode");

    c.bench_function("payload_encode", |b| {
        b.iter(|| encode(black_box(&payload)).expect("encode"));
    });

    c.bench_function("payload_decode", |b| {
        b.iter(|| {
            let out: SendReceipt = decode(black_box(&encoded)).expect("decode");
            black_box(out);
        });
    });
}

fn bench_enqueue(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");
    let payload = sample_payload();

    c.bench_function("enqueue_memory", |b| {
        // Each iteration enqueues into a fresh queue so the measurement is the
        // admission + envelope + insert cost, not the growth of a shared map.
        b.iter_batched(
            MemoryQueue::new,
            |queue| {
                rt.block_on(async {
                    let job = NewJob::encode::<SendReceipt>("send_receipt", "mail", &payload)
                        .expect("encode job");
                    queue.enqueue(black_box(job)).await.expect("enqueue");
                });
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_lease(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");
    let payload = sample_payload();
    let worker = WorkerId::from_string("bench-worker");
    let queue_name = QueueName::new("mail");

    // Lease a batch of 100 from a queue pre-filled with 1000 ready jobs, rebuilt
    // per iteration so leasing always starts from the same state.
    let prefill = 1_000usize;
    let batch = 100usize;

    c.bench_function("lease_memory_batch_100", |b| {
        b.iter_batched(
            || {
                let queue = MemoryQueue::new().with_max_depth(prefill * 2);
                rt.block_on(async {
                    for _ in 0..prefill {
                        let job = NewJob::encode::<SendReceipt>("send_receipt", "mail", &payload)
                            .expect("encode job");
                        queue.enqueue(job).await.expect("enqueue");
                    }
                });
                queue
            },
            |queue| {
                rt.block_on(async {
                    let leased = queue
                        .lease(black_box(&queue_name), black_box(&worker), batch)
                        .await
                        .expect("lease");
                    black_box(leased);
                });
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_payload_codec, bench_enqueue, bench_lease);
criterion_main!(benches);
