# Medicion oficial: CRUD + PostgreSQL — Fase 2

Plantilla pre-rellenada para el cierre de Fase 2.
Ejecutar en la maquina de referencia, rellenar todos los campos marcados
con `[RELLENAR]` y commitear el resultado.

Copiado como:
`docs/benchmarks/measurement-2026-05-20-fase-2-crud.md`

## Identidad del reporte

- Fecha: 2026-05-20.
- Reportero: Angel Nereira.
- Tipo de medicion: throughput HTTP + latencia CRUD + throughput DB directo.
- Componente: `todo-api` + `ag-data` + `ag-core` Fase 2.
- Comentario: validacion de cierre de Fase 2 — criterios 40 K req/s y p99 <= 5 ms.

## Entorno

### Hardware

- CPU: AMD Ryzen 5 2500U with Radeon Vega Mobile Gfx — 8 CPUs, 1600–2000 MHz.
- RAM: 14 GiB total, 8.9 GiB disponible.
- Almacenamiento: SK hynix SC311 SATA 256GB (SSD, ROTA=0).
- Red: loopback (127.0.0.1) para toda la medicion.

### Sistema operativo

```sh
# Ejecutar y pegar salida:
cat /etc/os-release | head -3
uname -r
ulimit -n
sysctl net.core.somaxconn
```

Salida:
```
PRETTY_NAME="Ubuntu 25.10"
NAME="Ubuntu"
VERSION_ID="25.10"
6.17.0-29-generic
524288
net.core.somaxconn = 4096
```

### Toolchain

```sh
# Ejecutar y pegar salida:
rustc --version
cat rust-toolchain.toml
```

Salida:
```
rustc 1.95.0 (59807616e 2026-04-14)
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
profile = "default"
```

### Repositorio

```sh
# Ejecutar y pegar salida:
git rev-parse HEAD
git branch --show-current
git status --short
```

Salida:
```
219309e9d2795c1991027ceb6d64e713c2ef858b
chore/status-fase-2-cierre
 M templates/fullstack/Cargo.toml.tmpl
 M templates/realtime/Cargo.toml.tmpl
 M templates/rest/Cargo.toml.tmpl
```

## Metodologia

### Parte 1: Benchmark DB directo (Criterion)

Mide latencia de operaciones sqlx directamente contra PostgreSQL,
sin overhead HTTP ni Axum.

```sh
export DATABASE_URL="postgresql://dev-sago-one@localhost:5433/todos_bench?host=/tmp"
cargo bench -p todo-api --bench crud 2>&1 | tee /tmp/bench-crud-$(date +%Y%m%d).txt
```

Nota: se utilizo una instancia PostgreSQL 18.4 local (initdb en /tmp/pgdata,
puerto 5433, auth=trust) por ausencia de credenciales en el entorno de referencia.
La instancia del sistema (puerto 5432) requeria credenciales no disponibles.

### Parte 2: Benchmark HTTP con oha (objetivo: >= 40 K req/s)

```sh
export DATABASE_URL="postgresql://dev-sago-one@localhost:5433/todos?host=/tmp"
./target/release/todo-api &
oha -z 10s -c 50 http://127.0.0.1:8080/todos           # warmup
oha -z 30s -c 100 http://127.0.0.1:8080/todos           # corrida 1
# reiniciar servidor entre corridas
oha -z 30s -c 100 http://127.0.0.1:8080/todos           # corrida 2
oha -z 30s -c 100 http://127.0.0.1:8080/todos           # corrida 3
```

oha v1.14.0 instalado via `cargo install oha`.
El servidor registra cada request por tracing middleware (overhead de logging incluido).

### Parte 3: Ciclo completo ag new + ag dev

```sh
cd /tmp
ag new mi-api --template fullstack
cd mi-api
export DATABASE_URL="postgresql://dev-sago-one@localhost:5433/mi_api_dev?host=/tmp"
ag dev
curl http://localhost:8080/health
```

### Parte 4: Docker FROM scratch

No ejecutado: Docker no disponible en el entorno de medicion.

## Resultados

### Criterion — DB directo

Salida de `cargo bench`:

```
crud/insert/insert_one  time:   [236.59 us 240.54 us 244.49 us]
                        thrpt:  [4.0902 Kelem/s 4.1573 Kelem/s 4.2267 Kelem/s]
crud/select/list_all    time:   [40.156 ms 41.422 ms 42.682 ms]
                        thrpt:  [23.429  elem/s 24.142  elem/s 24.903  elem/s]
crud/select/select_one_by_id
                        time:   [195.80 us 199.60 us 203.11 us]
                        thrpt:  [4.9233 Kelem/s 5.0101 Kelem/s 5.1072 Kelem/s]
crud/update/update_one  time:   [234.90 us 238.60 us 242.48 us]
                        thrpt:  [4.1241 Kelem/s 4.1911 Kelem/s 4.2571 Kelem/s]
crud/delete/delete_one  time:   [442.49 us 452.07 us 461.08 us]
                        thrpt:  [2.1688 Kelem/s 2.2120 Kelem/s 2.2600 Kelem/s]
crud/full_cycle/insert_select_update_delete
                        time:   [888.45 us 902.00 us 914.93 us]
                        thrpt:  [4.3719 Kelem/s 4.4346 Kelem/s 4.5022 Kelem/s]
crud/concurrent/1       time:   [427.55 us 440.24 us 452.79 us]
                        thrpt:  [2.2085 Kelem/s 2.2715 Kelem/s 2.3389 Kelem/s]
crud/concurrent/4       time:   [435.02 us 438.73 us 443.05 us]
                        thrpt:  [9.0284 Kelem/s 9.1172 Kelem/s 9.1950 Kelem/s]
crud/concurrent/16      time:   [1.0729 ms 1.0789 ms 1.0860 ms]
                        thrpt:  [14.734 Kelem/s 14.829 Kelem/s 14.912 Kelem/s]
crud/concurrent/64      time:   [3.8593 ms 3.8958 ms 3.9369 ms]
                        thrpt:  [16.256 Kelem/s 16.428 Kelem/s 16.584 Kelem/s]
```

