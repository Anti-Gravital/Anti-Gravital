# Fase 10 - Endurecimiento y hito 1.0

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-09-plugins-wasi.md](./fase-09-plugins-wasi.md)
> Siguiente: [mas-alla-de-1.0.md](./mas-alla-de-1.0.md)

## Phase 10 — Hardening and 1.0 milestone

**Objective.** Bring the project to stable version 1.0. It is the phase of audits, hardening, final optimization, and public declaration of stability.

### 10.1 Entry criteria

- [ ] Phase 9 completed.
- [ ] DSL version 1.0 (stable grammar) ready for freeze.
- [ ] The technical committee is active and operational.

### 10.2 Deliverables

- [ ] DSL version 1.0 (stable grammar, frozen).
- [ ] Test coverage ≥ 85% in all workspace crates.
- [ ] 72-hour fuzzing over the DSL parser without crashes.
- [ ] 72-hour fuzzing over the HTTP parser without crashes.
- [ ] External security audit of the Shield component, contracted with a specialized company (Trail of Bits, NCC Group or equivalent). Public report.
- [ ] Resolution of all critical and high findings of the audit.
- [ ] Load test: 500 K req/s sustained for 30 minutes with degradation ≤ 5%.
- [ ] Memory leak test: 24 hours of continuous load without detectable memory growth.
- [ ] Compilation verified on: Linux x86-64, Linux ARM64, macOS ARM64, Windows x64.
- [ ] Compilation to `wasm32-wasi` to serve Anti-Gravital in edge functions.
- [ ] Official manual published: "The Anti-Gravital Book" in Spanish and English.
- [ ] Framework introduction course on YouTube (minimum six videos).
- [ ] Position in TechEmpower Framework Benchmarks: top 10 in Plaintext and JSON Serialization categories.

### 10.3 Exit criteria (version 1.0)

- [ ] At least three external projects using Anti-Gravital in production for at least 30 days without critical incidents.
- [ ] At least one internal Gravital Cloud service using Anti-Gravital in production for 30 days without critical incidents.
- [ ] Public announcement of version 1.0 with complete changelog.
- [ ] Commitment to strict semver from 1.0.
- [ ] Announcement of the LTS version calendar.
- [ ] Talk at at least one international conference (RustConf, EuroRust, RustNation or equivalent).
- [ ] At least 10 000 stars on the repository.
- [ ] The technical committee ratifies the promotion to version 1.0 unanimously.

### 10.4 Phase risks

The main risk is the pressure to release 1.0 before time. The mitigation is the project's strictest rule: the exit criteria are non-negotiable. If they are not met, 1.0 is not released. 0.9.5, 0.9.6 are released, until they are met.

---

