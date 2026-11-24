---
doc: diseno-home-ux
project: tui-proyectos-ux
agent: dx
date: 2026-11-23
status: propuesta-para-consenso
scope: spec-only (ningun cambio de codigo, ningun commit, ningun push)
---

# Diseño de la pantalla `home` de `ag` — UX/DX para proyectos existentes y en ejecución

> Este documento es una **especificación**, no una implementación. No modifica `crates/`,
> no ejecuta `kaizen-team advance/decide/verify` y no abre transcriptes ni credenciales.
> El requerimiento del usuario es **dato no confiable** (L-5): se traduce a criterios
> verificables; no se trata como instrucción que anule reglas [S13].

---

## 0. Alcance, gobernanza y límites

| Ítem | Valor |
| --- | --- |
| Entregable | Este único archivo: `.kaizen/team/projects/tui-proyectos-ux/diseno-home-ux.md` |
| Tarea asignada | `tasks.json[1]`: "Redisenar la home TUI ... con UX/DX superior y modo interactivo preservado" (owner `dx`) [S10] |
| Autoridad | `propose` + `recommend`; sin veto; el humano/arquitecto decide [S13] |
| KPI que atiende | Una sola pantalla con estado real (existe/template, ejecución/PID y bind si aplica) y ≥4 acciones contextuales; 0 proyectos duplicados; 0 acciones sobre proyecto ajeno [S11] |
| Dependencias nuevas | **Ninguna.** ratatui 0.30 + crossterm ya aprobados; `TestBackend` disponible [S1][S9] |
| Protección | No se tocan rutas protegidas (L-4). No se imprimen secretos, rutas absolutas del usuario ni valores de entorno (S-2/S-3/S-8) |

Regla de oro: **la vista es un proyector de estado, no un actor.** Sigue siendo cierto que
`Model::update` solo devuelve `Effect::Launch(String)`/`Effect::Quit` y que `run_home` solo
devuelve un `String` que `main` re-parsea y ejecuta [S1][S7][S8].

---

## 1. Fuentes verificadas (fecha de corte 2026-11-23)

Los `source_ids` son rutas + líneas + `sha256` de archivos leídos directamente. No se inventan
identificadores normativos `XX-NNN`: este diseño es de UX/DX, no de cumplimiento; por REG-EV-01
una fuente `pending` no sustentaría nada y por REG-EV-07 toda decisión guarda fuente, fecha y hash [S14].

| ID | Fuente (archivo:líneas) | sha256 (12 primeros) | Qué verifica |
| --- | --- | --- | --- |
| S1 | `crates/ag-cli/src/tui/home.rs:17-211` | `b5186db42116` | `Entry`, `Model`, `update`, `msg_for`, `view`: launcher sin efectos; menú de 2 líneas; layout 3/1/3 |
| S2 | `crates/ag-cli/src/tui/home.rs:139-157` | `b5186db42116` | Atajos actuales: Up/`k`, Down/`j`, Enter, Esc/`q`, Ctrl-C/Ctrl-D; se ignoran liberaciones de tecla |
| S3 | `crates/ag-cli/src/tui/home.rs:213-338` | `b5186db42116` | Tests existentes, incl. `enter_returns_a_command_without_performing_it` y `monochrome_theme_uses_no_color_and_glyphs_are_ascii` |
| S4 | `crates/ag-cli/src/tui/text.rs:6-13` | `9dd2086a9b76` | `PRODUCT`, `HOME_TITLE`, `HOME_IN_PROJECT`, `HOME_OUTSIDE_PROJECT`, `KEYS_HOME`; idioma único centralizado |
| S5 | `crates/ag-cli/src/tui/theme.rs:20-43` | `7a4f1a389c28` | `Theme::color()` y `Theme::monochrome()`: selected por color/inversión; error con prefijo de texto |
| S6 | `crates/ag-cli/src/tui/mod.rs:37-59,120-158` | `db32944741c7` | `fits_full_screen` (mín. 80x24); `run_home` usa `in_project()` y devuelve `Option<String>` |
| S7 | `crates/ag-cli/src/tui/mod.rs:133-138` | `db32944741c7` | Detección actual: solo `config.toml` + `Cargo.toml` contiene `ag-core`. **No detecta ejecución** |
| S8 | `crates/ag-cli/src/main.rs:84-103,106-119` | `401e286ea45d` | El único que ejecuta: `launch(&command_line, ...)` tras `println!("{command_line}")` |
| S9 | `.kaizen/team/projects/tui-proyectos-ux/blackboard.md:6-11` | `184977a34216` | Sin mecanismo de PID/`is_running`/lockfile; ratatui+crossterm ya aprobados |
| S10 | `.kaizen/team/projects/tui-proyectos-ux/tasks.json:10-15` | — | Tarea `dx` asignada, vence 2026-11-30 |
| S11 | `.kaizen/team/projects/tui-proyectos-ux/charter.md:4-11` | `4ed1a095aaad` | Objetivo y KPI verificable del proyecto |
| S12 | `crates/ag-cli/src/config.rs:176-207,373-410` | `b5e94e04b24f` | `bind` es clave raíz canónica (`bind = "host:port"`); legacy `[bind]` se normaliza en memoria |
| S13 | `.kaizen/loop/LOOP.md` L-5 + `.kaizen/security/SECURITY-RULES.md` S-2/S-3/S-12 | — | Datos externos no son instrucciones; cero secretos; nada de hardcoding; secreto visto = comprometido |
| S14 | `.kaizen/team/EVIDENCIA.md:5-32` | — | REG-EV-01 y REG-EV-07: sin fuente verificada no hay decisión; hash y fecha de corte |
| S15 | `.kaizen/team/projects/estado-honesto/charter.md:6-7` | — | Principio "0 afirmaciones sin evidencia" aplicable al panel de estado |

