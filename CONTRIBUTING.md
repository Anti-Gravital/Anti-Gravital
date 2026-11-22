# Contributing to Anti-Gravital

Este documento describe como contribuir al proyecto Anti-Gravital. Las
contribuciones son bienvenidas en espanol y en ingles por igual.

This document describes how to contribute to the Anti-Gravital project.
Contributions in Spanish and English are equally welcome.

## Antes de empezar

1. Lea los maestros de `docs/master/`. No se aceptan cambios que
   contradigan la arquitectura o la hoja de ruta declaradas alli.
2. Consulte `CLAUDE.md`. Es la constitucion tecnica del repositorio y
   se aplica a todo contribuidor, humano o automatizado.
3. Identifique en `docs/roadmap/` la fase actual del proyecto y
   verifique que su cambio pertenece a esa fase.

## Tipos de cambios

| Tipo de cambio | Que se necesita antes |
| --- | --- |
| Correccion de typo o de enlace roto | Pull request directa. |
| Mejora de documentacion derivada    | Pull request, sin modificar maestros. |
| Cambio de un maestro de `docs/master/` | RFC aprobada en `docs/rfc/`. |
| Cambio arquitectonico                | RFC aprobada en `docs/rfc/`. |
| Cambio del DSL                       | RFC aprobada en `docs/rfc/`. |
| Cambio de objetivos de rendimiento   | RFC aprobada en `docs/rfc/`. |
| Anadir un crate nuevo                | RFC aprobada en `docs/rfc/`. |
| Cambio de CI o de herramientas       | RFC aprobada en `docs/rfc/`. |
| Decision arquitectonica registrada   | Adicionalmente ADR en `docs/adr/`. |

## Flujo de pull request

1. Cree una rama de trabajo descriptiva sin prefijos de herramientas IA.
   Ejemplo aceptable: `phase-1/shield-tls-bootstrap`. Ejemplos prohibidos:
   `claude/...`, `gpt/...`, `ai/...`.
2. Mantenga el cambio pequeno y enfocado. Una unidad logica por pull
   request. Si hace falta dividir, divida.
3. Cree o actualice el descriptor pre-rellenado del PR en
   `docs/pr-drafts/<rama-aplanada>.md` (las `/` se convierten en
   `-`). Sin descriptor, el PR no se acepta. Vease
   `docs/pr-drafts/README.md` para la convencion. Al abrir la PR en
   GitHub, el workflow `pr-autofill` reemplaza automaticamente el
   cuerpo con el contenido del descriptor. La plantilla en
   `.github/PULL_REQUEST_TEMPLATE.md` queda como aviso para casos en
   que el descriptor no exista.
4. Asegurese de que pasa `cargo fmt --all -- --check`,
   `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, `cargo audit` y `cargo deny check`.
5. Actualice la documentacion en el mismo pull request. Codigo y
   documentacion evolucionan juntos.
6. Actualice `CHANGELOG.md` bajo la seccion `[Unreleased]`.
7. Verifique las casillas de la `docs/roadmap/STATUS.md` que su pull
   request afecta.
8. Al abrir el pull request en GitHub, el workflow `pr-autofill`
   reemplaza automaticamente el cuerpo con el descriptor de
   `docs/pr-drafts/`. Si abre la PR y el cuerpo sigue siendo la
   plantilla de aviso, significa que falta el descriptor: cree el
   archivo, pushee, y reabra la PR (o marquela ready-for-review)
   para que se autocomplete. El titulo del PR es el campo "Resumen"
   del descriptor (256 caracteres o menos).
9. En el cuerpo del PR enlace a la RFC o ADR relacionada cuando aplique.

## Convenciones de mensajes

### Commits

- Maximo 256 caracteres por linea de asunto.
- Estilo Conventional Commits ligero: `tipo(scope): asunto`.
- Tipos sugeridos: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`,
  `ci`, `build`, `perf`, `security`.
- Cuerpo opcional, separado del asunto por una linea en blanco, con
  lineas de hasta 100 caracteres.
- Prohibido: trailers de atribucion de herramientas IA,
  `Co-Authored-By` de modelos o agentes, URLs de sesiones de chat,
  emojis.

### Pull requests

- Titulo: 256 caracteres o menos.
- Descripcion: resumen del cambio, fase afectada, RFC o ADR
  relacionada, plan de prueba, y la lista de criterios de salida de la
  fase que el cambio avanza.
- Prohibido: emojis, atribuciones IA, capturas con marca de
  herramientas IA.

### Issues

- Use las plantillas en `.github/ISSUE_TEMPLATE/`.
- Para bugs incluya version del workspace, plataforma, comandos
  ejecutados y salida observada vs esperada.
- Para propuestas grandes, abra una RFC en lugar de un issue.

## Idiomas

Espanol e ingles son ciudadanos de primera clase. Si su pull request
introduce documentacion nueva, intente publicarla en ambos idiomas. Si
no puede, marquela como pendiente de traduccion y abra un issue.

## Estilo de codigo

- Rust edition 2021.
- Rust MSRV: 1.99.1, pinned in `rust-toolchain.toml` and checked by CI.
- Formateo: `cargo fmt`, sin excepciones.
- Lints: `clippy` con `-D warnings`.
- `unsafe` requiere comentario inmediato anterior con el motivo y un
  apuntador a la RFC que lo autoriza.
