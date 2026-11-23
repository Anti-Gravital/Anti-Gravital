## Resumen
Migrate ag-workers config to the declared RFC-0020 PR 4 EnvLoader schema with byte-identical defaults, validation and per-field fallbacks.

## Componente afectado
Fase 4.8-B, crate `ag-workers` (configuracion), `ag-core` (dependency only). Tercera crate del orden de migracion A2, despues de `ag-storage` y `ag-observe`.

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [ ] documentation
- [x] refactor

## Documentos relacionados
- `docs/rfc/RFC-0020-environment-and-secrets.md` seccion 5 PR 4 y 4.2 (ya aprobada; sin RFC nueva).
- `docs/audits/oportunidades-arquitectura.md` A2 (esquema declarado, tests de paridad) y A1 (carreras de `set_var`).
- `CLAUDE.md` reglas 16, 22, 28; `Arquitectura Tecnica` 5.3 regla 2 (workers -> core).
- `CHANGELOG.md`, nota aditiva de implementation status en la RFC. Sin ADR nuevo.

## Decision de diseno

`workers_env_declarations()` es la unica lista auditable de las claves fijas del crate. Todas se declaran como `EnvType::String`, igual que en las migraciones de `ag-storage` y `ag-observe`: el codigo previo leia `std::env::var(k)` y aplicaba el parseo *por campo* (los numeros con `env_parse` caian al valor de TOML/default si el valor estaba ausente o era malformado; `mode`, `backend` y `shutdown timeout` propagaban un error tipado). `EnvLoader` valida el conjunto completo y aborta la carga si un valor falla su tipo declarado, asi que una declaracion tipada (`Int`/`Duration`) cambiaria ese fallback por campo. El parseo permanece en `WorkersConfig::apply_from_loaded_env`, identico al original. Ninguna clave es secreta: el loader nunca imprime valores en errores (S-1/S-8).

`apply_env()` conserva su firma publica y usa `EnvLoader::load_process()`. La fuente inyectada `apply_env_source()` es `#[cfg(test)]` para no ampliar la API publica. Como todas las declaraciones son `String` opcionales sin default, el unico error posible de `load_process()` es un valor no-UTF-8; se replaya lenientemente para conservar el fallback previo (`var(k).ok()` ignoraba ese valor). El unico `std::env::var` que queda es ese replay, identico al ya aceptado en la migracion de `ag-observe`.

**Nota de alcance (KPI del tablero).** El tablero `rfc0020-pr4-workers` fija el KPI "3 claves declaradas" (`AG_WORKERS_MODE`, `AG_WORKERS_BACKEND`, `AG_WORKERS_SHUTDOWN_TIMEOUT`, lecturas directas en config.rs ~269-275). Ademas de esas tres se declaran `AG_WORKERS_ENABLED`, `AG_WORKERS_MAX_PAYLOAD_BYTES` y `AG_WORKERS_POISON_GUARD_ATTEMPTS`, mas las claves dinamicas por cola `AG_WORKERS_QUEUE_<NAME>_MIN`/`_MAX`: el codigo previo las leia mediante los helpers `env_bool`/`env_parse`, que llamaban `std::env::var` directamente. Declararlas todas completa el objetivo del tablero ("migrar ag-workers de std::env::var directo a EnvLoader declarado"), no deja lecturas directas residuales y mantiene la semantica byte-identica; las tres claves del KPI quedan cubiertas. Los nombres `from_env()` y `read_env`/`read_env_usize` mencionados en la mission no existen en esta crate: la API publica real es `from_toml_str`/`apply_env`/`load`/`runtime_config` y se conserva sin cambios. No se toca la feature `postgres` ni el dep opcional `ag-data`.

## Plan de prueba

