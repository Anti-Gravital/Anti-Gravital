# Objetivos de rendimiento y metodologia de validacion

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 16.

## 16. Objetivos de rendimiento y metodología de validación

Esta sección sustituye los benchmarks absolutos del v3.0. Las cifras anteriores se presentaban como hechos cuando en realidad son extrapolaciones de componentes individuales. Esta versión las refrasea honestamente como **objetivos de diseño**, contra los cuales el proyecto se medirá públicamente.

### 16.1 Objetivos de diseño

| Métrica                                                    | Objetivo                  | Base de extrapolación                        |
|------------------------------------------------------------|---------------------------|----------------------------------------------|
| Throughput Hello World (plaintext)                         | ≥ 300 K req/s             | Axum + Tokio en TechEmpower                  |
| Throughput JSON simple                                     | ≥ 150 K req/s             | Axum + serde_json en benchmarks públicos     |
| Throughput CRUD con PostgreSQL                             | ≥ 40 K req/s              | sqlx + connection pool                       |
| Latencia p99 con DB query                                  | ≤ 5 ms                    | Mediciones de servicios Tokio en producción  |
| Memoria base (proceso idle, sin tráfico)                   | ≤ 15 MB                   | Tamaño de binarios Rust + Tokio              |
| Tiempo de arranque en frío                                 | ≤ 100 ms                  | Binarios Rust estáticos en Linux             |
| Tamaño del binario release con todos los módulos estándar  | ≤ 20 MB                   | Compilaciones de proyectos similares         |
| Conexiones WebSocket concurrentes en una instancia 2 vCPU  | ≥ 50 000                  | Tokio tasks stackless                        |

Estas cifras son objetivos técnicos. La especificación del proyecto exige que sean medidas con la suite `ag bench` en el repositorio, y que cada release publique los resultados reproducibles. Si una métrica no se alcanza, se publica como tal y se documenta el déficit. La credibilidad técnica del proyecto depende de no exagerar.

### 16.2 Metodología de medición

Toda comparación con frameworks competidores se hace bajo TechEmpower Framework Benchmarks, ejecutado por el equipo o por terceros independientes. Las comparaciones publicadas en la documentación incluyen: versión exacta del framework comparado, configuración usada, hardware del benchmark, número de runs y desviación estándar. Comparaciones que no cumplan estas reglas no se publican.

### 16.3 Hitos de validación para v1.0

La versión 1.0 estable se libera solo cuando se cumplen los siguientes hitos:

- Posición top-10 en TechEmpower Round (categorías Plaintext y JSON Serialization)
- Auditoría externa de seguridad sin findings críticos sin resolver
- 72 horas de fuzzing del parser DSL y el parser HTTP sin crashes
- Load test de 500 K req/s sostenidos por 30 minutos sin degradación >5%
- 24 horas de carga continua sin crecimiento de memoria detectable
- Binarios verificados en Linux x86-64, Linux ARM64, macOS ARM64, Windows x64
- Al menos un servicio en producción en Gravital Cloud por 30 días sin incidentes
- Al menos tres proyectos externos usando Anti-Gravital en producción

---

## Resultados disponibles

| Archivo | Fecha | Fase | Descripcion |
|---|---|---|---|
| `measurement-2026-05-21-fase-2-crud-ryzen5-2500u.md` | 2026-05-21 | 2 | CRUD con PostgreSQL nativo en Ryzen 5 2500U. GET 14 478 req/s, POST 8 934 req/s. |
| `measurement-2026-05-22-neon-real.md` | 2026-05-22 | 3 | Benchmark 2 horas contra Neon PostgreSQL serverless. 255 805 requests, 0 errores, peak 43 req/s. |
| `measurement-2026-05-22-neon-saturacion.md` | 2026-05-22 | 3 | Prueba de saturacion. Estable a 800 workers, quiebre a 1600 por pool de conexiones. |

