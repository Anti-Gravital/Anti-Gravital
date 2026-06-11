# Capitulo 18. Analisis de riesgos y mitigaciones

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 18
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [17-gobernanza-open-source.md](./17-gobernanza-open-source.md)
> Siguiente: [19-glosario.md](./19-glosario.md)

## 18. Risk analysis and mitigations

This section documents the real risks of the project and the planned mitigations. It is deliberately honest; a project that does not enumerate its risks does not deserve trust.

### 18.1 Risk: DSL compiler complexity

The DSL compiler is a several-year project on its own. The mitigation is the incremental implementation by DSL versions described in section 7. Version 0.1 covers only basic models and is deliverable in two months. Each version adds a well-defined subset. The stable 1.0 version of the DSL is the highest-risk milestone of the project and is planned for the end of the schedule.

### 18.2 Risk: Rust learning curve

Rust has a real learning curve. The mitigation is threefold. First, the DSL generates 80% of the scaffolding, so that the handlers the developer writes are simple Rust: a few `await`s, access to shared state, returning a `Result`. Second, the documentation includes a "Rust for Python/Node.js developers" guide with the minimum necessary concepts. Third, the integrated AI assistant can generate handlers that the developer supervises.

### 18.3 Risk: competition with big players

Spring, .NET, Express, and FastAPI have decades-old ecosystems. Anti-Gravital cannot compete frontally with them in breadth. The mitigation is to focus on niches where the incumbents have structural weaknesses: high-load applications, edge services, backends for Flutter, backends for AI applications with streaming.

### 18.4 Risk: bus factor

The initial project has a worryingly low bus factor (one maintainer). The mitigation is active: complete internal documentation from day one, incorporation of external contributors from phase 1, and transition to a technical committee before 1.0.

### 18.5 Risk: changes in the Rust ecosystem

The Rust ecosystem continues to evolve rapidly. Axum, Tokio, and sqlx may make breaking changes in future versions. The mitigation is conservative version pinning, exhaustive integration tests against each new version of the core dependencies, and active participation in their communities to anticipate changes.

### 18.6 Risk: community fragmentation

If the Anti-Gravital community fragments (for example, competing forks with divergent features emerge), the ecosystem weakens. The mitigation is an open RFC process that gives a real voice to the community, predictable releases, and a public roadmap.

### 18.7 Risk: post-launch security vulnerabilities

Although Rust eliminates many categories of vulnerabilities, it does not eliminate the logical ones (broken authorization, information leaks, application-level races). The mitigation is the external audit before 1.0, the responsible disclosure program, continuous fuzzing, and CI with static analysis (clippy, cargo-audit, cargo-deny).

---

