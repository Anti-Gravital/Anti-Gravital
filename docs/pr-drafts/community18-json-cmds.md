## Resumen

Emite `config.get` en `--output json` (RFC-0021) reutilizando el executor de config: sin comandos nuevos, sin version bump y sin mutaciones permitidas.

## Componente afectado

`ag-cli` (rutas de despacho JSON, executor de configuracion y contrato `docs/mcp/`). Tablero `roadmap-avance`, tarea P1.

## Tipo de cambio

- [ ] bug
- [x] enhancement
- [ ] tech-debt
- [ ] documentation
- [ ] refactor

## Documentos relacionados

- RFC-0021 seccion 4.3 y seccion 4 regla 3 (contrato ya aceptado: "Every command accepts --output json"; anadir forma es aditivo pre-0.5, sin RFC nuevo).
- RFC-0024 seccion 9 prerrequisito 3; RFC-0018 (config); ADR-0008 (ingles canonico).
- `docs/mcp/cli-json-contract-0.2.0.md`, `docs/mcp/README.md`, `docs/mcp/snapshots/unsupported_command.json`.

## Plan de prueba

- `CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p ag-cli`
- `cargo test --workspace --all-features` (mismo CARGO_TARGET_DIR)
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `bash scripts/ag-check`
- `git diff --check`
- Escaneo de contenido: sin secretos, atribucion de IA, emojis ni marcas retiradas (ag-check `prohibited`).

## Evidencia (salida real)

- `cargo test -p ag-cli`: RC=0; 9 suites, 248 passes / 0 failures (incluye 30 pruebas de `json_output.rs`).
- `cargo test --workspace --all-features`: RC=0; todas las suites en verde, 0 failures.
- `cargo fmt --all -- --check`: RC=0. `cargo clippy --workspace --all-targets -- -D warnings`: RC=0.
- `bash scripts/ag-check`: `PASS=8 FAIL=0 SKIP=0`, "all selected checks passed".
- `git diff --check`: RC=0. Escaneo de contenido: sin secretos, atribucion de IA, emojis ni supresiones.
- CLI real: `config get bind` emite `"source":"file"`; `rate_limit.per_ip_rps` emite `"source":"default"`; `config set` con `--output json` devuelve exit 2 y no modifica `config.toml`; la forma prose sigue imprimiendo solo el valor.

## Impacto en calidad (ratchet)

- Cobertura: sube (nuevas pruebas unitarias y de integracion para config.get y el rechazo de config set).
- Advertencias: igual (clippy -D warnings limpio).
- Deuda (TODO/FIXME): igual (ninguno anadido).

## Criterios de salida que avanza

- `ag --output json config get KEY` emite el shape 0.2.0 ya publicado (key, kind, value, source file|default).
- `value` byte-identico a la linea prose; `config set` sigue rechazando con exit 2 y sin escribir.
- Mismo `contract_version` 0.2.0: no se introduce comando ni se cambia shape existente.
- Contrato, README, snapshot de rechazo y CHANGELOG actualizados en el mismo commit.

## Aprendizaje (opcional)

`config get` ya tenia shape, schema y snapshot publicados desde 0.2.0; solo faltaba el cableo de despacho. Un archivo `config.toml` ausente es `read_failed` tambien en prose: no se cambio ese comportamiento.

## Nota S-3 (hardcoding)

El gancho de commit senalo direcciones `127.0.0.1:7000/8080/9000` en
`cli.rs`, `config.rs`, `json_mode.rs`, `tests/json_output.rs` y
`json_output_support/mod.rs`. Todas son direcciones de loopback en datos de
prueba: fixtures deterministas de `config.toml` y el valor de la instantanea
publicada `config.get` (snapshot del contrato 0.2.0). Ninguna aparece en la
configuracion de produccion ni sustituye a una opcion validada al arrancar;
el unico valor por defecto real (`rate_limit.per_ip_rps`) sigue viniendo de
`ShieldConfig::default()` a traves de `default_value()`.

## Cierre de issues

Closes: none

## Checklist

- [x] Compila
- [x] Pasa tests
- [x] `scripts/ag-check` en verde
- [x] Documentacion actualizada
- [x] Linea base no empeora
