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

Fase 2 MVP completada. El binario `ag` es funcional con tres comandos
y tres templates embebidos.

Implementado:
- `ag new <nombre>` — genera scaffold desde template (`rest`,
  `realtime` o `fullstack`) embebido via `include_str!`. Crea
  `Cargo.toml`, `src/main.rs`, `README.md` y `.gitignore`.
- `ag dev` — arranca el servidor con hot reload via `cargo-watch`.
  Responde en `http://localhost:8080`.
- `ag build` — compila en perfil release. Produce binario optimizado.

Verificado el 2026-05-21: los tres templates generan scaffold correcto,
compilan sin warnings con `cargo clippy -D warnings` y el binario
release funciona correctamente.

Comandos futuros (`ag deploy`, `ag migrate`, `ag plugin`) llegan en
fases posteriores segun la hoja de ruta.