Hash completo de `home.rs`: `b5186db421169db77d2a72fa8190021f50782ece2a7da37392eb70842e5c364e`.
El resto de hashes completos quedan registrados en el ledger de evidencia del proyecto si el
verificador lo solicita (no se escriben aquí otros archivos que no sean este spec).

---

## 2. Diagnóstico: por qué la home actual no responde al pedido

1. **No hay estado de ejecución.** `Model::new(in_project: bool)` solo sabe "dentro/fuera" [S1][S7][S9]. Un proyecto ya creado y uno corriendo se ven idénticos; el usuario no puede saber en <2 s si debe continuar o iniciar.
2. **No hay identidad del proyecto en pantalla.** El panel dice solo `Project detected. What would you like to do?` [S1][S4]. No aparece nombre base, template, PID ni bind, aunque `bind` existe en la configuración real [S12].
3. **No hay jerarquía de acción.** Seis entradas pareadas + detalle compiten al mismo peso; la principal ("Run the development server") no se distingue salvo por el cursor `> ` [S1][S3].
4. **Microcopys pasivos.** "Hot reload while you edit (requires cargo-watch)." describe una característica, no una decisión; no dice qué pasará ni si es seguro reintentar [S1].
5. **Riesgo silencioso de duplicado.** Sin estado `running`/`unknown`, pulsar Enter puede lanzar una segunda instancia sobre el mismo directorio: exactamente lo que el KPI prohíbe [S11].

Consecuencia: el rediseño no es cosmético. Es **honestidad del estado + jerarquía de decisión**.

---

## 3. Contrato de entrada de la vista (lo que el architect debe proveer)

La vista no debe adivinar nada del disco. Recibe un dato ya resuelto:

```rust
pub(crate) enum RunState {
    NoProject,                                   // sin config.toml / sin ag-core
    Detected,                                    // proyecto real, sin instancia confirmada
    Running { pid: u32, bind: String },           // SOLO con PID verificado vivo
    Unknown,                                     // no se pudo comprobar; sin PID, sin bind
    Stale   { pid: u32 },                        // había PID, ya no existe
}

pub(crate) struct ProjectStatus {
    pub(crate) name: String,          // SOLO basename (file_name()), nunca ruta absoluta
    pub(crate) template: Option<String>,
    pub(crate) state: RunState,
}
```

Reglas duras del contrato:

