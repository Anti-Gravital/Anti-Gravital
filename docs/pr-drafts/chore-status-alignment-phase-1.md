# Chore: alineacion de docs/roadmap/STATUS.md con el estado real de Fase 1

## Resumen

Marca como completas en STATUS.md las casillas de Fase 1 que la documentacion arrastraba como pendientes pese a estar entregadas y publicadas en main por PRs ya merged.

## Fase afectada

Fase 1 (Shield MVP). Limpieza documental tras el cierre tecnico de los
11 PRs del RFC-0002.

## Tipo de cambio

- [x] Documentacion
- [ ] Codigo
- [ ] Infraestructura o CI
- [ ] RFC nueva o actualizacion de RFC
- [ ] ADR nuevo
- [ ] Seguridad

## Documentos relacionados

- RFC: `docs/rfc/RFC-0002-diseno-shield-mvp.md` (esta cerrada).
- ADR: N/A.
- Maestro afectado: N/A.

## Diagnostico

Auditoria de `docs/roadmap/STATUS.md` revela siete casillas de Fase 1
desincronizadas con el estado real del codigo en main:

| Linea | Estado actual | Realidad | Accion |
| --- | --- | --- | --- |
| 62 | `[x]` con nota "Pendiente de verificacion del primer run" | CI ha corrido muchas veces, tres plataformas verde y la cuarta verde tras hotfix #11 | Limpiar la nota. |
| 85 | `[/] (En bootstrap.)` | `ag-core` cerrado con pipeline completa | `[x]` con resumen de capas. |
| 86 | `[ ] Soporte HTTP/1.1 y HTTP/2 via Axum + Tokio` | Entregado en PR 1 (bootstrap) | `[x]`. |
| 92 | `[ ] Middleware de logging estructurado con tracing` | Entregado en PR 1 (`shield::logging`) | `[x]`. |
| 94 | `[ ] Tests unitarios con cobertura >= 80% del crate` | 56 unit + 27 E2E + 1 doctest = 84 verde; medicion oficial pendiente | `[/]` con explicacion. |
| 106 | `[ ] CI pasa en las cuatro plataformas` | Tres ok desde el inicio; macos-arm64 verde tras hotfix #10; windows-x64 verde tras hotfix #11 | `[/]` con detalle del primer run post-merge pendiente de inspeccionar. |
| 107 | `[ ] Clippy sin warnings` | `cargo clippy --workspace --all-targets -- -D warnings` limpio en local y en CI | `[x]`. |
| 108 | `[ ] cargo audit sin vulnerabilidades` | `cargo deny check` pasa; `cargo audit` parte de quality.yml | `[/]` con detalle. |
| 109 | `[ ] Cero bloques unsafe no documentados` | `unsafe_code = "deny"` en workspace; sin `unsafe` en codigo propio | `[x]`. |

Las casillas de metricas duras (300K req/s, p99, memoria idle,
arranque), blog post tecnico y stars permanecen `[ ]` porque requieren
medicion en hardware de referencia o eventos de comunidad, no
trabajo en el repositorio.

## Plan de prueba

```sh
# Documentos siguen renderizando.
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# No toca codigo, asi que las suites siguen pasando.
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Inspeccion manual del archivo: `docs/roadmap/STATUS.md` muestra Fase 1
con todas las casillas internas marcadas y las casillas externas
claramente identificadas como pendientes con su explicacion.

## Criterios de salida que avanza

Documental. No afecta criterios de salida del codigo. Es la
contrapartida documental al cierre tecnico de Fase 1.

## Checklist

- [x] Titulo de PR de 256 caracteres o menos.
- [x] Sin emojis en ningun archivo modificado.
- [x] Sin atribuciones de herramientas IA.
- [x] Documentacion actualizada en el mismo PR.
- [x] CHANGELOG.md actualizado bajo `[Unreleased]`.
- [x] CLAUDE.md respetado: alcance documental; sin cambios en codigo;
  sin nuevas dependencias.
- [x] Descriptor pre-rellenado existe en `docs/pr-drafts/`.
