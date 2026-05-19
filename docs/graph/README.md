# Knowledge graph

> Regla 33 de `CLAUDE.md`.

Esta carpeta alojara el knowledge graph autogenerado del proyecto a
partir de la Fase 6. El graph se deriva del AST del DSL y se persiste
en `.ag/knowledge-graph.json` dentro de cada proyecto usuario; la
version curada del proyecto general vive aqui.

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

## Estado

Fase 0: placeholder. La generacion automatica se implementa en el
crate `ag-ai` durante la Fase 6.