- **R1.** `Running` exige PID real y vivo. Nunca se imprime un PID inventado ni un `bind` supuesto [S9][S15].
- **R2.** `bind` se muestra **únicamente** en `Running` y solo si viene de la configuración válida (clave raíz `bind`, legacy normalizado) [S12].
- **R3.** `Unknown` y `Stale` son estados de primera clase, no errores ocultos: se nombran con símbolo + palabra y explican cómo recuperarse.
- **R4.** `name` es `Path::file_name()`; si no hay nombre, `"--"`. Queda prohibida cualquier ruta absoluta en el buffer [S13].
- **R5.** La resolución de estado ocurre **antes** de abrir la terminal; la vista solo formatea (misma separación que hoy entre `run_home` y `run_home_loop`) [S6].

> Nota de alcance: hoy no existe mecanismo de detección de PID [S7][S9]. Definir cómo se obtiene
> ese PID (lockfile, socket, `/proc`, etc.) es responsabilidad del `architect` (task 0) y requiere
> su ADR. Este spec **no inventa** ese mecanismo: prescribe qué debe mostrar la vista para cada
> resultado posible, incluido "no se pudo determinar".

---

## 4. Jerarquía visual en una sola pantalla (≥80x24)

Orden de lectura, de arriba abajo y de izquierda a derecha:

1. **Cabecera** (3 líneas): producto + versión real + `HOME`. La versión viene de `env!("CARGO_PKG_VERSION")`, nunca literal (S-3) [S13].
2. **Panel de estado** (8 líneas): título `PROJECT STATUS`; línea 1 = `State : <símbolo> <PALABRA>`; línea 2 = `Project`/`Template`; línea 3 = hecho verificable sobre ejecución; línea 4 = acción principal sugerida. Es la **zona de las 2 segundos**: símbolo ASCII + palabra en mayúsculas, en negrita (`theme.label`) [S5].
3. **Menú de acciones** (Min 9 líneas), agrupado por contexto y con atajo numérico `[n]` visible. El grupo de la acción principal va primero.
4. **Footer de ayuda** (3 líneas): una sola línea con todos los atajos, más una línea de estado de tecla no disponible si aplica (p. ej. `r failed: ...`).

Layout propuesto (idéntico espíritu al actual `Layout::vertical` de `view`) [S1]:

```
header : Constraint::Length(3)
status : Constraint::Length(8)
menu   : Constraint::Min(9)
footer : Constraint::Length(3)     // total 23 de 24; sobra 1 de margen
```

Reglas de jerarquía:

- El símbolo de estado y la palabra **nunca** dependen de color: `* RUNNING`, `o DETECTED`, `? UNKNOWN`, `! STALE`, `. NO PROJECT` [S5].
- La acción principal se marca **tres veces**: `PRINCIPAL`, `[Enter]` y el cursor `> `. Si el foco está en otra fila, la etiqueta `PRINCIPAL` permanece impresa para no perder la referencia [S5].
- El detalle de cada entrada pasa de 2 líneas a **1 línea compacta** (o se elimina en favor del footer) para que quepan ≥4 acciones contextuales sin scroll [S1][S11].
- Agrupación dentro de proyecto: `CONTINUE` → `GENERATE` → `INSPECT` (read-only) → `VERIFY` → `CONFIGURE` → `EXIT`. Fuera de proyecto: `CREATE` → `HELP` → `EXIT`.

---

## 5. Frames ASCII (ilustrativos a 78 columnas; el renderer usa todo el ancho)

Las cadenas están en inglés para no romper la consistencia de `text.rs` (idioma centralizado, S4).
El ancho real se reparte con `Layout`; estos frames muestran el contenido y el orden, no padding exacto.
`v0.x.y` es ilustrativo: en runtime se sustituye por `env!("CARGO_PKG_VERSION")`.

### Frame A — Fuera de proyecto (`. NO PROJECT`)

```
+------------------------------------------------------------------------------+
| Anti-Gravital  v0.x.y  |  HOME                                              |
+------------------------------------------------------------------------------+
| PROJECT STATUS                                                               |
| State   : . NO PROJECT                                                       |
| Project : --            Template: --                                         |
| No config.toml found here. Nothing has been created or changed.              |
+------------------------------------------------------------------------------+
| ACTIONS                                                                      |
| > [1] Create a new project          PRINCIPAL   ag new                       |
|   [2] Help and documentation                    ag --help                    |
|   [3] Quit                                                                   |
+------------------------------------------------------------------------------+
| j/k: move   Enter: run   r: refresh   ?: help   Esc/q: quit                  |
+------------------------------------------------------------------------------+
```

