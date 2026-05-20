# Fase 2: The Core MVP y roundtrip completo

## Resumen

Implementacion de los entregables de Fase 2 segun la Hoja de Ruta §2.2:
modulo core en ag-core con extractores tipados y sistema de respuestas;
ag-data con pool PostgreSQL y migraciones embebidas; ag-cli operativo con
tres templates; ejemplo todo-api con CRUD completo y Dockerfile FROM scratch.

## Fase afectada

Fase 2 — The Core MVP y roundtrip completo.

## Tipo de cambio

- Implementacion de funcionalidad nueva (core, data, cli, ejemplo).
- Expansion de sistema de errores existente (AgError).
- Adicion de dependencias de workspace (sqlx, clap).

## Documentos relacionados

- `docs/roadmap/fase-02-core-mvp.md` — definicion de los entregables.
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` — criterios de salida de Fase 2.
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` capitulo 6 — diseno Shield/Core.
- `docs/manual/02-primera-api.md` — guia de usuario creada en esta PR.
- `docs/roadmap/STATUS.md` — actualizado con progreso de Fase 2.

## Cambios principales

### ag-core

- `src/core/mod.rs`: modulo Core completo con reexports de extractores
  (`State`, `Path`, `Query`), alias `ValidatedBody`, reexport `Claims`
  y modulo `response` con `PlainText` y `BodyStream`.
- `src/error.rs`: expandido con `NotFound` (404), `BadRequest` (400),
  `Conflict` (409) y `Database` (500).
- `src/lib.rs`: reexporta `AppState` en el nivel superior.

### ag-data (nuevo contenido funcional)

- `Cargo.toml`: dependencias sqlx (0.8, postgres, migrate, macros,
  runtime-tokio-rustls), ag-core, serde, thiserror, tokio, tracing.
- `src/lib.rs`: `DbPool`, `DataConfig`, `DataError`, `connect()`,
  `run_migrations()`, conversion `DataError -> AgError`, `sanitize_url`.
- `migrations/`: directorio placeholder para tests del crate.

### ag-cli (nuevo contenido funcional)

- `Cargo.toml`: dependencias clap (4, derive), serde, thiserror, toml.
  Declara el binario `ag`.
- `src/main.rs`: comandos `new`, `dev`, `build` con subcomandos clap;
  tres templates embebidos via `include_str!`; scaffold de proyectos con
  sustitucion de `{{name}}`; deteccion de cargo-watch para hot reload.

### templates/

- `templates/rest/`: Cargo.toml.tmpl, src/main.rs.tmpl, config.toml.tmpl.
- `templates/realtime/`: igual con handler WebSocket de ejemplo.
- `templates/fullstack/`: igual con ag-data, sqlx y migrations/.

### examples/todo-api/ (nuevo)

- `Cargo.toml`: workspace member; dependencias ag-core, ag-data, axum,
  sqlx, tokio, tracing, tracing-subscriber. Declara bench `crud`.
- `src/main.rs`: arranque con Shield + router + pool PostgreSQL.
- `src/models.rs`: structs `Todo`, `CreateTodo`, `UpdateTodo`.
- `src/handlers.rs`: cinco handlers CRUD tipados con AgError.
- `migrations/0001_create_todos.sql`: tabla `todos`.
- `benches/crud.rs`: placeholder de benchmark CRUD+DB con instrucciones
  para medicion en hardware de referencia.
- `Dockerfile`: compilacion MUSL + FROM scratch; imagen <= 20 MB.

### Workspace

- `Cargo.toml`: agrega `examples/todo-api` a los miembros; agrega
  `sqlx` (0.8) y `clap` (4) a `workspace.dependencies`; habilita la
  feature `query` de axum; agrega path-references `ag-core` y `ag-data`.

## Plan de prueba

- [x] `cargo build --workspace` limpio (sin errores, sin warnings).
- [x] `cargo test --workspace` verde (tests de DB ignorados por defecto).
- [x] `cargo clippy --workspace --all-targets -- -D warnings` limpio.
- [x] `cargo fmt --check` limpio.
- [ ] `cargo test --workspace -- --ignored` verde contra PostgreSQL real.
- [ ] `cargo run -p todo-api` arranca y sirve los cinco endpoints.
- [ ] `docker build` produce imagen funcional < 20 MB (requiere MUSL).

## Criterios de salida que avanza

Hoja de Ruta Fase 2, seccion 2.2 (entregables):

- [x] Crate `ag-core` con modulo `core` operativo.
- [x] Router Axum integrado con la Shield.
- [x] Extractores tipados.
- [x] Sistema de errores AgError extendido.
- [x] Sistema de respuestas JSON / plaintext / streams.
- [x] Crate `ag-data` con pool PostgreSQL.
- [x] Sistema de migraciones embebido.
- [x] Example app `todo-api` con CRUD completo.
- [/] Benchmark CRUD + DB (placeholder; medicion real pendiente).
- [x] Crate `ag-cli` con comandos `new`, `dev`, `build`.
- [x] Tres templates: `rest`, `realtime`, `fullstack`.

## Checklist final CLAUDE.md

- [x] Pertenece a la fase correcta (Fase 2).
- [x] Respeta la documentacion y el alcance.
- [x] No rompe arquitectura ni modularidad.
- [x] No anade complejidad innecesaria ni features fuera de scope.
- [x] No crea dependencias circulares.
- [x] Compila (`cargo build --workspace`).
- [x] Pasa tests (`cargo test --workspace`).
- [x] Pasa fmt (`cargo fmt --check`).
- [x] Pasa clippy (`cargo clippy --workspace --all-targets -- -D warnings`).
- [x] Tiene documentacion (`docs/manual/02-primera-api.md`).
- [/] Tiene benchmarks (placeholder en `examples/todo-api/benches/crud.rs`).
- [x] Tiene manejo de errores correcto (`AgError` con conversion HTTP).
- [x] Mantiene coherencia con Anti-Gravital v4.0.
