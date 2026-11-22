# docs(rfc): chrono replacement assessment (Refs #185)

- Fase afectada: mantenimiento / higiene de dependencias (post Fase 4.6).
- Tipo de cambio: documentacion (RFC propuesta, sin cambios de codigo).
- Documentos relacionados: `docs/rfc/RFC-0026-chrono-replacement.md`,
  issue #185, RUSTSEC-2026-0543, RFC-0012, ADR-0008.
- Modulos afectados: ninguno. No se modifica `Cargo.toml`, `Cargo.lock`,
  `deny.toml`, `crates/**`, `examples/**`, `README.md` ni `CHANGELOG.md`.

## Resumen

Registra la evaluacion `time` frente a `jiff`, el inventario exacto de
call sites de chrono y el bloqueo real: `oauth2 5.0.0` depende de chrono de
forma no opcional, por lo que no se puede eliminar el ignore de deny.toml
sin una RFC previa para sustituir o parchear oauth2.

## Plan de prueba

- `cargo tree -i chrono`: verifica las cuatro rutas (ag-domains, ag-workers,
  sqlx-core, oauth2 5.0.0).
- `cargo deny check advisories`: pasa con el ignore actual.
- `cargo deny --config <sin ignore RUSTSEC-2026-0543> check advisories`:
  falla con `error[unmaintained]: chrono is unmaintained`, incluyendo la
  ruta `oauth2 v5.0.0 -> ag-auth`.
- `cargo tree -i jiff`: no imprime nada (jiff no esta en el grafo).
- No se ejecutan fmt/clippy/test/audit porque no hay cambios de codigo.

## Criterios de salida que avanza

- Evaluation de `time` vs `jiff` con decision motivada (issue #185 paso 1).
- Inventario de call sites y orden de migracion recomendado.
- Evidencia concreta de que el ignore de deny.toml no puede eliminarse
  todavia, sin simular finalizacion.

## Cierre de issues

Refs #185
Closes: none

## Checklist final

- [x] Sin evidencia de herramientas IA ni emojis.
- [x] Mensajes y titulo por debajo de 256 caracteres.
- [x] Documentacion tecnica en ingles (ADR-0008).
- [x] Sin cambios de codigo, sin dependencias nuevas, sin merge a main.
