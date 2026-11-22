# Descriptor de PR

## Resumen

deps(reqwest): subida a 0.13 con rustls-no-provider y proveedor ring explicito (Closes #177)

## Fase afectada

Mantenimiento de dependencias previo a Fase 5. No avanza ninguna casilla de
`docs/roadmap/STATUS.md`.

## Tipo de cambio

- [x] Actualizacion de dependencias
- [x] Cambio de stack TLS (documentado en ADR-0018)
- [x] Documentacion tecnica y CHANGELOG
- [x] Tests de verificacion de handshake TLS
- [ ] Nueva feature de producto
- [ ] Cambio de arquitectura o DSL
- [ ] Cambio que rompe compatibilidad publica

## Contexto

`reqwest` 0.13 elimina `rustls-tls`: su feature `rustls` selecciona `aws-lc-rs`
y `rustls-no-provider` deja la eleccion del proveedor a la aplicacion. El
workspace estandariza en `ring`, por lo que se eligio `rustls-no-provider` mas
una instalacion explicita e idempotente de `ring` en cada punto de composicion.
Esto evita reintroducir el fallo de dos proveedores corregido en #181.

Se anadieron las features `form` y `query`, exigidas por los puntos de llamada
existentes en `ag-auth` y `ag-domains`. Se conservan los timeouts de 30 s de los
adaptadores DNS (hallazgo S7-4).

## Documentos relacionados

- `docs/adr/0018-reqwest-013-ring-provider.md` (nuevo; decision y alternatives,
  riesgo de ordenacion bajo `--all-features` y acoplamiento con OTLP/#149).
- `CHANGELOG.md` (seccion "reqwest 0.13 with the ring provider").
- `Cargo.toml`, `benchmarks/neon-real/Cargo.toml`, manifiestos de ag-auth,
  ag-domains, ag-core, ag-edge, examples y tests/integration.
- `crates/ag-core/tests/shield_tls.rs` (test de handshake TLS verificado).
- Issues #177, #181, #149; ADR-0008, ADR-0009.

## Plan de prueba

Ejecutados en el worktree con
`CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_TEST_DEBUG=0`:

```sh
cargo check --workspace --all-targets            # OK
cargo test --workspace                           # OK (0 failures)
cargo test --workspace --all-features            # OK (0 failures)
cargo fmt --all -- --check                       # OK
cargo clippy --workspace --all-targets -- -D warnings  # OK
cargo audit                                      # OK (sin vulnerabilidades)
cargo deny --all-features check                   # OK
```

Verificacion adicional de provider:

```sh
cargo tree -e features -i rustls                  # sin aws-lc-rs por reqwest
cargo tree --workspace --all-features -e features -i reqwest@0.13.5
# object_store/s3 sigue activando reqwest/rustls, pero el install explicito de
# ring gana porque reqwest consulta CryptoProvider::get_default() primero.
cargo check --manifest-path benchmarks/neon-real/Cargo.toml --all-targets  # OK
```

El benchmark nativo es un workspace independiente y su manifiesto ya estaba
roto (`rand = { workspace = true }` sin `workspace.dependencies`). Se declara
`rand 0.10` con `sys_rng` (idéntico al workspace raíz) para poder comprobar la
subida de `reqwest` y regenerar su `Cargo.lock`. Solo advertencias `dead_code`
preexistentes.

## Criterios de salida que avanza

Ninguna casilla de `docs/roadmap/STATUS.md`. Issue #177 queda resuelto y
verificado: workspace compila y testea en 0.13, el proveedor queda registrado
en ADR-0018 y el test de handshake cubre la instalacion explicita.

## Cierre de issues

Closes #177

## Checklist

- [x] Compila (`cargo check --workspace --all-targets`).
- [x] Pasa tests (`cargo test --workspace` y `--all-features`).
- [x] Pasa fmt y clippy con `-D warnings`.
- [x] Pasa `cargo audit` y `cargo deny --all-features check`.
- [x] Documentacion actualizada en el mismo cambio (ADR-0018 y CHANGELOG).
- [x] Sin `unsafe`, sin emojis, sin atribucion de herramientas IA, mensaje
      de commit por debajo de 256 caracteres.