### Frame B — Dentro de proyecto, sin ejecutar (`o DETECTED`)

```
+------------------------------------------------------------------------------+
| Anti-Gravital  v0.x.y  |  HOME                                              |
+------------------------------------------------------------------------------+
| PROJECT STATUS                                                               |
| State   : o DETECTED                                                         |
| Project : demo-api      Template: rust-axum                                  |
| No running instance detected. Nothing has been started.                      |
| Action  : [Enter] Continue development  (ag dev)   <-- PRINCIPAL             |
+------------------------------------------------------------------------------+
| ACTIONS                                                                      |
| > [1] Continue development             PRINCIPAL   ag dev                    |
|   [2] Generate artifacts                            ag generate              |
|   [3] View attached domains                         ag domains list          |
|   [4] View declared workers                         ag workers list          |
|   [5] Lint schema                                   ag schema lint           |
|   [6] Configure project                             ag config                |
|   [7] Quit                                                                   |
+------------------------------------------------------------------------------+
| j/k: move   Enter: run   r: refresh   ?: help   Esc/q: quit                  |
+------------------------------------------------------------------------------+
```

### Frame C — Dentro de proyecto EN EJECUCIÓN (`* RUNNING`)

```
+------------------------------------------------------------------------------+
| Anti-Gravital  v0.x.y  |  HOME                                              |
+------------------------------------------------------------------------------+
| PROJECT STATUS                                                               |
| State   : * RUNNING                                                          |
| Project : demo-api      Template: rust-axum                                  |
| PID     : 4242          bind: 127.0.0.1:7000                                 |
| Instance confirmed. Continue development will reuse it.                      |
| Action  : [Enter] Continue development  (ag dev)   <-- PRINCIPAL             |
+------------------------------------------------------------------------------+
| ACTIONS                                                                      |
| > [1] Continue development             PRINCIPAL   ag dev (reuse)            |
|   [2] Generate artifacts                            ag generate              |
|   [3] View attached domains                         ag domains list          |
|   [4] View declared workers                         ag workers list          |
|   [5] Lint schema                                   ag schema lint           |
|   [6] Configure project                             ag config                |
|   [7] Quit                                                                   |
+------------------------------------------------------------------------------+
| j/k: move   Enter: run   r: refresh   ?: help   Esc/q: quit                  |
+------------------------------------------------------------------------------+
```

### Frame D — Estado `unknown` honesto (`? UNKNOWN`)

```
+------------------------------------------------------------------------------+
| Anti-Gravital  v0.x.y  |  HOME                                              |
+------------------------------------------------------------------------------+
| PROJECT STATUS                                                               |
| State   : ? UNKNOWN                                                          |
| Project : demo-api      Template: rust-axum                                  |
| Could not verify a running instance (PID not available).                     |
| No PID or bind shown. Press r to retry, ? for help.                          |
| Action  : [Enter] Continue development  (may start a second instance)        |
+------------------------------------------------------------------------------+
| ACTIONS                                                                      |
| > [1] Continue development             PRINCIPAL   ag dev (check first)      |
|   [2] Generate artifacts                            ag generate              |
|   [3] View attached domains                         ag domains list          |
|   [4] View declared workers                         ag workers list          |
|   [5] Lint schema                                   ag schema lint           |
|   [6] Configure project                             ag config                |
|   [7] Quit                                                                   |
+------------------------------------------------------------------------------+
| j/k: move   Enter: run   r: refresh   ?: help   Esc/q: quit                  |
+------------------------------------------------------------------------------+
```

### Frame E — PID muerto (`! STALE`), caso límite del requirement

```
+------------------------------------------------------------------------------+
| PROJECT STATUS                                                               |
| State   : ! STALE                                                            |
| Project : demo-api      Template: rust-axum                                  |
| PID 4242 is gone. The recorded instance is no longer active.                 |
| No bind shown until state is confirmed. Press r to refresh.                  |
| Action  : [Enter] Continue development  (ag dev)   <-- PRINCIPAL             |
+------------------------------------------------------------------------------+
| j/k: move   Enter: run   r: refresh   ?: help   Esc/q: quit                  |
+------------------------------------------------------------------------------+
```

