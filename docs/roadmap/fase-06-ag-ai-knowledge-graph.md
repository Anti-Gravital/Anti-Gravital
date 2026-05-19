# Fase 6 - ag-ai y Knowledge Graph

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-05-ag-cloud.md](./fase-05-ag-cloud.md)
> Siguiente: [fase-07-ag-migrate.md](./fase-07-ag-migrate.md)

## Fase 6 — `ag-ai` y Knowledge Graph

**Objetivo.** Construir el módulo de IA con el knowledge graph y las capacidades asistidas.

### 6.1 Criterios de entrada

- [ ] Versión 0.5 (beta pública) liberada.
- [ ] Retroalimentación de los primeros usuarios incorporada en backlog.

### 6.2 Entregables

- [ ] Generador del knowledge graph desde el AST del DSL.
- [ ] Persistencia del graph en `.ag/knowledge-graph.json`.
- [ ] Generador de documentación arquitectónica Markdown desde el graph.
- [ ] Generador de diagramas C4 (Context, Container, Component) en Mermaid.
- [ ] Dashboard interactivo del graph en el dev server (`ag dev`).
- [ ] Comando `ag ai suggest-schema` con integración a proveedor configurable.
- [ ] Comando `ag ai review-migration`.
- [ ] Comando `ag ai analyze-architecture`.
- [ ] Soporte para proveedores: Anthropic Claude, OpenAI, Ollama local, vLLM local.
- [ ] Modo offline donde las funciones AI están deshabilitadas pero el framework funciona.
- [ ] Documentación: "Anti-Gravital + agentes IA: el flujo schema-first" con ejemplos completos.

### 6.3 Criterios de salida (puerta antes de Fase 7)

- [ ] El knowledge graph se regenera correctamente con cada `ag generate`.
- [ ] La documentación arquitectónica generada es legible y útil (revisada por tres personas externas al equipo).
- [ ] Al menos una organización usuaria reporta que ha integrado `ag ai` en su flujo de trabajo.
- [ ] Al menos 2 500 stars en el repositorio.

### 6.4 Riesgos de la fase

El riesgo principal es la dependencia de proveedores externos de IA. La mitigación es la abstracción del proveedor y el modo offline.

---
