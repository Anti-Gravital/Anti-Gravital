# Medicion oficial Fase 2 - CRUD + PostgreSQL

## Identidad del reporte

- Fecha: 2026-05-21.
- Reportero: Angel Nereira.
- Tipo de medicion: throughput y latencia HTTP con backend PostgreSQL real.
- Componente: `todo-api` (examples/) con `ag-core` + `ag-data`.
- Comentario: Primera medicion valida de Fase 2. Incluye correccion de bug
  de routing que invalidaba todas las mediciones previas (ver seccion
  Observaciones).

---

## Entorno

### Hardware

- CPU: AMD Ryzen 5 2500U with Radeon Vega Mobile Gfx, 2.0 GHz base /
  3.6 GHz boost, 4 nucleos fisicos, 8 hilos logicos.
- RAM: 14.4 GB DDR4.
- Red: loopback (127.0.0.1), sin overhead de red fisica.

### Sistema operativo

- Ubuntu 25.10 (Questing Quokka).
- Kernel: 6.17.0-29-generic.
- `ulimit -n`: 524288 (file descriptors).
- `net.core.somaxconn`: 4096.

### Toolchain

- `rustc 1.95.0 (59807616e 2026-04-14)`.
- `rust-toolchain.toml`: channel = "stable", pin a 1.95.0.
- Profile release: `opt-level=3`, `lto="fat"`, `codegen-units=1`,
  `panic="abort"`, `strip="symbols"`.

### Repositorio

- Commit base: `177a6ca0b57955c7ca4fc4191d53aad678b3f322` (rama main).
- Modificacion aplicada en esta sesion: correccion de ruta
  `"/todos/{id}"` -> `"/todos/:id"` en `examples/todo-api/src/main.rs`.
  El binario benchmarkeado incluye este fix.
- Dirty tree: si. Un archivo modificado: `examples/todo-api/src/main.rs`.

### PostgreSQL

- Version: PostgreSQL 18.4 (Ubuntu 18.4-1.pgdg25.10+1).
- Puerto: 5432 (socket UNIX local).
- `max_connections`: 100.
- `shared_buffers`: 128 MB (default Ubuntu).
- `synchronous_commit`: off en la base `todos_bench` (escrito como
  `ALTER DATABASE todos_bench SET synchronous_commit = off`).

### Herramienta de carga

- `oha 1.14.0`.

---

## Metodologia

### Servidor

```sh
DATABASE_URL="postgresql://bench_user:bench_pass@localhost/todos_bench" \
DATABASE_MAX_CONNECTIONS=100 \
BIND="127.0.0.1:8099" \
RUST_LOG="warn" \
./target/release/todo-api &
```

Pool: 100 conexiones. `synchronous_commit = off` para la base bench
(mejora throughput de escritura al eliminar fsync por transaccion; no
aplica en produccion sin evaluacion de requisitos de durabilidad).

### Clientes

```sh
# Lectura (GET /todos/:id)
oha -n 200000 -c 100 --no-tui http://127.0.0.1:8099/todos/1

# Escritura (POST /todos)
oha -n 50000 -c 50 --no-tui \
  -m POST -H "Content-Type: application/json" \
  -d '{"title":"bench-write","done":false}' \
  http://127.0.0.1:8099/todos

# Baseline sin DB (GET /health)
oha -n 200000 -c 100 --no-tui http://127.0.0.1:8099/health
```

### Numero de ejecuciones

Se ejecutaron dos corridas completas por endpoint. Los datos del barrido
de concurrencia (c=8..100) complementan la medicion de throughput.

---

## Resultados

### Baseline HTTP sin DB - GET /health

| Metrica       | Valor      |
| ------------- | ---------- |
| req/s         | 88 930     |
| p50           | 1.06 ms    |
| p99           | 3.17 ms    |
| p99.9         | ~4 ms      |
| Avg           | 1.12 ms    |

