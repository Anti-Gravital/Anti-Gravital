# PR: Configuracion TOML completa del Shield (Fase 1 PR 8 de 11)

## Resumen

ShieldConfig completo desde TOML con rechazo de claves desconocidas, carga desde archivo, ejemplo documentado y tests de round-trip de cada seccion.

## Fase afectada

Fase 1 (Shield MVP). PR 8 de los 11 incrementos previstos en
`docs/rfc/RFC-0002-diseno-shield-mvp.md`.

## Tipo de cambio

- [x] Documentacion
- [x] Codigo
- [ ] Infraestructura o CI
- [ ] RFC nueva o actualizacion de RFC
- [ ] ADR nuevo
- [ ] Seguridad

## Documentos relacionados

- RFC: `docs/rfc/RFC-0002-diseno-shield-mvp.md`, seccion 4.5.
- ADR: N/A (no introduce decisiones arquitectonicas nuevas; refina la
  superficie publica acordada en RFC-0002).
- Maestro afectado: N/A.

## Detalle del cambio

### Estricto al desconocido

Todas las structs de configuracion (`ShieldConfig`, `RuntimeConfig`,
`CorsConfig`, `CsrfConfig`, `RateLimitConfig`, `AuthConfig`,
`TlsConfig`) llevan `#[serde(deny_unknown_fields)]`. Una clave
tipeada o invalida en el TOML produce `AgError::Config` con el
nombre exacto del campo desconocido, en vez de ignorarse en silencio.
Defaults seguros ya existen via `Default`, asi que omitir secciones
sigue siendo aceptable.

### Carga desde archivo

`ShieldConfig::from_toml_str` permanece como el primitivo de bajo
nivel. Se agrega `ShieldConfig::from_path(path)` que lee el archivo
del filesystem y delega a `from_toml_str`, con errores de I/O
mapeados a `AgError::Config`.

### Ejemplo documentado

Se publica `crates/ag-core/config.example.toml` con todas las
secciones configurables, cada campo comentado con su default y su
significado. Sirve como referencia copy-paste para usuarios y como
fixture en tests.

### Round-trip por seccion

Tests unitarios nuevos que cubren cada seccion:

- Parseo desde TOML minimal (campos por defecto).
- Parseo desde TOML completo (todos los campos).
- Rechazo de claves desconocidas (deny_unknown_fields).
- Serializacion ShieldConfig -> TOML -> ShieldConfig produce el mismo
  valor (round-trip estable).
- Carga del fichero `config.example.toml` desde disco produce una
  configuracion valida.

## Plan de prueba

```sh
# Workspace completo.
cargo build --workspace
# Tests unitarios y E2E.
cargo test --workspace
# Tests especificos de config.
cargo test -p ag-core --lib config
# Fmt y clippy estricto.
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
# Politica de dependencias.
cargo deny check
# Documentacion sin warnings.
cargo doc --workspace --no-deps
# Carga manual del ejemplo (no falla).
cargo run --quiet -p ag-core --example load_config_example
```

## Criterios de salida que avanza

De `docs/roadmap/STATUS.md` Fase 1, esta PR marca:

- [x] Configuracion minima desde archivo TOML.

Sigue pendiente y se aborda en PRs posteriores:

- [ ] Tests unitarios con cobertura >= 80% del crate (PR 10).
- [ ] Benchmark Hello World (PR 9).
- [ ] Documentacion API en docs.rs y manual de usuario (PR 11).
- [ ] Metricas duras de cierre de Fase 1.

## Checklist

- [x] Titulo de PR de 256 caracteres o menos.
- [x] Sin emojis en ningun archivo modificado.
- [x] Sin atribuciones de herramientas IA.
- [x] Documentacion actualizada en el mismo PR (CHANGELOG, STATUS,
  config.example.toml).
- [x] CHANGELOG.md actualizado bajo `[Unreleased]`.
- [x] CLAUDE.md respetado: alcance limitado a Fase 1; sin nuevos
  crates; sin nuevas dependencias; sin `unsafe`; defaults seguros
  preservados.
- [x] Descriptor pre-rellenado existe en `docs/pr-drafts/`.