- `cargo test -p ag-workers` (exit 0; 88 tests unitarios de lib + integracion y doctest en verde).
- `cargo test -p ag-workers --all-features` (exit 0; 89 tests de lib; feature `postgres` compila con dependencias en cache).
- `cargo test --workspace --all-features` (exit 0; 90 lineas `test result: ok`, sin fallos).
- `cargo fmt --all -- --check` (exit 0 tras aplicar `cargo fmt --all`).
- `cargo clippy -p ag-workers --all-targets --all-features -- -D warnings` (exit 0).
- `bash scripts/kaizen check` (verde tras crear este descriptor).
- `bash scripts/ag-check` (verde, PASS=8/8). Una ejecucion intermedia fallo solo en `audit`/`deny` porque no pudieron descargar la base de datos de avisos sin red; se verifico offline con `cargo audit --no-fetch` y `cargo deny --offline check` (ambos exit 0, 4 avisos permitidos) y, tras calentar la cache local, `ag-check` quedo en verde completo.
- `bash scripts/check-dependency-direction.sh` (verde, 19 crates, 20 aristas; la arista `ag-workers -> ag-core` queda registrada). Prueba controlada y revertida: anadir `ag-mail` a ag-workers produce 3 violaciones (forbidden direction, unlisted edge y cycle), luego se restaura y el guard vuelve a PASS.
- `bash tools/check-file-size.sh` (OK: 319 archivos Rust, ninguno sobre 600 lineas); `git diff --check` (exit 0).
- Escaneo de contenido: sin secretos reales, sin atribucion a IA, sin emojis; sin `set_var`/`remove_var` reales en `crates/ag-workers/src/` (solo menciones en comentarios); sin supresiones S-10 anadidas.

## Impacto en calidad (ratchet)

- Cobertura: sube (12 pruebas de paridad/esquema nuevas; la unica prueba con `set_var` se convierte a fuente inyectada).
- Advertencias: igual (`clippy -D warnings` en verde).
- Deuda (TODO/FIXME): igual (no se anade ninguna).

## Criterios de salida que avanza

- Los defaults, el orden de resolucion (entorno de proceso sobre TOML sobre defaults) y la validacion son byte-identicos.
- `mode`/`backend` solo aceptan las variantes conocidas y propagan `ConfigError::Variant`; `shutdown_timeout` usa `parse_duration` y propaga `ConfigError::Duration`.
- `AG_WORKERS_ENABLED` solo acepta `1`/`true`/`yes` (insensible a mayusculas); los campos numericos hacen fallback por campo y recortan espacios; las claves por cola se aplican solo a colas declaradas.
- Sin valor secreto en logs, errores ni `Debug`; sin cambio de API publica ni de default; feature `postgres` intacta.
- Guard de dependencias en verde, con las direcciones prohibidas todavia detectadas; documentacion y CHANGELOG actualizados en el mismo cambio.

## Aprendizaje (opcional)

La compatibilidad estricta obligo de nuevo a declarar las claves como `String`: una declaracion tipada habria convertido un valor malformado en error de carga global, rompiendo el fallback por campo. El caso de las claves por cola muestra que un esquema declarado puede construirse dinamicamente a partir de las colas presentes en la configuracion sin leer el entorno de forma directa.

## Nota S-3 (hardcoding en fixtures de prueba)

Los valores `standalone`, `postgres`, `45s`, `999`, `5`, `mail`, `media` y `synthetic-value` usados en `config_tests.rs` son entradas sinteticas del oraculo de paridad, no hosts reales, credenciales, IDs de cuenta ni umbrales operativos. No se suprime ni se silencia ninguna advertencia; se documenta segun S-3/S-10.

## Cierre de issues

Closes: none

Refs RFC-0020 PR 4 y audit A2. No hay numero de issue asignado a esta unidad en el tablero disponible (la tarea se llama `workers-env-declarations`); K-10.

## Checklist

- [x] Compila
- [x] Pasa tests
- [x] `scripts/kaizen check` en verde
- [x] Documentacion actualizada
- [x] Linea base no empeora