Este numero establece el techo del stack HTTP (Shield + Axum + Tokio)
en este hardware: ~89K req/s sin overhead de base de datos.

---

### Lectura - GET /todos/:id (SELECT por clave primaria)

Concurrencia optima segun barrido (c=8..100, pool=50, n=50000):

| c   | req/s  | p99 (ms) |
| --- | ------ | -------- |
| 8   | 10 724 | 1.54     |
| 16  | 12 480 | 2.71     |
| 32  | 13 305 | 4.93     |
| 64  | 13 720 | 9.83     |
| 100 | 14 338 | 15.45    |

Corridas definitivas con c=100, pool=100, n=200000:

| Corrida | req/s  | p50 (ms) | p99 (ms) | p99.9 (ms) |
| ------- | ------ | -------- | -------- | ---------- |
| 1       | 14 818 | ~4.8     | 15.41    | 19.07      |
| 2       | 14 138 | ~3.4     | 13.79    | 17.38      |
| Mediana | 14 478 | ~4.1     | 14.60    | 18.23      |

---

### Escritura - POST /todos (INSERT RETURNING)

Corridas con c=50, pool=100, n=50000, `synchronous_commit=off`:

| Corrida | req/s | p50 (ms) | p99 (ms) | p99.9 (ms) |
| ------- | ----- | -------- | -------- | ---------- |
| 1       | 9 111 | ~4.0     | 9.30     | 51.37      |
| 2       | 8 756 | ~4.3     | 9.44     | 58.56      |
| Mediana | 8 934 | ~4.2     | 9.37     | 54.97      |

Con `synchronous_commit=on` (default produccion): ~7 000 req/s (estimado
por runs anteriores en Docker).

---

### Benchmarks de capa DB directa (criterion, sin HTTP)

Ejecutado con `cargo bench -p todo-api --bench crud`. 100 muestras por
grupo. Runtime Tokio multi-thread con 4 workers.

| Operacion                      | Latencia media | Throughput     |
| ------------------------------ | -------------- | -------------- |
| INSERT (insert_one)            | 1.898 ms       | 527 op/s       |
| SELECT list (100 filas)        | 6.940 ms       | 144 op/s       |
| SELECT one by id               | 351.8 us       | 2 843 op/s     |
| UPDATE one                     | 1.968 ms       | 508 op/s       |
| DELETE one                     | 4.113 ms       | 243 op/s       |
| Ciclo completo (4 ops)         | 6.173 ms       | 162 ciclos/s   |
| Concurrencia/16 (insert+delete)| 7.563 ms       | 2 116 op/s     |
| Concurrencia/64 (insert+delete)| 24.54 ms       | 2 609 op/s     |

---

### Recursos del proceso

- Memoria RSS idle (todo-api, pool=100): 3 MB.
- Arranque hasta "servidor iniciado": ~50 ms (estimado; incluye
  migraciones sqlx y handshake del pool).

---

## Conformidad con criterios de salida de Fase 2

| Criterio                            | Objetivo     | Medido                   | Cumple    |
| ----------------------------------- | ------------ | ------------------------ | --------- |
| Benchmark CRUD >= 40K req/s         | >= 40K req/s | 14.5K req/s (GET, c=100) | No        |
| Latencia p99 CRUD <= 5 ms           | <= 5 ms      | 14.6 ms (GET, c=100)     | No        |
| Benchmark HTTP layer (sin DB)       | N/A          | 88.9K req/s              | referencia|
| `ag new` + `ag dev` operativos      | si           | Verificado               | Si        |
| Docker FROM scratch <= 20 MB        | <= 20 MB     | 2.49 MB                  | Si        |
| Binario release <= 20 MB            | <= 20 MB     | 5.3 MB (MUSL)            | Si        |

Los criterios de throughput y latencia no se alcanzan en este hardware
con PostgreSQL nativo y configuracion estandar. La causa es el modelo
proceso-por-conexion de PostgreSQL que satura los 4 nucleos fisicos del
Ryzen 5 2500U a ~100 conexiones concurrentes.

