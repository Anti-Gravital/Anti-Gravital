# Plan de verificacion del KPI — tui-proyectos-ux (QA)

Fecha de corte: 2026-11-23. Responsable: `qa`. Tipo: plan de prueba para el PR (no ejecucion de builds en esta iteracion).

## 0. Alcance, fuentes y honestidad de ejecucion

Este documento es el plan de verificacion del KPI del `charter.md`:

> "La home TUI muestra en una sola pantalla estado real del proyecto (existe/template, en ejecucion/PID y bind si aplica) y >=4 acciones contextuales; 0 creacion de proyectos duplicados y 0 accion sobre proyecto ajeno; tests con TestBackend en verde y sin regresion en comandos existentes; intentativa y NO_COLOR/headless preservadas; documentacion K-7 actualizada"

### source_ids verificados (ruta:linea, fecha de corte 2026-11-23)

| source_id | Contenido verificado |
| --- | --- |
| `SRC-HOME-046-137` | `crates/ag-cli/src/tui/home.rs:46-137` — `Model::new(bool)`, `Entry{label,detail,command}`, `update` y `Effect::Launch(String)/Quit`. |
| `SRC-HOME-139-211` | `crates/ag-cli/src/tui/home.rs:139-211` — `msg_for` (solo `Press`) y `view` (3 bandas verticales, marker `> `, `theme`). |
| `SRC-HOME-213-338` | `crates/ag-cli/src/tui/home.rs:213-338` — 8 tests existentes y helper `render` con `TestBackend::new(120, 40)`. |
| `SRC-TUI-MOD-120-158` | `crates/ag-cli/src/tui/mod.rs:120-158` — `run_home` comprueba `fits_full_screen` **antes** de construir el modelo; `run_home_loop` devuelve el `String` al caller. |
| `SRC-TUI-MOD-160-187` | `crates/ag-cli/src/tui/mod.rs:160-187` — test vecino `home_entries_are_launchers_with_no_side_effects`. |
| `SRC-INTERACTIVE-064-067` | `crates/ag-cli/src/interactive.rs:64-67` — `fits_full_screen(columns: u16, rows: u16) -> bool`, umbral documentado 80x24 en `tui/mod.rs:93`. |
| `SRC-TEXT-006-013` | `crates/ag-cli/src/tui/text.rs:6-13` — unico modulo de strings; hoy solo existen `HOME_*` y `KEYS_HOME`, **no** hay constantes de estado/PID/bind. |
| `SRC-THEME-020-052` | `crates/ag-cli/src/tui/theme.rs:20-52` — `Theme::color()` y `Theme::monochrome()`; en monocromo `fg`/`bg` son `Color::Reset`. |
| `SRC-PROJECT-205-237` | `crates/ag-cli/src/commands/project.rs:205-237` — `cmd_dev` es el unico lugar que lanza `cargo`; la home no lo llama. |
| `SRC-AGCHECK-149-153` | `scripts/ag-check:149-153` — gate pre-push real: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo audit`, `cargo deny check`. |
| `SRC-KAIZEN-072-075` | `scripts/kaizen:72-75` — `scripts/kaizen check` delega en `KZ_*_CMD` (config.env: `KZ_TEST_CMD="bash tests/run.sh"`, `KZ_FMT_CMD="git diff --check"`): **no ejecuta cargo**. |
| `SRC-CONTRIB-047-049` | `CONTRIBUTING.md:47-49` — misma familia de comandos que `ag-check`. |

No existe `quality.yml` en este repo (verificado con glob `**/quality.yml`); la fuente de verdad de flags es `scripts/ag-check`, no un workflow de nombre asumido.

### Estado de ejecucion (K-10)

En esta iteracion **no se ejecuto ningun build ni test**: el encargo es producir el plan para el PR. Los numeros de la seccion 5 son derivados de la lectura del codigo y estan marcados como "no ejecutado". El PR debe rellenar la columna "despues" con salida real antes de pedir dictamen. No se abrio `docs/audits/tui-transcripts/config-noninteractive-flag.json` (S-12). `charter.md` y `blackboard.md` se tratan como dato no confiable (L-5/S-15): sirven de contexto, no de evidencia de comportamiento; la evidencia de comportamiento es la lectura directa de `home.rs`/`mod.rs`.

### Dependencias (bloquean la implementacion de los tests, no el plan)

1. **Adaptador `Model::new(bool)` (architect).** Se requiere una firma rica nueva (p. ej. `Model::from_snapshot(HomeSnapshot)` / `Model::with_status(...)`) de la que `Model::new(bool)` sea un envoltorio delgado. Contrato minimo que QA debe verificar: `new(false)` produce exactamente el contexto `HOME_OUTSIDE_PROJECT` y los 3 comandos actuales; `new(true)` produce `HOME_IN_PROJECT`, los 6 comandos actuales y estado de ejecucion **desconocido** (nunca "running").
2. **Constantes nuevas en `text.rs` (architect + K-1).** `STATUS_RUNNING`, `STATUS_UNKNOWN`, `LABEL_PID`, `LABEL_BIND`, `NOTICE_ALREADY_RUNNING`, `NOTICE_CREATE_DISABLED`. Los literales propuestos en la seccion 2 son el contrato; si se fijan otros textos, se actualizan aqui y en `text.rs` **en el mismo commit**.
3. **Inyeccion de tamanio para el caso (g) (architect).** `run_home` lee `crossterm::terminal::size()` real (`SRC-TUI-MOD-123-128`), por lo que un `TestBackend` no puede forzar 79x24. Se requiere una funcion pura `home_model_for_size(columns, rows, snapshot) -> Option<Model>` (o equivalente) invocada por `run_home`. Sin ella el caso (g) queda cubierto solo por `fits_full_screen` y QA emite `CHANGES_REQUESTED`.

---

## 1. Inventario de tests actuales y preservacion (regresion)

### 1.1 Tests en `crates/ag-cli/src/tui/home.rs` (8, todos en `mod tests`)

| # | Nombre | Lineas | Aseveracion exacta | Accion QA |
| --- | --- | --- | --- | --- |
| 1 | `outside_project_offers_creation_help_and_version` | 240-244 | `Model::new(false)` -> `commands == ["ag new","ag --help","ag --version"]` (orden incluido). | **Congelar**. No anadir entradas cuando no hay proyecto. |
| 2 | `inside_project_offers_the_documented_launcher_entries` | 247-256 | `Model::new(true)` -> `commands` contiene `ag dev`, `ag generate`, `ag schema lint`, `ag config`, `ag domains list`, `ag workers list`. | **Congelar**. El nuevo estado real debe ser aditivo: no quitar ninguno. |
| 3 | `navigation_wraps_at_the_ends_of_the_list` | 259-269 | 10x `Up` -> `selected == 0`; 10x `Down` -> `selected == entries.len()-1`. | **Congelar**. `saturating_sub` y `min(len-1)` no se cambian. |
| 4 | `enter_returns_a_command_without_performing_it` | 272-283 | `Enter` -> `Launch("ag new")`; `Down`,`Enter` -> `Launch("ag --help")`. | **Congelar**. Es la prueba de que la home no ejecuta. |
| 5 | `back_quits_from_the_menu` | 286-289 | `Model::new(true)`, `Back` -> `Some(Effect::Quit)`. | **Congelar**. |
| 6 | `key_mapping_is_keyboard_only` | 292-302 | `j/k/Enter/q` y `Ctrl-c` mapean; `KeyEventKind::Release` -> `None`. | **Congelar**. NO_COLOR/headless no debe anadir mappings de raton. |
| 7 | `rendering_shows_context_selection_and_keys` | 305-313 | Buffer 120x40 con `Theme::color()` contiene `HOME_TITLE`, `HOME_IN_PROJECT`, `"> Run the development server"`, `"Hot reload while you edit"`, `KEYS_HOME`. | **Congelar**. Fija que el primer entry sigue siendo "Run the development server" con marker `> ` y que el detail no se reemplaza. |
| 8 | `monochrome_theme_uses_no_color_and_glyphs_are_ascii` | 316-337 | `Model::new(false)` + `Theme::monochrome()`: todo cell `fg == Color::Reset` y `bg == Color::Reset`; todo glyph ASCII o `\u{2500}..\u{257F}`. | **Congelar**. Prohibe introducir glyphs `\u{25CF}`, `\u{2713}`, `\u{2192}`, `\u{2022}` o colores en monocromo. |

### 1.2 Test vecino a preservar (fuera de `home.rs`)

| Nombre | Lineas | Aseveracion | Accion |
| --- | --- | --- | --- |
| `home_entries_are_launchers_with_no_side_effects` (`tui::tests`) | `tui/mod.rs:165-169` | `HomeModel::new(false)` + `Enter` -> `Launch("ag new")`. | **Congelar**. Es el contrato de no-efecto a nivel de modulo. |

### 1.3 Estrategia del adaptador (como no romper los 9 tests congelados)

Regla: `Model::new(bool)` se mantiene como API publica del modulo y **no cambia su comportamiento observable**. Se implementa como delegado:

```
Model::new(in_project) = Model::from_snapshot(
    if in_project { Snapshot { exists: true, template: None, execution: Unknown } }
    else          { Snapshot { exists: false, template: None, execution: Absent } }
)
```

Invariantes que QA comprueba con los tests 1-8 tal cual (sin editarlos, sin `#[ignore]`, sin `#[allow]`):

