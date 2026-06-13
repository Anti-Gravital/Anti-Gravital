# Knowledge graph

> Regla 33 de `CLAUDE.md`.

Esta carpeta aloja el knowledge graph curado del proyecto. El graph
autogenerado por-proyecto se deriva del AST del DSL y se persiste en
`.ag/knowledge-graph.json` dentro de cada proyecto usuario (entregable de la
Fase 6); la version curada del proyecto general vive aqui y se mantiene a
mano a partir de los manifiestos, la CLI y el pipeline del DSL.

Componentes previstos:

- Mapa de modulos y sus relaciones.
- Mapa de crates y sus dependencias permitidas.
- Mapa de comandos de la CLI y sus efectos.
- Mapa de eventos del DSL.
- Mapa de ejemplos y de RFC/ADR vigentes.

Formatos preferidos:

- JSON estructurado para consumo automatizado.
- Markdown para lectura humana.
- OpenAPI cuando aplique.
- Mermaid embebido en Markdown para visualizacion.

## Contenido disponible

- `knowledge-graph.json` — graph curado, legible por maquina: crates con su
  clasificacion, relaciones de dependencia, reglas de dependencia, comandos
  de la CLI y artefactos del DSL.
- `crates.md` — espejo legible por humanos del mismo contenido.

Los diagramas Mermaid asociados viven en `docs/diagrams/`. La generacion
automatica del graph por-proyecto se implementa en el crate `ag-ai` durante
la Fase 6 y no reemplaza a estos archivos curados.