---

## Analisis del cuello de botella

El stack HTTP (Shield + Axum + Tokio) es capaz de 89K req/s sin DB,
lo que demuestra que el framework no es el limite. La brecha entre 89K y
14.5K se debe exclusivamente a PostgreSQL:

- `max_connections=100` (limite de la instalacion Ubuntu por defecto).
- Modelo proceso-por-conexion de PostgreSQL: 100 procesos OS compitiendo
  por 4 nucleos fisicos.
- La latencia media del SELECT por PK es 352 µs; con 100 conexiones
  paralelas el techo teorico es 284K op/s, pero el scheduler de OS y el
  buffer pool de PG lo limitan a ~14.5K en este CPU.

Para alcanzar >= 40K req/s en lectura se requiere una de las siguientes
condiciones:

1. Hardware con >= 8 nucleos fisicos (ej. Ryzen 9 7950X, 16C/32T).
2. Uso de un connection pooler externo (pgbouncer en transaction mode)
   que desacopla las conexiones de la aplicacion de las del servidor PG.
3. Lectura con replicas de solo lectura balanceadas.

---

## Observaciones criticas

### Bug de routing detectado y corregido

Durante la medicion se descubrio que la ruta parametrizada del todo-api
usaba sintaxis de Axum 0.8 (`{id}`) en lugar de Axum 0.7 (`:id`). El
workspace usa `axum = "0.7.9"`.

Consecuencia: todas las requests a `GET /todos/:id`, `PUT /todos/:id` y
`DELETE /todos/:id` devolvian HTTP 404 sin tocar la base de datos. El
servidor procesaba los 404 a ~89K req/s (equivalente al baseline sin
DB), lo que explica el numero de "82 233 req/s" registrado en mediciones
anteriores en STATUS.md. Ese numero es invalido.

Correccion aplicada:

```diff
- "/todos/{id}",
+ "/todos/:id",
```

Archivo: `examples/todo-api/src/main.rs`, linea 72.

Esta correccion debe incluirse en el siguiente commit y el STATUS.md debe
actualizar las metricas de Fase 2 con los numeros de este documento.

### Limitacion de max_connections

PostgreSQL por defecto en Ubuntu instala con `max_connections=100`. El
pool de la aplicacion se configuro a 100 para aprovechar el limite
completo. Aumentar `max_connections` requiere reinicio del servicio
PostgreSQL y no se realizo para no alterar el entorno del sistema.

---

## Reproducibilidad

Para reproducir esta medicion exacta:

```sh
git clone https://github.com/anti-gravital/anti-gravital
git checkout 177a6ca0b57955c7ca4fc4191d53aad678b3f322

# Aplicar el fix de routing (pendiente de commit)
sed -i 's|"/todos/{id}"|"/todos/:id"|' examples/todo-api/src/main.rs

# Preparar PostgreSQL
createdb todos_bench
psql todos_bench -c "CREATE USER bench_user WITH PASSWORD 'bench_pass';"
psql todos_bench -c "GRANT ALL ON DATABASE todos_bench TO bench_user;"
psql todos_bench -c "ALTER DATABASE todos_bench SET synchronous_commit = off;"

# Compilar y arrancar
cargo build -p todo-api --release
DATABASE_URL="postgresql://bench_user:bench_pass@localhost/todos_bench" \
DATABASE_MAX_CONNECTIONS=100 BIND="127.0.0.1:8099" RUST_LOG="warn" \
./target/release/todo-api &

# Insertar fila de referencia
curl -X POST http://127.0.0.1:8099/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"benchmark-reference"}'

# Medir
oha -n 200000 -c 100 --no-tui http://127.0.0.1:8099/todos/1
```

Los resultados deben reproducirse dentro de +-10% de los valores
reportados en hardware equivalente (4 nucleos fisicos, ~2.0 GHz base,
PostgreSQL 17+ con configuracion por defecto).