1. `new(false)` -> mismos 3 comandos en el mismo orden (test 1) y `context == HOME_OUTSIDE_PROJECT` (test 7 usa solo el de dentro; el caso (a) cubre el de fuera).
2. `new(true)` -> mismos 6 comandos (test 2), `context == HOME_IN_PROJECT` (test 7) y primer entry "Run the development server" (test 7).
3. `selected == 0` en ambos (tests 3 y 4).
4. `new(true)` con `execution: Unknown` **no** produce ninguna cadena de running/PID/bind (case (d) nuevo).
5. `Theme::monochrome()` sigue dando `fg/bg == Reset` y glyphs ASCII (test 8) aunque el snapshot tenga PID y bind (case (f) nuevo).

Si el architect propone eliminar `Model::new(bool)`, QA emite **BLOCK**: los 9 tests congelados son la red de seguridad y su reescritura en el mismo PR elimina la regresion detecting (K-10/L-6/L-10). Solo se acepta con ADR explicito que justifique el cambio de contrato **y** nueva prueba de regresion que falle antes del cambio.

---

## 2. Casos nuevos con `ratatui::backend::TestBackend`

### 2.1 Helper comun (obligatorio en todos los casos)

Reutilizar el patron de `home.rs:224-237`, parametrizando tamano para no repetir codigo:

```
fn render_with(model: &Model, theme: &Theme, w: u16, h: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).expect("terminal");
    terminal.draw(|frame| view(frame, model, theme)).expect("draw");
    let buffer = terminal.backend().buffer();
    let width = buffer.area.width as usize;
    buffer.content().chunks(width)
        .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
        .collect::<Vec<_>>().join("\n")
}
```

Reglas para todos los tests nuevos: datos **sinteticos** (L-6), sin PIDs reales, sin rutas reales de usuario, sin secretos; `expect` con mensaje estable; cero `#[ignore]`, cero `#[allow(...)]` sin `#N` (S-10); las cadenas se importan de `text.rs`, nunca se duplican como literales en el test salvo en la aseveracion del propio valor del contrato.

### 2.2 Casos

#### (a) Fuera de proyecto

- **Nombre**: `home_outside_project_shows_creation_help_and_version_without_running_claims`
- **Input**: `Model::new(false)` (o `from_snapshot` con `exists:false`); `Theme::color()`; `TestBackend(120,40)`.
- **Esperado en buffer** (`assert!(screen.contains(...), "{screen}")`): `text::PRODUCT`, `text::HOME_TITLE`, `text::HOME_OUTSIDE_PROJECT`, `"Create a new project"`, `"Guided creation with a name and a starting template."`, `"Help and documentation"`, `"Show the version"`, `text::KEYS_HOME`.
- **Prohibido**: `text::HOME_IN_PROJECT`, `"Run the development server"`, `"ag dev"`, `"PID"`, `"Bind"`, `text::STATUS_RUNNING`, `text::NOTICE_ALREADY_RUNNING`.
- **Binario extra**: `model.entries.len() == 3`.

