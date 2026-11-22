# Add CI verification for the ag-observe OTLP feature (issue #149, RFC-0028)

## Resumen

Registra RFC-0028 (aceptada por BDFL, periodo de comentarios renunciado) y anade el job `ci / ag-observe otlp feature` que comprueba, testea y lintea `ag-observe` con el feature opt-in `otlp`. Sin `protoc`, sin collector y sin cambios en la matriz de features por defecto.

## Fase afectada

Fase 4.7 (calidad y cierre de deuda tecnica de `ag-observe`, ADR-0019). Sin cambio de fase, alcance ni roadmap.

## Tipo de cambio

- [x] RFC nueva (requerida por la regla 28: cambio de pipeline de CI)
- [x] Cambio de CI (job aditivo de verificacion de feature)
- [x] Documentacion (indice de RFC, ADR-0019, CHANGELOG)
- [ ] Codigo de crates (ninguno; el feature ya estaba implementado en `community/dev5`)

## Documentos relacionados

- Issue #149 (principal; permanece abierta, `Refs`).
- ADR-0019 (ag-observe OTLP transport without protoc).
- ADR-0009 regla 2 (default nativo preservado).
- RFC-0009 seccion 4.8 (precedente del job `mail-mta` en `ci.yml`).
- Regla CLAUDE.md 28 (RFC para cambios de CI) y 35 (ADR).

## Cambios concretos

- `docs/rfc/RFC-0028-otlp-feature-ci-verification.md`: RFC nueva, estado aceptada, decision BDFL con periodo de comentarios renunciado; declara explictamente que el job no hace obligatorios `protoc` ni collector externo.
- `docs/rfc/README.md`: fila 0028 en el indice de RFC.
- `.github/workflows/ci.yml`: job `otlp` con pasos `cargo check --locked -p ag-observe --features otlp --all-targets`, `cargo test --locked -p ag-observe --features otlp` y `cargo clippy --locked -p ag-observe --features otlp --all-targets -- -D warnings`. Cache propio; matriz por defecto intacta.
- `docs/adr/0019-ag-observe-otlp-transport.md`: nota de trazabilidad hacia RFC-0028; la decision de no requerir `protoc`/collector queda intacta.
- `CHANGELOG.md`: entrada bajo `[Unreleased]` con evidencia y proposito.
- `docs/pr-drafts/community2-otlp.md`: este descriptor.

## Plan de prueba

Ejecutado localmente el 2026-11-22 con
`CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0`:

```
cargo check --locked -p ag-observe --features otlp --all-targets
cargo test --locked -p ag-observe --features otlp
cargo fmt --all -- --check
cargo clippy --locked -p ag-observe --features otlp --all-targets -- -D warnings
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo audit
cargo deny --all-features check
PROTOC=/nonexistent/protoc cargo check --locked -p ag-observe --features otlp  # target-dir limpio
```

Evidencia: check exit 0; test 14 passed / 0 failed (1 doc-test ignored), exit 0; fmt exit 0; clippy feature exit 0; clippy workspace all-features exit 0; audit exit 0 (4 warnings permitidos: astral-tokio-tar, chrono, humantime, paste); deny exit 0 (`advisories ok, bans ok, licenses ok, sources ok`); build sin protoc exit 0. No se inicio ningun collector.

## Criterios de salida que avanza

- Issue #149 "Done when" 1 (situacion `protoc` documentada/mitigada): CUMPLIDO, sin cambios de CI que lo hagan obligatorio (ADR-0019 + RFC-0028).
- Issue #149 "Done when" 2 (feature opt-in, default nativo, `cargo doc`): CUMPLIDO en la implementacion previa; el job anade verificacion reproducible.
- Issue #149 "Done when" 3 (marcadores TECH-DEBT eliminados): CUMPLIDO en la implementacion previa.
- Cierre final de #149: PENDIENTE de que el coordinador fusione esta rama y confirme el job `ci / ag-observe otlp feature` en verde en el primer CI de `main`. Por eso se usa `Refs`, no `Closes`.

## Cierre de issues

Refs #149
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (4.7, calidad; sin cambio de alcance)
- [x] Respeta la documentacion (regla 28: RFC antes de cambiar CI)
- [x] No anade dependencias ni cambia features por defecto
- [x] No introduce `protoc` ni collector como requisito
- [x] Compila, pasa tests, fmt, clippy, audit y deny
- [x] Documentacion actualizada en el mismo cambio (RFC, indice, ADR, CHANGELOG)
- [x] Sin push ni merge desde esta rama
