# Descriptor de PR: community4/hooks

## Resumen

Add opt-in local git hooks and a single ag-check verification gate for contributors

## Fase afectada

Herramientas de desarrollo y gobernanza local (wave 4). No cambia la fase del
roadmap ni los documentos maestros. No modifica `.github/`, por lo que no se
requiere RFC: es una ayuda local opcional, no un cambio de CI.

## Tipo de cambio

- [x] Herramientas locales (`scripts/ag-check`, `scripts/install-hooks.sh`)
- [x] Hooks de git (`.githooks/commit-msg`, `.githooks/pre-push`)
- [x] Documentacion (`CONTRIBUTING.md`, `CHANGELOG.md`)
- [x] Tests (`tests/hooks/test-hooks.sh`)
- [ ] Codigo de crates
- [ ] Cambio de arquitectura, DSL o dependencias

## Documentos relacionados

- `CONTRIBUTING.md` (secciones "Verificacion local opt-in" / "Opt-in local verification")
- `CHANGELOG.md` (seccion "Opt-in local verification workflow")
- `.github/workflows/docs.yml` (origen de los hashes maestros y de la politica
  de contenido prohibido que replica `ag-check`)
- `docs/pr-drafts/README.md` (convencion de descriptores y cierre de issues)
- Referencia de diseno leida en modo solo lectura: `Gravital-Fabric/.githooks/`
  y `Gravital-Fabric/scripts/kaizen`.

## Plan de prueba

Ejecutado en este worktree:

1. `bash tests/hooks/test-hooks.sh` → 34 afirmaciones, 0 fallos, exit 0.
2. `scripts/ag-check --list` → los ocho nombres de verificacion.
3. `AG_CHECK_ROOT=<clone temporal> scripts/ag-check --only descriptors` →
   falla sin descriptor, pasa con `Closes: none`, falla sin palabra clave.
4. `scripts/ag-check --only masters` → pass con documentos intactos, fail con
   un documento modificado.
5. `scripts/ag-check --only prohibited` → pass limpio, fail con contenido
   prohibido generado en tiempo de ejecucion.
6. `AG_TEST_CMD=false scripts/ag-check --only test` → fail;
   `AG_TEST_CMD=true scripts/ag-check --only test` → pass.
7. Instalacion en clone temporal: `core.hooksPath=.githooks` solo local,
   hooks ejecutables, removal con `git config --unset core.hooksPath`.
8. `pre-push` propaga el codigo de salida de `ag-check` (verificado con 7).
9. `scripts/ag-check` completo → ver evidencia en las notas del agente.

## Criterios de salida que avanza

- Comunidad: flujo proactivo y auditable que reproduce las compuertas de CI
  antes de pushear, sin descargas ni acciones privilegiadas.
- Ninguna verificacion se reporta como pasada si fue omitida (`SKIP`).
- El descriptor de rama con palabra clave de cierre es obligatorio fuera de
  la rama por defecto, evitando PRs sin trazabilidad.
- Los documentos maestros quedan intactos: sus hashes siguen siendo los de
  `.github/workflows/docs.yml`.

## Cierre de issues

Closes: none

## Checklist

- [x] Pertenece a la fase correcta (herramientas locales, sin cambio de fase).
- [x] Respeta la documentacion (no toca maestros ni `.github/`).
- [x] No rompe arquitectura (no anade crates ni dependencias).
- [x] No anade complejidad innecesaria (un solo script de verificacion).
- [x] No crea dependencias circulares.
- [x] Compila (no hay codigo Rust nuevo).
- [x] Pasa tests (`bash tests/hooks/test-hooks.sh`, 34/34).
- [x] Pasa fmt (no hay codigo Rust nuevo).
- [x] Pasa clippy (no hay codigo Rust nuevo).
- [x] Pasa audit (sin cambios de dependencias; verificado en la rama).
- [x] Pasa deny (sin cambios de dependencias; verificado en la rama).
- [x] Tiene documentacion (`CONTRIBUTING.md`, `CHANGELOG.md`).
- [x] Tiene observabilidad (salida PASS/FAIL/SKIP por verificacion).
- [x] Tiene manejo de errores correcto (codigos de salida propagados).
- [x] Tiene trazabilidad (descriptor y notas con evidencia real).
- [x] Mantiene coherencia con Anti-Gravital v4.1.
