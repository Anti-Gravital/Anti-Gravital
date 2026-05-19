# ag-wasm-host

> Capitulo de arquitectura: `docs/architecture/09-plugins-wasi.md`.
> README del crate: `crates/ag-wasm-host/README.md`.
> Criticidad: Nucleo.
> Fase de implementacion: Fase 9.

## Dominio

Host de plugins WASI sobre wasmtime. Sandbox de memoria, fuel y timeout. ABI estable via WIT.

## Dependencias internas permitidas

Depende de ag-core.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-wasm-host/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
