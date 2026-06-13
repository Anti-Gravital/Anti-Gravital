# Diagramas

> Regla 34 de `CLAUDE.md`.

Esta carpeta contiene los diagramas oficiales del proyecto. Toda
arquitectura importante requiere diagrama logico, diagrama fisico,
flujo de requests, mapa de dependencias y diagrama de despliegue.

## Formatos aceptados

- Mermaid (`.mmd`) cuando el diagrama puede expresarse como texto
  declarativo. Se renderiza automaticamente en GitHub.
- SVG cuando se necesita control fino o cuando el diagrama no es
  amigable con Mermaid.
- PNG exportado de SVG como respaldo para presentaciones, no como
  fuente.

No se aceptan diagramas embebidos unicamente en PDFs porque rompen la
auditabilidad por diff.

## Convenciones

- Nombre del archivo: `categoria-objeto.formato`.
  Ej: `flow-request-shield-to-core.mmd`, `dep-crates.svg`.
- Cada diagrama acompana un breve `.md` con su contexto, su fecha de
  ultima revision y el capitulo del maestro al que pertenece.
- Los diagramas reemplazan a un fragmento textual cuando el grafo es
  mas claro que la prosa, nunca como adorno.

## Diagramas disponibles

- `workspace-dependencies.md` — mapa de dependencias entre crates.
- `request-flow-shield.md` — flujo de un request a traves del Shield.
- `dsl-pipeline.md` — pipeline de compilacion del Anti-DSL.
- `mail-workers-dataflow.md` — flujo de correo y jobs (`ag-mail`/`ag-workers`).

Las fuentes Mermaid van embebidas en su `.md` (bloque ```mermaid```), que
GitHub renderiza y que se audita por diff. Se anaden mas diagramas a medida
que crecen las fases.
