# ag-cli

> Capitulo de arquitectura: `docs/architecture/05-ecosistema-modulos.md`.
> README del crate: `crates/ag-cli/README.md`.
> Criticidad: Nucleo.
> Fase de implementacion: Fase 2 a Fase 9.

## Dominio

Binario `ag`: orquesta creacion de proyectos, generacion DSL, hot reload, build, deploy, migracion y administracion de plugins.

## Dependencias internas permitidas

Depende de todos los crates a traves de Cargo features.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 2 + Fase 3 completadas. El binario `ag` es funcional con seis comandos.

## Comandos implementados

### Fase 2 (scaffold y desarrollo)

- `ag new <nombre>` — genera scaffold desde template (`rest`, `realtime`
  o `fullstack`) embebido via `include_str!`. Crea `Cargo.toml`,
  `src/main.rs`, `README.md` y `.gitignore`.
- `ag dev` — arranca el servidor con hot reload via `cargo-watch`.
- `ag build` — compila en perfil release.

### Fase 3 (DSL)

- `ag generate [--schema <path>] [--output <dir>]` — compila `schema.ag`
  y escribe los artefactos generados (SQL, Rust, TypeScript, OpenAPI).
- `ag schema lint [--schema <path>]` — reporta errores y warnings del
  schema usando `ag_dsl::lint()`. Muestra warnings aunque no haya errores
  (a diferencia de `ag generate` que solo muestra errores bloqueantes).
- `ag schema diff <ref> [--schema <path>]` — detecta cambios BREAKING
  vs additive comparando el schema actual contra un commit o rama git.

## Notas de implementacion

`ag schema lint` fue refactorizado en Fase 3 (commit 81f9dae) para usar
directamente `ag_dsl::lint()` en lugar de `ag_dsl::compile()`. Esto
garantiza que los warnings siempre son visibles, incluso cuando el schema
compila sin errores.

Comandos futuros (`ag deploy`, `ag migrate`, `ag plugin`) llegan en
fases posteriores segun la hoja de ruta.
