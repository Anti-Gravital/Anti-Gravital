# PR: Documentacion API y capitulo del manual de Shield (Fase 1 PR 11 de 11)

## Resumen

Rustdoc enriquecido en `ag-core` con ejemplos compilados y capitulo del manual de usuario sobre Shield como libreria. Cierra el ultimo entregable de Fase 1.

## Fase afectada

Fase 1 (Shield MVP). PR 11 y ultimo de los 11 incrementos previstos en
`docs/rfc/RFC-0002-diseno-shield-mvp.md`.

## Tipo de cambio

- [x] Documentacion
- [ ] Codigo
- [ ] Infraestructura o CI
- [ ] RFC nueva o actualizacion de RFC
- [ ] ADR nuevo
- [ ] Seguridad

## Documentos relacionados

- RFC: `docs/rfc/RFC-0002-diseno-shield-mvp.md`.
- ADR: N/A.
- Maestro afectado: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
  seccion 6 (arquitectura del nucleo). El capitulo del manual extiende
  esa seccion con orientacion practica para desarrolladores.

## Detalle del cambio

### Rustdoc enriquecido en `ag-core`

- Documentacion crate-level en `crates/ag-core/src/lib.rs` con un
  recorrido por las capas Shield, los tipos publicos clave
  (`Shield`, `ShieldConfig`, `AgError`, `Claims<T>`,
  `ValidatedJson<T>`) y un ejemplo `no_run` completo de uso.
- Documentacion module-level para cada submodulo (`config`, `error`,
  `runtime`, `shield`, `core`) con que ofrece y donde mirar primero.
- Cobertura completa de items publicos: el lint `missing_docs` ya
  obliga, este PR llena los huecos de pulido (resumenes mas claros,
  enlaces cruzados, ejemplos integrados).

### Capitulo del manual: Shield como libreria

`docs/manual/01-shield-as-library.md` cubre, en orden:

- Que es la Shield y donde encaja en el ecosistema.
- Como anadir `ag-core` al `Cargo.toml`.
- Como construir y servir un router minimo con `Shield::with_defaults`.
- Como activar y configurar cada capa (TLS, JWT, CSRF, CORS,
  rate-limit, validation, logging).
- Como escribir handlers con los extractores `Claims<T>` y
  `ValidatedJson<T>`.
- Como cargar la configuracion desde TOML con `from_path`.
- Consideraciones de despliegue (defaults seguros, ConnectInfo bajo
  TLS, integracion con balanceadores externos).
- Que no es la Shield y que se aborda en otras fases (DSL en Fase 3,
  Core completo en Fase 2, observabilidad avanzada en Fase 4).

El indice del manual vive en `docs/manual/README.md`.

## Plan de prueba

```sh
# Documentacion sin warnings (incluye examples compilando en doctests).
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Doctests del crate (los ejemplos del rustdoc se compilan).
cargo test --workspace --doc

# Suite completa sigue verde.
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
```

## Criterios de salida que avanza

De `docs/roadmap/STATUS.md` Fase 1, esta PR marca:

- [x] Documentacion API generada con `cargo doc`.
- [x] Capitulo del manual de usuario sobre uso de Shield como
  libreria.

Cierra los entregables de Fase 1 entregables-en-repo (1.2). Quedan
fuera de este PR y a cargo del operador:

- [ ] Benchmark Hello World >= 300K req/s en hardware de referencia.
- [ ] Latencia p99 <= 1 ms a 100K req/s.
- [ ] Memoria idle <= 15 MB.
- [ ] Arranque <= 100 ms.
- [ ] Blog post tecnico sobre la arquitectura de Shield.
- [ ] >= 10 stars en el repositorio.
- [ ] Publicacion del crate en docs.rs (requiere `cargo publish` desde
  el mantenedor con cuenta autorizada; este PR deja el crate listo).

## Checklist

- [x] Titulo de PR de 256 caracteres o menos.
- [x] Sin emojis en ningun archivo modificado.
- [x] Sin atribuciones de herramientas IA.
- [x] Documentacion actualizada en el mismo PR (rustdoc, manual,
  CHANGELOG, STATUS).
- [x] CHANGELOG.md actualizado bajo `[Unreleased]`.
- [x] CLAUDE.md respetado: alcance limitado a documentacion; sin
  cambios de codigo funcional; sin nuevas dependencias.
- [x] Descriptor pre-rellenado existe en `docs/pr-drafts/`.
