# ag-wasm-host

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 9.
> Criticidad: Nucleo.
> Capitulo de arquitectura: docs/architecture/09-plugins-wasi.md

## Dominio

Sistema de plugins WASI sobre wasmtime. Define interfaces WIT, especifica `plugin.toml`, implementa el ciclo de vida de plugin (descubrimiento, validacion, carga, activacion, descarga) con sandbox de memoria, fuel y timeout. Soporta plugins escritos en Rust, Go (TinyGo) y AssemblyScript.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/09-plugins-wasi.md`.
- Hoja de ruta del crate: `docs/modules/ag-wasm-host/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
