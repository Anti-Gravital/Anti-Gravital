# Descriptor de PR

## Resumen

ag-storage: anadir codificacion AVIF tras la feature `avif` (encoder pure-Rust)

## Fase afectada

Completa el soporte de formatos de imagen de `ag-storage` (modulo de
procesamiento de imagen). No avanza una fase de la Hoja de Ruta ni toca otros
crates.

## Tipo de cambio

- [ ] Correccion de bug
- [x] Nueva feature (codificacion AVIF tras feature Cargo)
- [x] Documentacion (cabecera del modulo)
- [ ] CI / seguridad
- [ ] Cambio que rompe compatibilidad

## Contexto

`ImageProcessor` soportaba JPEG, PNG y WebP; AVIF estaba pendiente (issue #145).
Esta PR anade `ImageProcessor::to_avif`, que codifica cualquier entrada soportada
a AVIF mediante el camino pure-Rust del crate `image` (`ravif`/`rav1e`), sin
ninguna biblioteca del sistema.

Queda tras una feature Cargo (`avif`) porque arrastra el stack de `ravif`
(CLAUDE.md regla 15: dependencia justificada y aislada; ADR-0009 regla 2: el
camino nativo por defecto se conserva intacto). Solo se anade codificacion: la
decodificacion AVIF requeriria la biblioteca C `dav1d`, lo que queda fuera de
alcance; por eso se acepta cualquier formato de entrada (JPEG/PNG/WebP) como
origen, como permite el "Done when" del issue ("decode and/or encode").

## Cambios

- `crates/ag-storage/Cargo.toml`: nueva feature `avif = ["image/avif"]`.
- `crates/ag-storage/src/image.rs`: metodo `to_avif(data, quality)` tras
  `#[cfg(feature = "avif")]`; cabecera `//!` actualizada a la realidad; test
  `to_avif_produces_valid_container` que valida la estructura ISOBMFF/`ftyp` y
  la marca `avif` del contenedor.

## Plan de prueba

```sh
cargo fmt --check -p ag-storage
cargo clippy -p ag-storage --features avif -- -D warnings
cargo test  -p ag-storage --features avif     # incluye to_avif_produces_valid_container
# El camino por defecto (sin feature) sigue intacto:
cargo clippy -p ag-storage -- -D warnings
cargo test  -p ag-storage
```

Resultado verificado en este entorno (Rust 1.95.0): build, clippy y los 75
tests en verde con y sin la feature; el test AVIF nuevo pasa.

## Criterios de salida que avanza

- `ImageProcessor` codifica AVIF tras feature, con el camino nativo por defecto
  preservado.
- Cabecera `//!` de `image.rs` refleja el soporte real.
- Round-trip de codificacion cubierto por un test con fixture pequeno.

## Cierre de issues

Closes #145

## Checklist final

- [x] Pertenece a la fase correcta y respeta la documentacion.
- [x] No rompe arquitectura; la dependencia nueva queda aislada tras feature.
- [x] No crea dependencias circulares.
- [x] Compila; pasa fmt, clippy y tests con y sin la feature.
- [x] Tiene documentacion (cabecera del modulo y doc del metodo) y test.
- [x] Sin evidencia de herramientas IA; atribuido a la persona autora.
