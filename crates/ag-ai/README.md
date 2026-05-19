# ag-ai

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 6.
> Criticidad: Opcional.
> Capitulo de arquitectura: docs/architecture/11-ai-knowledge-graph.md

## Dominio

Genera el knowledge graph desde el AST del DSL, produce documentacion arquitectonica y diagramas C4 en Mermaid, ofrece sugerencias de schema, revision de migraciones y analisis arquitectonico. Soporta proveedores Anthropic Claude, OpenAI, Ollama y vLLM. Funciona en modo offline con las features IA deshabilitadas.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/11-ai-knowledge-graph.md`.
- Hoja de ruta del crate: `docs/modules/ag-ai/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
