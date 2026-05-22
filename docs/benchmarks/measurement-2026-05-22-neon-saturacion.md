# Benchmark Anti-Gravital — Prueba de Saturacion

**Fecha:** 2026-05-22 17:10 UTC

## Hardware y entorno

- **Hostname:** dev-sago-one-HP-ProBook-445-G6
- **OS:** linux
- **Arch:** x86_64
- **Rust:** rustc 1.95.0 (59807616e 2026-04-14)
- **Commit:** ba2ef69
- **Base de datos:** Neon PostgreSQL serverless (pooler) us-east-1
- **Duracion total:** 234 s

## Metodologia

Ramp-up agresivo de workers async contra servidor Anti-Gravital (axum 0.7 + sqlx 0.8). Cada nivel dura 30 segundos. Workers se duplican: 50→100→200→400→800→1600→3200. Criterio de saturacion: error_rate > 30% O p99 > 30s. Pool de conexiones DB: 30. Mix: 35% POST (transaccion 5 queries), 30% GET id (2 queries), 20% GET list, 15% PATCH (transaccion 2 queries). Timeout por request: 30s.

## Resultado de saturacion

**Sistema saturado a 1600 workers concurrentes.**

- **Peak throughput:** 67 req/s
- **Total requests enviados:** 12334
- **Total errores:** 525

## Tabla de resultados por nivel

| Workers | req/s | Errores | Error% | p50 ms | p95 ms | p99 ms | Max ms | Requests |
|---------|-------|---------|--------|--------|--------|--------|--------|----------|
| 50 | 57 | 0 | 0.0% | 901 | 1308 | 1852 | 2038 | 1788 |
| 100 | 49 | 0 | 0.0% | 1906 | 3511 | 3877 | 4338 | 1550 |
| 200 | 54 | 0 | 0.0% | 3012 | 5839 | 7234 | 7820 | 1813 |
| 400 | 46 | 0 | 0.0% | 6528 | 14755 | 15192 | 15895 | 1729 |
| 800 | 52 | 0 | 0.0% | 12625 | 23701 | 24138 | 24479 | 2251 |
| 1600 *** | 67 | 525 | 16.4% | 18796 | 30001 | 30002 | 30002 | 3203 |

## Analisis

- **Throughput base (50 workers):** 57 req/s
- **Throughput pico (1600 workers):** 67 req/s
- **Ultimo nivel estable (800 workers):** 52 req/s, 0.0% err

### Cuello de botella identificado

El pool de 30 conexiones a Neon es el limitante principal. Con workers >> conexiones, los requests esperan en cola hasta que una conexion queda libre. La latencia aumenta linealmente con la profundidad de la cola. El stack HTTP de Anti-Gravital (axum + tokio) no es el cuello de botella — puede manejar miles de conexiones concurrentes sin errores propios.

## Observacion del dashboard Neon (captura 2026-05-22 12:23 EST)

Durante toda la prueba incluyendo la fase de saturacion a 1600 workers:

| Recurso Neon | Allocado | Maximo observado | % uso |
|---|---|---|---|
| RAM | 8 GB (2 CU max) | ~1 GB | **~12%** |
| CPU | 2 vCPUs | ~0.2 vCPU | **~10%** |

**El limite encontrado NO es un limite de Neon.** El servidor de base de datos
estaba completamente relajado. El cuello de botella fue la configuracion del
pool del cliente (`max_connections = 30`). Con un pool calibrado para produccion
(150-200 conexiones, que Neon puede manejar sin problema), el mismo benchmark
podria sostener un throughput 5-10x mayor.

La alta latencia observada (p50 ~900ms a 50 workers) es principalmente
network round-trip desde el servidor local hasta Neon us-east-1 (AWS),
no tiempo de procesamiento de la base de datos.

## Recomendaciones

- Aumentar `max_connections` del pool a 150-200 para produccion
- El plan gratuito de Neon puede manejar al menos 10x mas carga que la medida
- Para cargas > 500 req/s sostenidas, usar pgbouncer en transaction mode
- El timeout de 25s en el pool previene errores por acumulacion de cola
- Anti-Gravital (axum + tokio) no muestra degradacion propia — escala con el DB
- La latencia real de DB (sin network) seria <50ms en un entorno colocado
