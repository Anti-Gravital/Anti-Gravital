# Capitulo 17. Modelo de gobernanza Open Source

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 17
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [16-rendimiento-y-validacion.md](./16-rendimiento-y-validacion.md)
> Siguiente: [18-riesgos-y-mitigaciones.md](./18-riesgos-y-mitigaciones.md)

## 17. Open Source governance model

### 17.1 License and promise

The license is Apache 2.0 for the entire ecosystem. There is not and will not be a closed Enterprise version with features reserved for paying customers. The commitment is explicit and is documented in the README. Any future license change would require the approval of the entire community of maintainers, and the ecosystem remains forkable.

### 17.2 Maintenance model

The project adopts an initial BDFL model with a transition plan to explicit meritocracy. In the initial phase (0.x versions), Angel Nereira is the principal maintainer. Starting from version 1.0, a technical committee of five people elected among the contributors with the greatest track record is established. The committee approves RFCs (Request For Comments) for major changes.

### 17.3 RFCs

Any change that affects the public API, the DSL, the plugin architecture, or the security model requires an RFC. The process is: the proposer opens an issue in `anti-gravital-rfcs/`, the community debates for at least two weeks, the technical committee votes. Once approved, the RFC is moved to the "Accepted" state and is implemented in a specific version.

### 17.4 Compatibility

After version 1.0, the project commits to strict semver in the public API. Breaking changes only in majors. The LTS versions are announced with a public calendar, with at least 18 months of security support.

### 17.5 Economic sustainability

The project is sustained on three legs. The first is Gravital Labs (Nereira Technology and Business Solutions), which finances the initial development as a strategic investment. The second is professional services: adoption consulting, training, and premium support for companies that want an SLA, without this closing product features. The third, in the future, are corporate sponsors (GitHub Sponsors, Open Collective) of companies that depend on the project.

---

