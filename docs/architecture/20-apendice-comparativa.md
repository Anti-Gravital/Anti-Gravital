# Capitulo 20. Apendice: comparativa de mercado

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 20
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [19-glosario.md](./19-glosario.md)

## 20. Appendix: market comparison

This comparison is offered as a technical reference. The figures for the competitors are based on verifiable public benchmarks (TechEmpower, GitHub issues, official documentation). Those of Anti-Gravital are design objectives, not measurements.

| Criterion                      | Spring Boot   | .NET Core     | FastAPI      | NestJS       | Anti-Gravital (objective)   |
|--------------------------------|---------------|---------------|--------------|--------------|-----------------------------|
| Runtime                        | JVM           | CLR           | CPython      | Node.js V8   | None (native binary)        |
| Base memory                    | ~350 MB       | ~120 MB       | ~60 MB       | ~80 MB       | <= 15 MB                    |
| Startup time                   | ~6 s          | ~0.8 s        | ~0.8 s       | ~1.2 s       | <= 0.1 s                    |
| Hello World throughput         | ~75 K req/s   | ~200 K req/s  | ~28 K req/s  | ~45 K req/s  | >= 300 K req/s              |
| CRUD + DB throughput           | ~15 K req/s   | ~30 K req/s   | ~5 K req/s   | ~8 K req/s   | >= 40 K req/s               |
| Memory safety                  | Partial       | Partial       | Yes          | No           | Total (Rust compiler)       |
| GC pauses                      | Yes (JVM GC)  | Yes (CLR GC)  | Not applicable | Yes (V8 GC) | No (no GC)                  |
| Deployment as single binary    | No            | Partial       | No           | No           | Yes                         |
| Schema-first DX                | No            | No            | Partial      | No           | Yes (Anti-DSL)              |
| Compile-time verified queries   | No           | No            | No           | No           | Yes (sqlx)                  |
| Native DX for AI agents        | No            | No            | Partial      | No           | Yes                         |
| Native cross-compilation       | No            | No            | No           | No           | Yes                         |
| License                        | Apache 2.0    | MIT           | MIT          | MIT          | Apache 2.0                  |

---

**End of the Technical Architecture document.**
Complementary document: *Roadmap and Verification Gates.*
Unified PDF version: *Anti-Gravital Blueprint v4.0 — Master Document.*

---