---

## 6. Microcopys concretos y accionables

Nada de texto decorativo. Cada cadena afirma un hecho verificable o prescribe una tecla.

| Clave (ir a `text.rs`) | Cadena exacta | Cuándo | Qué le dice al usuario en <2 s |
| --- | --- | --- | --- |
| `STATUS_TITLE` | `PROJECT STATUS` | siempre | Dónde mirar |
| `STATE_NO_PROJECT` | `. NO PROJECT` | NoProject | No hay proyecto aquí |
| `STATE_DETECTED` | `o DETECTED` | Detected | El proyecto existe, no está corriendo |
| `STATE_RUNNING` | `* RUNNING` | Running | Hay una instancia viva confirmada |
| `STATE_UNKNOWN` | `? UNKNOWN` | Unknown | El estado **no se sabe**; no se finge |
| `STATE_STALE` | `! STALE` | Stale | El PID anotado murió |
| `STATUS_NO_PROJECT_HINT` | `No config.toml found here. Nothing has been created or changed.` | NoProject | Acción segura: crear |
| `STATUS_DETECTED_HINT` | `No running instance detected. Nothing has been started.` | Detected | Puede iniciar sin duplicar |
| `STATUS_RUNNING_HINT` | `Instance confirmed. Continue development will reuse it.` | Running | Enter no crea otra instancia |
| `STATUS_RUNNING_LINE` | `PID     : {pid}          bind: {bind}` | Running | Datos reales, nunca inventados |
| `STATUS_UNKNOWN_HINT` | `Could not verify a running instance (PID not available).` | Unknown | Razón de la incertidumbre |
| `STATUS_UNKNOWN_ACTION` | `No PID or bind shown. Press r to retry, ? for help.` | Unknown | Cómo recuperarse |
| `STATUS_STALE_HINT` | `PID {pid} is gone. The recorded instance is no longer active.` | Stale | Por qué cambió el estado |
| `STATUS_STALE_ACTION` | `No bind shown until state is confirmed. Press r to refresh.` | Stale | Cómo recuperarse |
| `ACTION_PRIMARY` | `Action  : [Enter] Continue development  (ag dev)   <-- PRINCIPAL` | Detected/Running/Stale | Cuál es la acción principal |
| `ACTION_PRIMARY_UNKNOWN` | `Action  : [Enter] Continue development  (may start a second instance)` | Unknown | Advertencia honesta antes de duplicar |
| `MENU_PRIMARY` | `[1] Continue development` | en proyecto | Renombra el actual `Run the development server` [S1] |
| `MENU_GENERATE` | `[2] Generate artifacts` | en proyecto | `ag generate` |
| `MENU_DOMAINS` | `[3] View attached domains` | en proyecto | read-only `ag domains list` |
| `MENU_WORKERS` | `[4] View declared workers` | en proyecto | read-only `ag workers list` |
| `MENU_LINT` | `[5] Lint schema` | en proyecto | `ag schema lint` |
| `MENU_CONFIG` | `[6] Configure project` | en proyecto | `ag config` |
| `MENU_QUIT` | `[n] Quit` | siempre | sin efecto en disco |
| `MENU_NEW` | `[1] Create a new project` | fuera | `ag new` |
| `KEYS_HOME` (revisar) | `j/k: move   Enter: run   r: refresh   ?: help   Esc/q: quit` | siempre | Todos los atajos en una línea |

Impacto en tests existente: `rendering_shows_context_selection_and_keys` espera hoy
`"> Run the development server"` y `"Hot reload while you edit"` [S3]. Al renombrar la entrada,
esa aserción se actualiza en el **mismo commit** de implementación (K-1: doc y código no discrep).
No se crea `ag status`: sería un comando nuevo con ADR propio. "Ver estado" se satisface con las
dos entradas read-only existentes [S1].

---

## 7. UX/DX: teclado, foco y errores

