# benchmarks/

Suite oficial de benchmarks del workspace Anti-Gravital. Cada
benchmark se ejecuta con `cargo bench` y produce cifras reproducibles
sobre hardware de referencia documentado.

## Benchmarks previstos

- `hello-world`: latencia y throughput minimos del pipeline Shield
  desde Fase 1.
- `json-crud`: CRUD con PostgreSQL desde Fase 2.
- `plaintext`: handler trivial estilo TechEmpower desde Fase 2.

## Reglas

- Toda cifra publica viene con: hardware, sistema operativo, version
  Rust, commit, configuracion, metodologia, numero de ejecuciones y
  desviacion estandar. Regla 17 de `CLAUDE.md`.
- Sin trampas. Sin benchmark cherry-picked. Si un benchmark
  competidor existe (TechEmpower, oha, wrk), se ejecuta el mismo aqui
  y se publica.
- Resultados historicos en `docs/benchmarks/` con su contexto.

## Estado

Esta carpeta (suite de escenarios cross-crate) sigue vacia. Los
benchmarks existentes son de nivel crate, con criterion, en
`crates/*/benches/`:

- `crates/ag-core/benches/shield_hello_world.rs` — pipeline Shield
  hello-world (Fase 1).
- `crates/ag-workers/benches/queue_throughput.rs` — encode/decode de
  payloads, enqueue y lease en lote sobre el backend en memoria
  (Fase 4.6-D; metodologia en `crates/ag-workers/benches/README.md`).

Las cifras publicadas y su contexto viven en `docs/benchmarks/`.
