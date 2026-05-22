# Benchmark Anti-Gravital — Neon PostgreSQL Real

**Fecha:** 2026-05-22 16:21 UTC

## Hardware y entorno

- **Hostname:** dev-sago-one-HP-ProBook-445-G6
- **OS:** linux
- **Arch:** x86_64
- **Rust:** rustc 1.95.0 (59807616e 2026-04-14)
- **Commit:** ba2ef69
- **Base de datos:** Neon PostgreSQL (pooler) — postgresql://***:***@ep-snowy-tooth-a4hy4h0p-pooler.us-east-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require
- **Duracion total:** 120 min 11 s

## Metodologia

Servicio HTTP Anti-Gravital (axum 0.7 + sqlx 0.8 + tokio rt-multi-thread) ejecutando 7 fases de carga contra Neon PostgreSQL serverless via pooler. Cada fase tiene un numero fijo de workers async concurrentes. Mix de operaciones: 35% POST (crear factura con 3 items + log en transaccion), 30% GET (factura + items, 2 queries), 20% GET list (filtrado por client_id+status), 15% PATCH (actualizar estado + insertar log en transaccion). Workers sin think time salvo cold-start (50ms). Pool de conexiones DB: 20 (Neon free tier).

## Resumen global

| Metrica | Valor |
|---|---|
| Total requests | 255805 |
| Total errores | 0 |
| Error rate | 0.00% |
| Throughput promedio | 35.5 req/s |
| Throughput pico | 43.0 req/s |

### Por tipo de operacion (global)

| Operacion | Requests | Errores | Mean ms | p50 ms | p95 ms | p99 ms | Max ms |
|---|---|---|---|---|---|---|---|
| GET  /invoices | 51179 | 0 | 1035.8 | 767.4 | 2181.7 | 3126.0 | 8633.1 |
| GET  /invoices/:id | 76744 | 0 | 2031.2 | 1499.1 | 4265.1 | 5982.7 | 11430.8 |
| PATCH /invoices/:id/status | 38495 | 0 | 1304.4 | 1034.3 | 2488.6 | 3549.9 | 10519.3 |
| POST /invoices | 89387 | 0 | 1535.0 | 1259.9 | 2756.9 | 3909.8 | 12660.8 |

## Resultados por fase

### Fase: cold-start (5 workers, 300 s)

- Requests: 3079  |  Errores: 0  |  Throughput: 10.3 req/s

| Operacion | Req | Err | Mean ms | p50 | p95 | p99 | Max |
|---|---|---|---|---|---|---|---|
| GET  /invoices | 639 | 0 | 181.7 | 143.6 | 236.2 | 959.6 | 4780.5 |
| GET  /invoices/:id | 883 | 0 | 369.9 | 287.8 | 461.5 | 2215.9 | 8907.3 |
| PATCH /invoices/:id/status | 475 | 0 | 452.8 | 360.0 | 528.0 | 3599.5 | 9776.0 |
| POST /invoices | 1082 | 0 | 696.7 | 577.7 | 928.7 | 3898.4 | 12660.8 |

### Fase: low-load (20 workers, 900 s)

- Requests: 27920  |  Errores: 0  |  Throughput: 31.0 req/s

| Operacion | Req | Err | Mean ms | p50 | p95 | p99 | Max |
|---|---|---|---|---|---|---|---|
| GET  /invoices | 5544 | 0 | 302.5 | 256.8 | 574.0 | 863.4 | 6535.4 |
| GET  /invoices/:id | 8401 | 0 | 589.0 | 514.3 | 986.1 | 1446.0 | 10520.2 |
| PATCH /invoices/:id/status | 4167 | 0 | 625.0 | 533.8 | 1080.9 | 1612.4 | 10519.3 |
| POST /invoices | 9808 | 0 | 895.2 | 783.3 | 1465.8 | 2150.9 | 11161.0 |

### Fase: normal (40 workers, 1801 s)

- Requests: 64668  |  Errores: 0  |  Throughput: 35.9 req/s

| Operacion | Req | Err | Mean ms | p50 | p95 | p99 | Max |
|---|---|---|---|---|---|---|---|
| GET  /invoices | 12892 | 0 | 690.0 | 630.2 | 1091.5 | 1536.6 | 2997.9 |
| GET  /invoices/:id | 19502 | 0 | 1359.9 | 1237.9 | 2097.2 | 2743.0 | 4471.6 |
| PATCH /invoices/:id/status | 9619 | 0 | 967.4 | 875.2 | 1516.4 | 2080.8 | 3986.6 |
| POST /invoices | 22655 | 0 | 1205.0 | 1086.0 | 1869.0 | 2525.0 | 4531.2 |

