# Fase 6 - ag-ai y Knowledge Graph

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-05-ag-cloud.md](./fase-05-ag-cloud.md)
> Siguiente: [fase-07-ag-migrate.md](./fase-07-ag-migrate.md)

## Phase 6 — `ag-ai` and Knowledge Graph

**Objective.** Build the AI module with the knowledge graph and the assisted capabilities.

### 6.1 Entry criteria

- [ ] Version 0.5 (public beta) released.
- [ ] Feedback from the first users incorporated into the backlog.

### 6.2 Deliverables

- [ ] Knowledge graph generator from the DSL AST.
- [ ] Persistence of the graph in `.ag/knowledge-graph.json`.
- [ ] Markdown architectural documentation generator from the graph.
- [ ] C4 diagram generator (Context, Container, Component) in Mermaid.
- [ ] Interactive graph dashboard in the dev server (`ag dev`).
- [ ] `ag ai suggest-schema` command with integration to a configurable provider.
- [ ] `ag ai review-migration` command.
- [ ] `ag ai analyze-architecture` command.
- [ ] Support for providers: Anthropic Claude, OpenAI, local Ollama, local vLLM.
- [ ] Offline mode where the AI functions are disabled but the framework works.
- [ ] Documentation: "Anti-Gravital + AI agents: the schema-first flow" with complete examples.

### 6.3 Exit criteria (gate before Phase 7)

- [ ] The knowledge graph regenerates correctly with each `ag generate`.
- [ ] The generated architectural documentation is readable and useful (reviewed by three people external to the team).
- [ ] At least one user organization reports that it has integrated `ag ai` into its workflow.
- [ ] At least 2 500 stars on the repository.

### 6.4 Phase risks

The main risk is the dependency on external AI providers. The mitigation is provider abstraction and offline mode.

---

