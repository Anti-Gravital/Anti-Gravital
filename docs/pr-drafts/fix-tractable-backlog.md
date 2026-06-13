# Descriptor de PR

## Resumen

Resolver dos quick wins del backlog: quitar unwrap() en to_snake_case (ag-dsl) y emitir ETag fuerte completo (ag-storage)

## Fase afectada

Cierre pre-Fase 5 / Fase 4.6. Correcciones puntuales de calidad; no avanza fase.

## Tipo de cambio

- [x] Correccion (quita panic path; corrige ETag debil)
- [ ] Documentacion / gobernanza
- [x] Tests (cobertura nueva para ambas correcciones)
- [ ] Nueva feature
- [ ] Cambio que rompe compatibilidad

## Cambios

- `ag-dsl` (`to_snake_case`): `ch.to_lowercase().next().unwrap()` -> `extend`
  con el mapeo Unicode completo; elimina el unico `unwrap()` fuera de tests y
  su marcador TECH-DEBT. Test nuevo de PascalCase/camelCase/borde.
- `ag-storage` (`etag_for`): ETag fuerte con el digest blake3 completo (256
  bits) en vez de truncado a 64 bits; elimina el marcador TECH-DEBT. Test
  nuevo de formato fuerte y resistencia a colision.

## Documentos relacionados

- Issues #150 y #147 (ambos abiertos desde la reconciliacion de #135).

## Plan de prueba

```sh
cargo test -p ag-dsl
cargo test -p ag-storage
cargo clippy -p ag-dsl -p ag-storage --all-targets -- -D warnings
grep -n "unwrap()" crates/ag-dsl/src/ast.rs   # solo dentro de #[cfg(test)]
```

## Criterios de salida que avanza

- Ningun `unwrap()` alcanzable desde el codigo del DSL fuera de tests.
- ETag fuerte conforme a RFC 7232 (sin colisiones por truncado).

## Cierre de issues

Closes #150
Closes #147

## Checklist final

- [x] Pertenece a la fase correcta y respeta la documentacion.
- [x] No rompe arquitectura ni anade dependencias.
- [x] Compila; pasa fmt, clippy y tests de los crates tocados.
- [x] Marcadores TECH-DEBT eliminados junto con la deuda.
- [x] Sin evidencia de herramientas IA; atribuido a la persona autora.
