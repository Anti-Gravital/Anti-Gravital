## Resumen
Migrate ag-auth config to the declared RFC-0020 PR 4 EnvLoader schema with byte-identical defaults, JWT error mapping, OAuth Option semantics and secret classification (four secrets).

## Componente afectado
Fase 4.8-B, crate `ag-auth` (configuracion), `ag-core` (dependency only). Quinta crate del orden de migracion A2, despues de `ag-storage`, `ag-observe`, `ag-workers` y `ag-realtime`.

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [ ] documentation
- [x] refactor

## Documentos relacionados
- `docs/rfc/RFC-0020-environment-and-secrets.md` seccion 5 PR 4 y 4.2 (ya aprobada; sin RFC nueva).
- `docs/audits/oportunidades-arquitectura.md` A2 (esquema declarado, tests de paridad) y A1 (carreras de `set_var`).
- `CLAUDE.md` reglas 16, 22, 28; `Arquitectura Tecnica` 5.3 regla 2 (auth -> core).
- `CHANGELOG.md`, nota aditiva de implementation status en la RFC. Sin ADR nuevo.

## Decision de diseno

`auth_env_declarations()` es la unica lista auditable de las 8 claves que `config.rs` lee: `JWT_PRIVATE_KEY`, `JWT_PUBLIC_KEY`, `WEBAUTHN_RP_ID`, `WEBAUTHN_ORIGIN`, `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`. coincide con el KPI del tablero "8 claves config declaradas". Todas se declaran como `EnvType::String`, igual que en las migraciones previas: el codigo original aplicaba `std::env::var(k)` con un fallback *por campo* (unset, no-UTF-8), de modo que una declaracion tipada convertiria un valor en un error de carga global y romperia ese fallback independiente. El parseo permanece en `AuthConfig::from_loaded_env`, identico al original byte por byte; el loader aporta la lista declarada, los defaults declarados, la clasificacion de secretos, el aislamiento del entorno de proceso (solo lee claves declaradas) y errores sin valores.

Clasificacion de secretos: cuatro claves son secretas y se declaran `secret(true)`: `JWT_PRIVATE_KEY`, `JWT_PUBLIC_KEY`, `GOOGLE_CLIENT_SECRET`, `GITHUB_CLIENT_SECRET`. El loader las entrega como `Secret<String>` (Debug/Display redactan, `expose()` explicito); los bytes se copian a los campos publicos `String`/`Option<String>` de `AuthConfig` solo en `from_loaded_env` mediante `expose()`. No se cambia la API publica: `AuthConfig` conserva sus 8 campos y sus tipos. `WEBAUTHN_RP_ID`, `WEBAUTHN_ORIGIN`, `GOOGLE_CLIENT_ID` y `GITHUB_CLIENT_ID` no son secretos. Ningun valor secreto se imprime en errores, logs ni `Debug` del loader; el error `AuthConfigError::MissingVar` solo lleva el nombre de la variable.

`from_env()` conserva su firma publica `Result<Self, AuthConfigError>` y resuelve mediante `EnvLoader::load_process()`. Las dos claves JWT se declaran opcionales y sin default para que su ausencia siga produciendo `AuthConfigError::MissingVar` con el mismo mensaje que antes (`JWT_PRIVATE_KEY` primero, luego `JWT_PUBLIC_KEY`). Como todas las declaraciones son `String` opcionales con defaults validos, el unico error posible del loader es un valor de proceso no-UTF-8; se replaya lenientemente (`std::env::var(...).ok()` ignora el valor) para conservar el comportamiento previo. La fuente inyectada `from_env_source()` es `#[cfg(test)]` para no ampliar la API publica. No se cambia ningun default, firma publica ni semantica de las features `persistent`/`mail`.

## Plan de prueba

