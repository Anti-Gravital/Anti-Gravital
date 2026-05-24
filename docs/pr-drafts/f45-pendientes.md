# fix(fase-4.5): ag-lsp v0.7 + DSL from→domain + ACME + docs

## Fase afectada

Fase 4.5 — Completar entregables pendientes: ag-lsp, ACME, documentacion.

## Tipo de cambio

Correcciones y completar entregables pendientes de Fase 4.5 (`fix` + `docs`).

## Documentos relacionados

- `docs/manual/03-dominio-tls-correo.md` — nuevo capitulo del manual
- `docs/manual/README.md` — indice actualizado
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` seccion 4.5.2 entregables

## Resumen

Cuatro entregables de Fase 4.5 que no formaron parte del PR #42:

**ag-lsp — soporte DSL v0.7** (`crates/ag-lsp/src/backend.rs`):
- Hover con documentacion completa para `mail`, `domain`, `template`
- Completions para keywords v0.7 y sus propiedades (`provider`, `from`,
  `subject`, `vars`, `dkim_selector`, `dmarc_policy`, `dmarc_rua`)
- 6 tests nuevos (total: 15 tests en ag-lsp)

**ag-domains — correccion ACME renewal** (`crates/ag-domains/src/acme/renewal.rs`):
- Bug fix: `check_interval - renew_threshold` causaba underflow (Duration
  de 1 dia menos 30 dias panickea en debug mode)
- Nuevo calculo: `sleep_secs = renew_before_days.max(1) * 86400`
- TECH-DEBT documentado para cuando se implemente parseo de `notAfter`
- 4 tests nuevos para `CertConfig`, `IssuedCert` y serde roundtrip
- `acme/mod.rs`: eliminar comentario "Skeleton de la Fase 4.5"

**ag-dsl — validacion from→domain** (`crates/ag-dsl/src/semantic.rs`):
- Warning si el hostname del `from` de un bloque `mail` no coincide con
  ningun `domain_name` declarado en los bloques `domain` del schema
- 2 tests nuevos (total: 153 tests en ag-dsl)

**STATUS.md + README.md** — Fase 4.5 marcada como completada con todos
los criterios de salida marcados.

**Manual cap. 3** (`docs/manual/03-dominio-tls-correo.md`):
- Guia: "Configurar dominio, TLS y correo transaccional con Anti-Gravital"
- 6 secciones: DSL, DNS sync, propagacion, ACME, correo transaccional,
  ag-auth + ag-mail. Con ejemplos de codigo compilables y comandos CLI.

## Plan de prueba

- `cargo test -p ag-lsp` — 15 tests, 0 fallos
- `cargo test -p ag-domains` — 28 tests, 0 fallos
- `cargo clippy -p ag-lsp -p ag-domains -- -D warnings` — 0 errores
- `cargo fmt --all -- --check` — sin cambios pendientes

## Criterios de salida avanzados

- ag-lsp provee hover y autocompletado para bloques DSL v0.7
- ACME renewal no panickea en debug mode con renew_before_days tipico
- Manual cap. 3 documenta el flujo dominio/TLS/correo de extremo a extremo

## Checklist final

- [x] Pertenece a la fase correcta (4.5 segun ADR-0007)
- [x] Respeta la documentacion
- [x] No rompe arquitectura
- [x] No anade complejidad innecesaria
- [x] No crea dependencias circulares
- [x] Compila
- [x] Pasa tests
- [x] Pasa fmt
- [x] Pasa clippy
- [x] Tiene documentacion (manual cap. 3)
- [x] Tiene manejo de errores correcto (bug fix renewal)
- [x] Mantiene coherencia con Anti-Gravital v4.0
