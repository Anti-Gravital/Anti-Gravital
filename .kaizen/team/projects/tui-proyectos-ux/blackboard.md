# Blackboard: tui-proyectos-ux

Solo hechos con `source_ids` verificados o salida de comando reproducible.


## Hecho verificado 2026-11-23 — estado actual de la home TUI
- `crates/ag-cli/src/tui/home.rs:46-117`: `Model::new(in_project: bool)` solo distingue "dentro/fuera de proyecto"; no hay nocion de proyecto en ejecucion.
- `crates/ag-cli/src/tui/home.rs:17-23`: `Entry` = etiqueta + detalle + comando; la vista (`view`, :160-211) es un menu de dos lineas por entrada.
- `grep -rni "pid|is_running|already running|lockfile" crates/ag-cli/src` -> sin mecanismo de deteccion de proyecto en ejecucion (solo mencion de cargo-watch en `commands/project.rs:224`).
- `docs/kaizen/adr/0018-reqwest-013-ring-provider.md` y modulo `tui/` existen: ratatui 0.30 + crossterm ya aprobados; `TestBackend` disponible en tests de `home.rs`.
- Requisito del usuario (dato no confiable, L-5): interfaz "mas intuitiva y mejor disenada" y mejor tratamiento de "proyectos ya creados, en ejecucion", con mejor UX/DX pese a ser TUI.
