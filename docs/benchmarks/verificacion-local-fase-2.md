# Verificacion local de Fase 2

Lista completa de comandos para ejecutar en la maquina Linux con PostgreSQL.
Al terminar, rellena `docs/benchmarks/measurement-fase-2-crud.md` con los
numeros reales y commitea.

## Requisitos previos

```sh
# Rust (stable, segun rust-toolchain.toml del proyecto)
rustup show   # debe mostrar 1.79 o superior

# PostgreSQL corriendo localmente
pg_isready    # debe responder "accepting connections"

# oha (cliente HTTP de carga)
cargo install oha
# o: apt install oha / brew install oha

# Docker (para la parte de FROM scratch)
docker info   # debe responder sin error

# Target MUSL para el build estatico
rustup target add x86_64-unknown-linux-musl
# Dependencias del sistema para MUSL:
sudo apt-get install -y musl-tools   # Debian/Ubuntu
# o: sudo dnf install musl-gcc       # Fedora
```

## Paso 1: Clonar y compilar

```sh
git clone https://github.com/anti-gravital/anti-gravital
cd anti-gravital
git checkout main
cargo build --release -p todo-api
cargo build --release -p ag-cli
```

Binario CLI disponible en: `./target/release/ag`

## Paso 2: ag new + ag dev

```sh
# Crear la base de datos de desarrollo
createdb mi_api_dev

# Crear y arrancar un proyecto nuevo
cd /tmp
/ruta/al/repositorio/target/release/ag new mi-api --template fullstack
cd mi-api

export DATABASE_URL="postgresql://postgres:postgres@localhost/mi_api_dev"

# Arrancar en modo dev (compila y ejecuta)
/ruta/al/repositorio/target/release/ag dev
```

En otra terminal, verificar:
```sh
curl http://localhost:8080/health
# Esperado: {"status":"ok","service":"mi-api"}
```

Criterio cumplido si arranca sin errores y /health responde 200.

## Paso 3: Benchmark Criterion (DB directo)

```sh
cd /ruta/al/repositorio

# Base de datos dedicada al bench (no mezclar con dev)
createdb todos_bench

export DATABASE_URL="postgresql://postgres:postgres@localhost/todos_bench"

# Ejecutar (puede tardar 5-10 minutos)
cargo bench -p todo-api --bench crud 2>&1 | tee /tmp/bench-crud-fase2.txt

# Los resultados en HTML quedan en:
# target/criterion/crud/*/report/index.html
```

Anotar los numeros en `docs/benchmarks/measurement-fase-2-crud.md`.

## Paso 4: Benchmark HTTP con oha (criterio 40 K req/s)

```sh
cd /ruta/al/repositorio

# Preparar base de datos y tabla
createdb todos
export DATABASE_URL="postgresql://postgres:postgres@localhost/todos"

# Arrancar servidor en release
./target/release/todo-api &
SERVER_PID=$!
sleep 2

# Insertar datos de muestra para que GET /todos no devuelva lista vacia
for i in $(seq 1 50); do
  curl -s -X POST http://localhost:8080/todos \
       -H "Content-Type: application/json" \
       -d "{\"title\": \"tarea $i\"}" > /dev/null
done

# Warmup (no registrar)
oha -z 10s -c 50 http://127.0.0.1:8080/todos > /dev/null 2>&1

# Corrida 1
oha -z 30s -c 100 http://127.0.0.1:8080/todos 2>&1 | tee /tmp/oha-r1.txt

# Reiniciar servidor
kill $SERVER_PID && sleep 2
./target/release/todo-api &
SERVER_PID=$!
sleep 2

# Corrida 2
oha -z 30s -c 100 http://127.0.0.1:8080/todos 2>&1 | tee /tmp/oha-r2.txt

# Reiniciar servidor
kill $SERVER_PID && sleep 2
./target/release/todo-api &
SERVER_PID=$!
sleep 2

# Corrida 3
oha -z 30s -c 100 http://127.0.0.1:8080/todos 2>&1 | tee /tmp/oha-r3.txt

kill $SERVER_PID

# Extraer req/s y p99 de cada corrida:
grep -E "Requests/sec|p99" /tmp/oha-r1.txt /tmp/oha-r2.txt /tmp/oha-r3.txt
```

Criterios:
- `Requests/sec` (mediana de 3 corridas) >= 40 000.
- p99 <= 5 ms.

## Paso 5: Build MUSL y tamano del binario

```sh
cd /ruta/al/repositorio

# Build estatico con MUSL
cargo build --release --target x86_64-unknown-linux-musl -p todo-api

# Medir tamano del binario sin strip
ls -lh target/x86_64-unknown-linux-musl/release/todo-api

# Medir tamano con strip (lo que va en la imagen Docker)
strip target/x86_64-unknown-linux-musl/release/todo-api
ls -lh target/x86_64-unknown-linux-musl/release/todo-api
```

Criterio: binario con strip <= 20 MB.

## Paso 6: Docker FROM scratch

```sh
cd /ruta/al/repositorio

# Build de la imagen (usa el Dockerfile en examples/todo-api/)
docker build -f examples/todo-api/Dockerfile -t todo-api:fase2 .

# Tamano de la imagen
docker images todo-api:fase2 --format "Tamano: {{.Size}}"

# Arrancar y verificar
docker run -d \
  -e DATABASE_URL="postgresql://host.docker.internal/todos" \
  -p 8080:8080 \
  --name todo-api-test \
  todo-api:fase2

sleep 2
curl http://localhost:8080/health

# Limpiar
docker stop todo-api-test && docker rm todo-api-test
```

Criterios:
- Build termina sin error.
- Imagen pesa <= 20 MB (la imagen, no solo el binario).
- `/health` responde `{"status":"ok","service":"todo-api"}`.

## Paso 7: Registrar resultados

Una vez completados los pasos anteriores:

1. Copiar `docs/benchmarks/measurement-fase-2-crud.md` a
   `docs/benchmarks/measurement-YYYY-MM-DD-fase-2-crud.md`.
2. Rellenar todos los campos `[RELLENAR]` con los valores reales.
3. Actualizar `docs/roadmap/STATUS.md`:
   - Marcar `[x]` los criterios cumplidos.
   - Para los numericos: anotar el valor medido junto al `[x]`.
4. Commitear y crear PR hacia main.

## Notas de entorno recomendadas

Para reducir varianza en los benchmarks:

```sh
# Deshabilitar Turbo Boost (Intel) durante el bench
echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo

# Governor a performance
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Aumentar file descriptors
ulimit -n 65536

# Aumentar backlog de conexiones
sudo sysctl -w net.core.somaxconn=65535
sudo sysctl -w net.ipv4.tcp_max_syn_backlog=65535

# Restaurar al terminar
echo 0 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo
echo powersave | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```
