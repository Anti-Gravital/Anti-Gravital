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

## Estado

Fase 0: vacia salvo este README. Los primeros diagramas tecnicos
llegan con la Fase 1 (Shield) y la Fase 2 (Core).
