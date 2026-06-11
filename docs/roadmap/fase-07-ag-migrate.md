# Fase 7 - ag-migrate importadores

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-06-ag-ai-knowledge-graph.md](./fase-06-ag-ai-knowledge-graph.md)
> Siguiente: [fase-08-ag-mobile.md](./fase-08-ag-mobile.md)

## Phase 7 — `ag-migrate` importers

**Objective.** Build the migration importers from legacy frameworks. It is probably the phase with the greatest impact on real adoption.

### 7.1 Entry criteria

- [ ] Phase 6 completed.
- [ ] Research of real samples: at least ten schemas/projects of each target framework collected as a testing corpus.

### 7.2 Deliverables

- [ ] `ag-migrate` crate with five importers:
  - [ ] OpenAPI 3.0 and 3.1 importer.
  - [ ] Prisma importer.
  - [ ] Django importer.
  - [ ] FastAPI importer.
  - [ ] Sequelize importer.
  - [ ] GraphQL SDL importer.
- [ ] `ag migrate from <framework> <path>` command.
- [ ] Official migration guides per framework with complete examples.
- [ ] Documented case study: real migration of a medium-sized FastAPI application.

### 7.3 Exit criteria (gate before Phase 8)

- [ ] Each importer has test coverage ≥ 80% over the corpus of real projects.
- [ ] The FastAPI migration guide has been validated by at least one external team that migrated its application.
- [ ] At least 3 500 stars on the repository.

### 7.4 Phase risks

The importers cover the translation of the contract, not the business logic. The risk is generating exaggerated expectations. The mitigation is honest documentation about what is imported and what is not.

---

