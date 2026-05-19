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

Fase 0: vacio. El primer benchmark llega con la Fase 1 (Shield MVP).
