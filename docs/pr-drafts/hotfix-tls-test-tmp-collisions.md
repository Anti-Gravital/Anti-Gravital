# Hotfix: colision de paths temporales entre tests TLS paralelos en Windows

## Resumen

Fix de colision de archivos /tmp en tests TLS por baja resolucion de `SystemTime::as_nanos()` en Windows. Genera paths unicos con pid+counter en lugar de timestamp.

## Fase afectada

Fase 1 (Shield MVP). Segundo hotfix del CI tras la merge del primer
hotfix (`1f456b7` en main). Macos-arm64 paso a verde, pero windows-x64
empezo a fallar de forma intermitente en `shield_full_pipeline.rs`.

## Tipo de cambio

- [ ] Documentacion
- [x] Codigo
- [x] Infraestructura o CI
- [ ] RFC nueva o actualizacion de RFC
- [ ] ADR nuevo
- [ ] Seguridad

## Documentos relacionados

- RFC: `docs/rfc/RFC-0002-diseno-shield-mvp.md`.
- ADR: N/A.
- Maestro afectado: N/A.

## Diagnostico

CI run #20 (commit `1f456b7`) mostro `build (windows-x64)` rojo con
tres tests fallando en `tests/shield_full_pipeline.rs`. Los tres
fallidos varian entre corridas, lo que indica race en lugar de bug
deterministico. Las otras tres plataformas (linux-x86-64, linux-arm64,
macos-arm64) verde.

Causa raiz: las funciones helper `generate_tls_cert` (en
`tests/shield_full_pipeline.rs`) y `generate_cert_pair` (en
`tests/shield_tls.rs`) construyen rutas temporales con
`SystemTime::now().duration_since(UNIX_EPOCH).as_nanos()`. En Linux y
macOS la resolucion del reloj es del orden de microsegundos. En
Windows, la resolucion efectiva de `SystemTime::now()` es de unos 15
ms, asi que dos tests que corren en paralelo dentro de la misma
ventana de 15 ms generan exactamente el mismo path.

Cuando A escribe su cert/key y B escribe los suyos en el mismo path,
B sobreescribe a A. Si A esta en medio de leer el archivo (en
`build_acceptor`) mientras B escribe, A puede leer contenido
parcial y fallar el handshake TLS. Tambien provoca mezcla de
cert/key entre tests, lo que rompe la verificacion.

El mismo patron afecta a `tmpfile` en
`crates/ag-core/src/shield/tls.rs` (tests internos), aunque alli el
riesgo es menor porque los tests internos no corren TLS handshakes.

## Fix

Sustituye el uso de `SystemTime` por un counter atomico + pid del
proceso, garantizando unicidad sin depender de la resolucion del
reloj:

```rust
static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_temp(prefix: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    std::env::temp_dir().join(format!("{prefix}-{pid}-{n}.pem"))
}
```

Aplicado en:

- `crates/ag-core/tests/shield_full_pipeline.rs::generate_tls_cert`.
- `crates/ag-core/tests/shield_tls.rs::generate_cert_pair`.
- `crates/ag-core/src/shield/tls.rs::tests::tmpfile`.

## Plan de prueba

```sh
# Reproducir el flakiness localmente con paralelismo alto en cualquier
# plataforma (no requiere Windows).
RUST_TEST_THREADS=16 cargo test -p ag-core --test shield_full_pipeline
RUST_TEST_THREADS=16 cargo test -p ag-core --test shield_tls

# Suite completa.
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
```

CI confirma el fix cuando `build (windows-x64)` vuelve a verde.

## Criterios de salida que avanza

No introduce nuevos criterios. Restaura el verde en CI para los
entregables ya marcados como completos en Fase 1.

## Checklist

- [x] Titulo de PR de 256 caracteres o menos.
- [x] Sin emojis en ningun archivo modificado.
- [x] Sin atribuciones de herramientas IA.
- [x] Documentacion actualizada (CHANGELOG).
- [x] CHANGELOG.md actualizado bajo `[Unreleased]`.
- [x] CLAUDE.md respetado: cambio acotado a infraestructura de tests;
  sin cambios en codigo de produccion; sin nuevas dependencias.
- [x] Descriptor pre-rellenado existe en `docs/pr-drafts/`.
