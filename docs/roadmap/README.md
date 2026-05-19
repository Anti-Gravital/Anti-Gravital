# Hoja de Ruta de Anti-Gravital - Indice navegable

> Fuente verbatim: `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`.

Esta carpeta contiene la descomposicion verbatim de la Hoja de Ruta
del proyecto. Cada fase del maestro vive en un archivo propio. El
estado vivo del tablero (que casillas estan marcadas y cuales
pendientes) se mantiene en `STATUS.md`.

## Indice de fases

| Fase | Nombre | Archivo | Duracion estimada |
| --- | --- | --- | --- |
| -- | Preambulo: como leer este documento | `preambulo.md` | -- |
| 0 | Fundaciones y gobernanza | `fase-00-fundaciones-y-gobernanza.md` | 1-2 meses |
| 1 | The Shield MVP | `fase-01-shield-mvp.md` | 2-3 meses |
| 2 | The Core MVP y roundtrip completo | `fase-02-core-mvp.md` | 2 meses |
| 3 | Anti-DSL alpha (v0.1 a v0.4) | `fase-03-anti-dsl-alpha.md` | 3 meses |
| 4 | Modulos estandar | `fase-04-modulos-estandar.md` | 3 meses |
| 5 | ag-cloud y version 0.5 beta | `fase-05-ag-cloud.md` | 2 meses |
| 6 | ag-ai y Knowledge Graph | `fase-06-ag-ai-knowledge-graph.md` | 2 meses |
| 7 | ag-migrate importadores | `fase-07-ag-migrate.md` | 2 meses |
| 8 | ag-mobile Flutter bridge | `fase-08-ag-mobile.md` | 2 meses |
| 9 | Sistema de plugins WASI | `fase-09-plugins-wasi.md` | 2 meses |
| 10 | Endurecimiento y hito 1.0 | `fase-10-endurecimiento-y-1.0.md` | 3 meses |
| -- | Mas alla de la 1.0 | `mas-alla-de-1.0.md` | -- |
| -- | Reglas de oro del proceso | `reglas-de-oro.md` | -- |

Duracion total estimada: 24 a 28 meses desde el inicio.

## Estado vivo

`STATUS.md` resume el estado por fase y enumera las casillas marcadas
y las pendientes. Se actualiza con cada pull request que avance la
hoja de ruta.

## Calendario publico

`calendar.md` mantiene las fechas estimadas de cada release y los
hitos publicos del proyecto.

## Reglas

1. Los archivos de fase son verbatim del maestro. Para modificarlos,
   modifique el maestro y regenere con `bash tools/split-masters.sh`.
2. Las casillas markdown `- [ ]` se preservan exactamente; cuando un
   criterio se cumple se marca `- [x]` y se documenta el commit que lo
   completa en `STATUS.md`.
3. Una fase no se considera concluida hasta que todas sus casillas de
   criterio de salida estan marcadas. Regla 1 del proceso, verbatim en
   `reglas-de-oro.md`.
