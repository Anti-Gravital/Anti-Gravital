## Resumen

Migrate ag-observe config loading to the declared RFC-0020 PR 4 EnvLoader schema with byte-identical defaults.

## Componente afectado

Fase 4.8-B, crate `ag-observe` (configuracion), `ag-core` (dependency only). Segunda crate del orden de migracion A2, despues de `ag-storage`.

## Tipo de cambio

- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [ ] documentation
- [x] refactor

## Documentos relacionados

- `docs/rfc/RFC-0020-environment-and-secrets.md` seccion 5 PR 4 y 4.2 (ya aprobada; sin RFC nueva).
- `docs/audits/oportunidades-arquitectura.md` A2 (esquema declarado, tests de paridad) y A1 (carreras de `set_var`).
- `CLAUDE.md` reglas 16, 22, 28; `Arquitectura Tecnica` 5.3 regla 2 (observabilidad -> core).
- `CHANGELOG.md`, nota aditiva de implementation status en la RFC. Sin ADR nuevo.

## Decision de diseno

`observe_env_declarations()` es la unica lista auditable de las seis claves. Todas se declaran como `EnvType::String`, igual que en la migracion de `ag-storage`: el codigo previo aplicaba `var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(DEFAULT)` por campo, de modo que un valor ausente o malformado caida al default de ese campo de forma independiente. `EnvLoader` valida el conjunto completo y aborta la carga si un solo valor falla su tipo, asi que una declaracion tipada (`Int`/`Url`) cambiaria esa semantica. El parseo permanece en `from_loaded_env`, identico al original. Ninguna clave es secreta: `AG_PROJECT_ID` es una etiqueta opaca y acotada, y el loader nunca imprime valores en errores. `from_env()` conserva su firma y usa `load_process()`; la fuente inyectada es `#[cfg(test)]` para no ampliar la API publica.

`SERVICE_NAME` se declara con su default `ag-service` para completar el inventario, pero su lectura sigue directa en `layer::build_resource`: esa funcion esta detras de la feature `otlp` y se ejecuta durante `init()`, no en la ruta `from_env`. Anadir un campo publico a `ObserveConfig` seria un cambio de API publica vedado por este PR. El fallback se mantiene byte-identico.

## Plan de prueba

- `cargo test -p ag-observe` (30 passed).
- `cargo test -p ag-observe --all-features` (32 passed).
- `cargo test --workspace --all-features` (exit 0; 90 lineas `test result: ok`).
- `cargo fmt --all -- --check`.
- `cargo clippy -p ag-observe --all-targets --all-features -- -D warnings`.
- `bash scripts/kaizen check` (verde, exit 0).
- `bash scripts/ag-check` (verde, PASS=8/8).
- `bash scripts/check-dependency-direction.sh` (verde, 19 aristas; la arista `ag-observe -> ag-core` queda registrada y las direcciones prohibidas siguen detectandose).
- `bash tools/check-file-size.sh`; `git diff --check`.
- Escaneo de contenido: sin secretos reales, sin atribucion a IA, sin emojis, sin `set_var` en tests de la crate.

## Impacto en calidad (ratchet)

- Cobertura: sube (12 pruebas de paridad, esquema y default; se elimina 1 prueba con `set_var`).
- Advertencias: igual (clippy -D warnings en verde).
- Deuda (TODO/FIXME): igual (no se anade ninguna).

## Criterios de salida que avanza

- Los defaults y el orden de resolucion (entorno de proceso, luego default) son byte-identicos.
- `LOG_FORMAT` solo acepta `json` en minusculas; `AG_DEV_CONSOLE` solo `true`/`1`; `AG_PROJECT_ID` conserva incluso la cadena vacia y su validacion sigue en `init`.
- El fallback `ag-service` de `SERVICE_NAME` no cambia y queda documentado en esquema y codigo.
- Sin valor secreto en logs, errores ni `Debug`; sin cambio de API publica ni de default.
- Guard de dependencias en verde y documentacion/CHANGELOG actualizados en el mismo cambio.

## Aprendizaje (opcional)

La compatibilidad estricta obligo de nuevo a declarar las claves como `String`: una declaracion tipada habria convertido un valor malformado en error de carga global, rompiendo el fallback por campo. El caso `SERVICE_NAME` muestra un limite real del patron: una variable leida en una ruta feature-gated y fuera de la configuracion no puede migrarse sin ampliar la API publica, asi que se documenta y se deja intacta.

## Nota S-3 (hardcoding en fixtures de prueba)

El hook de seguridad senala `config_tests.rs:91`: el valor
`http://127.0.0.1:4318/v1/traces` es un endpoint sintetico de loopback usado
solo como entrada del oraculo de paridad, igual que el que ya usan los tests de
`layer.rs`. `9000` es un puerto de fixture, `demo-project.1` una etiqueta
sintetica y `synthetic-*` valores de ejemplo. No hay hosts reales,
credenciales, IDs de cuenta ni umbrales operativos. No se suprime ni se
silencia la advertencia; se documenta aqui segun S-3/S-10.

## Cierre de issues

Closes: none

Refs RFC-0020 PR 4 y audit A2. No hay numero de issue asignado a esta unidad en el tablero disponible (K-10).

## Checklist

- [x] Compila
- [x] Pasa tests
- [x] `scripts/kaizen check` en verde
- [x] Documentacion actualizada
- [x] Linea base no empeora
