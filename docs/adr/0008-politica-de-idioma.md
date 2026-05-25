# ADR-0008 - Politica de idioma: ingles canonico y vitrina bilingue

- Estado: aceptado
- Fecha: 2026-05-24
- RFC origen: RFC-0008
- Supersede: ADR-0002 (parcialmente; ver Contexto)

## Contexto

`ADR-0002` (2026-05-19) establecio documentacion bilingue con espanol como
predeterminado y dejo registrado que la decision "se revisa cuando el comite
tecnico se forme en Fase 4". El proyecto cierra la Fase 4.5 y se prepara para
Fase 5 (`ag-cloud`), donde el objetivo es ampliar la adopcion.

Dos hechos fuerzan la revision:

1. ADR-0002 no legislo el idioma de los comentarios de codigo. El vacio se
   lleno por defecto con espanol: ~3200 lineas de comentarios en 118 archivos
   `.rs`.
2. La documentacion crecio a 147 archivos markdown. Hacerla toda bilingue
   inline duplicaria el volumen y garantizaria desincronizacion.

`RFC-0008` analiza las alternativas y propone la politica que esta ADR
registra.

## Decision

El **ingles es el idioma canonico** del codigo y de la documentacion tecnica
profunda. Los **documentos vitrina** (README raiz, los tres maestros de
`docs/master/`, y los capitulos del manual) son **bilingues en el mismo
archivo**, con la seccion en ingles primero (canonica) y la seccion en
espanol despues. Todos los comentarios de codigo (`//`, `///`, `//!`) se
escriben en ingles.

## Consecuencias

Positivas:

- El codigo y los docs tecnicos quedan accesibles a contribuidores globales,
  alineados con la norma del open source de infraestructura.
- La vitrina bilingue preserva la accesibilidad para el foco inicial
  latinoamericano: lo primero que ve un recien llegado sigue estando en
  espanol.
- Un solo idioma canonico de codigo elimina ambiguedad para futuros
  contribuidores.

Negativas:

- Costo puntual de convertir ~3200 comentarios a ingles. Mitigacion: se hace
  una sola vez en la fase de cierre documental, crate por crate, con
  verificacion `cargo`.
- La seccion espanola de la vitrina puede desincronizarse de la inglesa.
  Mitigacion: la inglesa es canonica; si avanza, la espanola se marca
  pendiente.

## Alternativas consideradas

- Mantener espanol por defecto: rechazada por perpetuar la barrera de entrada
  internacional.
- Todo en ingles sin espanol: rechazada por contradecir el posicionamiento
  del proyecto.
- Todo bilingue inline: rechazada como mala practica para 147 archivos
  (duplicacion, desincronizacion, diffs ruidosos).
- Carpetas espejo `es/` + `en/` completas: rechazada por costo de
  mantenimiento desproporcionado.

Detalle completo en `RFC-0008`.

## Notas

- Supersede el aspecto de "espanol predeterminado" de `ADR-0002`; la
  estructura de carpetas `docs/es/` y `docs/en/` como indices se conserva.
- Cumple el punto de revision que `ADR-0002` dejo anticipado para Fase 4.
- Regla operativa registrada en `CLAUDE.md`.
- Maestro de arquitectura, seccion 1: "documentacion bilingue desde el dia
  cero" se reinterpreta como "vitrina bilingue, tecnica en ingles canonico".
