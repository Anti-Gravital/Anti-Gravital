## Resumen

Migrate ag-storage config loading to the declared RFC-0020 PR 4 EnvLoader schema with byte-identical defaults.

## Componente afectado

Fase 4.8-B, crate `ag-storage` (configuracion), `ag-core` (dependency only). Primera crate del orden de migracion A2.

## Tipo de cambio

- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [ ] documentation
- [x] refactor

## Documentos relacionados

- `docs/rfc/RFC-0020-environment-and-secrets.md` seccion 5 PR 4 y 4.2 (ya aprobada; sin RFC nueva).
- `docs/audits/oportunidades-arquitectura.md` A2 (esquema declarado, tests de paridad) y A1 (carreras de `set_var`).
- `CLAUDE.md` reglas 16, 22, 28; `ADR-0009` regla 2 (backend nativo por defecto, externos opt-in).
- `CHANGELOG.md`, nota aditiva de implementation status en la RFC. Sin ADR nuevo.

## Decision de diseno

`storage_env_declarations()` es la unica lista auditable de las 17 claves. Todas se declaran como `EnvType::String` a proposito: el codigo previo aplicaba `var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(DEFAULT)` por campo, de modo que un valor ausente o malformado caida al default de ese campo de forma independiente. `EnvLoader` valida el conjunto completo y aborta la carga si un solo valor falla su tipo, asi que una declaracion tipada (`Int`/`Bool`) cambiaria esa semantica (p. ej. `STORAGE_PORT=abc` con `STORAGE_ROOT` valido perderia el root). El parseo permanece en `from_loaded_env`, identico al original; el loader aporta la lista declarada, defaults por campo, redaccion de secretos y aislamiento del entorno de proceso. Cuatro claves son `secret(true)` y se leen solo con `expose()`. `from_env()` conserva su firma y usa `load_process()`; la fuente inyectada es `#[cfg(test)]` para no ampliar la API publica.

## Plan de prueba

- `cargo test -p ag-storage` (79 passed; 74 antes).
- `cargo test -p ag-storage --all-features` (87 passed).
- `cargo test --workspace --all-features` (exit 0; 90 lineas `test result: ok`).
- `cargo fmt --all -- --check`.
- `cargo clippy -p ag-storage --all-targets --all-features -- -D warnings`.
- `bash scripts/kaizen check` (todo en verde, exit 0).
- `bash scripts/ag-check` (verde, PASS=8/8; una primera ejecucion tuvo errores
  transitorios de descarga de la base de avisos, resueltos al calentar la cache
  local; `audit` y `deny` tambien se verificaron con `cargo audit --no-fetch` y
  `cargo deny --offline check`). `bash tools/check-file-size.sh`; `git diff --check`.
- Escaneo de contenido: sin secretos reales, sin atribucion a IA, sin emojis.

## Impacto en calidad (ratchet)

- Cobertura: sube (9 pruebas nuevas de paridad, esquema y redaccion; se eliminan 3 pruebas con `set_var`).
- Advertencias: igual (clippy -D warnings en verde).
- Deuda (TODO/FIXME): igual (no se anade ninguna).

## Criterios de salida que avanza

- Los 17 defaults y el orden de resolucion (entorno de proceso, luego default) son byte-identicos.
- Ningun valor secreto aparece en logs, errores ni `Debug` del entorno cargado.
- Backend nativo sigue siendo el default; S3/MinIO siguen detras de la feature `s3`.
- Sin `std::env::set_var` en los tests de la crate; sin cambio de API publica.
- Documentacion y CHANGELOG actualizados en el mismo cambio.

## Aprendizaje (opcional)

La compatibilidad estricta obligo a declarar las claves como `String`: un declaration tipada habria convertido un valor malformado en un error de carga global, rompiendo el fallback por campo. La paridad se demuestra con un oraculo que replica el codigo anterior leyendo un mapa inyectado.

## Cierre de issues

Closes: none

Refs RFC-0020 PR 4 y audit A2. No hay numero de issue asignado a esta unidad en el tablero disponible (K-10).

## Checklist

- [x] Compila
- [x] Pasa tests
- [x] `scripts/kaizen check` en verde
- [x] Documentacion actualizada
- [x] Linea base no empeora
