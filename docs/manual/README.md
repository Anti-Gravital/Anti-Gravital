English | Espanol

---

# Anti-Gravital Manual

This directory gathers the chapters of the official Anti-Gravital
manual. Each chapter is self-contained and published in markdown so it
is readable from the repository without depending on another renderer.

The master `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` remains
the architectural source of truth. The manual chapters **apply** the
architecture to concrete use cases: how to build, configure and deploy
ecosystem components.

## Index

| Chapter | Topic | Status |
| --- | --- | --- |
| `01-shield-as-library.md` | Use the `ag-core` Shield as a library | Published |
| `02-primera-api.md` | Build the first API with the DSL | Published |
| `03-dominio-tls-correo.md` | Configure domain, TLS and transactional email | Published |
| `04-instalacion-y-onboarding.md` | Installation, ag new, dev server, troubleshooting | Published |

## Convention

- One chapter per file, numbered at the start (`01-`, `02-`, ...).
- Main title H1, sections H2.
- No emojis.
- No attribution to AI tools.
- Compilable code examples; where applicable, copy the example from an
  `ag-core` test to keep them consistent.
- Cross references to the master and the Roadmap when the chapter
  content extends them.
- Showcase chapters are bilingual (ADR-0008): canonical English section
  first, a horizontal rule, then the Spanish section.

## How to contribute chapters

1. Identify the domain in `docs/roadmap/STATUS.md` and its corresponding
   phase.
2. Draft on a dedicated branch with its descriptor under
   `docs/pr-drafts/`.
3. PR with review; approved content moves into the index.

As the phases advance, this manual grows with chapters on Core, DSL,
batteries-included modules, deployment, AI integration, mobile and
plugins.

---

# Manual de Anti-Gravital

Este directorio reune los capitulos del manual oficial de
Anti-Gravital. Cada capitulo es autocontenido y se publica en
markdown para que sea leible desde el repositorio sin depender de
otro renderer.

El maestro `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` sigue
siendo la fuente de verdad arquitectonica. Los capitulos del manual
**aplican** la arquitectura a casos concretos de uso: como construir,
configurar y desplegar componentes del ecosistema.

## Indice

| Capitulo | Tema | Estado |
| --- | --- | --- |
| `01-shield-as-library.md` | Usar la Shield de `ag-core` como libreria | Publicado |
| `02-primera-api.md` | Crear la primera API con el DSL | Publicado |
| `03-dominio-tls-correo.md` | Configurar dominio, TLS y correo transaccional | Publicado |
| `04-instalacion-y-onboarding.md` | Instalacion, ag new, dev server, troubleshooting | Publicado |

## Convencion

- Un capitulo por archivo, numerado al inicio (`01-`, `02-`, ...).
- Titulo principal H1, secciones H2.
- Sin emojis.
- Sin atribuciones a herramientas IA.
- Ejemplos de codigo compilables; cuando corresponda, copiar el
  ejemplo desde un test de `ag-core` para mantener consistencia.
- Referencias cruzadas al maestro y a la Hoja de Ruta cuando el
  contenido del capitulo lo extienda.
- Los capitulos vitrina son bilingues (ADR-0008): seccion inglesa
  canonica primero, una regla horizontal, despues la seccion espanola.

## Como contribuir capitulos

1. Identificar el dominio en `docs/roadmap/STATUS.md` y la fase
   correspondiente.
2. Borrador en una rama dedicada con su descriptor en
   `docs/pr-drafts/`.
3. PR con revision; el contenido aprobado pasa a indice.

A medida que las fases avanzan, este manual crece con capitulos de
Core, DSL, modulos batteries-included, despliegue, integracion IA,
mobile y plugins.