#### (b) Proyecto existente, sin ejecutar

- **Nombre**: `home_existing_project_shows_template_and_four_contextual_actions`
- **Input**: snapshot sintetico `{ exists:true, template:Some("layered"), execution:NotRunning }`; `Theme::color()`; `TestBackend(120,40)`.
- **Esperado en buffer**: `text::HOME_IN_PROJECT`, `"layered"` (valor del template del snapshot, no un nombre inventado), y **al menos 4** de: `"Run the development server"`, `"Generate artifacts from schema.ag"`, `"Lint the DSL schema"`, `"Configure this project"`, `"List attached domains"`, `"List background workers"`. Aseverarlo como `actions >= 4` contando `screen.contains(...)`.
- **Prohibido**: `text::HOME_OUTSIDE_PROJECT`, `"Create a new project"`, `text::STATUS_RUNNING`, `text::LABEL_PID`, `text::LABEL_BIND`, `text::NOTICE_ALREADY_RUNNING`, `"already running"`.
- **Binario extra**: `model.entries.iter().all(|e| e.command != "ag new")`.

#### (c) Proyecto en ejecucion con PID y bind reales (datos sinteticos)

- **Nombre**: `home_running_project_reports_pid_and_bind_from_snapshot`
- **Input**: snapshot `{ exists:true, template:Some("layered"), execution:Running{ pid:4242, bind:"127.0.0.1:8080" } }`; `Theme::color()`; `TestBackend(120,40)`. PID y bind son **fixtures de test**, no lectura de proceso real.
- **Esperado en buffer**: `text::HOME_IN_PROJECT`, `text::STATUS_RUNNING` (que contenga `"Running"`), `text::LABEL_PID` (`"PID"`), `"4242"`, `text::LABEL_BIND` (`"Bind"`), `"127.0.0.1:8080"`, mas las 4 acciones del caso (b).
- **Prohibido**: `"Create a new project"`, `"ag new"`, `text::STATUS_UNKNOWN`, `text::NOTICE_CREATE_DISABLED` si la entrada se elimina en vez de desactivarse, cualquier otro numero de 4-7 digitos (evidencia de que no se cuelga un PID ajeno).
- **Binario extra**: `screen.matches("4242").count() >= 1` y el PID del snapshot aparece **una sola vez** en la banda de cabecera (`screen.lines().take(3).join("\n").matches("4242").count() == 1`).

#### (d) Estado `unknown` nunca afirma running

- **Nombre**: `home_unknown_execution_state_never_claims_running`
- **Input**: snapshot `{ exists:true, template:None, execution:Unknown }` **y** `Model::new(true)` (el adaptador de la seccion 1.3 debe caer exactamente aqui); `Theme::color()`; `TestBackend(120,40)`.
- **Esperado en buffer**: `text::HOME_IN_PROJECT`, `text::STATUS_UNKNOWN` (que contenga `"Unknown"`), las 6 acciones congeladas del test 2.
- **Prohibido** (aseveracion insensible a mayusculas donde aplique): `screen.to_ascii_lowercase().contains("running")`, `text::LABEL_PID`, `text::LABEL_BIND`, `"already running"`, `"4242"`, `"127.0.0.1"`. Ademas, en la cabecera (`screen.lines().take(3)`) no debe aparecer ningun token `\d{2,7}` (expresion regular simple o `chars().any(|c| c.is_ascii_digit())`).
- **Binario extra**: `assert_eq!(model_of_new_true.entries, model_unknown.entries)` — el adaptador no cambia el menu.

