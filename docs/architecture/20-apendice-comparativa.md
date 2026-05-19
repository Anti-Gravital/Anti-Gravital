# Capitulo 20. Apendice: comparativa de mercado

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 20
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [19-glosario.md](./19-glosario.md)

## 20. Apéndice: comparativa de mercado

Esta comparativa se ofrece como referencia técnica. Las cifras de los competidores se basan en benchmarks públicos verificables (TechEmpower, GitHub issues, documentación oficial). Las de Anti-Gravital son objetivos de diseño, no mediciones.

| Criterio                       | Spring Boot   | .NET Core     | FastAPI      | NestJS       | Anti-Gravital (objetivo)    |
|--------------------------------|---------------|---------------|--------------|--------------|-----------------------------|
| Runtime                        | JVM           | CLR           | CPython      | Node.js V8   | Ninguno (binario nativo)    |
| Memoria base                   | ~350 MB       | ~120 MB       | ~60 MB       | ~80 MB       | ≤ 15 MB                     |
| Tiempo de arranque             | ~6 s          | ~0.8 s        | ~0.8 s       | ~1.2 s       | ≤ 0.1 s                     |
| Throughput Hello World         | ~75 K req/s   | ~200 K req/s  | ~28 K req/s  | ~45 K req/s  | ≥ 300 K req/s               |
| Throughput CRUD + DB           | ~15 K req/s   | ~30 K req/s   | ~5 K req/s   | ~8 K req/s   | ≥ 40 K req/s                |
| Memory safety                  | Parcial       | Parcial       | Sí           | No           | Total (compilador Rust)     |
| Pausas de GC                   | Sí (JVM GC)   | Sí (CLR GC)   | No aplica    | Sí (V8 GC)   | No (sin GC)                 |
| Despliegue como binario único  | No            | Parcial       | No           | No           | Sí                          |
| Schema-first DX                | No            | No            | Parcial      | No           | Sí (Anti-DSL)               |
| Queries verificadas compile-time | No          | No            | No           | No           | Sí (sqlx)                   |
| DX nativa para agentes AI      | No            | No            | Parcial      | No           | Sí                          |
| Compilación cruzada nativa     | No            | No            | No           | No           | Sí                          |
| Licencia                       | Apache 2.0    | MIT           | MIT          | MIT          | Apache 2.0                  |

---

**Fin del documento de Arquitectura Técnica.**
Documento complementario: *Hoja de Ruta y Puertas de Verificación.*
Versión PDF unificada: *Anti-Gravital Blueprint v4.0 — Documento Maestro.*
