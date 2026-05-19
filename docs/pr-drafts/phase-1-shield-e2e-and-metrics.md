# PR: Tests E2E del pipeline completo y plantilla de medicion de metricas duras (Fase 1 PR 10 de 11)

## Resumen

Tests E2E del pipeline Shield completo, fix de ConnectInfo en Shield::serve, ejemplo release-ready para wrk/oha y plantilla de medicion de metricas duras de cierre de Fase 1.

## Fase afectada

Fase 1 (Shield MVP). PR 10 de los 11 incrementos previstos en
`docs/rfc/RFC-0002-diseno-shield-mvp.md`.

## Tipo de cambio

- [x] Documentacion
- [x] Codigo
- [ ] Infraestructura o CI
- [ ] RFC nueva o actualizacion de RFC
- [ ] ADR nuevo
- [x] Seguridad

## Documentos relacionados

- RFC: `docs/rfc/RFC-0002-diseno-shield-mvp.md`.
- ADR: N/A (no introduce decisiones arquitectonicas nuevas).
- Maestro afectado: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
  seccion 16 (objetivos de rendimiento) y seccion 6 (organizacion del
  nucleo).

## Detalle del cambio

### Fix de seguridad operacional: `ConnectInfo` en `Shield::serve`

La capa de rate-limit identifica clientes por la IP que extrae del
extractor `ConnectInfo<SocketAddr>` de Axum. Hasta este PR,
`Shield::serve` usaba `Router::into_make_service()`, que no inyecta
ese extractor: en consecuencia, rate-limit pasaba transparente sobre
cualquier proceso arrancado con `Shield::serve` (en tests aislados se
verificaba con un make-service distinto, lo que ocultaba el bug en
produccion).

Cambio: `Shield::serve` ahora usa
`Router::into_make_service_with_connect_info::<SocketAddr>()` en el
camino plano. En el camino TLS, la funcion `serve_tls` captura el
`peer_addr` del `TcpStream` aceptado y lo inyecta como
`ConnectInfo<SocketAddr>` en las extensiones de cada request antes
de pasarla al servicio. La firma publica de `Shield::serve` no
cambia.

### Tests E2E del pipeline completo

`crates/ag-core/tests/shield_full_pipeline.rs` arranca un servidor
con todas las capas activas simultaneamente (TLS, auth-jwt, csrf,
cors, rate-limit, validation, logging) y valida:

- Request valido pasa por todas las capas y llega al handler.
- Token JWT invalido se rechaza con 401 (capa auth).
- Origen no listado se rechaza por CORS (sin header allow-origin).
- POST sin CSRF token + cookie se rechaza con 403 (capa csrf).
- Rate-limit se dispara tras agotar el burst.

### Ejemplo release-ready para medicion

`crates/ag-core/examples/hello_world.rs`: binario de ejemplo que
arranca un Shield con configuracion minima (rate-limit, CSRF, CORS
deshabilitados; TLS deshabilitado para medicion plain o
configurables via TOML) y sirve `GET /` con body `hello, world`.
Pensado para correr `cargo run --release -p ag-core --example hello_world`
y medir con `oha` o `wrk` desde un cliente externo.

### Plantilla para metricas duras de cierre de Fase 1

`docs/benchmarks/measurement-template.md` es la plantilla que el
operador rellena tras correr `oha`/`wrk` y `/usr/bin/time` contra el
binario release. Cubre los cuatro objetivos duros de la Hoja de Ruta:

- Throughput >= 300K req/s.
- Latencia p99 <= 1 ms a 100K req/s.
- Memoria idle del proceso <= 15 MB (medida con `ps` o `cargo-bloat`).
- Tiempo de arranque <= 100 ms (medido con `time` desde fork hasta
  bind ready).

La plantilla exige todos los datos de la regla 17: hardware, OS,
version Rust, commit, configuracion, metodologia, ejecuciones,
desviacion estandar. Sin esos datos, el numero no se publica.

## Plan de prueba

```sh
# Tests E2E + unit del crate. El test nuevo del pipeline completo
# corre TLS, JWT, CSRF, CORS y rate-limit a la vez.
cargo test --workspace

# Fmt, clippy y deny.
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check

# Compilacion release del ejemplo y arranque manual local.
cargo run --release -p ag-core --example hello_world &
sleep 1
curl -i http://127.0.0.1:8080/
kill %1

# Medicion oficial (regla 17). El operador rellena la plantilla:
oha -z 30s -c 100 http://127.0.0.1:8080/ > /tmp/oha-report.txt
# y copia el reporte a docs/benchmarks/measurement-<fecha>.md siguiendo
# docs/benchmarks/measurement-template.md.
```

## Criterios de salida que avanza

De `docs/roadmap/STATUS.md` Fase 1, esta PR marca:

- [x] Tests de integracion end-to-end del pipeline Shield.

Sigue pendiente y a cargo del operador (este PR aporta la
infraestructura para medirlo, no los numeros):

- [ ] Benchmark Hello World >= 300K req/s en hardware de referencia.
- [ ] Latencia p99 <= 1 ms a 100K req/s.
- [ ] Memoria idle <= 15 MB.
- [ ] Arranque <= 100 ms.
- [ ] Documentacion API y manual (PR 11).
- [ ] Cobertura de tests >= 80% del crate (este PR la incrementa; la
  validacion oficial necesita `cargo-llvm-cov` o similar y no se
  ejecuta aqui).

## Checklist

- [x] Titulo de PR de 256 caracteres o menos.
- [x] Sin emojis en ningun archivo modificado.
- [x] Sin atribuciones de herramientas IA.
- [x] Documentacion actualizada en el mismo PR (CHANGELOG, STATUS,
  measurement-template, hello_world example).
- [x] CHANGELOG.md actualizado bajo `[Unreleased]`.
- [x] CLAUDE.md respetado: alcance Fase 1; sin nuevos crates; sin
  nuevas dependencias de runtime; sin `unsafe`; defaults seguros
  preservados; el fix de ConnectInfo cierra un hueco de seguridad
  operacional documentado.
- [x] Descriptor pre-rellenado existe en `docs/pr-drafts/`.
