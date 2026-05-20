# Medicion oficial: CRUD + PostgreSQL — Fase 2

Plantilla pre-rellenada para el cierre de Fase 2.
Ejecutar en la maquina de referencia, rellenar todos los campos marcados
con `[RELLENAR]` y commitear el resultado.

Copiar como:
`docs/benchmarks/measurement-YYYY-MM-DD-fase-2-crud.md`

## Identidad del reporte

- Fecha: [RELLENAR] (YYYY-MM-DD).
- Reportero: [RELLENAR] (nombre humano).
- Tipo de medicion: throughput HTTP + latencia CRUD + throughput DB directo.
- Componente: `todo-api` + `ag-data` + `ag-core` Fase 2.
- Comentario: validacion de cierre de Fase 2 — criterios 40 K req/s y p99 <= 5 ms.

## Entorno

### Hardware

- CPU: [RELLENAR] — `lscpu | grep -E "Model name|CPU\(s\)|MHz"`.
- RAM: [RELLENAR] — `free -h`.
- Almacenamiento: [RELLENAR] — `lsblk -d -o NAME,MODEL,ROTA`.
- Red: loopback (127.0.0.1) para toda la medicion.

### Sistema operativo

```sh
# Ejecutar y pegar salida:
cat /etc/os-release | head -3
uname -r
ulimit -n
sysctl net.core.somaxconn
```

Salida: [RELLENAR]

### Toolchain

```sh
# Ejecutar y pegar salida:
rustc --version
cat rust-toolchain.toml
```

Salida: [RELLENAR]

### Repositorio

```sh
# Ejecutar y pegar salida:
git rev-parse HEAD
git branch --show-current
git status --short
```

Salida: [RELLENAR]

## Metodologia

### Parte 1: Benchmark DB directo (Criterion)

Mide latencia de operaciones sqlx directamente contra PostgreSQL,
sin overhead HTTP ni Axum.

```sh
export DATABASE_URL="postgresql://postgres:postgres@localhost/todos_bench"
createdb todos_bench   # si no existe
cargo bench -p todo-api --bench crud 2>&1 | tee /tmp/bench-crud-$(date +%Y%m%d).txt
```

### Parte 2: Benchmark HTTP con oha (objetivo: >= 40 K req/s)

Mide throughput real de la API REST con conexiones concurrentes.

```sh
# Terminal 1: arrancar el servidor
export DATABASE_URL="postgresql://postgres:postgres@localhost/todos"
createdb todos  # si no existe
cargo build --release -p todo-api
./target/release/todo-api &
SERVER_PID=$!

# Esperar a que arranque
sleep 2

# Terminal 2: warmup (no registrar)
oha -z 10s -c 50 http://127.0.0.1:8080/todos

# Corrida 1 (registrar)
oha -z 30s -c 100 http://127.0.0.1:8080/todos 2>&1 | tee /tmp/oha-run1.txt

# Reiniciar servidor para corrida 2
kill $SERVER_PID && sleep 1
./target/release/todo-api &
SERVER_PID=$!
sleep 2

# Corrida 2
oha -z 30s -c 100 http://127.0.0.1:8080/todos 2>&1 | tee /tmp/oha-run2.txt

# Reiniciar servidor para corrida 3
kill $SERVER_PID && sleep 1
./target/release/todo-api &
SERVER_PID=$!
sleep 2

# Corrida 3
oha -z 30s -c 100 http://127.0.0.1:8080/todos 2>&1 | tee /tmp/oha-run3.txt

kill $SERVER_PID
```

Instalar oha si no esta disponible:
```sh
cargo install oha
# o:
# brew install oha
# o descargar desde https://github.com/hatoo/oha/releases
```

### Parte 3: Ciclo completo ag new + ag dev

```sh
# En un directorio temporal
cd /tmp
ag new mi-api --template fullstack
cd mi-api
export DATABASE_URL="postgresql://postgres:postgres@localhost/mi_api_dev"
createdb mi_api_dev
ag dev
# Verificar que el servidor arranca y responde en http://localhost:8080
```

### Parte 4: Docker FROM scratch

```sh
# En la raiz del repositorio clonado
docker build -f examples/todo-api/Dockerfile -t todo-api:fase2 .

# Verificar tamano de imagen
docker images todo-api:fase2

# Verificar que arranca
docker run -e DATABASE_URL="postgresql://host.docker.internal/todos" \
           -p 8080:8080 todo-api:fase2 &
curl http://localhost:8080/health
```

## Resultados

### Criterion — DB directo

Pegar salida de `cargo bench`:

```
[RELLENAR — pegar output de criterion aqui]
```

Resumen:

| Benchmark | Tiempo medio | Throughput estimado |
| --- | --- | --- |
| crud/insert/insert_one | [RELLENAR] ms | [RELLENAR] ops/s |
| crud/select/list_all | [RELLENAR] ms | [RELLENAR] ops/s |
| crud/select/select_one_by_id | [RELLENAR] ms | [RELLENAR] ops/s |
| crud/update/update_one | [RELLENAR] ms | [RELLENAR] ops/s |
| crud/delete/delete_one | [RELLENAR] ms | [RELLENAR] ops/s |
| crud/full_cycle | [RELLENAR] ms | [RELLENAR] ciclos/s |
| crud/concurrent/1 | [RELLENAR] ms | [RELLENAR] ops/s |
| crud/concurrent/4 | [RELLENAR] ms | [RELLENAR] ops/s |
| crud/concurrent/16 | [RELLENAR] ms | [RELLENAR] ops/s |
| crud/concurrent/64 | [RELLENAR] ms | [RELLENAR] ops/s |

### oha — Throughput HTTP GET /todos (criterio principal)

| Corrida | req/s | p50 (ms) | p95 (ms) | p99 (ms) | p99.9 (ms) |
| --- | --- | --- | --- | --- | --- |
| 1 | [RELLENAR] | | | | |
| 2 | [RELLENAR] | | | | |
| 3 | [RELLENAR] | | | | |
| Mediana | [RELLENAR] | | | | |

### Docker

- Tamano de imagen: [RELLENAR] MB (`docker images todo-api:fase2 --format "{{.Size}}"`).
- Arranque hasta primer log: [RELLENAR] ms.
- `curl /health` responde: [RELLENAR] (si/no).

### ag new + ag dev

- Scaffold creado correctamente: [RELLENAR] (si/no).
- Servidor arranca hasta listening: [RELLENAR] (si/no).
- `curl http://localhost:8080/health` responde 200: [RELLENAR] (si/no).

## Conformidad con criterios de cierre de Fase 2

| Criterio | Objetivo | Medido | Cumple |
| --- | --- | --- | --- |
| Throughput HTTP CRUD | >= 40 K req/s | [RELLENAR] | si/no |
| Latencia p99 HTTP | <= 5 ms | [RELLENAR] | si/no |
| ag new + ag dev funcional | si | [RELLENAR] | si/no |
| Docker FROM scratch arranque | si | [RELLENAR] | si/no |
| Tamano binario MUSL | <= 20 MB | [RELLENAR] MB | si/no |

## Observaciones

[RELLENAR — notas sobre el entorno, anomalias, condiciones del sistema]

## Reproducibilidad

Cualquier tercero con el mismo commit y hardware similar debe poder
reproducir los numeros dentro de +-15%. Si no es posible, se considera
fallo segun regla 36 de CLAUDE.md.
