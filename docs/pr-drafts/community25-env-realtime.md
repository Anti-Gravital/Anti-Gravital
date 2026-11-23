## Resumen
Migrate ag-realtime config to the declared RFC-0020 PR 4 EnvLoader schema with byte-identical defaults, mode/bool parsing and per-field fallbacks.

## Componente afectado
Fase 4.8-B, crate `ag-realtime` (configuracion), `ag-core` (dependency only). Cuarta crate del orden de migracion A2, despues de `ag-storage`, `ag-observe` y `ag-workers`.

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [ ] documentation
- [x] refactor

## Documentos relacionados
- `docs/rfc/RFC-0020-environment-and-secrets.md` seccion 5 PR 4 y 4.2 (ya aprobada; sin RFC nueva).
- `docs/audits/oportunidades-arquitectura.md` A2 (esquema declarado, tests de paridad) y A1 (carreras de `set_var`).
- `CLAUDE.md` reglas 16, 22, 28; `Arquitectura Tecnica` 5.3 regla 2 (realtime -> core).
- `CHANGELOG.md`, nota aditiva de implementation status en la RFC. Sin ADR nuevo.

## Decision de diseno

`realtime_env_declarations()` es la unica lista auditable de las 11 claves que `config.rs` lee: `NATS_MODE`, `NATS_URL`, `RT_BROADCAST_CAPACITY`, `NATS_TLS`, `NATS_TLS_CA`, `NATS_TLS_CERT`, `NATS_TLS_KEY`, `NATS_JETSTREAM`, `NATS_JS_STREAM`, `NATS_JS_MAX_MSGS` y `NATS_JS_MAX_BYTES`. coincide con el KPI del tablero "11 claves config declaradas". Todas se declaran como `EnvType::String`, igual que en las migraciones previas: el codigo original aplicaba `std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(DEFAULT)` *por campo*, de modo que una declaracion tipada (`Int`/`Bool`/`Path`) convertiria un valor malformado en un error de carga global y romperia ese fallback independiente. El parseo permanece en `RealtimeConfig::from_loaded_env`, identico al original byte por byte; el loader aporta la lista declarada, los defaults de las dos claves de texto, el aislamiento del entorno de proceso (solo lee claves declaradas) y errores sin valores.

Clasificacion de secretos: ninguna clave es secreta. `NATS_TLS_CERT` y `NATS_TLS_KEY` son **rutas** al certificado y a la clave privada del cliente, no los bytes de la credencial; por eso no se marcan `secret(true)`, igual que `STORAGE_JWT_PUBLIC_KEY_PATH` en `ag-storage`. Aun asi el loader nunca imprime valores en errores (S-1/S-8). Se descarto `EnvType::Path` para esas dos claves porque su validacion rechazaria una cadena vacia y aplicaria validacion de conjunto, cambiando el fallback previo (`var(k).ok()` devolvia `Some("")` y no abortaba la carga). los valores por defecto declarados solo estan en `NATS_URL` y `NATS_JS_STREAM`, las unicas claves de texto con default no vacio; los demas defaults (1024, false, 1_000_000, 1_073_741_824) siguen aplicandose campo por campo en la conversion.

`from_env()` conserva su firma publica y resuelve mediante `EnvLoader::load_process()` a traves del helper `resolve_process_env()`. Como todas las declaraciones son `String` opcionales con defaults validos, el unico error posible es un valor de proceso no-UTF-8; se replaya lenientemente para conservar el comportamiento previo (`var(k).ok()` ignoraba ese valor). La fuente inyectada `from_env_source()` es `#[cfg(test)]` para no ampliar la API publica. No se anade `PartialEq` a `NatsMode` ni se cambia ningun default, firma publica o semantica de las features `nats-external`/`event-persistence`.

**Alcance de external.rs.** Las tres lecturas de `NATS_URL` en `crates/ag-realtime/src/external.rs` (lineas 168, 232 y 268) estan dentro de `#[cfg(test)] mod tests` y son compuertas de integracion con un servidor NATS real (`nats_available()`). Se dejan **completamente intactas** y no se declara una segunda entrada `NATS_URL` para ellas, igual que se hicieron las compuertas S3/MinIO en `ag-storage`. La unica lectura de `NATS_URL` en el esquema es la de `config.rs`.

## Plan de prueba

