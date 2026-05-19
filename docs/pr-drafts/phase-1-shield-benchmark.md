# PR: Benchmark Hello World del Shield con criterion (Fase 1 PR 9 de 11)

## Resumen

Benchmark Hello World con criterion sobre el pipeline Shield: baseline Axum vs Shield default vs Shield con todas las capas, en latencia por request a nivel Tower.

## Fase afectada

Fase 1 (Shield MVP). PR 9 de los 11 incrementos previstos en
`docs/rfc/RFC-0002-diseno-shield-mvp.md`.

## Tipo de cambio

- [x] Documentacion
- [x] Codigo
- [ ] Infraestructura o CI
- [ ] RFC nueva o actualizacion de RFC
- [ ] ADR nuevo
- [ ] Seguridad

## Documentos relacionados

- RFC: `docs/rfc/RFC-0002-diseno-shield-mvp.md`, seccion 4.
- ADR: N/A.
- Maestro afectado: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
  seccion 16 (objetivos de rendimiento). El cumplimiento de los
  objetivos duros (300K req/s, p99 <=1ms, idle <=15MB, arranque
  <=100ms) se valida en PR 10 con carga sostenida; este PR mide
  overhead de middleware por request.

## Detalle del cambio

### Suite de benchmarks

Se publica `crates/ag-core/benches/shield_hello_world.rs` con tres
grupos criterion:

- `bare_axum`: router Axum minimo sin Shield, como linea base de
  referencia. Mide solamente el costo de Axum + Tokio + tower al
  enrutar una peticion GET `/`.
- `shield_default`: Shield con configuracion por defecto (capa de
  logging estructurado siempre activa, resto deshabilitadas). Mide el
  overhead minimo aceptable.
- `shield_full_default`: Shield con todas las capas opcionales que se
  activan sin requerir keys (CORS configurado con un origen permitido,
  CSRF activado con tokens dummy, rate-limit con burst alto, sin auth
  ni TLS porque exigen material criptografico). Mide overhead con
  cuatro capas Tower activas a la vez.

Cada grupo emite N peticiones GET `/` a traves del router como
`tower::Service`, sin pasar por la pila de red. Esto produce
mediciones deterministas a nivel de microsegundos por request.

### Configuracion criterion

`[[bench]]` con `harness = false` para que criterion provea su propio
runner. `criterion = 0.5` como dev-dependency del workspace.

### Methodologia documentada

`crates/ag-core/benches/README.md` documenta el hardware de
referencia exigido, version Rust (`rust-toolchain.toml`), comando
exacto de ejecucion y la regla del proyecto: ningun numero publicado
sin contexto (hardware, OS, version, commit, std-dev) segun regla 17
de `CLAUDE.md`.

### Limitacion explicita

Este PR no mide throughput sostenido en req/s ni recursos del proceso
(memoria idle, tiempo de arranque). Esas metricas requieren carga
real por red con `wrk` u `oha` contra un binario release. Se entregan
en PR 10. Aqui medimos overhead determinista de la pipeline de
middleware.

## Plan de prueba

```sh
# Compilacion del bench sin ejecutarlo (verificacion rapida).
cargo bench -p ag-core --no-run

# Ejecucion completa del bench (toma varios minutos).
cargo bench -p ag-core --bench shield_hello_world

# Tests, fmt, clippy y deny siguen pasando.
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
```

Resultados criterion se publican bajo `target/criterion/` con
graficos HTML. Para reporte oficial, el operador anota:

- Hardware (CPU, RAM, kernel).
- Sistema operativo y version.
- Version Rust (cat rust-toolchain.toml).
- Commit (git rev-parse HEAD).
- Configuracion (release sin debug-info).
- Numero de ejecuciones (default criterion 100 samples).
- Desviacion estandar (criterion la reporta).

## Criterios de salida que avanza

De `docs/roadmap/STATUS.md` Fase 1, esta PR marca:

- [x] Benchmark Hello World ejecutable: `cargo bench` produce cifras
  reproducibles.

Sigue pendiente para PRs siguientes y para el cierre de Fase 1:

- [ ] Benchmark Hello World >= 300K req/s en hardware de referencia.
- [ ] Latencia p99 <= 1 ms a 100K req/s.
- [ ] Memoria idle <= 15 MB.
- [ ] Arranque <= 100 ms.
- [ ] Tests E2E del pipeline completo (PR 10).
- [ ] Documentacion API y manual (PR 11).

Las metricas absolutas se validaran en PR 10 con carga real.

## Checklist

- [x] Titulo de PR de 256 caracteres o menos.
- [x] Sin emojis en ningun archivo modificado.
- [x] Sin atribuciones de herramientas IA.
- [x] Documentacion actualizada en el mismo PR (CHANGELOG, STATUS,
  benches/README.md).
- [x] CHANGELOG.md actualizado bajo `[Unreleased]`.
- [x] CLAUDE.md respetado: alcance Fase 1; sin nuevos crates; nueva
  dependencia (criterion) justificada como estandar de facto Rust;
  sin `unsafe`; reglas 17 de benchmarks aplicadas.
- [x] Descriptor pre-rellenado existe en `docs/pr-drafts/`.