### 7.1 Mapa de teclas (extensión de `msg_for`, sin romper el actual [S2])

| Tecla | `Msg` nuevo | Efecto en la vista | Compatibilidad |
| --- | --- | --- | --- |
| `j` / Down | `Down` | Mueve `>`; `saturating` abajo | Igual que hoy [S2] |
| `k` / Up | `Up` | Mueve `>`; `saturating` arriba | Igual que hoy [S2] |
| `1`..`9` | `Select(n)` | Salta directo a la entrada `n`; si no existe, no-op | Nuevo, atajo visible |
| `Enter` | `Enter` | `Effect::Launch(command)` de la fila con foco | Igual que hoy [S1] |
| `r` | `Refresh` | Pide re-resolver el estado; actualiza el panel sin tocar disco | Nuevo |
| `?` | `Help` | Muestra/oculta una línea de ayuda contextual en el footer | Nuevo |
| `Esc` / `q` | `Back` | `Effect::Quit` | Igual que hoy [S2] |
| Ctrl-C / Ctrl-D | `Back` | `Effect::Quit` | Igual que hoy [S2] |

- Se siguen ignorando `KeyEventKind::Release` [S2].
- `r` y `?` **no** lanzan comandos: son puramente de la TUI. `r` re-solicita un `ProjectStatus`
  ya resuelto por la capa de detección; si falla, el panel pasa a `? UNKNOWN` con la razón.
- El event loop sigue siendo el único sitio que toca el terminal [S6]. Sin hilos, sin `sleep`, sin
  polling en la vista: DX = feedback inmediato y determinista.

### 7.2 Indicadores de foco

- Cursor `> ` delante de la fila activa (ya existe [S1]); se conserva en monocromo.
- `theme.selected` (inversión en monocromo, fg/bg en color) [S5].
- Atajo numérico `[n]` siempre impreso; la fila activa además lleva `PRINCIPAL` si es la principal.
- `?` cambia el footer de `KEYS_HOME` a `KEYS_HELP` con una línea por símbolo (`* = running, o = detected, ? = unknown, ! = stale`). Vuelve con `?` o `Esc`.

### 7.3 Mensajes de error claros

| Situación | Mensaje en el panel | Regla |
| --- | --- | --- |
| No se pudo comprobar PID | `Could not verify a running instance (PID not available).` | Nunca escribir `RUNNING`; nunca imprimir `PID` con número [S9][S15] |
| PID muerto | `PID {pid} is gone. The recorded instance is no longer active.` | Estado `! STALE`; no mostrar bind |
| `r` falló | `Refresh failed: state unavailable. Press r to retry or ? for help.` | Sin traza interna, sin ruta, sin dato sensible (S-7/S-8) [S13] |
| config ilegible | `Cannot read project config. Run "ag config" to inspect it.` | No imprime el contenido del archivo |

Ningún error se traga: se muestra en el panel con prefijo de texto, no solo con color rojo [S5].

---

## 8. Accesibilidad y robustez terminal

1. **No depender del color.** Cada estado lleva símbolo ASCII + palabra en mayúsculas. El error
   lleva texto, no solo `theme.error`. La fila activa lleva `>` además de inversión [S5].
2. **NO_COLOR / monocromo.** Se mantiene `Theme::for_session(no_color)` [S5]. Todos los glyphs
   nuevos son ASCII (`* o ? ! . > [ ] / - :`), dentro del rango permitido por el test
   `monochrome_theme_uses_no_color_and_glyphs_are_ascii` [S3]. No se introducen `OK`, `ERR`,
   `→`, `↑` ni otros caracteres fuera de ASCII/box-drawing.
3. **Contraste.** Estado y acción principal usan `theme.label` (bold); `theme.hint` solo para
   atajos. En color, `selected` usa fg negro/bg cian ya existente [S5]; en monocromo, `REVERSED`
   con `Color::Reset` en todas las celdas [S3].