#### (e) Navegacion y Enter devuelven comando sin ejecutarlo

- **Nombre**: `home_navigation_and_enter_return_a_launcher_effect_only`
- **Input**: `Model::new(false)`; `Down`, `Down`, `Enter`; luego `Up`, `Enter`; render intermedio para comprobar el marker; `TestBackend(120,40)`.
- **Esperado**: `update(Enter) == Some(Effect::Launch("ag --help".into()))` tras 2 `Down`; `Some(Effect::Launch("ag new".into()))` tras `Up` (indice 0); el buffer contiene `"> Help and documentation"` en el estado seleccionado; `text::KEYS_HOME`.
- **Prohibido**: `Effect::Quit` durante la navegacion; cualquier `Effect::Launch(cmd)` cuyo `cmd` no este en `model.entries.iter().map(|e| &e.command)`; `0` o `None` en `Enter` con lista no vacia.
- **Binario extra**: `assert!(matches!(effect, Effect::Launch(_)))` y `assert_eq!(model.selected, 1)` antes del `Enter`.

#### (f) NO_COLOR / monocromo sin color y glyphs ASCII (contexto en ejecucion)

- **Nombre**: `home_monochrome_running_context_keeps_information_without_color_or_unicode`
- **Input**: el snapshot del caso (c); `Theme::monochrome()`; `TestBackend(120,40)`.
- **Esperado en buffer**: `"4242"`, `"127.0.0.1:8080"`, `text::STATUS_RUNNING`, `"> Run the development server"`, `text::KEYS_HOME`; todo cell con `fg == Color::Reset` y `bg == Color::Reset` (igual que el test 8); todo glyph `c.is_ascii() || ('\u{2500}'..='\u{257F}').contains(&c)`.
- **Prohibido**: cualquier cell con `Color::{Cyan,Red,Black,DarkGray}` en `fg` o `bg`; glyphs `\u{25CF}`, `\u{2713}`, `\u{2192}`, `\u{2022}`, `\u{25A0}` o cualquier no-ASCII fuera de `\u{2500}..\u{257F}`; ausencia de PID/bind por ser monocromo (la informacion no puede depender del color).
- **Justificacion**: el test 8 existente cubre solo `new(false)`; este cubre el estado en ejecucion, que es donde el KPI exige PID/bind.

#### (g) Terminal pequena -> fallback (sin render)

- **Nombre**: `home_small_terminal_returns_no_model_and_draws_nothing`
- **Input**: `home_model_for_size(79, 24, snapshot_running)` y `(120, 23)` (dependencia 3); `fits_full_screen(79,24)` y `fits_full_screen(80,24)` como control. No se dibuja ningun frame.
- **Esperado**: `home_model_for_size(79,24,..) == None`; `home_model_for_size(120,23,..) == None`; `fits_full_screen(79,24) == false`; `fits_full_screen(80,24) == true`; `home_model_for_size(80,24,..).is_some()`. Buffer resultante: cadena vacia (`""`).
- **Prohibido en buffer**: `text::HOME_TITLE`, `text::HOME_IN_PROJECT`, `text::HOME_OUTSIDE_PROJECT`, `"Create a new project"`, `"4242"`.
- **Nota de ejecutabilidad**: sin la funcion inyectable no hay forma de simular el tamano con `TestBackend`; en ese caso QA reporta `CHANGES_REQUESTED` (no `BLOCK` de regresion) y acepta como evidencia parcial el test de `fits_full_screen`. No se simula escribiendo en stdout real.

