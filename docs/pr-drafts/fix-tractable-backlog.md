# Descriptor de PR

## Resumen

Resolver issues del backlog: quitar unwrap en DSL, ETag fuerte, feature auth JWT y WebP lossy en ag-storage, y property tests

## Fase afectada

Cierre pre-Fase 5 / Fase 4.6. Correcciones de calidad y cobertura; no avanza fase.

## Tipo de cambio

- [x] Correccion (quita panic path; ETag debil)
- [x] Nueva feature opt-in (`ag-storage` `auth` JWT, `webp-lossy`)
- [x] Tests (property-based + unit)
- [ ] Documentacion / gobernanza
- [ ] Cambio que rompe compatibilidad

## Cambios

- `ag-dsl` (#150): `to_snake_case` sin `unwrap()` (mapeo Unicode completo) + test.
- `ag-storage` (#147): ETag fuerte con digest blake3 completo (256 bits) + test.
- `ag-storage` (#148): feature `auth` implementada; valida el Bearer como JWT
  Ed25519 via `ag-auth` con clave publica PEM configurable
  (`STORAGE_JWT_PUBLIC_KEY`/`_PATH`); modo token estatico sigue por defecto.
  El check de bind publico trata la clave JWT como autenticacion. Tests JWT.
- `ag-storage` (#146): feature `webp-lossy`; `to_webp` honra `quality` via el
  encoder nativo `webp` (libwebp embebido); lossless por defecto. Test de
  tamano por calidad.
- Property tests (#155): ag-auth (api keys), ag-mail (address, template vars),
  ag-domains (hostname parse/idempotencia), ag-dsl (compile/tokenize) con
  `proptest`.

## Plan de prueba

```sh
cargo test -p ag-dsl -p ag-auth -p ag-mail -p ag-domains
cargo test -p ag-storage
cargo test -p ag-storage --features auth
cargo test -p ag-storage --features webp-lossy
cargo clippy --workspace --all-targets -- -D warnings
```

## Criterios de salida que avanza

- Sin `unwrap()` alcanzable fuera de tests en el DSL; ETag fuerte (RFC 7232).
- `ag-storage` deja de declarar una capacidad `auth` inexistente (ADR-0009).
- Cobertura property-based en parsers/validators de cuatro crates.

## Cierre de issues

Closes #150
Closes #147
Closes #148
Closes #146
Closes #155

## Checklist final

- [x] Pertenece a la fase correcta y respeta la documentacion.
- [x] No rompe arquitectura; nuevas deps (webp, proptest) feature/dev-gated y de
      licencia permitida (MIT/Apache/BSD-3).
- [x] Compila; pasa fmt, clippy y tests por crate y feature.
- [x] Marcadores TECH-DEBT eliminados junto con su deuda; CHANGELOG actualizado.
- [x] Sin evidencia de herramientas IA; atribuido a la persona autora.
