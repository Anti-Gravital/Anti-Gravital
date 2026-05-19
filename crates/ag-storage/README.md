# ag-storage

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 4.
> Criticidad: Estandar.
> Capitulo de arquitectura: docs/architecture/08-modulos-batteries-included.md

## Dominio

Almacenamiento de objetos con adaptadores S3, MinIO y filesystem local. URLs firmadas pre-signed, procesamiento basico de imagenes, deduplicacion por hash y politicas de retencion. API unica independiente del backend.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/08-modulos-batteries-included.md`.
- Hoja de ruta del crate: `docs/modules/ag-storage/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