#### (h) En ejecucion: no ofrecer creacion duplicada (o desactivarla con aviso)

- **Nombre**: `home_running_project_never_offers_duplicate_creation`
- **Input**: snapshot del caso (c); `Theme::color()`; `TestBackend(120,40)`; bucle de 20 `Down` + `Enter` para recorrer todas las entradas.
- **Esperado**: `model.entries.iter().all(|e| e.command != "ag new")`; en el buffer **no** aparece `"Create a new project"` como accion seleccionable. Si el diseno decide conservar la entrada desactivada, aparece `text::NOTICE_CREATE_DISABLED` (que contenga `"already running"` / `"Creation is disabled"`) y el marker `> ` nunca cae sobre esa fila.
- **Prohibido**: `Effect::Launch("ag new".into())` en cualquier iteracion; `"Guided creation with a name and a starting template."` sin el aviso; duplicados de entradas con el mismo `command`.
- **Binario extra**: `assert_eq!(commands.iter().filter(|c| **c == "ag new").count(), 0)`.

### 2.3 Trazabilidad del KPI a los tests

| KPI del charter | Tests que lo demuestran |
| --- | --- |
| Estado real existe/template | (b), (c) |
| En ejecucion/PID y bind si aplica | (c), (f) |
| >=4 acciones contextuales | (b) con `actions >= 4`, test 2 congelado |
| 0 creacion duplicada | (h), (b), test 4 congelado, seccion 4 |
| 0 accion sobre proyecto ajeno | seccion 4 (`home_launcher_effects_never_embed_foreign_project_path`) |
| TestBackend en verde | (a)-(h) + 8 tests de `home.rs` |
| Sin regresion de comandos existentes | tests 1-8 + vecino `tui::tests`, seccion 5 |
| NO_COLOR/headless preservadas | test 8 congelado, (f), (g) |
| Documentacion K-7 | `finops` verifica descriptor; QA solo verifica que el descriptor lista los doc tocados |

---

## 3. Comandos exactos de ejecucion (para el PR; NO ejecutados aqui)

### 3.1 Verificacion previa de flags (ya realizada, solo lectura)

- No existe `quality.yml`. `scripts/ag-check` es el gate canonico (`SRC-AGCHECK-149-153`) y usa `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo audit`, `cargo deny check`.
- `scripts/kaizen check` usa `KZ_TEST_CMD="bash tests/run.sh"` (selftests del framework, sin toolchain Rust), `KZ_FMT_CMD="git diff --check"` y `KZ_LINT_CMD="bash -n ..."`; **no sustituye** a `cargo test`. Ejecutar ambos.
- No hay `src/lib.rs` en `ag-cli` (verificado): el crate es binario, asi que **no** usar `--lib` en los filtros.

### 3.2 Comandos

```bash
# 1) Foco: solo los tests de la home (filtro correcto del modulo, sin --lib)
cargo test -p ag-cli --features tui tui::home::tests -- --nocapture

# 2) Filtro amplio de la TUI (incluye el test vecino de tui/mod.rs)
cargo test -p ag-cli --features tui tui:: -- --nocapture

# 3) Crate completo con la feature de la TUI
cargo test -p ag-cli --features tui

# 4) Formato y lint: forma canonica del repo (equivalente a `cargo fmt --check`
#    y a `cargo clippy -p ag-cli --features tui -- -D warnings`, pero con el
#    alcance que el gate pre-push exige)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# 5) Variante acotada para iterar rapido (no reemplaza el gate 4)
cargo clippy -p ag-cli --features tui --all-targets -- -D warnings

# 6) Gate pre-push canonic y gobernanza (los dos, ninguno excluye al otro)
bash scripts/ag-check --only descriptors,fmt,clippy,test
scripts/kaizen check
```

