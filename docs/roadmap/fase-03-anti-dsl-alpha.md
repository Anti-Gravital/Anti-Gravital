# Fase 3 - Anti-DSL alpha (v0.1 a v0.4)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-02-core-mvp.md](./fase-02-core-mvp.md)
> Siguiente: [fase-04-modulos-estandar.md](./fase-04-modulos-estandar.md)

## Phase 3 — Anti-DSL alpha (versions 0.1 to 0.4 of the DSL)

**Objective.** Build the DSL compiler with a deliverable subset of the grammar. This phase delivers the first functional codegen: models, basic endpoints, validations and relations. No declarative auth yet, no events yet. The product is the first version of the "define → generate → implement" flow.

### 3.1 Entry criteria

- [ ] Phase 2 completed with all of its exit criteria checked.
- [ ] `ag-dsl` crate started.
- [ ] Final decision on the compiler's base libraries (logos for lexer, chumsky for parser, askama and quote for codegen). Documented in RFC.

### 3.2 Deliverables

- [ ] DSL version 0.1: basic models with primitive annotations (`@primary`, `@unique`, `@auto`).
- [ ] DSL version 0.2: endpoints (method, path, body, response).
- [ ] DSL version 0.3: validations (`@min`, `@max`, `@email`, `@regex`, `@length`).
- [ ] DSL version 0.4: relations between models (`1:1`, `1:N`, `N:M`).
- [ ] Rust generator: structs with serde, validators, sqlx query builders.
- [ ] SQL generator: idempotent migrations.
- [ ] TypeScript generator: types and HTTP client.
- [ ] OpenAPI 3.1 generator.
- [ ] `ag generate` command that reads `schema.ag` and produces all the artifacts.
- [ ] `ag schema lint` command that reports best-practices warnings.
- [ ] `ag schema diff <ref>` command that reports breaking vs non-breaking changes.
- [ ] Readable diagnostics for common DSL errors (model not found, unknown type, invalid annotation).
- [ ] Basic LSP server (`ag-lsp`) with autocompletion and diagnostics.
- [ ] VS Code plugin published on the marketplace.
- [ ] Compiler test suite with coverage ≥ 85%.
- [ ] Parser fuzzing with `cargo-fuzz`: 24 hours without crashes.
- [ ] DSL reference documentation version by version.

### 3.3 Exit criteria (gate before Phase 4)

- [ ] A complete project can be created, defined in `schema.ag`, generated, and executed using only the CLI.
- [ ] The `ecommerce-api` example is completely rewritten with DSL and works.
- [ ] The benchmarks are maintained: DSL-generated CRUD is not slower than hand-written CRUD.
- [ ] The VS Code plugin has ≥ 100 installations.
- [ ] At least one external collaborator has contributed to the compiler.
- [ ] The DSL documentation is complete and reviewed by at least two people.
- [ ] At least 200 stars on the repository.

### 3.4 Phase risks

The DSL compiler is the technically most complex component of the project. The main risk is underestimating the effort and exceeding the schedule. The mitigation is incremental implementation by subversions: if the phase runs long, subversion 0.4 (relations) can be postponed to phase 4 without blocking advancement.

The secondary risk is the compiler's error messages. A compiler with incomprehensible messages ruins the experience. The mitigation is to prioritize readable diagnostics from day one, with specific tests that verify that the messages are useful.

---

