//! CRUD + PostgreSQL benchmark for todo-api.
//!
//! Measures the latency and throughput of the five elementary operations
//! (INSERT, SELECT list, SELECT one, UPDATE, DELETE) directly against
//! the sqlx pool, without the HTTP layer. This gives the lower latency bound
//! before network and Axum overhead come into play.
//!
//! The HTTP load benchmark (target >= 40 K req/s with oha/wrk against
//! the running server) is documented separately in:
//!   docs/benchmarks/measurement-fase-2-crud.md
//!
//! # Requirements
//!
//! - PostgreSQL accessible.
//! - DATABASE_URL environment variable pointing to a bench database.
//!   A dedicated database is recommended to avoid interfering with development data.
//!
//! # Running
//!
//! ```sh
//! export DATABASE_URL="postgresql://postgres:postgres@localhost/todos_bench"
//! cargo bench -p todo-api --bench crud 2>&1 | tee /tmp/bench-crud.txt
//! ```
//!
//! If DATABASE_URL is not defined, all benchmarks are skipped with a
//! warning and the process exits successfully (compatible with CI without a database).

#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::runtime::Runtime;

// --- Infrastructure ---------------------------------------------------------

fn build_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("failed to build the Tokio runtime")
}

fn connect_and_prepare(rt: &Runtime) -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "\n[crud bench] DATABASE_URL not defined; all benchmarks are skipped.\n\
                 Set DATABASE_URL to run against a real PostgreSQL instance.\n\
                 Example:\n\
                   export DATABASE_URL=\"postgresql://postgres:postgres@localhost/todos_bench\"\n\
                   cargo bench -p todo-api --bench crud\n"
            );
            return None;
        }
    };

    Some(rt.block_on(async {
        let pool = PgPoolOptions::new()
            .max_connections(16)
            .min_connections(4)
            .connect(&url)
            .await
            .expect("failed to connect to PostgreSQL — check DATABASE_URL");

        // Bench table isolated from the production table.
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS bench_todos (
                id    BIGSERIAL PRIMARY KEY,
                title TEXT      NOT NULL,
                done  BOOLEAN   NOT NULL DEFAULT FALSE
            )",
        )
        .execute(&pool)
        .await
        .expect("failed to create bench_todos");

        // Clean state before the benchmark.
        sqlx::query("TRUNCATE bench_todos RESTART IDENTITY")
            .execute(&pool)
            .await
            .expect("failed to truncate bench_todos");

        pool
    }))
}

// --- Individual benchmarks --------------------------------------------------

fn bench_insert(c: &mut Criterion, rt: &Runtime, pool: &PgPool) {
    let mut group = c.benchmark_group("crud/insert");
    group.throughput(Throughput::Elements(1));

    group.bench_function("insert_one", |b| {
        b.iter(|| {
            rt.block_on(async {
                sqlx::query_scalar::<_, i64>(
                    "INSERT INTO bench_todos (title) VALUES ($1) RETURNING id",
                )
                .bind("benchmark task")
                .fetch_one(pool)
                .await
                .expect("insert failed")
            })
        });
    });

    group.finish();
}

fn bench_select_list(c: &mut Criterion, rt: &Runtime, pool: &PgPool) {
    // Pre-load rows so the table is not empty during SELECT.
    rt.block_on(async {
        for i in 0..100i32 {
            sqlx::query("INSERT INTO bench_todos (title) VALUES ($1)")
                .bind(format!("preload {i}"))
                .execute(pool)
                .await
                .expect("preload failed");
        }
    });

    let mut group = c.benchmark_group("crud/select");
    group.throughput(Throughput::Elements(1));

    group.bench_function("list_all", |b| {
        b.iter(|| {
            rt.block_on(async {
                sqlx::query_as::<_, (i64, String, bool)>(
                    "SELECT id, title, done FROM bench_todos ORDER BY id",
                )
                .fetch_all(pool)
                .await
                .expect("select list failed")
            })
        });
    });

    group.bench_function("select_one_by_id", |b| {
        b.iter(|| {
            rt.block_on(async {
                sqlx::query_as::<_, (i64, String, bool)>(
                    "SELECT id, title, done FROM bench_todos WHERE id = $1",
                )
                .bind(1i64)
                .fetch_optional(pool)
                .await
                .expect("select one failed")
            })
        });
    });

    group.finish();
}