- `cargo test -p ag-auth` (exit 0; 58 tests de lib en verde, 2 doctests en verde).
- `cargo test -p ag-auth --all-features` (exit 0; 63 tests de lib en verde, 3 doctests; features `persistent` y `mail` compilan).
- `cargo test --workspace --all-features` (exit 0; 90 lineas `test result: ok`, 0 fallos).
- `cargo fmt --all -- --check` (exit 0 tras aplicar `cargo fmt --all`).
- `cargo clippy -p ag-auth --all-targets --all-features -- -D warnings` (exit 0).
- `RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps` (exit 0; sin enlaces intra-doc a items privados desde documentacion publica).
- `bash scripts/ag-check` (verde tras crear este descriptor).
- `bash scripts/kaizen check` (verde).
- `bash scripts/check-dependency-direction.sh` (verde, 19 crates, 22 aristas; la arista `ag-auth -> ag-core` queda registrada; no existe entrada FORBIDDEN en conflicto). Prueba controlada y revertida: anadir `ag-auth` a `ag-mail` y `ag-mail` a `ag-auth` produce violaciones (forbidden direction y cycle); tras restaurar, el guard vuelve a PASS.
- `bash tools/check-file-size.sh` (OK: 321 archivos Rust rastreados, ninguno sobre 600 lineas; `config.rs` 195 y `config_tests.rs` 293 lineas segun `wc -l`); `git diff --check` (exit 0).
- Escaneo de contenido: sin secretos reales, sin atribucion a IA, sin emojis; sin `set_var`/`remove_var` nuevos en `config.rs` (el unico replay usa `std::env::var(...).ok()` como en las migraciones previas); sin supresiones S-10 anadidas.

## Impacto en calidad (ratchet)

- Cobertura: sube (12 tests en `config_tests.rs` frente a 2 inline originales: 2 convertidos a fuente inyectada y 10 nuevos de paridad/esquema/redaccion).
- Advertencias: igual (`clippy -D warnings` y doc gate en verde).
- Deuda (TODO/FIXME): igual (no se anade ninguna).

## Criterios de salida que avanza

- Las 8 claves de `config.rs` quedan declaradas exactamente una vez; los 4 secretos marcados `secret(true)` y los demas no.
- Defaults byte-identicos (`WEBAUTHN_RP_ID` "localhost", `WEBAUTHN_ORIGIN` "http://localhost:8080"); valores vacios preservados; campos OAuth `Option` independientes; ausencia de JWT keys mapea a `AuthConfigError::MissingVar` con el mismo mensaje.
- Orden de resolucion, errores, API publica y features son byte-identicos; ningun valor secreto en logs, errores ni `Debug` del loader.
- Guard de dependencias en verde con las direcciones prohibidas todavia detectadas; documentacion y CHANGELOG actualizados en el mismo cambio.

## Aprendizaje (opcional)

La compatibilidad estricta volvio a exigir `EnvType::String`: una declaracion tipada habria convertido un valor no-UTF-8 o malformado en un error de carga global y roto el fallback por campo. Ademas, las claves JWT se declaran `required(false)` sin default: declararlas `required(true)` habria sustituido `AuthConfigError::MissingVar` por un `EnvValidationError` de `ag-core`, cambiando el tipo de error publico (prohibido en este PR).

## Nota S-3 (hardcoding en fixtures de prueba)

Los valores `synthetic-private`, `synthetic-public`, `synthetic-google-secret`, `synthetic-github-secret` y `synthetic-canary-private-key` usados en `config_tests.rs` son entradas sinteticas del oraculo de paridad, no credenciales, hosts ni IDs reales. No se suprime ni se silencia ninguna advertencia; se documenta segun S-3/S-10.

## Cierre de issues

Closes: none

Refs RFC-0020 PR 4 y audit A2. No hay numero de issue asignado a esta unidad en el tablero disponible; K-10.

## Checklist

- [x] Compila
- [x] Pasa tests
- [x] `scripts/kaizen check` en verde
- [x] Documentacion actualizada
- [x] Linea base no empeora
