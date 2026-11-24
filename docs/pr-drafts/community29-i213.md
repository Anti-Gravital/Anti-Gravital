## Resumen
Remove the crate-level `#![allow(dead_code)]` on the shared JSON snapshot test support module by splitting single-binary scenario fixtures into their own module; both test binaries compile with `-D warnings` and zero suppressions.

## Componente afectado
Solo codigo de prueba: `crates/ag-cli/tests/json_output_support/mod.rs` (reduce al surface compartido), `crates/ag-cli/tests/json_output_scenarios/mod.rs` (nuevo, fixtures usados solo por `json_output`), `crates/ag-cli/tests/json_output.rs` (incluye el modulo nuevo), `CHANGELOG.md` y este descriptor. Sin cambios bajo `crates/*/src/`, sin API publica, sin default ni feature nuevo.

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [x] tech-debt
- [ ] documentation
- [ ] refactor

## Documentos relacionados
- `docs/kaizen/pr-drafts/_TEMPLATE.md` (formato de descriptor).
- RFC-0021 seccion 4.3 (`--output json` snapshot tests, contrato byte-exacto).
- `.kaizen/security/SECURITY-RULES.md` S-10 (supresiones requieren referencia a issue): la supresion `#213` se elimina, no se reemplaza.
- `.kaizen/loop/LOOP.md` L-6 (reproducir antes de arreglar) y L-10 (anti-trampas).

## Analisis y decision de diseno

### Reproduccion (falla primero)
Con el `#![allow(dead_code)] // #213` eliminado y `cargo clippy -p ag-cli --tests -- -D warnings`, el binario de prueba `json_output_config` produce 9 errores `dead_code`: `SNAPSHOT_DIR`, `clean_schema`, `warning_schema`, `invalid_schema`, `reference_schema`, `current_schema`, `dns_manifest`, `workers_schema`, `docker_project` (todos definidos en `json_output_support/mod.rs`). Exit 101. El binario `json_output` si usa los 16 items; `json_output_config` solo usa 7.

### Por que no feature-gating por binario
Las features de Cargo son por paquete, no por destino de prueba. `required-features` en `[[test]]` solo evita compilar un objetivo cuando la feature no esta activa: no permite que `json_output` y `json_output_config` se compilen con features distintas en una misma invocacion `cargo test -p ag-cli`. Activar ambas features las unifica y el dead_code reaparece; ademas los tests dejarian de compilarse en la invocacion por defecto. Descartado.

### Fix aplicado (split de modulos)
- `json_output_support/mod.rs` conserva SOLO los 7 helpers usados por ambos binarios: `snapshot`, `scratch_dir`, `write`, `ag`, `stdout`, `assert_snapshot`, `config_toml`.
- `json_output_scenarios/mod.rs` (nuevo) contiene los 9 helpers exclusivos de `json_output`; `docker_project` importa `scratch_dir`/`write` desde `crate::json_output_support`. Se incluye con `mod json_output_scenarios;` unicamente en `json_output.rs`.
- Se elimina la supresion `#![allow(dead_code)] // #213` y su comentario de deuda. Los `include_str!` de snapshots no se mueven: `snapshot()` permanece en `json_output_support`, por lo que las rutas relativas siguen validas.

Ningun test se debilita: se conservan 21 tests en `json_output.rs` y 9 en `json_output_config.rs`, con las mismas comparaciones byte-exactas.

## Plan de prueba

- `cargo clippy -p ag-cli --tests -- -D warnings` (exit 0; antes del fix exit 101 con 9 dead_code).
- `cargo test -p ag-cli` (50 tests: 21 + 9 JSON, mas los del crate; 0 fallos).
- `cargo check --workspace --all-features` (exit 0).
- `cargo clippy --workspace --all-targets -- -D warnings` (exit 0; gate de `ag-check`).
- `bash scripts/ag-check` (verde; descriptor duplicado en `docs/pr-drafts/` para este gate).
- `bash tools/check-file-size.sh` (exit 0; ambos modulos muy por debajo de 600 lineas).
- `git diff --check` (sin whitespace errors).
- Escaneo de contenido: 0 `allow(dead_code)`, 0 `#213` en fuentes `.rs`, sin secretos, sin atribucion a IA, sin emojis, sin supresiones S-10 nuevas.

## Impacto en calidad (ratchet)

- Cobertura: igual (mismos tests, solo se reubican helpers).
- Advertencias: baja (se elimina 1 supresion; ambos binarios compilan con `-D warnings`).
- Deuda (TODO/FIXME): igual; disminuye la deuda de supresiones S-10 en 1.

## Criterios de salida que avanza

- 0 supresiones `#![allow(dead_code)]` en `crates/ag-cli/tests/`.
- Ambos binarios de prueba compilan con `-D warnings` y la suite completa pasa.
- Issue #213 queda con una correccion real verificada (no sin cerrar en falso).

## Aprendizaje (opcional)
Cuando un modulo de soporte es compartido por varios binarios de prueba, la division correcta es por superficie de uso (helpers comunes vs. especificos de un binario), no por features de Cargo: las features son por paquete y se unifican entre destinos de prueba, por lo que no pueden dar visibilidad distinta por binario en una sola invocacion.

## Cierre de issues
Closes #213

## Checklist
- [x] Compila
- [x] Pasa tests
- [x] `scripts/ag-check` en verde
- [x] Documentacion y CHANGELOG actualizados
- [x] Linea base no empeora (0 supresiones nuevas)
