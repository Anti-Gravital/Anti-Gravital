# fix(todo-api): correccion routing Axum 0.7 y benchmarks reales Fase 2

## Resumen

Correccion de bug critico de routing en `todo-api`: la ruta parametrizada
usaba sintaxis `{id}` de Axum 0.8 en lugar de `:id` de Axum 0.7. El workspace
usa `axum = "0.7.9"`. El bug causaba que GET, PUT y DELETE /todos/:id devolvieran
404 sin consultar PostgreSQL, invalidando todas las mediciones previas de
throughput para esas rutas.

Primera medicion real de CRUD con PostgreSQL nativo en Ryzen 5 2500U:
- HTTP layer sin DB: 88 930 req/s (Shield + Axum + Tokio)
- GET /todos/:id (SELECT por PK): 14 478 req/s mediana, p99 = 14.6 ms
- POST /todos (INSERT): 8 934 req/s mediana, p99 = 9.4 ms
- Cuello de botella: PostgreSQL con max_connections=100 en 4 nucleos fisicos

Los criterios de 40K req/s y p99 <= 5ms no se alcanzan en este hardware
con configuracion estandar de PostgreSQL. Documentado con analisis de causa.

## Fase afectada

Fase 2 - The Core MVP.

## Tipo de cambio

- Correccion de bug (routing Axum 0.7)
- Documentacion (benchmarks reales, actualizacion STATUS.md, CHANGELOG)

## Documentos relacionados

- `docs/benchmarks/measurement-2026-05-21-fase-2-crud-ryzen5-2500u.md`
- `docs/roadmap/STATUS.md` (criterios 2.3 actualizados)
- `CHANGELOG.md` (entrada nueva bajo [Unreleased])

## Archivos modificados

- `examples/todo-api/src/main.rs`: `"/todos/{id}"` -> `"/todos/:id"`
- `docs/benchmarks/measurement-2026-05-21-fase-2-crud-ryzen5-2500u.md`: nuevo
- `docs/roadmap/STATUS.md`: criterios benchmark y latencia actualizados
- `CHANGELOG.md`: entrada de correccion y documentacion

## Plan de prueba

- [x] `cargo fmt --check -p todo-api` pasa sin cambios
- [x] `cargo clippy -p todo-api --all-targets -- -D warnings` pasa limpio
- [x] `cargo test -p todo-api` pasa (0 tests, sin regresion)
- [x] `cargo build -p todo-api --release` compila correctamente
- [x] `GET /todos/1` devuelve 200 con datos reales (verificado con curl)
- [x] `POST /todos` crea registros correctamente (verificado con curl)
- [x] Benchmarks ejecutados con oha 1.14.0 contra servidor real + PG 18.4

## Criterios de salida que avanza

Ninguno nuevo se cierra: los criterios de throughput (40K req/s) y latencia
(p99 <= 5ms) quedan marcados como `[ ]` con numeros reales y explicacion.
Este PR aporta honestidad tecnica: reemplaza numeros invalidos por mediciones
verificables y documentadas.

## Checklist final

- [x] Pertenece a la fase correcta (Fase 2)
- [x] Respeta la documentacion y no altera arquitectura
- [x] No anade complejidad innecesaria (cambio minimo: 1 caracter en 1 linea)
- [x] Compila sin warnings
- [x] Pasa fmt y clippy
- [x] Documentacion actualizada junto al codigo
- [x] Mantiene coherencia con Anti-Gravital v4.0
