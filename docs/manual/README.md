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

## Convencion

- Un capitulo por archivo, numerado al inicio (`01-`, `02-`, ...).
- Titulo principal H1, secciones H2.
- Sin emojis.
- Sin atribuciones a herramientas IA.
- Ejemplos de codigo compilables; cuando corresponda, copiar el
  ejemplo desde un test de `ag-core` para mantener consistencia.
- Referencias cruzadas al maestro y a la Hoja de Ruta cuando el
  contenido del capitulo lo extienda.

## Como contribuir capitulos

1. Identificar el dominio en `docs/roadmap/STATUS.md` y la fase
   correspondiente.
2. Borrador en una rama dedicada con su descriptor en
   `docs/pr-drafts/`.
3. PR con revision; el contenido aprobado pasa a indice.

A medida que las fases avanzan, este manual crece con capitulos de
Core, DSL, modulos batteries-included, despliegue, integracion IA,
mobile y plugins.
