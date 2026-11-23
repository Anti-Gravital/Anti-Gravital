# Capitulo 18. Analisis de riesgos y mitigaciones

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 18
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [17-gobernanza-open-source.md](./17-gobernanza-open-source.md)
> Siguiente: [19-glosario.md](./19-glosario.md)

Any change that affects the public API, the DSL, the plugin architecture, or the security model requires an RFC. The process is: the proposer opens an issue in `anti-gravital-rfcs/`, the community debates for at least two weeks, the technical committee votes. Once approved, the RFC is moved to the "Accepted" state and is implemented in a specific version.

### 17.4 Compatibility

After version 1.0, the project commits to strict semver in the public API. Breaking changes only in majors. The LTS versions are announced with a public calendar, with at least 18 months of security support.

### 17.5 Economic sustainability

The project is sustained on three legs. The first is Gravital Labs (Nereira Technology and Business Solutions), which finances the initial development as a strategic investment. The second is professional services: adoption consulting, training, and premium support for companies that want an SLA, without this closing product features. The third, in the future, are corporate sponsors (GitHub Sponsors, Open Collective) of companies that depend on the project.

---

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

