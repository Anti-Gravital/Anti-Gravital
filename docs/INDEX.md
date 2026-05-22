# Indice maestro de la documentacion de Anti-Gravital

Este archivo es la entrada principal a la documentacion del proyecto.
Su funcion es enumerar todos los documentos navegables y senalar a un
agente (humano o automatizado) que falta al proyecto en cada momento.

## Fuente de verdad

Los maestros son la unica fuente de verdad. Viven en `docs/master/` y
no se editan fuera de un procedimiento RFC. Todos los demas archivos
bajo `docs/` son derivados verbatim de los maestros.

| Documento maestro | Archivo |
| --- | --- |
| Vision, posicionamiento y alcance | `docs/master/ANTI-GRAVITAL-Blueprint-v4.0.pdf` |
| Arquitectura tecnica e implementacion | `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` |
| Hoja de ruta y puertas de verificacion | `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` |
| Registro de integridad de los maestros | `docs/master/VERSION.md` |

## Como leer la documentacion

1. Empiece por la version vigente en `docs/master/VERSION.md`.
2. Lea el preambulo de la Hoja de Ruta:
   `docs/roadmap/preambulo.md`.
3. Identifique la fase activa en `docs/roadmap/STATUS.md`.
4. Lea el capitulo de arquitectura correspondiente a la fase activa
   bajo `docs/architecture/`.
5. Lea el README del modulo o crate especifico bajo
   `docs/modules/<crate>/`.
6. Lea RFC y ADR vigentes en `docs/rfc/` y `docs/adr/`.

## Mapa de derivados

### docs/architecture/

Descomposicion verbatim del maestro de Arquitectura Tecnica en 20
archivos numerados, uno por capitulo. Indice en
`docs/architecture/README.md`.

### docs/roadmap/

Descomposicion verbatim de la Hoja de Ruta en archivos por fase y por
seccion (preambulo, resumen, reglas de oro, calendario, mas alla de
1.0). Estado vivo del tablero en `docs/roadmap/STATUS.md`.

### docs/modules/

Un subdirectorio por cada crate del workspace, con README propio que
referencia el capitulo de arquitectura correspondiente y la fase de
implementacion.

### docs/dsl/

Subsecciones del capitulo 7 de Arquitectura Tecnica, dedicadas al
Anti-DSL. Incluye la referencia implementada v0.1–v0.4
(`referencia-v01-v04.md`) y la hoja de ruta del servidor LSP
(`lsp-roadmap.md`).

### docs/fuzz/

Documentacion del harness de fuzzing del compilador DSL (cargo-fuzz).
Incluye instrucciones para el gate manual de 24h requerido antes de
cerrar Fase 3 e historial de crashes encontrados y corregidos.

### docs/benchmarks/, docs/security/, docs/governance/

Vistas verbatim de los capitulos 16, 15 y 17 del maestro de
Arquitectura, respectivamente.

### docs/rfc/ y docs/adr/

Procesos formales del proyecto: RFC para decisiones tecnicas
importantes y ADR para decisiones arquitectonicas estables. Plantillas
en `docs/rfc/template.md` y `docs/adr/template.md`.

### docs/diagrams/, docs/graph/, docs/examples/

Espacios reservados para diagramas, knowledge graph y catalogo de
ejemplos. Se pueblan a partir de la fase indicada en cada README.

### docs/es/ y docs/en/

Indices por idioma. Funcionan como atajo bilingue al resto de `docs/`.

## Politica de derivados

Si un derivado contradice un maestro, gana el maestro. Para corregir
un derivado, se regenera a partir del maestro con
`bash tools/split-masters.sh`. Para corregir un maestro, se abre RFC.

## Que falta al proyecto

Para saber que esta pendiente en cada momento, consulte
`docs/roadmap/STATUS.md`. Ese archivo agrega el estado de cada casilla
de cada fase y se actualiza con cada pull request que avanza la hoja
de ruta.