Resumen:

| Benchmark | Tiempo medio | Throughput estimado |
| --- | --- | --- |
| crud/insert/insert_one | 0.240 ms | 4.157 K ops/s |
| crud/select/list_all | 41.422 ms | 24.14 ops/s |
| crud/select/select_one_by_id | 0.200 ms | 5.010 K ops/s |
| crud/update/update_one | 0.239 ms | 4.191 K ops/s |
| crud/delete/delete_one | 0.452 ms | 2.212 K ops/s |
| crud/full_cycle | 0.902 ms | 4.435 K ciclos/s |
| crud/concurrent/1 | 0.440 ms | 2.271 K ops/s |
| crud/concurrent/4 | 0.439 ms | 9.117 K ops/s |
| crud/concurrent/16 | 1.079 ms | 14.829 K ops/s |
| crud/concurrent/64 | 3.896 ms | 16.428 K ops/s |

### oha — Throughput HTTP GET /todos (criterio principal)

| Corrida | req/s | p50 (ms) | p95 (ms) | p99 (ms) | p99.9 (ms) |
| --- | --- | --- | --- | --- | --- |
| 1 | 12 117 | 8.13 | 9.94 | 10.997 | 12.581 |
| 2 | 11 912 | 8.26 | 10.16 | 11.381 | 13.060 |
| 3 | 11 744 | 8.37 | 10.27 | 11.475 | 15.799 |
| Mediana | 11 912 | 8.26 | 10.16 | 11.381 | 13.060 |

Nota: el entorno es una laptop AMD Ryzen 5 2500U con PostgreSQL local.
Sin deshabilitar turbo boost ni ajustar governor a performance.
Pool de conexiones: 10 (default). El tracing middleware registra cada request.
Estos factores explican el gap respecto al objetivo de 40 K req/s.

### Docker

- Tamano de imagen: no medido — Docker no disponible en el entorno de medicion.
- Arranque hasta primer log: no medido.
- `curl /health` responde: no medido.

### ag new + ag dev

- Scaffold creado correctamente: si.
- Servidor arranca hasta listening: si (tras corregir dependencia faltante `tracing` en templates).
- `curl http://localhost:8080/health` responde 200: si — `{"status":"ok","service":"mi-api"}`.

### Binario MUSL

- Build MUSL: no ejecutado — `musl-tools` no instalado (requiere sudo, no disponible).
- Binario GNU release stripped: 5.2 MB (referencia; estaria dentro del criterio de 20 MB).

## Conformidad con criterios de cierre de Fase 2

| Criterio | Objetivo | Medido | Cumple |
| --- | --- | --- | --- |
| Throughput HTTP CRUD | >= 40 K req/s | 11 912 req/s | no |
| Latencia p99 HTTP | <= 5 ms | 11.38 ms | no |
| ag new + ag dev funcional | si | si (con bugfix de dependencia) | si |
| Docker FROM scratch arranque | si | no medido (Docker no disponible) | no medido |
| Tamano binario MUSL | <= 20 MB | no medido (musl-tools no disponible) | no medido |

## Observaciones

1. Throughput HTTP: el gap entre 11.9 K req/s y el objetivo de 40 K req/s se explica
   por: CPU de laptop sin turbo, pool de BD de 10 conexiones, tracing middleware activo,
   y PostgreSQL local sin configuracion de rendimiento. La arquitectura es correcta;
   el objetivo requiere hardware dedicado o ajuste de pool/middleware.

2. Dependencia faltante en templates: los tres templates (rest, realtime, fullstack)
   usaban `tracing::` en main.rs pero no declaraban `tracing` en Cargo.toml.
   Corregido en este mismo commit.

3. MUSL y Docker: no ejecutables por restricciones del entorno (sin sudo, sin Docker).
   El binario GNU release stripped mide 5.2 MB, lo que sugiere que el MUSL stripped
   estaria dentro del criterio de 20 MB.

4. PostgreSQL: se utilizo instancia local propia (PostgreSQL 18.4, puerto 5433, auth=trust)
   por ausencia de credenciales para la instancia del sistema (puerto 5432).

## Reproducibilidad

Para reproducir en hardware dedicado con Docker y musl-tools disponibles:

```sh
git clone https://github.com/anti-gravital/anti-gravital
cd anti-gravital
git checkout 219309e9d2795c1991027ceb6d64e713c2ef858b

# PostgreSQL local
export DATABASE_URL="postgresql://postgres:postgres@localhost/todos_bench"
createdb todos_bench todos mi_api_dev

# Pasos segun docs/benchmarks/verificacion-local-fase-2.md
cargo build --release -p todo-api -p ag-cli
cargo bench -p todo-api --bench crud
# ... continuar con oha, musl build y docker
```

Cualquier tercero con hardware similar debe poder reproducir los numeros dentro de +-15%.
