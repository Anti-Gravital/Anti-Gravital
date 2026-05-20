//! Benchmark CRUD + PostgreSQL de la todo-api.
#![allow(missing_docs)]
//!
//! Requiere una base de datos PostgreSQL accesible via DATABASE_URL.
//! Si la variable no esta definida, el benchmark se salta con un aviso.
//!
//! Ejecutar:
//! ```sh
//! export DATABASE_URL="postgresql://postgres:postgres@localhost/todos_bench"
//! cargo bench -p todo-api --bench crud
//! ```

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_placeholder(c: &mut Criterion) {
    // El benchmark real requiere un runtime Tokio y una conexion PostgreSQL.
    // Se implementa aqui como placeholder que documenta la intencion;
    // la medicion definitiva se registra en docs/benchmarks/ siguiendo
    // la plantilla measurement-template.md con hardware de referencia.
    //
    // Objetivo de Fase 2: >= 40 K req/s en CRUD simple, p99 <= 5 ms.

    if std::env::var("DATABASE_URL").is_err() {
        eprintln!(
            "[crud bench] DATABASE_URL no definida; benchmark omitido. \
             Define DATABASE_URL para ejecutar con una base de datos real."
        );
        return;
    }

    c.bench_function("todo_crud_placeholder", |b| {
        b.iter(|| {
            // Sustituir por operaciones reales contra el pool cuando
            // DATABASE_URL este disponible. Ver docs/benchmarks/ para
            // la metodologia y la plantilla de medicion.
            std::hint::black_box(42u64)
        });
    });
}

criterion_group!(benches, bench_placeholder);
criterion_main!(benches);
