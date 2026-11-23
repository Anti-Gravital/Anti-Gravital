## Resumen

Add `ag describe`, a read-only project description with an additive versioned `project.describe` JSON shape (RFC-0024).

## Componente afectado

Fase 4.8-E, crate `ag-cli` (CLI, dispatch, JSON mode, output shapes), contrato `docs/mcp/` 0.2.0.

## Tipo de cambio

- [ ] bug
- [x] enhancement
- [ ] tech-debt
- [ ] documentation
- [ ] refactor

## Documentos relacionados

- RFC-0024 section 4.2 (autoriza `project.describe` y el comando de solo lectura).
- RFC-0021 section 4, rule 3 (extension aditiva pre-0.5 sin bump de version).
- `docs/mcp/cli-json-contract-0.2.0.md`, `docs/mcp/cli-json-0.2.0.schema.json`,
  `docs/mcp/snapshots/project.describe.json`, `docs/mcp/README.md`.
- README (EN+ES), `crates/ag-cli/README.md`, `CHANGELOG.md`.
- No hay ADR nuevo: no cambia arquitectura, crates, DSL ni limites de modulo.

## Decision de contrato

No existia una forma concreta para `project.describe` ni `schema.describe`: el
contrato 0.2.0 los listaba explicitamente como no definidos (seccion 1). Se
agrega `project.describe` de forma aditiva: nuevo valor de `command` y nueva
definicion de `data`, sin tocar ninguna forma existente. Conforme a RFC-0021
seccion 4 regla 3, antes de 0.5 eso no incrementa `contract_version`, que
permanece en `0.2.0`. Se eligio `project.describe` (no `schema.describe`) porque
el comando describe el proyecto actual (layout, artefactos y estado generado),
no un schema DSL. El builder compartido `commands/describe.rs::describe_report`
alimenta prosa y JSON para que no puedan diverger.

## Plan de prueba

- `cargo test -p ag-cli` (170 unitarios; 9 nuevos en `describe_cli.rs`; integracion en verde).
- `cargo test --workspace --all-features`.
- `cargo fmt --all -- --check`.
- `cargo clippy -p ag-cli --all-targets --all-features -- -D warnings`.
- `bash scripts/ag-check`.
- `bash tools/check-file-size.sh` y `git diff --check`.
- Escaneo de contenido: sin secretos, sin atribucion a IA, sin emojis.

## Impacto en calidad (ratchet)

- Cobertura: sube (9 pruebas nuevas de contrato, snapshot, redaccion y read-only).
- Advertencias: igual (clippy -D warnings en verde).
- Deuda (TODO/FIXME): igual (no se anade ninguna).

## Criterios de salida que avanza

- `ag describe` describe el directorio actual sin mutar (prosa y `--output json`).
- Forma versionada documentada, esquematizada y con snapshot byte-exacto.
- `contract_version` presente; conjunto de campos cerrado; sin secretos ni rutas absolutas.
- Comandos mutantes siguen rechazando JSON con `unsupported_command` (exit 2).
- Documentacion EN+ES y CHANGELOG actualizados en el mismo cambio.

## Aprendizaje (opcional)

`status_report` de `deploy` ya normaliza el estado "no aplicado" como datos
neutrales; reutilizarlo evito inventar una segunda logica de sincronizacion y
mantubo `describe` util en directorios vacios.

## Cierre de issues

Closes: none

Refs RFC-0024 (fase 4.8-E). No se inventa numero de issue: no hay uno asignado
en el tablero disponible para esta unidad de trabajo (K-10).

## Checklist

- [x] Compila
- [x] Pasa tests
- [x] `scripts/kaizen check` en verde
- [x] Documentacion actualizada
- [x] Linea base no empeora