fn bench_update(c: &mut Criterion, rt: &Runtime, pool: &PgPool) {
    let mut group = c.benchmark_group("crud/update");
    group.throughput(Throughput::Elements(1));

    group.bench_function("update_one", |b| {
        b.iter(|| {
            rt.block_on(async {
                sqlx::query("UPDATE bench_todos SET done = NOT done WHERE id = $1")
                    .bind(1i64)
                    .execute(pool)
                    .await
                    .expect("update failed")
            })
        });
    });

    group.finish();
}

fn bench_delete(c: &mut Criterion, rt: &Runtime, pool: &PgPool) {
    let mut group = c.benchmark_group("crud/delete");
    group.throughput(Throughput::Elements(1));

    // Delete removes a real row; re-inserts before each iteration.
    group.bench_function("delete_one", |b| {
        b.iter(|| {
            rt.block_on(async {
                let id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO bench_todos (title) VALUES ($1) RETURNING id",
                )
                .bind("ephemeral row")
                .fetch_one(pool)
                .await
                .expect("insert before delete failed");

                sqlx::query("DELETE FROM bench_todos WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await
                    .expect("delete failed")
            })
        });
    });

    group.finish();
}

fn bench_full_cycle(c: &mut Criterion, rt: &Runtime, pool: &PgPool) {
    // Full cycle: INSERT -> SELECT -> UPDATE -> DELETE.
    // This is the canonical benchmark for the Fase 2 >= 40 K req/s criterion.
    // 1 iteration = 4 DB operations (throughput = 4x elements).

    let mut group = c.benchmark_group("crud/full_cycle");
    group.throughput(Throughput::Elements(4));

    group.bench_function("insert_select_update_delete", |b| {
        b.iter(|| {
            rt.block_on(async {
                // INSERT
                let id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO bench_todos (title) VALUES ($1) RETURNING id",
                )
                .bind("full cycle")
                .fetch_one(pool)
                .await
                .expect("cycle insert failed");

                // SELECT one
                sqlx::query_as::<_, (i64, String, bool)>(
                    "SELECT id, title, done FROM bench_todos WHERE id = $1",
                )
                .bind(id)
                .fetch_one(pool)
                .await
                .expect("cycle select failed");

                // UPDATE
                sqlx::query("UPDATE bench_todos SET done = TRUE WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await
                    .expect("cycle update failed");

                // DELETE
                sqlx::query("DELETE FROM bench_todos WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await
                    .expect("cycle delete failed")
            })
        });
    });

    group.finish();
}

// Concurrency benchmark: N concurrent async tasks measuring aggregate throughput.
// Simulates real sustained load with concurrent connections.
fn bench_concurrent(c: &mut Criterion, rt: &Runtime, pool: &PgPool) {
    let mut group = c.benchmark_group("crud/concurrent");

    for concurrency in [1usize, 4, 16, 64] {
        group.throughput(Throughput::Elements(concurrency as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            &concurrency,
            |b, &n| {
                b.iter(|| {
                    rt.block_on(async {
                        let handles: Vec<_> = (0..n)
                            .map(|_| {
                                let pool = pool.clone();
                                tokio::spawn(async move {
                                    let id = sqlx::query_scalar::<_, i64>(
                                        "INSERT INTO bench_todos (title) VALUES ($1) RETURNING id",
                                    )
                                    .bind("concurrent task")
                                    .fetch_one(&pool)
                                    .await
                                    .expect("concurrent insert failed");

                                    sqlx::query("DELETE FROM bench_todos WHERE id = $1")
                                        .bind(id)
                                        .execute(&pool)
                                        .await
                                        .expect("concurrent delete failed");
                                })
                            })
                            .collect();

                        for h in handles {
                            h.await.expect("concurrent task panicked");
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

// --- Entry point ------------------------------------------------------------

fn run_all(c: &mut Criterion) {
    let rt = build_runtime();
    let pool = match connect_and_prepare(&rt) {
        Some(p) => p,
        None => return,
    };

    bench_insert(c, &rt, &pool);
    bench_select_list(c, &rt, &pool);
    bench_update(c, &rt, &pool);
    bench_delete(c, &rt, &pool);
    bench_full_cycle(c, &rt, &pool);
    bench_concurrent(c, &rt, &pool);

    // Final cleanup.
    rt.block_on(async {
        let _ = sqlx::query("DROP TABLE IF EXISTS bench_todos")
            .execute(&pool)
            .await;
    });
}

criterion_group!(benches, run_all);
criterion_main!(benches);