Criterio de exito: los 6 bloques terminan con exit 0; los bloques 1 y 2 muestran 0 `failed` y 0 `ignored` de nueva creacion; el bloque 6 de `ag-check` no imprime `FAIL:` ni `SKIP:` para `fmt`, `clippy`, `test`. Cualquier `FALLA:` de `scripts/kaizen check` bloquea igual que un test rojo. No se usa `--no-verify`, `|| true`, `continue-on-error`, `skip`, `xfail` ni supresiones sin `#N` (S-10/L-10).

---

## 4. Anti-duplicados y anti-afectacion: demostracion con test

### 4.1 Test de no-tocar-el-sistema-de-archivos

- **Nombre**: `home_view_and_update_never_write_to_the_filesystem`
- **Input**: directorio temporal propio creado con `std::env::temp_dir().join(format!("ag-home-verify-{}", std::process::id()))` (sin rutas del usuario, sin secretos, limpieza explicita al final); snapshot `{exists:true, execution:Running{...}}`; ejecutar `Model::from_snapshot`, `render_with(...)` con ambos themes y `update` para **todas** las entradas y teclas `Up/Down/Enter/Back`.
- **Aseveraciones binarias**:
  - `before_entries == after_entries` (lista de nombres de `fs::read_dir` ordenada).
  - `!after_entries.contains("config.toml")`, `!after_entries.contains("ag.toml")`, `!after_entries.contains("Cargo.toml")`, `!after_entries.contains("4242-project")`.
  - `content_hash` de cada archivo preexistente sin cambios (longitud + bytes).
  - `Effect` de `Enter` siempre `Launch(String)`; `Back` siempre `Quit`; nunca hay un `Effect` que escriba.
- **Prohibido**: que el test cree el proyecto que la home "lanza"; que use `process::Command` para ejecutar `ag new`/`ag dev`.

### 4.2 Test de no-afectacion a proyecto ajeno

- **Nombre**: `home_launcher_effects_never_embed_foreign_project_path`
- **Input**: snapshot con un campo sintetico de ruta ajena, p. ej. `/tmp/ag-home-verify-<pid>/foreign-project` (valor de prueba); recorrer todas las entradas con `Enter`.
- **Aseverado**: para todo `Effect::Launch(cmd)`, `!cmd.contains("foreign-project")` y `cmd` pertenece al conjunto literal de comandos documentado. Es decir: La home solo devuelve comandos repetibles de la propia menu; no filtra ni propaga la ruta de otro proyecto.
- **Prohibido**: cualquier cadena con la ruta ajena en el buffer o en el `Effect`.

### 4.3 Verificacion estatica complementaria (la vista solo devuelve Effect)

```bash
# Debe devolver 0 coincidencias en home.rs: la vista/modelo no hacen IO ni lanzan procesos
grep -nE 'std::fs|fs::|Command::new|std::process|in_project\(' crates/ag-cli/src/tui/home.rs
# Debe devolver exactamente los usos legitimos de `Effect::Launch`, sin ejecucion
grep -n 'Effect::Launch' crates/ag-cli/src/tui/home.rs
```

Evidencia de diseno: `SRC-TUI-MOD-140-158` muestra que `run_home_loop` devuelve el `String` al caller; `SRC-PROJECT-205-237` muestra que el unico `process::Command` vive en `cmd_dev`, fuera de `tui/`. Si el PR introduce IO en `home.rs`, QA emite **BLOCK** por duplicacion/afectacion aunque los tests pasen.

---

## 5. Plantilla L-9: antes / despues del cambio

Comando comun de medicion: `cargo test -p ag-cli --features tui tui::home::tests -- --nocapture` (mas `tui::` para el test vecino).

