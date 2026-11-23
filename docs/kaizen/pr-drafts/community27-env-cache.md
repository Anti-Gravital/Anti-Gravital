## Resumen
Migrate ag-cache config to the declared RFC-0020 PR 4 EnvLoader schema with byte-identical defaults, per-field fallback and secret classification for REDIS_URL.

## Componente afectado
Fase 4.8-B, crate `ag-cache` (configuracion). `ag-core` ya es dependencia y la arista `ag-cache -> ag-core` ya esta permitida; no se toca el guard. Sexta crate del orden de migracion A2, despues de `ag-storage`, `ag-observe`, `ag-workers`, `ag-realtime` y `ag-auth`.

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [ ] documentation
- [x] refactor

## Documentos relacionados
- `docs/rfc/RFC-0020-environment-and-secrets.md` seccion 5 PR 4 y 4.2/4.3 (ya aprobada; sin RFC nueva).
- `docs/audits/oportunidades-arquitectura.md` A2 (esquema declarado, tests de paridad) y A1 (carreras de `set_var`).
- `Arquitectura Tecnica` 5.3 regla 2 (`ag-cache -> ag-core`, ya registrada en `scripts/check-dependency-direction.sh:67`).
- `CHANGELOG.md` y nota aditiva de implementation status en la RFC. Sin ADR nuevo, sin cambio de API publica ni de defaults.

## Decision de diseno

`cache_env_declarations()` es la unica lista auditable de las 6 claves que `config.rs` lee: `CACHE_L1_MAX_CAPACITY`, `CACHE_L1_TTL_SECS`, `REDIS_URL`, `REDIS_POOL_SIZE`, `CACHE_NATIVE_SERVER`, `CACHE_NATIVE_PORT`. coincide con el KPI del tablero "6 claves config declaradas". Todas se declaran como `EnvType::String` y `required(false)`: el codigo original aplicaba `std::env::var(k)` con un fallback *por campo* (unset, no-UTF-8 o malformado), de modo que una declaracion tipada (`Int`/`Bool`) convertiria un valor malformado en un error global de carga y romperia ese fallback independiente. El parseo permanece en `CacheConfig::from_loaded_env`, identico al original byte por byte (sin `trim`, `to_lowercase()` Unicode para el flag); el loader aporta la lista declarada, la clasificacion de secretos, el aislamiento del entorno de proceso (solo lee claves declaradas) y errores sin valores.

Clasificacion de secretos: `REDIS_URL` se declara `secret(true)` porque una URL de Redis puede incrustar credenciales `user:pass@host` (S-1/S-8). El loader la entrega como `Secret<String>` (Debug/Display redactan, `expose()` explicito); los bytes se copian al campo publico `Option<String>` de `CacheConfig` solo en `from_loaded_env` mediante `expose()`. No se cambia la API publica: `CacheConfig` conserva sus 6 campos y sus tipos. Las otras cinco claves son knobs operativos (tamanos, un flag y un puerto) y no son secretos. El valor del secreto nunca se imprime en errores del loader ni en su estado `Debug`; un test canario lo demuestra.

`from_env()` conserva su firma publica `Self` y resuelve mediante `EnvLoader::load_process()`. Como todas las declaraciones son `String` opcionales sin default, el unico error posible del loader es un valor de proceso no-UTF-8; se replaya lenientemente (`std::env::var(...).ok()` ignora el valor) para conservar el comportamiento previo por campo. La fuente inyectada `from_env_source()` es `#[cfg(test)]` para no ampliar la API publica y elimina las carreras de `set_var`/`remove_var` (audit A1). No se cambia ningun default (10_000, 300, 10, false, 6379, None), dimension de pool, semantica del flag ni las features `native-server`/`redis-l2`.

## Plan de prueba

- `cargo test -p ag-cache` (exit 0; 34 tests de lib en verde, 1 ignored, 2 doctests en verde; antes 24, ahora 10 mas de config).
- `cargo test -p ag-cache --all-features` (exit 0; features `native-server` y `redis-l2` compilan y pasan).
- `cargo test --workspace --all-features` (exit 0; todas las suites `test result: ok`, 0 fallos).
- `cargo fmt --all -- --check` (exit 0 tras aplicar `cargo fmt --all`).
- `cargo clippy -p ag-cache --all-targets --all-features -- -D warnings` (exit 0).
- `RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps` (exit 0; se corrigio un enlace intra-doc privado en el doc del modulo).
- `bash scripts/ag-check` (verde tras crear este descriptor).
- `bash scripts/kaizen check` (verde).
- `bash scripts/check-dependency-direction.sh` (verde, sin cambios: la arista `ag-cache -> ag-core` ya estaba en la allow-list; las direcciones prohibidas siguen detectandose).
- `bash tools/check-file-size.sh` (OK; `config.rs` 164 lineas y `config_tests.rs` 275, ambos bajo 600); `git diff --check` (exit 0).
- Escaneo de contenido: sin secretos reales, sin atribucion a IA, sin emojis; sin `set_var`/`remove_var` nuevos en `config.rs` (el unico replay usa `std::env::var(...).ok()` como en las migraciones previas); sin supresiones S-10 anadidas.

## Impacto en calidad (ratchet)

- Cobertura: sube (13 tests de config frente a 3 inline originales: 2 conservados, 1 convertido a fuente inyectada y 10 nuevos de paridad/esquema/redaccion).
- Advertencias: igual (`clippy -D warnings` y doc gate en verde).
- Deuda (TODO/FIXME): igual (no se anade ninguna).

## Criterios de salida que avanza

- Las 6 claves de `config.rs` quedan declaradas exactamente una vez; `REDIS_URL` marcado `secret(true)` y los otros cinco no.
- Defaults byte-identicos (10_000, 300, 10, false, 6379, None); valores vacios preservados; bool solo "1"/"true" case-insensitive sin `trim`; numeros sin `trim`.
- Orden de resolucion, errores, API publica y features son byte-identicos; ningun valor secreto en errores ni `Debug` del loader.
- Guard de dependencias en verde, sin tocar el guard; documentacion y CHANGELOG actualizados en el mismo cambio.

## Aprendizaje (opcional)

La compatibilidad estricta volvio a exigir `EnvType::String`: una declaracion `Bool` habria aceptado "yes"/"0" y aplicado `trim`, y una `Int` habria convertido un valor malformado en un error de carga global; ambas cosas cambian el comportamiento observado. El doc gate `RUSTDOCFLAGS="-D warnings"` exige no enlazar items `pub(crate)` desde el doc publico del modulo: se uso codigo inline en lugar de un intra-doc link.

## Nota S-3 (hardcoding en fixtures de prueba)

Los valores `synthetic-canary`, `cache.example.invalid` y demas usados en `config_tests.rs` son entradas sinteticas del oraculo de paridad, no credenciales, hosts ni IDs reales. No se suprime ni se silencia ninguna advertencia; se documenta segun S-3/S-10.

## Cierre de issues

Closes: none

Refs RFC-0020 PR 4 y audit A2. No hay numero de issue asignado a esta unidad en el tablero disponible; K-10.

## Checklist

- [x] Compila
- [x] Pasa tests
- [x] `scripts/ag-check` en verde
- [x] `scripts/kaizen check` en verde
- [x] Documentacion actualizada
- [x] Linea base no empeora