- `cargo test -p ag-realtime` (exit 0; 41 tests de lib en verde, 1 test `ignored` de escalabilidad en `tests/load_50k.rs`, 1 doctest en verde).
- `cargo test -p ag-realtime --all-features` (exit 0; 60 tests de lib en verde, 1 ignored, 1 doctest; features `nats-external` y `event-persistence` compilan).
- `cargo test --workspace --all-features` (exit 0; 90 lineas `test result: ok`, 0 fallos).
- `cargo fmt --all -- --check` (exit 0 tras aplicar `cargo fmt --all`).
- `cargo clippy -p ag-realtime --all-targets --all-features -- -D warnings` (exit 0).
- `RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps` (exit 0; sin enlaces intra-doc a items privados desde documentacion publica: `realtime_env_declarations` se referencia con comillas invertidas en los docos del modulo).
- `bash scripts/ag-check` (verde, PASS=8/8 tras crear este descriptor; `audit` y `deny` en verde con la cache local).
- `bash scripts/kaizen check` (verde).
- `bash scripts/check-dependency-direction.sh` (verde, 19 crates, 21 aristas; la arista `ag-realtime -> ag-core` queda registrada; no existe entrada FORBIDDEN en conflicto). Prueba controlada y revertida: anadir `ag-auth` a `ag-mail` y `ag-mail` a `ag-realtime` produce 4 violaciones (forbidden direction, unlisted edge x2 y cycle); tras restaurar, el guard vuelve a PASS.
- `bash tools/check-file-size.sh` (OK: 320 archivos Rust rastreados, ninguno sobre 600 lineas; `config.rs` 240 y `config_tests.rs` 398 lineas segun `wc -l`); `git diff --check` (exit 0).
- Escaneo de contenido: sin secretos reales, sin atribucion a IA, sin emojis; sin `set_var`/`remove_var` nuevos en `config.rs` (el unico replay usa `std::env::var(...).ok()` como en las migraciones previas); sin supresiones S-10 anadidas.

## Impacto en calidad (ratchet)

- Cobertura: sube (27 tests en `config_tests.rs` frente a 14 inline originales: 14 convertidos a fuente inyectada y 13 nuevos de paridad/esquema).
- Advertencias: igual (`clippy -D warnings` y doc gate en verde).
- Deuda (TODO/FIXME): igual (no se anade ninguna).

## Criterios de salida que avanza

- Las 11 claves de `config.rs` quedan declaradas exactamente una vez; `NATS_MODE` solo acepta `"external"` exacto en minusculas; los flags booleanos solo aceptan `"true"` insensible a mayusculas; capacidad y limites hacen fallback por campo; las rutas TLS son opcionales y se preservan incluida la cadena vacia.
- Defaults, orden de resolucion (entorno de proceso sobre defaults), errores, API publica y features son byte-identicos; ningun valor secreto en logs, errores ni `Debug`.
- Las 3 lecturas `#[cfg(test)]` de `NATS_URL` en `external.rs` siguen intactas y no se declaran en el esquema.
- Guard de dependencias en verde con las direcciones prohibidas todavia detectadas; documentacion y CHANGELOG actualizados en el mismo cambio.

## Aprendizaje (opcional)

La compatibilidad estricta volvio a exigir `EnvType::String`: `EnvType::Path` habria validado no-vacio y convertido una ruta TLS vacia en error de carga global. El test de aislamiento del loader mostro que los defaults declarados se resuelven incluso cuando no se inyecta ninguna clave; la asercion se ajusto a comprobar que la clave no declarada se ignora y que solo `NATS_URL` y `NATS_JS_STREAM` se resuelven por default.

## Nota S-3 (hardcoding en fixtures de prueba)

Los valores `nats://myserver:4222`, `/synthetic/ca.pem`, `CUSTOM_EVENTS` y `synthetic-ignored` usados en `config_tests.rs` son entradas sinteticas del oraculo de paridad, no hosts reales, credenciales, IDs de cuenta ni umbrales operativos. No se suprime ni se silencia ninguna advertencia; se documenta segun S-3/S-10.

## Cierre de issues

Closes: none

Refs RFC-0020 PR 4 y audit A2. No hay numero de issue asignado a esta unidad en el tablero disponible (la tarea se llama `realtime-env-declarations`); K-10.

## Checklist

- [x] Compila
- [x] Pasa tests
- [x] `scripts/kaizen check` en verde
- [x] Documentacion actualizada
- [x] Linea base no empeora