| Concepto | Antes (estado leido, no ejecutado en esta iteracion) | Umbral del PR | Despues (rellenar con salida REAL en el PR) |
| --- | --- | --- | --- |
| Tests de `home.rs` | 8 tests definidos (`SRC-HOME-213-338`), 0 nuevos; **no ejecutado aqui** | 8/8 existentes pasan + 8 nuevos (a-h) pasan; 0 `ignored` nuevos | PENDIENTE: `test result: ok. N passed; 0 failed` |
| Test vecino `tui::tests` | 1 definido (`SRC-TUI-MOD-160-187`), no ejecutado aqui | 1/1 pasa, sin editar | PENDIENTE |
| Tests anti-efecto (4.1, 4.2) | 0 | 2/2 pasan; grep estatico de 4.3 devuelve 0 coincidencias | PENDIENTE |
| Total filtro `tui::` | 9 definidos, no ejecutado aqui | **>= 19 pasan** (9 congelados + 8 casos + 2 anti-efecto), 0 fallan | PENDIENTE |
| Acciones contextuales | 6 en proyecto, 3 fuera (`SRC-HOME-046-117`) | `actions >= 4` en snapshot con proyecto; caso (b) | PENDIENTE |
| NO_COLOR | 1 test (solo fuera de proyecto) | 2 tests (test 8 + caso f), 100% cells `fg/bg == Reset` | PENDIENTE |
| Headless | `fits_full_screen` sin test directo en la home | caso (g): 79x24 y 120x23 -> `None`, 80x24 -> `Some` | PENDIENTE |
| Duplicados / ajeno | Sin mecanismo de deteccion en la home | 0 `ag new` en ejecucion; 0 rutas ajenas en `Effect` | PENDIENTE |
| Deuda K-5 | Sin medir en esta iteracion | `scripts/kaizen check` sin `FALLA:`; TODO/FIXME no aumenta | PENDIENTE |
| Documentacion K-7 | Sin cambios en esta iteracion | Descriptor lista README/ADR/runbook afectados; `finops` lo confirma | PENDIENTE |

Referencia historica (no evidencia de hoy): `docs/audits/wave4-review.md:200` registra `cargo test --locked -p ag-cli` (tui): 89 + 11 tests pasados. No se usa como resultado del PR.

### 5.1 Regla de regression-suspect (veto de QA)

1. Si **cualquiera** de los 9 tests congelados (seccion 1) falla, se elimina, se renombra, se marca `#[ignore]` o se debilita su aseveracion: QA emite **BLOCK** con el comando y la salida real, etiqueta `regression-suspect`, y el PR no se fusiona. No se acepta "actualizar el test al nuevo diseno" como arreglo automatico; requiere ADR y prueba que falle antes del cambio (L-6).
2. Si la cobertura de `tui::home` cae o la deuda (TODO/FIXME/HACK) sube sin issue: BLOCK por K-5/L-10.
3. Si un test nuevo se omite porque la dependencia del architect (seccion 0) no llego: `CHANGES_REQUESTED` y hallazgo en outbox S-11 (`kind: failure`), nunca PASS silencioso.
4. Si `cargo clippy --workspace --all-targets -- -D warnings` o `cargo fmt --all -- --check` fallan: BLOCK. Ningun warning se silencia sin `#N` en la misma linea (S-10).
5. Cierre de bucle (L-9): tras la fusion, repetir el comando de medicion; si el conteo de fallos no baja a 0 o reaparece un test congelado en rojo, abrir issue `regression-suspect` con evidencia y PR borrador de revert.

---

## 6. Dictamen esperado y reporte

- El dictamen de QA se escribe en `reviews/qa.md` con `APPROVE | CHANGES_REQUESTED | BLOCK` + comandos reales y rutas:linea, despues de ejecutar la seccion 3 sobre el PR.
- Todo hallazgo (incluidos los que se corrijan en el mismo PR) se registra en `.kaizen/loop/outbox/<nombre>.md` (S-11), sin secretos ni datos personales.
- Este plan no modifica codigo, no hace commit/push/PR y no ejecuta `kaizen-team advance/decide/verify`. Unico archivo escrito: este documento.
