# Descriptor de PR: community4/layered

## Resumen

Add opt-in layered project template with ag.toml manifest and inward-only crate dependencies (RFC-0019 stage A)

## Fase afectada

Additive Phase 4.8, stage A. No phase or roadmap state changes; the default
`ag new` behavior is untouched.

## Tipo de cambio

- [x] Codigo (`ag-cli`: `Template`, flag parser, scaffolder, wizard expectations)
- [x] Templates (`templates/layered/**`, new tree)
- [x] Tests de integracion (`crates/ag-cli/tests/layered.rs`)
- [x] Documentacion tecnica (`templates/README.md`, RFC-0019 status, CHANGELOG)
- [ ] Cambio de arquitectura del ecosistema
- [ ] Cambio de seguridad / stack / dependencias del workspace
- [ ] Cambio en `docs/master/`

## Documentos relacionados

- `docs/rfc/RFC-0019-project-layered-architecture.md` (secciones 4.1-4.5, 5 y el
  nuevo "Stage A implementation status")
- `templates/README.md` (seccion "Layered template")
- `CHANGELOG.md` (seccion "Opt-in layered project template")
- `CLAUDE.md` reglas 5, 8, 26 y 28 (documentacion antes que codigo)

## Plan de prueba

Ejecutados en este worktree:

1. `cargo test -p ag-cli` (54 unit + 3 layered + 6 non-interactive: 0 fallos)
2. `cargo test --workspace --all-features` (exit 0, 80 bloques "test result: ok",
   0 FAILED; los 3 tests de `tests/layered.rs` incluidos)
3. `cargo fmt --all -- --check` (limpio)
4. `cargo clippy --workspace --all-targets --all-features -- -D warnings` (limpio)
5. `cargo audit` (4 allowed warnings, 0 vulnerabilidades)
6. `cargo deny --all-features check` (advisories, bans, licenses, sources OK)
7. Scaffold fresco en `/tmp`: `ag new demo --template layered`, seguido de
   `cargo metadata --no-deps` (5 paquetes resueltos) y `cargo check` (compila
   contra el `rev` fijado, sin warnings).

Nota de entorno: los comandos 2 y 4 se ejecutaron con un `CARGO_TARGET_DIR`
privado del worktree. El directorio compartido de Cargo es usado por varios
agentes concurrencialmente y el binario `ag` en `debug/ag` puede ser
sobrescrito por otra rama durante la ejecucion de tests (esto produjo un fallo
espurio en un primer intento). En CI, con un solo worktree, no aplica.

## Criterios de salida que avanza

- RFC-0019 stage A: layout de cuatro crates + raiz, regla de dependencias
  hacia adentro y manifiesto `ag.toml` entregados y verificados.
- `templates/README.md` documenta el nuevo template y su alcance.
- No se cierra ningun issue: #154 se referencia porque el trabajo es
  incremental y el cambio de defaults (seccion 9.1) queda para otro PR.

## Cambios que el coordinador debe saber

- Se anade una cuarta variante a `Template::ALL`. Dos tests del wizard TUI
  tenian expectativas fijas: el ultimo elemento seleccionado (ahora `layered`)
  y la altura del frame que mostraba la lista de archivos. Son ajustes
  obligatorios para que la suite pase; la logica del wizard no cambio.
- Conflicto esperado con las ramas `tui`, `machinejson` y `envsecrets`: todas
  tocan `ag-cli`. Los cambios aqui son aditivos y estan confinados a
  `request.rs`, `cli.rs`, `commands/project.rs`, `main.rs` (comentario) y el
  test del wizard.

## Cierre de issues

Refs #154
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (Fase 4.8 stage A, aditivo).
- [x] Respeta la documentacion (RFC-0019; desviacion del default documentada).
- [x] No rompe arquitectura (los tres templates por defecto quedan identicos).
- [x] Compila (workspace y proyecto generado).
- [x] Pasa tests (workspace --all-features).
- [x] Pasa fmt.
- [x] Pasa clippy con -D warnings.
- [x] Pasa audit (4 allowed warnings).
- [x] Pasa deny --all-features check.
- [x] Tiene documentacion y CHANGELOG.
- [x] Mantiene coherencia con Anti-Gravital v4.1.
- [ ] No cierra #154: el cambio de defaults de la seccion 9.1 del RFC sigue
      pendiente de un PR posterior.
