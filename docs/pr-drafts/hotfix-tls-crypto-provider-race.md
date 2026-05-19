# Hotfix: race del crypto provider global de rustls bajo tests paralelos en macos-arm64

## Resumen

Fix de race en `ServerConfig::builder()` que usaba el provider rustls global instalado por proceso: bajo tests paralelos en macos-arm64 fallaba intermitentemente. Inyecta el provider explicito por TlsAcceptor.

## Fase afectada

Fase 1 (Shield MVP). Hotfix del CI introducido por la merge de
`phase-1/shield-e2e-and-metrics` (PR #9, commit `8d4c1af`).

## Tipo de cambio

- [ ] Documentacion
- [x] Codigo
- [x] Infraestructura o CI
- [ ] RFC nueva o actualizacion de RFC
- [ ] ADR nuevo
- [x] Seguridad

## Documentos relacionados

- RFC: `docs/rfc/RFC-0002-diseno-shield-mvp.md` seccion 4.1 (stack TLS).
- ADR: N/A.
- Maestro afectado: N/A. El fix es interno; la superficie publica no
  cambia.

## Diagnostico

Tras mergear PR 10 (tests E2E del pipeline completo) el job
`build (macos-arm64)` empezo a fallar de forma intermitente en tres
de los seis tests de `tests/shield_full_pipeline.rs`. Linux x86-64,
Linux arm64 y Windows x64 seguian verde.

Causa raiz: `rustls::ServerConfig::builder()` usa el `CryptoProvider`
instalado en el proceso. La instalacion se realiza con
`rustls::crypto::ring::default_provider().install_default()`, que es
idempotente pero **no esta serializada con la lectura del default**.
Bajo paralelismo agresivo (los runners macos-arm64 corren tests en
varios threads simultaneos), un test puede leer el default antes de
que otro termine de instalarlo, lo que produce un panic
`no process-level CryptoProvider available`. El panic se observa en
unos pocos tests por corrida, no en todos.

Esto era un bug latente desde PR 7 (introduccion de TLS): el unico
test que tocaba TLS antes de PR 10 era `shield_tls.rs`, que llama a
`install_default()` una sola vez al inicio.

## Fix

`shield::tls::build_acceptor` deja de depender del provider global.
Construye el `ServerConfig` con
`ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))`
y `with_safe_default_protocol_versions()`. Cada `TlsAcceptor` lleva su
propio provider en su Arc, sin tocar estado de proceso. Esto elimina
la race por construccion.

Los tests que llamaban `rustls::crypto::ring::default_provider().install_default()`
para anticiparse al problema pasan a no necesitarlo. Se conservan los
`let _ = ...install_default();` por compatibilidad si otros componentes
del proceso siguieran usando el default (ejemplos: reqwest cliente
con `rustls-tls` puede instalar su propio default; no colisiona).

## Plan de prueba

```sh
# Reproducir el flakiness localmente con paralelismo alto.
RUST_TEST_THREADS=8 cargo test -p ag-core --test shield_full_pipeline

# Suite completa.
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
```

En CI se observa la matriz de cuatro plataformas. El verde en
macos-arm64 es el indicador de que el hotfix funciona.

## Criterios de salida que avanza

Restaura el verde de los workflows ya completados de Fase 1; no
introduce nuevos criterios de salida. Sigue siendo cierto:

- [x] Tests de integracion end-to-end del pipeline Shield (volvera a
  pasar en macos-arm64 con este hotfix).

## Checklist

- [x] Titulo de PR de 256 caracteres o menos.
- [x] Sin emojis en ningun archivo modificado.
- [x] Sin atribuciones de herramientas IA.
- [x] Documentacion actualizada en el mismo PR (CHANGELOG).
- [x] CHANGELOG.md actualizado bajo `[Unreleased]`.
- [x] CLAUDE.md respetado: cambio acotado a fix de seguridad
  operacional; sin nuevos crates; sin nuevas dependencias; sin
  `unsafe`; no debilita defaults seguros.
- [x] Descriptor pre-rellenado existe en `docs/pr-drafts/`.