### Fase: peak (80 workers, 1802 s)

- Requests: 60475  |  Errores: 0  |  Throughput: 33.6 req/s

| Operacion | Req | Err | Mean ms | p50 | p95 | p99 | Max |
|---|---|---|---|---|---|---|---|
| GET  /invoices | 12190 | 0 | 1666.1 | 1475.4 | 2645.1 | 4089.0 | 8633.1 |
| GET  /invoices/:id | 17949 | 0 | 3293.2 | 2941.9 | 5088.1 | 7204.7 | 11430.8 |
| PATCH /invoices/:id/status | 9173 | 0 | 1953.3 | 1731.5 | 3159.8 | 4749.1 | 9628.5 |
| POST /invoices | 21163 | 0 | 2209.6 | 1957.7 | 3520.9 | 5370.4 | 10119.4 |

### Fase: max-stress (120 workers, 902 s)

- Requests: 37691  |  Errores: 0  |  Throughput: 41.8 req/s

| Operacion | Req | Err | Mean ms | p50 | p95 | p99 | Max |
|---|---|---|---|---|---|---|---|
| GET  /invoices | 7540 | 0 | 2076.8 | 2015.4 | 2674.3 | 3326.1 | 4011.6 |
| GET  /invoices/:id | 11185 | 0 | 4129.0 | 4004.9 | 5368.6 | 6368.6 | 7234.4 |
| PATCH /invoices/:id/status | 5693 | 0 | 2306.8 | 2236.0 | 2936.4 | 3683.9 | 4463.7 |
| POST /invoices | 13273 | 0 | 2504.8 | 2431.9 | 3146.9 | 3944.4 | 5084.9 |

### Fase: cooldown (25 workers, 900 s)

- Requests: 38733  |  Errores: 0  |  Throughput: 43.0 req/s

| Operacion | Req | Err | Mean ms | p50 | p95 | p99 | Max |
|---|---|---|---|---|---|---|---|
| GET  /invoices | 7784 | 0 | 315.9 | 306.1 | 407.4 | 511.6 | 705.5 |
| GET  /invoices/:id | 11784 | 0 | 602.5 | 595.6 | 682.5 | 961.0 | 1262.0 |
| PATCH /invoices/:id/status | 5777 | 0 | 540.4 | 523.3 | 681.5 | 874.2 | 1453.1 |
| POST /invoices | 13388 | 0 | 734.1 | 719.7 | 810.5 | 1172.3 | 1444.6 |

### Fase: sustained (50 workers, 602 s)

- Requests: 23239  |  Errores: 0  |  Throughput: 38.6 req/s

| Operacion | Req | Err | Mean ms | p50 | p95 | p99 | Max |
|---|---|---|---|---|---|---|---|
| GET  /invoices | 4590 | 0 | 849.2 | 805.2 | 1217.3 | 1690.1 | 2890.4 |
| GET  /invoices/:id | 7040 | 0 | 1661.6 | 1570.1 | 2381.2 | 2870.9 | 4039.9 |
| PATCH /invoices/:id/status | 3591 | 0 | 1090.5 | 1033.4 | 1536.8 | 2038.4 | 2919.5 |
| POST /invoices | 8018 | 0 | 1314.9 | 1237.2 | 1897.4 | 2387.6 | 3344.7 |

## Criterio Fase 3: CRUD DSL vs manual

Este benchmark mide el stack Anti-Gravital (axum + sqlx) contra una base de datos PostgreSQL real (Neon serverless). Los handlers fueron escritos manualmente (equivalente al codigo generado por ag-dsl v0.4). La comparativa directa DSL-generado vs manual requiere ejecutar `cargo bench -p todo-api` con el mismo schema, pendiente en gate de Fase 4.

## Notas

- Neon PostgreSQL serverless en us-east-1 (AWS). Latencia de red incluida.
- Pool de 20 conexiones compartidas entre todos los workers.
- Operaciones de escritura (POST, PATCH) usan transacciones explicitas.
- Tablas con prefijo `ag_bench_` creadas para el benchmark.
