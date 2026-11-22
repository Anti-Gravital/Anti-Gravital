# Descriptor de PR

## Resumen

Registro de 33 templates, parser CLI y selector con catalogo, scaffolding generado en build y documentacion del contrato template.toml

## Fase afectada

Wave 6 (catalogo de templates). No avanza una fase de la Hoja de Ruta ni abre Phase 5.

## Tipo de cambio

- [ ] Correccion de bug
- [x] Documentacion (`templates/CATALOG.md`, `templates/README.md`, CHANGELOG)
- [x] Nueva feature (catalogo, wiring CLI/TUI, scaffolding data-driven)
- [ ] Cambio que rompe compatibilidad

## Contexto

La wave 6 define al menos 20 templates preconstruidos. Solo este rol puede
editar el parser de clap, el scaffolding y `templates/CATALOG.md`; los demas
roles anaden directorios `templates/<name>/` despues de que exista el esquema.
El scaffolding pasa a ser generico para que esos directorios sean utilizables
tras la fusion sin nuevas ediciones de codigo CLI. Quedan utilizables sin
editar el parser.

## Cambios

- `crates/ag-cli/src/catalog.rs`: fuente de verdad con 33 nombres, categoria,
  proposito y requisitos externos. `rest` es el primero y el default.
- `crates/ag-cli/src/cli.rs`: `value_parser` construido desde el catalogo;
  nombres desconocidos siguen siendo error de uso.
- `crates/ag-cli/build.rs` y `crates/ag-cli/src/scaffold.rs`: descubrimiento en
  tiempo de build de `templates/<name>/`, con sustitucion `{{name}}` y
  eliminacion del sufijo `.tmpl`. `template.toml` no se copia.
- `crates/ag-cli/src/commands/project.rs`: los cuatro templates originales usan
  el mismo scaffolding generico; sus archivos generados no cambian.
- `crates/ag-cli/src/tui/new_wizard/*` y `tui/text.rs`: ventana de seis
  entradas con posicion e indicios de desplazamiento, navegacion acotada,
  categoria y requisitos externos en la revision.
- `templates/CATALOG.md`: registro completo y contrato `template.toml`.
- `templates/README.md` y `CHANGELOG.md` actualizados.

## Documentos relacionados

- `templates/CATALOG.md` (este cambio)
- `docs/examples/alpha-prototype.md`, `CLAUDE.md` (regla 26 y 32)

## Plan de prueba

```sh
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0
cargo test -p ag-cli
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
cargo deny check
```

Evidencia: pruebas nuevas de parseo de los 33 nombres, default `rest`, error
de nombre desconocido, ventana del wizard y scaffolding de directorios
incorporados. Sin push ni fusion.

## Criterios de salida que avanza

- Catalogo documentado con 33 nombres, categorias, propositos e infraestructura
  externa.
- `--template` acepta todo nombre del catalogo; desconocido sigue siendo error.
- Paridad de flags y `ag new --no-interactive` sin cambios.
- Wizard legible y acotado con lista larga.

## Cierre de issues

Closes: none

## Checklist final

- [x] Pertenece a la fase correcta y respeta la documentacion.
- [x] No rompe arquitectura ni anade dependencias externas.
- [x] Compila, pasa tests, fmt y clippy segun evidencia en notas.
- [x] Documentacion actualizada junto al cambio.
- [x] Sin evidencia de herramientas IA; atribuido a la persona autora.
