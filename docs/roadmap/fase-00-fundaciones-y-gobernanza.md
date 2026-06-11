# Fase 0 - Fundaciones y gobernanza

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [preambulo.md](./preambulo.md)
> Siguiente: [fase-01-shield-mvp.md](./fase-01-shield-mvp.md)

## Phase 0 — Foundations and governance

**Objective.** Create the project's foundations: repository, license, governance documentation, CI, contributors, communication with the community. No code yet. The product of this phase is an open source project fit to receive collaborators.

### 0.1 Entry criteria

- [ ] Final decision to begin Anti-Gravital as a formal Gravital Labs project.
- [ ] Approval of Apache 2.0 license without restrictions.
- [ ] Public commitment from Ángel Nereira as initial maintainer.

### 0.2 Deliverables

- [ ] Repository `github.com/gravital-labs/anti-gravital` created and public.
- [ ] `LICENSE` file with complete Apache 2.0 text.
- [ ] Bilingual `README.md` file (Spanish + English) with value proposition.
- [ ] `CONTRIBUTING.md` file with contribution guide, code conventions, pull request process.
- [ ] `CODE_OF_CONDUCT.md` file adopting Contributor Covenant 2.1.
- [ ] `SECURITY.md` file with responsible disclosure policy and address `anti@gravitalcloud.com` (backup: `angelnereira@gravitalcloud.com`).
- [ ] `GOVERNANCE.md` file describing initial BDFL model and transition plan.
- [ ] CI configuration with GitHub Actions: build on Linux x86-64, Linux ARM64, macOS ARM64, Windows x64.
- [ ] Issue templates (bug report, feature request, RFC) and pull request template.
- [ ] Basic branding: logo, color palette, typography. Applied to the README.
- [ ] Official project Discord with channels `#español`, `#english`, `#announcements`, `#help`.
- [ ] Project account on X/Bluesky for announcements.
- [ ] Domain `antigravital.dev` registered and pointing to a minimal landing page.
- [ ] Institutional email `anti@gravitalcloud.com` operational (project root email).
- [ ] Public release calendar published.

### 0.3 Exit criteria (gate before Phase 1)

- [ ] The repository receives its first unsolicited external star.
- [ ] At least five external people have joined the Discord.
- [ ] The monorepo's folder structure is defined and committed (although without functional content).
- [ ] The Cargo workspace is initialized with the empty crates: `ag-core`, `ag-dsl`, `ag-cli`, `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe`, `ag-ui`, `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`, `ag-wasm-host`.
- [ ] The CI successfully builds the empty workspace on the four target platforms.
- [ ] The landing page describes in one paragraph what the project is, what it is not, and where it is on the roadmap.

### 0.4 Phase risks

The main risk is procrastination due to perfectionism. Phase 0 does not produce code that runs, which tempts to postpone it. The mitigation is a strict timebox: 8 weeks maximum. If by the end not all deliverables are in place, it concludes with whatever exists and the pending items are documented as phase 0 technical debt to be resolved during phase 1.

---

