# Modulos de Anti-Gravital - Indice

> Fuente verbatim: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`,
> seccion 5.

Esta carpeta describe los 17 crates del workspace. Cada subcarpeta
contiene un README con el dominio del crate, su criticidad, sus
dependencias internas permitidas, la fase en la que se implementa y
un puntero al capitulo correspondiente de la arquitectura tecnica.

El conteo paso de 15 a 17 crates con la introduccion de la Fase 4.5
(`ag-mail` estandar diferido, `ag-domains` opcional infra), oficializada
en `docs/adr/0007-ag-mail-ag-domains.md`.

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
| `ag-mail` | Estandar diferido | 8.8 | Fase 4.5 |
| `ag-ui` | Opcional | 5 | Fase 4 o posterior |
| `ag-cloud` | Opcional | 10 | Fase 5 |
| `ag-domains` | Opcional infra | 8.9 / 10.6 | Fase 4.5 |
| `ag-ai` | Opcional | 11 | Fase 6 |
| `ag-mobile` | Opcional | 13 | Fase 8 |
| `ag-migrate` | Opcional | 12 | Fase 7 |

## Reglas

- Las siete reglas de dependencia entre crates estan en el capitulo 5
  del maestro de arquitectura y en `docs/architecture/05-ecosistema-modulos.md`.
  Las dos ultimas (sexta y septima) fueron introducidas por `ADR-0007` para
  documentar la direccionalidad `ag-auth -> ag-mail` (sin ciclo) y la
  dependencia opcional `ag-cloud -> ag-domains`.
- La regla operativa fundamental: `ag-core` no depende de ningun otro
  crate Anti-Gravital.
