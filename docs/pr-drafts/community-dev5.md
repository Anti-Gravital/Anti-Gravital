# Resumen

Restore ag-observe OTLP trace export as an opt-in HTTP/protobuf feature, without protoc (Refs #149)

## Fase afectada

Mantenimiento de deuda tecnica de la Fase 4 (`ag-observe`). No cambia ninguna casilla de la Hoja de Ruta ni el estado de una fase.

## Tipo de cambio

- [x] Codigo (feature de Cargo, modulo `layer`, tests)
- [x] Documentacion tecnica (ADR-0019, README del crate, documentacion Rust)
- [x] Dependencias (nueva dependencia opcional `opentelemetry-otlp` 0.33)
- [x] CHANGELOG
- [ ] Maestros documentales (no tocados)
- [ ] Cambio de CI (no necesario: ningun paso requiere `protoc`)

## Documentos relacionados

- `docs/adr/0019-ag-observe-otlp-transport.md` (nuevo; decision de transporte)
- `docs/audits/infrastructure-review-112-149-141.md` (premisa corregida)
- `crates/ag-observe/README.md`, `crates/ag-observe/src/lib.rs`
- `ADR-0009` (default nativo sin colector externo), `ADR-0011` (sin marcas comerciales), `ADR-0008` (ingles canonico en documentacion tecnica)

## Cambio concreto

- `Cargo.toml`: `opentelemetry-otlp` 0.33 con `default-features = false`, `features = ["http-proto", "reqwest-blocking-client"]`; se elimina el marcador `TECH-DEBT` #149. Nunca se habilita `grpc-tonic`.
- `crates/ag-observe/Cargo.toml`: feature `otlp` (OFF por defecto) = `dep:opentelemetry-otlp` + `dep:rustls`; se elimina el marcador `TECH-DEBT` #149.
- `crates/ag-observe/src/layer.rs`: `init()` anade una capa `tracing-opentelemetry` con `SdkTracerProvider` y `BatchSpanProcessor` (hilo propio) solo si la feature `otlp` esta activa y `OTEL_EXPORTER_OTLP_ENDPOINT` esta definida. Sin la feature, el endpoint sigue devolviendo `ObserveError::OtlpSetup`. Instalacion idempotente del proveedor `ring` antes de construir el cliente, por la unification de features de reqwest 0.13 (#177).
- `SERVICE_NAME` (default `ag-service`) pasa a ser el atributo `service.name` del recurso OTLP.

## Plan de prueba

```sh
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --all-features
cargo audit
cargo deny --all-features check
RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps
# Sin protoc, target dir limpio:
PROTOC=/nonexistent/protoc CARGO_TARGET_DIR=/tmp/opencode/dev5-protoc-check \
  cargo check -p ag-observe --all-features
```

Evidencia: todos los comandos anteriores salieron 0 en el worktree de la rama. El build aislado con `PROTOC=/nonexistent/protoc` compilo `opentelemetry-proto`, `opentelemetry-otlp`, `tonic` y `tonic-prost` desde cero sin el compilador de protobuf.

## Criterios de salida que avanza

- Issue #149: ambos marcadores `TECH-DEBT` eliminados; exporter reintegrado detras de una feature con default nativo preservado; documentado que no se necesita `protoc` en CI multiplataforma. Pendiente de verificacion en los runners de CI antes de cerrar el issue.

## Cierre de issues

Closes: none
Refs #149 (implementacion completa; cierre pendiente de CI verde en los tres runners)

## Checklist

- [x] Pertenece a la fase correcta (mantenimiento de deuda Fase 4)
- [x] Respeta la documentacion y no anade features fuera de alcance
- [x] No rompe arquitectura ni crea dependencias circulares
- [x] Default nativo sin colector externo (ADR-0009)
- [x] Compila, pasa fmt, clippy, tests, audit y deny
- [x] Documentacion actualizada en la misma rama
- [x] Sin emojis, atribucion de herramientas ni marcas comerciales prohibidas