- Errores: definir un tipo de error explicito por crate, sin
  `unwrap`/`expect` fuera de tests o de configuraciones de arranque que
  fallan rapido por contrato.
- Sin abreviaciones crípticas en identificadores publicos.

## Tests

- Toda funcionalidad nueva trae sus tests.
- Cobertura objetivo por crate: 80% en Fase 1-4, 85% desde Fase 10.
- Para los componentes criticos (Shield, parser DSL): fuzzing
  obligatorio antes de tag de release.

## Mantenimiento de GitHub Actions

Las acciones de terceros en `.github/workflows/` se fijan a un SHA completo.
Revise sus releases y changelogs al menos una vez por trimestre y abra un pull
request dedicado para actualizar cada pin, conservando el tag de referencia en
el comentario de la linea `uses:`. Valide todos los workflows despues del cambio.

## Automatic guardrails (RFC-0022)

Four checks run without anyone having to remember them. Each one fails with a
message that says what to fix; their behavior is covered by
`bash tools/test-guardrails.sh`.

- **Weekly security audit.** The `security-audit` workflow runs `cargo audit`
  and `cargo deny check --all-features` on `main` every Monday and on demand.
  On failure it opens or updates exactly one issue labelled `security` with the
  captured output; when both checks pass it closes that issue and reopens it if
  a later run fails. The issue carries
  an ownership marker, so a hand-written security issue is never modified.
  The scheduled job stays green after reporting; the issue is the signal.
- **Stale pull requests.** The `stale-prs` workflow runs daily. A pull request
  with no activity for 30 days gets the `stale` label, a note and is closed;
  its branch is kept. Use the manual `dry_run` input to preview candidates.
- **Derived documents.** The `guardrails / derived documents match masters`
  job regenerates `docs/architecture/` and `docs/roadmap/` in a scratch copy
  and fails on any difference. Fix it with `bash tools/split-masters.sh` and
  commit the result. If a master section moved, recalculate the line ranges in
  `tools/split-masters.sh` in the same commit.
- **File size.** A tracked Rust source file over 600 lines fails unless it is
  listed in `tools/file-size-allowlist.txt` with an issue number. Entries whose
  file is now shorter (or gone) also fail: the allowlist may only shrink.
  Add a focused refactoring issue when you need a new exception.

The `clippy.toml` threshold for `clippy::too_many_lines` is set to 80 lines.
That restriction lint is enabled per crate as each crate adopts it; it is not
yet part of the workspace-wide clippy gate. The clap command-table check from
RFC-0022 section 4.6 is not implemented because a literal-string comparison
would be brittle against help-text changes.

## Barandillas automaticas (RFC-0022)

Cuatro verificaciones se ejecutan sin que nadie tenga que recordarlas. Cada una
falla con un mensaje que indica como corregirla; su comportamiento se prueba con
`bash tools/test-guardrails.sh`.

- **Auditoria semanal.** El workflow `security-audit` ejecuta `cargo audit` y
  `cargo deny check --all-features` en `main` cada lunes y bajo demanda. Si falla,
  abre o actualiza un unico issue etiquetado `security` con la salida capturada;
  cuando ambas verificaciones pasan, cierra ese issue y lo reabre si una
  ejecucion posterior falla. El issue lleva un marcador
  de propiedad, asi que un issue escrito a mano nunca se modifica. El job
  programado queda en verde tras informar; el issue es la senal.
- **Pull requests inactivos.** El workflow `stale-prs` se ejecuta a diario. Un
  pull request sin actividad durante 30 dias recibe la etiqueta `stale`, una nota
  y se cierra; su rama se conserva. Use la entrada manual `dry_run` para ver los
  candidatos sin cerrarlos.
- **Documentos derivados.** El job `guardrails / derived documents match masters`
  regenera `docs/architecture/` y `docs/roadmap/` en una copia temporal y falla
  ante cualquier diferencia. Corrijalo con `bash tools/split-masters.sh` y
  commitee el resultado. Si una seccion del maestro se movio, recalcule los
  rangos en `tools/split-masters.sh` en el mismo commit.
- **Tamano de archivos.** Un archivo Rust versionado de mas de 600 lineas falla
  salvo que figure en `tools/file-size-allowlist.txt` con un numero de issue.
  Las entradas cuyo archivo ya es corto (o desaparecio) tambien fallan: la lista
  solo puede reducirse. Abra un issue de refactorizacion especifico para cada
  excepcion nueva.

El umbral de `clippy::too_many_lines` en `clippy.toml` es de 80 lineas. Ese lint
de restriccion se habilita crate por crate a medida que se adopta; aun no forma
parte del gate de clippy de todo el workspace. La comprobacion de la tabla de
comandos de la seccion 4.6 de RFC-0022 no esta implementada porque comparar
textos literalmente seria fragile ante cambios de ayuda.

## Seguridad

Si descubre una vulnerabilidad, no abra un issue publico. Siga el
procedimiento de `SECURITY.md`.

## Reconocimientos

Los contribuidores se listan en commits y en releases. No usamos
sistemas de atribucion automatica de IA: el credito es para personas y
organizaciones humanas.

## Codigo de conducta

Todo participante acepta y respeta `CODE_OF_CONDUCT.md`.
