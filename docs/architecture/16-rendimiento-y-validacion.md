# Capitulo 16. Objetivos de rendimiento y metodologia de validacion

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 16
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [15-seguridad.md](./15-seguridad.md)
> Siguiente: [17-gobernanza-open-source.md](./17-gobernanza-open-source.md)

## 16. Performance objectives and validation methodology

This section replaces the absolute benchmarks of v3.0. The previous figures were presented as facts when in reality they are extrapolations of individual components. This version rephrases them honestly as **design objectives**, against which the project will measure itself publicly.

### 16.1 Design objectives

| Metric                                                     | Objective                 | Extrapolation basis                          |
|------------------------------------------------------------|---------------------------|----------------------------------------------|
| Hello World throughput (plaintext)                         | >= 300 K req/s            | Axum + Tokio in TechEmpower                  |
| Simple JSON throughput                                     | >= 150 K req/s            | Axum + serde_json in public benchmarks       |
| CRUD throughput with PostgreSQL                            | >= 40 K req/s             | sqlx + connection pool                       |
| p99 latency with DB query                                  | <= 5 ms                   | Measurements of Tokio services in production |
| Base memory (idle process, no traffic)                     | <= 15 MB                  | Size of Rust + Tokio binaries                |
| Cold start time                                            | <= 100 ms                 | Static Rust binaries on Linux                |
| Release binary size with all standard modules              | <= 20 MB                  | Compilations of similar projects             |
| Concurrent WebSocket connections on a 2 vCPU instance      | >= 50 000                 | Tokio stackless tasks                        |

These figures are technical objectives. The project specification requires that they be measured with the `ag bench` suite in the repository, and that each release publish the reproducible results. If a metric is not reached, it is published as such and the deficit is documented. The technical credibility of the project depends on not exaggerating.

### 16.2 Measurement methodology

Any comparison with competing frameworks is done under the TechEmpower Framework Benchmarks, run by the team or by independent third parties. The comparisons published in the documentation include: the exact version of the compared framework, the configuration used, the benchmark hardware, the number of runs, and the standard deviation. Comparisons that do not comply with these rules are not published.

### 16.3 Validation milestones for v1.0

The stable 1.0 version is released only when the following milestones are met:

- Top-10 position in a TechEmpower Round (Plaintext and JSON Serialization categories)
- External security audit without unresolved critical findings
- 72 hours of fuzzing of the DSL parser and the HTTP parser without crashes
- Load test of 500 K req/s sustained for 30 minutes without degradation >5%
- 24 hours of continuous load without detectable memory growth
- Binaries verified on Linux x86-64, Linux ARM64, macOS ARM64, Windows x64
- At least one service in production on Gravital Cloud for 30 days without incidents
- At least three external projects using Anti-Gravital in production

---

