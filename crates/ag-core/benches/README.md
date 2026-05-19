# Benchmarks de `ag-core`

Esta carpeta contiene la suite de microbenchmarks del crate, ejecutada
con `criterion`. Sigue la regla 17 de `CLAUDE.md`: ningun numero se
publica sin contexto completo.

## Suite vigente

### `shield_hello_world.rs`

Tres grupos comparables a nivel Tower (sin pasar por la pila de red):

| Grupo                       | Que mide                                                     |
| --- | --- |
| `bare_axum_hello`           | Linea base: router Axum minimo sin Shield.                   |
| `shield_default_hello`      | Shield con configuracion default (solo logging activo).      |
| `shield_full_default_hello` | Shield con CORS, CSRF y rate-limit activos al mismo tiempo.  |

TLS y JWT no entran en esta suite porque requieren material
criptografico (cert/key, par de claves). Su latencia se mide en PR 10
con tests E2E especificos por capa.

## Como ejecutar

Desde la raiz del repositorio:

```sh
# Compilacion sin ejecutar (verifica que el bench compila).
cargo bench -p ag-core --no-run

# Ejecucion completa (varios minutos).
cargo bench -p ag-core --bench shield_hello_world
```

Criterion publica los resultados en `target/criterion/` con graficos
HTML reproducibles. El report final se genera en
`target/criterion/report/index.html`.

## Que se reporta junto con cualquier numero

Ningun resultado se publica sin acompanarlo de:

- Hardware: CPU (modelo, frecuencia base y boost), nucleos fisicos y
  logicos, memoria RAM total, sistema de almacenamiento.
- Sistema operativo y kernel (`uname -a` en Linux).
- Version Rust (`cat rust-toolchain.toml` mas `rustc --version`).
- Commit del repositorio (`git rev-parse HEAD`).
- Configuracion de build: profile (release), debug info, codegen
  units, LTO.
- Numero de ejecuciones (criterion usa 100 samples por defecto).
- Desviacion estandar (criterion la reporta).
- Comando exacto utilizado.

## Hardware de referencia para reportes oficiales

Cuando un numero se publica como "metrica oficial Anti-Gravital", se
ejecuta sobre:

- CPU: AMD Ryzen 9 7950X o equivalente x86-64 con 16 nucleos fisicos.
- RAM: 32 GB DDR5.
- OS: Linux x86-64 reciente (kernel >= 6.0).
- Toolchain: Rust stable congelado en `rust-toolchain.toml`.

Resultados sobre hardware distinto se publican como "hardware
informativo" y se etiquetan claramente.

## Limitaciones

Estos benchmarks no miden throughput sostenido en req/s ni el consumo
de recursos (memoria idle, tiempo de arranque). Para esas metricas se
usa carga real por red con `wrk` u `oha` contra un binario release
(PR 10 de la Fase 1).
