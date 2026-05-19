# ag-migrate

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 7.
> Criticidad: Opcional.
> Capitulo de arquitectura: docs/architecture/12-migracion-ag-migrate.md

## Dominio

Importadores de migracion desde frameworks legacy. Cubre OpenAPI 3.0 y 3.1, Prisma, Django, FastAPI/Pydantic, Sequelize y GraphQL SDL. Genera el archivo `schema.ag` correspondiente; la logica de negocio queda fuera de alcance y se documenta en cada guia.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/12-migracion-ag-migrate.md`.
- Hoja de ruta del crate: `docs/modules/ag-migrate/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
