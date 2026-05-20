//! Benchmark CRUD + PostgreSQL del todo-api.
//!
//! Mide la latencia y el throughput de las cinco operaciones elementales
//! (INSERT, SELECT list, SELECT one, UPDATE, DELETE) directamente contra
//! el pool de sqlx, sin capa HTTP. Esto da el limite inferior de latencia
//! antes de que el overhead de red y Axum entren en juego.
//!
//! El benchmark HTTP de carga (objetivo >= 40 K req/s con oha/wrk contra
//! el servidor en ejecucion) se documenta por separado en:
//!   docs/benchmarks/measurement-fase-2-crud.md
//!
//! # Requisitos
//!
//! - PostgreSQL accesible.
//! - Variable de entorno DATABASE_URL apuntando a una base de datos de bench.
//!   Se recomienda una base dedicada para no interferir con datos de desarrollo.
//!
//! # Ejecucion
//!
//! ```sh
//! export DATABASE_URL="postgresql://postgres:postgres@localhost/todos_bench"
//! cargo bench -p todo-api --bench crud 2>&1 | tee /tmp/bench-crud.txt
//! ```
//!
//! Si DATABASE_URL no esta definida, todos los benchmarks se saltan con un
//! aviso y el proceso termina con exito (compatible con CI sin base de datos).

#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::runtime::Runtime;

// --- Infraestructura --------------------------------------------------------

fn build_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("no se pudo construir el runtime Tokio")
}

fn connect_and_prepare(rt: &Runtime) -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "\n[crud bench] DATABASE_URL no definida; todos los benchmarks se omiten.\n\
                 Define DATABASE_URL para ejecutar contra PostgreSQL real.\n\
                 Ejemplo:\n\
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
            .expect("fallo la conexion a PostgreSQL — verifica DATABASE_URL");

        // Tabla de bench aislada de la tabla de produccion.
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS bench_todos (
                id    BIGSERIAL PRIMARY KEY,
                title TEXT      NOT NULL,
                done  BOOLEAN   NOT NULL DEFAULT FALSE
            )",
        )
        .execute(&pool)
        .await
        .expect("fallo la creacion de bench_todos");

        // Estado limpio antes del benchmark.
        sqlx::query("TRUNCATE bench_todos RESTART IDENTITY")
            .execute(&pool)
            .await
            .expect("fallo el truncate de bench_todos");

        pool
    }))
}

// --- Benchmarks individuales ------------------------------------------------

fn bench_insert(c: &mut Criterion, rt: &Runtime, pool: &PgPool) {
    let mut group = c.benchmark_group("crud/insert");
    group.throughput(Throughput::Elements(1));

    group.bench_function("insert_one", |b| {
        b.iter(|| {
            rt.block_on(async {
                sqlx::query_scalar::<_, i64>(
                    "INSERT INTO bench_todos (title) VALUES ($1) RETURNING id",
                )
                .bind("tarea de benchmark")
                .fetch_one(pool)
                .await
                .expect("fallo insert")
            })
        });
    });

    group.finish();
}

fn bench_select_list(c: &mut Criterion, rt: &Runtime, pool: &PgPool) {
    // Pre-carga filas para que la tabla no este vacia durante SELECT.
    rt.block_on(async {
        for i in 0..100i32 {
            sqlx::query("INSERT INTO bench_todos (title) VALUES ($1)")
                .bind(format!("precarga {i}"))
                .execute(pool)
                .await
                .expect("fallo precarga");
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
                .expect("fallo select list")
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
                .expect("fallo select one")
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
                    .expect("fallo update")
            })
        });
    });

    group.finish();
}

fn bench_delete(c: &mut Criterion, rt: &Runtime, pool: &PgPool) {
    let mut group = c.benchmark_group("crud/delete");
    group.throughput(Throughput::Elements(1));

    // El delete borra una fila real; re-inserta antes de cada iteracion.
    group.bench_function("delete_one", |b| {
        b.iter(|| {
            rt.block_on(async {
                let id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO bench_todos (title) VALUES ($1) RETURNING id",
                )
                .bind("fila efimera")
                .fetch_one(pool)
                .await
                .expect("fallo insert antes de delete");

                sqlx::query("DELETE FROM bench_todos WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await
                    .expect("fallo delete")
            })
        });
    });

    group.finish();
}

fn bench_full_cycle(c: &mut Criterion, rt: &Runtime, pool: &PgPool) {
    // Ciclo completo: INSERT -> SELECT -> UPDATE -> DELETE.
    // Este es el benchmark canonico para el criterio >= 40 K req/s de Fase 2.
    // 1 iteracion = 4 operaciones DB (throughput = 4x elementos).

    let mut group = c.benchmark_group("crud/full_cycle");
    group.throughput(Throughput::Elements(4));

    group.bench_function("insert_select_update_delete", |b| {
        b.iter(|| {
            rt.block_on(async {
                // INSERT
                let id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO bench_todos (title) VALUES ($1) RETURNING id",
                )
                .bind("ciclo completo")
                .fetch_one(pool)
                .await
                .expect("fallo insert del ciclo");

                // SELECT one
                sqlx::query_as::<_, (i64, String, bool)>(
                    "SELECT id, title, done FROM bench_todos WHERE id = $1",
                )
                .bind(id)
                .fetch_one(pool)
                .await
                .expect("fallo select del ciclo");

                // UPDATE
                sqlx::query("UPDATE bench_todos SET done = TRUE WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await
                    .expect("fallo update del ciclo");

                // DELETE
                sqlx::query("DELETE FROM bench_todos WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await
                    .expect("fallo delete del ciclo")
            })
        });
    });

    group.finish();
}

// Benchmark de concurrencia: N tareas async en paralelo midiendo throughput
// agregado. Simula carga sostenida real con conexiones concurrentes.
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
                                    .expect("fallo insert concurrente");

                                    sqlx::query("DELETE FROM bench_todos WHERE id = $1")
                                        .bind(id)
                                        .execute(&pool)
                                        .await
                                        .expect("fallo delete concurrente");
                                })
                            })
                            .collect();

                        for h in handles {
                            h.await.expect("tarea concurrente fallo con panic");
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

    // Limpieza final.
    rt.block_on(async {
        let _ = sqlx::query("DROP TABLE IF EXISTS bench_todos")
            .execute(&pool)
            .await;
    });
}

criterion_group!(benches, run_all);
criterion_main!(benches);
