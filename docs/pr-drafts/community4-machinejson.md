# Descriptor de PR: community4/machinejson

## Resumen

Add versioned `--output json` for read-only CLI commands (RFC-0021 stage E substrate, RFC-0024 prerequisite)

## Fase afectada

Fase 4.8, etapa E (interfaz de maquina). Avance parcial: solo la parte de
salida JSON versionada de comandos de solo lectura. No se marca la etapa E como
completada: faltan `ag describe`, tabla completa de codigos de salida,
`--plan`/`--apply` y registro de auditoria.

## Tipo de cambio

- [x] Codigo (`ag-cli`: flag global, envelope versionado, reportes de lectura)
- [x] Tests (12 tests de snapshot de integracion + tests unitarios de parser y envelope)
- [x] Documentacion (`docs/mcp/`, CHANGELOG, README, revision de preparacion MCP)
- [x] Contrato de datos versionado (JSON Schema + snapshots)
- [ ] Cambio de arquitectura
- [ ] Cambio de security / stack
- [ ] Nuevo crate o dependencia externa (solo `serde_json`, ya presente en el workspace)

## Documentos relacionados

- `docs/rfc/RFC-0021-server-install-remote-console-and-machine-interface.md`, seccion 4.3
- `docs/rfc/RFC-0024-mcp-shared-machine-interface.md`, secciones 4.2, 4.3 y 9
- `docs/audits/mcp-readiness-review.md`, secciones 1, 2, 8 y nueva seccion 12
- `docs/mcp/README.md`, `docs/mcp/cli-json-contract-0.1.0.md`,
  `docs/mcp/cli-json-0.1.0.schema.json`, `docs/mcp/snapshots/`
- `README.md`, `crates/ag-cli/README.md`, `docs/modules/ag-cli/README.md`, `CHANGELOG.md`

## Plan de prueba

Ejecutados en este worktree con el target de Cargo compartido:

1. `cargo test -p ag-cli` (64 unitarios + 12 snapshots + 6 no interactivos, 0 fallos)
2. `cargo test --workspace --all-features` (0 fallos)
3. `cargo fmt --all -- --check`
4. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
5. `cargo audit` (4 allowed warnings conocidos, 0 vulnerabilidades)
6. `cargo deny --all-features check` (advisories, bans, licenses y sources OK)

## Criterios de salida que avanza

- RFC-0021 4.3.1 "Structured output": contrato `0.1.0` para `schema.lint`,
  `schema.diff` y `workers.list`, con `contract_version` y snapshots byte a byte.
- RFC-0021 4.3.2 "Stable exit codes": 0 exito, 1 fallo de lectura/validacion,
  2 uso no soportado, para los comandos cubiertos. La tabla completa sigue abierta.
- RFC-0024 seccion 9, prerequisito 1 (parte de JSON de solo lectura): satisfecho.
- Los prerequisitos 1 (plan/apply y auditoria), 2, 3, 4-7 siguen abiertos y asi
  queda registrado en `docs/audits/mcp-readiness-review.md`.

## Cierre de issues

Closes: none

## Checklist

- [x] Pertenece a la fase correcta (4.8-E, avance parcial documentado).
- [x] Respeta la documentacion (RFC-0021 y RFC-0024; sin codigo MCP).
- [x] No rompe arquitectura (un solo modulo nuevo en `ag-cli`, sin crates nuevos).
- [x] Compila.
- [x] Pasa tests.
- [x] Pasa fmt.
- [x] Pasa clippy con -D warnings.
- [x] No imprime secretos (test de canario en rechazos).
- [x] Contrato, snapshots y CHANGELOG actualizados en la misma rama.
