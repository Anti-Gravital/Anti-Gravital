# Capitulo 17. Modelo de gobernanza Open Source

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 17
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [16-rendimiento-y-validacion.md](./16-rendimiento-y-validacion.md)
> Siguiente: [18-riesgos-y-mitigaciones.md](./18-riesgos-y-mitigaciones.md)


- Top-10 position in a TechEmpower Round (Plaintext and JSON Serialization categories)
- External security audit without unresolved critical findings
- 72 hours of fuzzing of the DSL parser and the HTTP parser without crashes
- Load test of 500 K req/s sustained for 30 minutes without degradation >5%
- 24 hours of continuous load without detectable memory growth
- Binaries verified on Linux x86-64, Linux ARM64, macOS ARM64, Windows x64
- At least one service in production on Gravital Cloud for 30 days without incidents
- At least three external projects using Anti-Gravital in production

---

## 17. Open Source governance model

### 17.1 License and promise

The license is Apache 2.0 for the entire ecosystem. There is not and will not be a closed Enterprise version with features reserved for paying customers. The commitment is explicit and is documented in the README. Any future license change would require the approval of the entire community of maintainers, and the ecosystem remains forkable.

### 17.2 Maintenance model

The project adopts an initial BDFL model with a transition plan to explicit meritocracy. In the initial phase (0.x versions), Angel Nereira is the principal maintainer. Starting from version 1.0, a technical committee of five people elected among the contributors with the greatest track record is established. The committee approves RFCs (Request For Comments) for major changes.

### 17.3 RFCs

