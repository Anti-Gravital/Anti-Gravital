# Descriptor de PR: community3/chrono

## Resumen

Replace chrono with time across workspace crates and DSL-generated projects (issue #185)

## Fase afectada

Mantenimiento / higiene de dependencias (post Fase 4.6), sin cambio de fase ni de roadmap.

## Tipo de cambio

- [x] Codigo (`ag-workers`, `ag-domains`, `workers-scheduled`)
- [x] Dependencias de Cargo (sqlx `chrono` -> `time`; workspace `chrono` -> `time`)
- [x] Cambio de DSL (`FieldType::Timestamp` mapping y adaptadores serde emitidos)
- [x] Documentacion tecnica y CHANGELOG
- [x] Tests golden de serde y test ignorado de TIMESTAMPTZ
- [ ] Cambio de arquitectura
- [ ] Cambio de security / stack

## Documentos relacionados

- `docs/rfc/RFC-0026-chrono-replacement.md` (call sites y decision de diseño)
- `docs/rfc/RFC-0025-ag-data-orm-rls.md` seccion 4.5
- `docs/modules/ag-data/data-layer-rls-tenancy.md` (restricciones de formato)
- `docs/dsl/referencia-v01-v04.md`, `docs/modules/ag-workers/README.md`
- `docs/rfc/RFC-0012-ag-workers.md`, `crates/ag-auth/README.md`
- `CHANGELOG.md` (seccion "Date/time migration: chrono to time")

## Plan de prueba

Ejecutados en este worktree con el target de Cargo compartido:

1. `cargo check --workspace --all-targets`
2. `cargo test --workspace --all-features` (todas las unidades: 0 fallos)
3. `cargo fmt --all -- --check`
4. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
5. `cargo audit` (4 allowed warnings, sin vulnerabilidades)
6. `cargo deny --all-features check` (advisories, bans, licenses y sources OK; el
   ignore de RUSTSEC-2026-0543 se conserva deliberadamente)
7. `cargo test -p ag-generated-rust-fixture --test serde_time` (5 golden tests)
8. `cargo tree -i chrono`: unica ruta restante es `oauth2 5.0.0 -> ag-auth`,
   eliminada en la rama/community3/oauthimpl ya integrada en `main`.
9. `cargo test -p ag-workers --features postgres --test timestamptz_roundtrip --no-run`
   (compila; el test esta `#[ignore]` porque requiere `DATABASE_URL`).

## Criterios de salida que avanza

- #185 "No workspace crate depends on chrono": se eliminan las dependencias
  directas de `ag-domains`, `ag-workers`, `workers-scheduled` y el feature
  `chrono` de sqlx. Queda solo la arista de `oauth2` que otra rama retira.
- #185 "serialized formats unchanged": se conservan byte a byte los segundos
  enteros y las fracciones de 9 digitos. Cambian dos formas textuales,
  documentadas como breaking changes: fracciones con ceros finales
  (`.100Z` -> `.1Z`) y `+00:00` -> `Z` en formateo explicito.
- #185 verificar round-trip de TIMESTAMPTZ: se anade test `#[ignore]` con
  precision microsegundo; NO se ejecuto (sin base de datos). No se afirma verificado.

## Cambios que el coordinador debe aplicar al fusionar

- Eliminar el ignore de RUSTSEC-2026-0543 en `deny.toml` despues de integrar
  esta rama y la de oauth2, y verificar que `cargo tree -i chrono` este vacio.
- Resolver los conflictos esperados en `Cargo.toml` (entry `chrono` de main,
  entry `time` de esta rama) y en `Cargo.lock`; conservar `time` y el feature
  `time` de sqlx. Conservar tambien el follow-up de RFC-0026 que main anadio.

## Cierre de issues

Refs #185
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (mantenimiento, sin cambio de fase).
- [x] Respeta la documentacion (RFC-0026 y RFC-0025 seccion 4.5).
- [x] No rompe arquitectura (mismo limite de crates, solo tipos de fecha/hora).
- [x] Compila.
- [x] Pasa tests (workspace --all-features y 5 golden tests de serde).
- [x] Pasa fmt.
- [x] Pasa clippy con -D warnings.
- [x] Pasa audit (4 allowed warnings).
- [x] Pasa deny --all-features check con el ignore todavia presente.
- [x] Tiene documentacion y documenta los breaking changes de formato.
- [x] Mantiene coherencia con Anti-Gravital v4.1.
- [ ] No cierra #185: queda abierto hasta que el coordinador elimine el ignore
      y verifique el arbol de dependencias sin chrono.