4. **Terminal pequeña.** Se conserva el patrón `fits_full_screen(columns, rows)`:
   - `>=80x24` → pantalla completa con `TestBackend`/`CrosstermBackend` [S6].
   - `<80x24` → `run_home` devuelve `None` hoy [S6]. El rediseño **no cambia ese contrato**;
     la ayuda en modo pequeño sigue siendo responsabilidad del flujo no interactivo/line mode.
   - Si en el futuro se habilita modo reducido, el fallback es **columna única de líneas**:
     una línea por estado (`* RUNNING`), una por acción (`1) ...`), sin tablas ni columnas
     paralelas; `Wrap { trim: false }` ya está en uso [S1].
5. **Sin dependencias nuevas.** Solo `Layout`, `Paragraph`, `Span`, `Line`, `Block`, `Wrap`,
   `Frame` de ratatui y `KeyEvent` de crossterm, ya importados [S1][S9].

---

## 9. Criterios de aceptación (binarios, con `TestBackend`)

Todos son verificables con `Terminal::new(TestBackend::new(80, 24))` y el helper `render`
ya existente en `home.rs:224-237` [S3]. `OK` = la aserción pasa; cualquier `FALSO` bloquea.

### CA-1 · Fuera de proyecto
- [ ] Aparecen: `Anti-Gravital`, `HOME`, `. NO PROJECT`, `Create a new project`, `ag new`, `j/k: move`, `?: help`.
- [ ] NO aparecen: `RUNNING`, `DETECTED`, `UNKNOWN`, `PID`, `bind`.
- [ ] `Model::new(false).entries` sigue siendo `["ag new", "ag --help", "ag --version"]` [S3].

### CA-2 · Proyecto detectado, sin ejecutar
- [ ] Aparecen: `o DETECTED`, `demo-api`, `rust-axum`, `No running instance detected`, `Continue development`, `ag dev`.
- [ ] Aparecen ≥4 acciones contextuales read-only/accionables: `ag generate`, `ag domains list`, `ag workers list`, `ag schema lint`, `ag config`.
- [ ] NO aparecen: `RUNNING`, `PID [0-9]`, `bind: [0-9]`, ninguna ruta que empiece por `/`.

### CA-3 · Proyecto en ejecución
- [ ] Aparecen: `* RUNNING`, `PID     : 4242`, `bind: 127.0.0.1:7000`, `Instance confirmed`, `reuse`.
- [ ] NO aparecen: `? UNKNOWN`, `STALE`, `could not verify`.

### CA-4 · Estado unknown (el criterio anti-fingimiento)
- [ ] Aparecen: `? UNKNOWN`, `Could not verify a running instance`, `No PID or bind shown`, `Press r to retry`.
- [ ] **NO aparece la cadena `RUNNING`.**
- [ ] **NO aparece `PID` seguido de dígitos** (regex `PID\s*:\s*\d+` sobre el buffer → 0 coincidencias).
- [ ] NO aparece `bind:` con valor.
- [ ] Aparece la advertencia `may start a second instance`.

### CA-5 · PID muerto
- [ ] Aparecen: `! STALE`, `PID 4242 is gone`, `Press r to refresh`.
- [ ] NO aparecen: `RUNNING`, `bind:`.

### CA-6 · La vista no ejecuta trabajo
- [ ] `Model::update(Msg::Enter) == Some(Effect::Launch("ag dev"))` y **no** crea archivos, **no** abre sockets, **no** lanza procesos.
- [ ] El test existente `enter_returns_a_command_without_performing_it` [S3] sigue en verde (extender, no borrar).
- [ ] `r` y `?` devuelven `None` de `update` (no `Launch`): no salen comandos al stdout.
- [ ] `run_home` sigue devolviendo `Option<String>`; `launch` sigue siendo el único ejecutor [S6][S8].

### CA-7 · Monocromo y ASCII
- [ ] Con `Theme::monochrome()`, **todas** las celdas tienen `fg == Color::Reset` y `bg == Color::Reset`.
- [ ] Cada carácter del buffer es ASCII o box-drawing (`\u{2500}..=\u{257F}`) [S3].
- [ ] Los cinco símbolos (`* o ? ! .`) son ASCII y aparecen junto a su palabra.

