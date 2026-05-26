# ag-cli

> Status: Phases 2-4.5 — implemented. The `ag` binary exposes `new`, `dev`, `build`,
> `generate`, `schema lint`, `schema diff`, `domains check/sync` and `mail test`.
> `deploy`/`ai`/`migrate`/`plugin` subcommands arrive in later phases. See `docs/DEBT.md`.
> Criticidad: Nucleo.
> Capitulo de arquitectura: docs/architecture/05-ecosistema-modulos.md

## Dominio

Binario `ag`. Orquesta todos los comandos del ecosistema: creacion de proyectos (`ag new`), generacion desde DSL (`ag generate`), servidor de desarrollo con hot reload (`ag dev`), build (`ag build`), despliegue (`ag deploy`), migraciones (`ag migrate`), administracion de plugins (`ag plugin`) y operaciones de IA (`ag ai`).

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/05-ecosistema-modulos.md`.
- Hoja de ruta del crate: `docs/modules/ag-cli/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
