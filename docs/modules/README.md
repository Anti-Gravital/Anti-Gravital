# Modulos de Anti-Gravital - Indice

> Fuente verbatim: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`,
> seccion 5.

Esta carpeta describe los 15 crates del workspace. Cada subcarpeta
contiene un README con el dominio del crate, su criticidad, sus
dependencias internas permitidas, la fase en la que se implementa y
un puntero al capitulo correspondiente de la arquitectura tecnica.

## Mapa del ecosistema

| Crate | Criticidad | Capitulo | Fase de implementacion |
| --- | --- | --- | --- |
| `ag-core` | Nucleo | 6 | Fase 1 y 2 |
| `ag-dsl` | Nucleo | 7 | Fase 3 a 10 |
| `ag-cli` | Nucleo | 5 | Fase 2 a 9 |
| `ag-wasm-host` | Nucleo | 9 | Fase 9 |
| `ag-auth` | Estandar | 8 | Fase 4 |
| `ag-data` | Estandar | 8 | Fase 2 a 4 |
| `ag-realtime` | Estandar | 8 | Fase 4 |
| `ag-cache` | Estandar | 8 | Fase 4 |
| `ag-storage` | Estandar | 8 | Fase 4 |
| `ag-observe` | Estandar | 14 | Fase 4 |
| `ag-ui` | Opcional | 5 | Fase 4 o posterior |
| `ag-cloud` | Opcional | 10 | Fase 5 |
| `ag-ai` | Opcional | 11 | Fase 6 |
| `ag-mobile` | Opcional | 13 | Fase 8 |
| `ag-migrate` | Opcional | 12 | Fase 7 |

## Reglas

- Las cinco reglas de dependencia entre crates estan en el capitulo 5
  del maestro de arquitectura y en `docs/architecture/05-ecosistema-modulos.md`.
- La regla operativa fundamental: `ag-core` no depende de ningun otro
  crate Anti-Gravital.