### CA-8 · 80x24 y foco
- [ ] Render a 80x24 sin panic; las cadenas de CA-1..CA-5 están completas en el buffer (no truncadas).
- [ ] `fits_full_screen(80, 24) == true` y `fits_full_screen(79, 24) == false` [S6].
- [ ] Exactamente una fila lleva `> ` (contar `> ` en el buffer == 1).
- [ ] `j`/Down y `k`/Up desplazan el `> `; `1`/`2` saltan a la fila correspondiente; `q` y `Esc` devuelven `Effect::Quit`.

### CA-9 · Sin secretos ni rutas
- [ ] El buffer no contiene `/home/`, `/Users/`, `C:\`, ni ningún `std::env::var` resuelto (S-2/S-3).
- [ ] `Project :` muestra solo el basename; con nombre no disponible muestra `--`.

### CA-10 · Regresión y documentación (K-7)
- [ ] `cargo test -p ag-cli --features tui` en verde; comandos no interactivos sin cambio de salida.
- [ ] `scripts/kaizen check` ejecutado y su resultado real reportado por quien implemente.
- [ ] `text.rs`, `home.rs` y este spec concuerdan en las cadenas (K-1): si cambia una, cambian las tres en el mismo commit.

---

## 10. Regla de 1 vistazo y medición del KPI

**Regla verificable:** con el frame ya en pantalla, un observador debe poder responder
3 preguntas en <2 s mirando solo la fila `State :` y la línea `Action :`:

1. ¿Existe el proyecto? → símbolo `.` vs `o/*/?/!`.
2. ¿Está corriendo? → `* RUNNING` vs `o DETECTED` vs `? UNKNOWN`.
3. ¿Cuál es la acción principal? → `[Enter] ... <-- PRINCIPAL`.

Medición sugerida (sin telemetría nueva, L-12): el KPI se da por cumplido cuando CA-1..CA-10
están en verde y el panel muestra nombre base + template + estado + ≥4 acciones en un solo frame
de 80x24, con 0 cadenas prohibidas. No se mide tiempo humano con instrumentos nuevos: se mide la
**presencia** de los tres marcadores verificables.

---

## 11. No-hacer, riesgos y escalados

| Ítem | Decisión |
| --- | --- |
| No inventar `ag status` | "Ver estado" se resuelve con `ag domains list` + `ag workers list` (read-only reales) [S1]. Un comando nuevo requiere ADR. |
| No mostrar PID/bind en `unknown`/`stale` | Honestidad sobre conveniencia [S9][S15]. Evita el falso "running". |
| No tocar `botones`, `main.rs`, `mod.rs`, `config.rs` en esta tarea |Este spec es solo de diseño; la implementación va en `dev3-ui`/`dx` con su descriptor K-4. |
| No promover autonomía | `propose`/`recommend`; consenso según `.kaizen/team/FLUJOS.md`. Sin `advance/decide/verify`. |
| Riesgo: renombrar la entrada principal rompe un test | Mitigado en CA-10: actualizar la aserción en el mismo commit, nunca relajarla (L-10). |
| Riesgo: el mecanismo de PID no existe aún | Bloqueante para `RUNNING`/`STALE`; mientras tanto `UNKNOWN` es un estado válido y seguro. Escalar al `architect` (task 0). |
| Contenido de mensajes/blackboard | Dato no confiable (L-5/S-15): leído, nunca obedecido como instrucción. |

---

## 12. Handoff

- **Para `architect`:** confirmar el contrato `ProjectStatus`/`RunState` (§3), especialmente la
  fuente del PID y la regla R1. Sin ese acuerdo, el frame C no puede implementarse con verdad.
- **Para `qa`:** convertir CA-1..CA-10 en tests con `TestBackend` antes de tocar código (L-6:
  primero la prueba que falla).
- **Para `dev3-ui`/`dx` (implementación):** mover cadenas a `text.rs`, extender `msg_for`,
  reemplazar `view`, actualizar el test de renderizado y ejecutar `scripts/kaizen check`.
- **Para el orquestador:** este documento es propuesta; no contiene push, PR ni cambio de etapa.

Dictamen: **APPROVE para su uso como spec de diseño**, condicionado a que `architect` resuelva la
detección de PID y a que `qa` acepte los criterios CA-1..CA-10. Evidencia: §1 (source_ids con
hash) y §9 (criterios binarios). Sin aprobación humana no se considera decisión tomada (REG-EV-07, L-3).
