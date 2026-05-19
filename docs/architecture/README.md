# Arquitectura Tecnica de Anti-Gravital - Indice navegable

> Fuente verbatim: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.

Esta carpeta contiene la descomposicion verbatim del documento maestro
de arquitectura. Cada capitulo del maestro vive en un archivo propio,
con su contenido copiado palabra por palabra. Los breadcrumb al inicio
de cada archivo apuntan al maestro de origen y a los capitulos
anterior y siguiente.

## Capitulos

| Numero | Titulo | Archivo |
| --- | --- | --- |
| 1 | Resumen ejecutivo | `01-resumen-ejecutivo.md` |
| 2 | Manifiesto y posicionamiento | `02-manifiesto-y-posicionamiento.md` |
| 3 | Que es Anti-Gravital y que no es (alcance y limites) | `03-alcance-y-limites.md` |
| 4 | Analisis del estado del arte | `04-estado-del-arte.md` |
| 5 | Arquitectura del ecosistema: modulos y responsabilidades | `05-ecosistema-modulos.md` |
| 6 | Arquitectura del nucleo (ag-core): Shield y Core | `06-nucleo-shield-y-core.md` |
| 7 | El lenguaje Anti-DSL (ag-dsl) | `07-anti-dsl.md` |
| 8 | Modulos batteries-included | `08-modulos-batteries-included.md` |
| 9 | Sistema de plugins WASI (ag-wasm-host) | `09-plugins-wasi.md` |
| 10 | Subsistema de despliegue (ag-cloud) | `10-despliegue-ag-cloud.md` |
| 11 | Integracion con IA (ag-ai) y el Knowledge Graph | `11-ai-knowledge-graph.md` |
| 12 | Framework de migracion (ag-migrate): importadores | `12-migracion-ag-migrate.md` |
| 13 | Puente de aplicaciones nativas (ag-mobile) | `13-mobile-ag-mobile.md` |
| 14 | Observabilidad (ag-observe) | `14-observabilidad-ag-observe.md` |
| 15 | Modelo de seguridad | `15-seguridad.md` |
| 16 | Objetivos de rendimiento y metodologia de validacion | `16-rendimiento-y-validacion.md` |
| 17 | Modelo de gobernanza Open Source | `17-gobernanza-open-source.md` |
| 18 | Analisis de riesgos y mitigaciones | `18-riesgos-y-mitigaciones.md` |
| 19 | Glosario tecnico | `19-glosario.md` |
| 20 | Apendice: comparativa de mercado | `20-apendice-comparativa.md` |

## Reglas de uso

1. El contenido de cada archivo es verbatim del maestro. Para
   modificarlo, modifique el maestro y regenere con
   `bash tools/split-masters.sh`.
2. Estos archivos no contienen informacion adicional. Si necesita
   anadir contexto, hagalo en una RFC o en un ADR.
3. Para citas en codigo o en pull requests, prefiera referenciar el
   archivo derivado correspondiente a referenciar el maestro completo.
