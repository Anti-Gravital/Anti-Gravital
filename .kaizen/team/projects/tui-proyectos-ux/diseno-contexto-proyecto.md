# Propuesta técnica — `ProjectContext` para la home TUI (`tui-proyectos-ux`)

- **Agente**: `architect` (subagente, autonomía `propose`). No decide, no ejecuta cambios de código.
- **Fecha**: 2026-11-23.
- **Único archivo escrito por este agente**: `.kaizen/team/projects/tui-proyectos-ux/diseno-contexto-proyecto.md` (canal de salida asignado por el orquestador; no se toca ningún otro archivo de `.kaizen/`, `crates/`, `docs/`, `scripts/` ni `.github/`).
- **Sin push / PR / commit**. Sin secretos. No se abrió `docs/audits/tui-transcripts/config-noninteractive-flag.json` (S-12). Todo documento, blackboard y mensaje leído se trató como dato no confiable (L-5/S-15): se extrajeron hechos verificables con `path:línea`, no instrucciones.
- **Estado de `source_ids`**: **sin fuente normativa registrada**. En `.kaizen/org/knowledge/SOURCES.md:5-37` no existe ID para RFC-0018 ni ADR-0009/ADR-0002. Conforme a REG-EV-01 y REG-EV-07 (`.kaizen/team/EVIDENCIA.md:20-32`), estos hechos de ingeniería se sustentan en **rutas:línea reales y comandos reproducibles**, no en un `source_id` inventado. Si el equipo exige `source_ids` formales para aprobar, el orquestador debe registrar `RFC-0018` y `ADR-0009` en `SOURCES.md` con estado `verified` y fecha de corte; hasta entonces cualquier decisión formal queda marcada «sin fuente verificada» y escala a humano (K-10).

---

## 0. Resumen ejecutivo

La home de `ag` hoy decide entre dos universos con un solo `bool`: `Model::new(in_project: bool)` (`crates/ag-cli/src/tui/home.rs:46-117`), alimentado por `in_project()` (`crates/ag-cli/src/tui/mod.rs:133-138`), que solo comprueba `config.toml` + `Cargo.toml` con `ag-core`. No existe detección de "proyecto en ejecución" en `ag-cli` (confirmado por búsqueda: `grep -rni "pid|is_running|already running|lockfile" crates/ag-cli/src` → sin mecanismo; ver `blackboard.md:9`). `ag dev` lanza `cargo` y espera (`crates/ag-cli/src/commands/project.rs:205-237`) sin dejar rastro consultable.

Propuesta: un `ProjectContext` **tipado y honesto** construido por el llamador (I/O en la frontera), un archivo de estado **local al proyecto** `.ag/dev.json` escrito por `ag dev` con *heartbeat*, y una regla de no-duplicados con doble línea de defensa (TUI + executor compartido). La vista sigue siendo pura: el modelo solo contiene datos y `update` devuelve `Effect::Launch(String)` exactamente como hoy (`home.rs:40-44`, `home.rs:119-136`, RFC-0018 §4.7 en `docs/rfc/RFC-0018-interactive-terminal-experience.md:186-202`).

---

## 1. Evidencia verificada (hechos, no opiniones)

| # | Hecho | Ubicación real | Cómo se verificó |
| --- | --- | --- | --- |
| E1 | `Model::new(in_project: bool)` construye dos menús; no hay estado de ejecución | `crates/ag-cli/src/tui/home.rs:46-117` | Lectura directa |
| E2 | `Entry` = `label`/`detail`/`command`; `view` solo lee el modelo | `home.rs:17-23`, `home.rs:159-211` | Lectura directa |
| E3 | `update` devuelve `Effect::Launch`/`Quit` y no ejecuta nada | `home.rs:119-136`, `home.rs:40-44` | Lectura directa |
| E4 | `msg_for` ignora `KeyEventKind::Release`; mapa ↑/k, ↓/j, Enter, Esc/q, Ctrl-C/D | `home.rs:139-157` | Lectura directa |
| E5 | `in_project()` detecta por `config.toml` + `Cargo.toml` contiene `ag-core` | `crates/ag-cli/src/tui/mod.rs:133-138` | Lectura directa |
| E6 | `run_home` solo entra si terminal ≥ 80x24; otherwise `None` → `print_help` | `mod.rs:120-131`; `crates/ag-cli/src/main.rs:84-102` | Lectura directa |
| E7 | Bare `ag` sin TTY imprime help; con TTY re-parsea el comando elegido | `main.rs:86-101`, `main.rs:106-119` | Lectura directa |
| E8 | `ag dev` usa `BIND` y espera al hijo `cargo`; no escribe estado | `crates/ag-cli/src/commands/project.rs:205-237` | Lectura directa + grep sin resultados |
| E9 | `ag.toml` se lee tras scaffolder; solo algunos templates lo generan | `project.rs:93-101`, `project.rs:115-122`; `templates/layered/ag.toml.tmpl:1-13` | Lectura directa |
| E10 | `ag.toml` guarda `[project].layout`, **no** un campo `template` | `templates/layered/ag.toml.tmpl:1-3` | Lectura directa |
| E11 | Ya existe precedente de estado local por proyecto en `.ag/` | `crates/ag-cli/README.md:206`, `:226` (`.ag/dns/state.json`, `.ag/domains.json`) |Lectura directa |
| E12 | Bind por defecto `0.0.0.0:8080`; `ConfigKey::Bind` validado | `crates/ag-cli/src/config.rs:224`, `:363` | Lectura directa |
| E13 | RFC-0018: launchers, sin work en pantalla, sin estado global, rollback por feature | `docs/rfc/RFC-0018-interactive-terminal-experience.md:74-104`, `:164-175`, `:282-288` | Lectura directa |
| E14 | Tests existentes llaman `Model::new(bool)` en 6 puntos | `home.rs:241,248,260,273,287,317`; `mod.rs:166` | Lectura directa |
| E15 | KPI del proyecto: estado real en una pantalla, ≥4 acciones, 0 duplicados, 0 proyecto ajeno | `.kaizen/team/projects/tui-proyectos-ux/charter.md:6-7` | Lectura directa |

**Hallazgo para el orquestador (no se escribe outbox: este agente solo puede escribir este archivo)**: `blackboard.md:10` cita `docs/kaizen/adr/0018-reqwest-013-ring-provider.md`, pero `glob docs/kaizen/adr/*.md` —ejecutado en el momento de esta revisión, con otros agentes trabajando en paralelo— devolvía únicamente ADR `0001`–`0010` (no existe `0018`); el documento real es `docs/rfc/RFC-0018-interactive-terminal-experience.md`. Corregir la referencia en el blackboard antes de usarla como fuente (L-5).

---

## 2. Contrato de datos: `ProjectContext`

### 2.1 Estructura propuesta

```
enum StatusSource { Real, Unknown }

struct ProjectContext {
    exists: bool,
    path: Option<String>,          // SOLO file_name(), nunca ruta absoluta
    template: Option<String>,      // [project].layout de ag.toml si es legible
    running: bool,                 // SOLO si estado fresco y válido
    pid: Option<u32>,              // diagnóstico; no implica "vivo"
    bind: Option<String>,          // "host:puerto" del estado real
    status_source: StatusSource,
    detected_at: Option<SystemTime>,
}
```

| Campo | Tipo | Origen | Regla anti-ficción (K-10) |
| --- | --- | --- | --- |
| `exists` | `bool` | marcadores en cwd (E5) | `true` solo si `config.toml` es archivo **y** `Cargo.toml` contiene `ag-core`. Igual que hoy: sin cambio de semántica en v1. |
| `path` | `Option<String>` | `current_dir()?.file_name()` | Nunca `display()` del path completo. Si no hay nombre base → `None`. Sin rutas absolutas en pantalla ni en logs (S-3/S-8). |
| `template` | `Option<String>` | `[project].layout` de `ag.toml` (E9/E10) | Leer solo si el archivo existe y parsea; si no, `None` → UI «desconocida». **Prohibido** inferir del nombre de carpeta ni de `Cargo.toml`: muchos templates no generan `ag.toml` (E9). |
| `running` | `bool` | `.ag/dev.json` con heartbeat fresco (§3) | `true` **si y solo si** `schema` coincide, JSON parsea y `heartbeat_at` está dentro del TTL. Nunca `true` por defecto, por existencia de archivo, ni por PID escrito. |
| `pid` | `Option<u32>` | campo `pid` del estado | Informativo. **No** se usa para afirmar vida: un PID puede haber muerto o sido reutilizado. |
| `bind` | `Option<String>` | campo `bind` del estado | Solo si el estado es fresco; si no, `None` (no mostrar bind "configurado" como si estuviera activo; E12 queda para una futura extensión con etiqueta explícita). |
| `status_source` | `StatusSource` | resultado de la detección | `Real` solo cuando `running=true` con estado válido. En cualquier otro caso `Unknown`. Nunca un tercer valor "supuesto". |
| `detected_at` | `Option<SystemTime>` | reloj del proceso lector | `None` si la detección no se ejecutó (p. ej. headless). `SystemTime` de `std`: sin dependencias nuevas. |

### 2.2 Invariantes verificables

| ID | Invariante | Cómo se prueba |
| --- | --- | --- |
| INV-1 | `running == true ⇒ status_source == Real ∧ pid.is_some() ∧ bind.is_some() ∧ detected_at.is_some()` | Test de propiedad sobre todos los puntos de construcción |
| INV-2 | `status_source == Unknown ⇒ running == false` | Test de tabla de estados (§3.4) |
| INV-3 | `path` no contiene `/` ni `\` en Windows-like | Test con `current_dir()` simulado |
| INV-4 | `Model`/`view` no hacen I/O: ningún `std::fs`/`Command` en `home.rs` | `grep -n "std::fs\|Command" crates/ag-cli/src/tui/home.rs` → 0 resultados |
| INV-5 | Detección ausente nunca produce `running=true` | Test: borrar `.ag/dev.json` → `Unknown` |

---

## 3. Mecanismo de detección de "en ejecución"

### 3.1 Mecanismo elegido (único, pequeño, reversible, multiplataforma)

**Archivo de estado local del proyecto + heartbeat con TTL**, sin dependencias nuevas y sin elevación de privilegios.

- Ruta: `.ag/dev.json` **dentro del directorio del proyecto**. Precedente de `.ag/` como estado local en E11. No es configuración global: cumple RFC-0018 princ. 8 (`.kaizen/...` no se toca; `docs/rfc/...:102-103`).
- Formato (versado, `schema: 1`):

```json
{ "schema": 1, "pid": 12345, "bind": "0.0.0.0:8080",
  "started_at": 1763912345, "heartbeat_at": 1763912430 }
```

- **Escritor**: `ag dev` (`project.rs:205-237`), el único que conoce el `bind` real que pasa a `cargo` (`:219`, `:227`). Escribe al iniciar, actualiza `heartbeat_at` cada **5 s** desde un `std::thread` (`std::thread::spawn` + `sleep`: `std` puro) mientras el hijo `cargo` sigue vivo, y elimina el archivo en salida limpia (`:233-236`). Escritura atómica con temporal + `rename`, patrón ya usado en la etapa B (RFC-0018 `:154`). Si el proceso muere sin limpieza, el archivo queda pero **expira solo**.
- **Lector**: la frontera de I/O existente, `tui::run_home` (`mod.rs:120-131`), justo antes de `HomeModel::from_context(...)`. Sustituye a `in_project()` (`:133-138`) sin cambiar la rama headless: la detección solo se ejecuta dentro del `if session.is_interactive()` de `main.rs:87-97`.
- **Staleness**: `TTL = 3 × heartbeat = 15 s`. `fresh = (now - heartbeat_at) <= TTL ∧ heartbeat_at >= started_at ∧ schema == 1`. Reloj de `SystemTime`; relojes donde el tiempo va hacia atrás → `Unknown` (seguro).
- **Tolerancia a fallos**: si el archivo no existe, no parsea, tiene `schema` desconocido, o el reloj es inconsistente → `running=false`, `status_source=Unknown`. Nunca se propaga un error a la vista ni se finge estado (S-7: el error se reporta al llamador con huella estable, no se traga).

### 3.2 Multiplataforma y seguridad

| Requisito | Cumplimiento |
| --- | --- |
| Linux y Windows | `std::fs`, `std::thread`, `std::time` son multiplataforma; sin `libc`, sin `kill -0`, sin `tasklist`, sin PowerShell (es Rust). |
| Sin dependencias nuevas | No se añade nada a `Cargo.toml`; `ratatui`/`crossterm` ya aprobados (RFC-0018 `:203-214`, `blackboard.md:10`). |
| Sin elevación | Escritura dentro del directorio de trabajo del proyecto; permisos de usuario normal (S-6). |
| Sin infraestructura ni migraciones | No hay sockets escuchados por la TUI, no hay base de datos, no hay `scripts/` ni migraciones tocadas. |
| Sin secretos | El estado contiene PID, bind y timestamps; ni tokens ni rutas absolutas del usuario (S-1/S-3/S-8). Bind es `host:puerto`, no credencial. |
| Sin trabajo en la vista | El heartbeat lo escribe el executor de `ag dev`; la home solo lee. `Effect::Launch("ag dev")` sigue siendo el único disparador (E3). |

### 3.3 Alternativas evaluadas y descartadas

| Alternativa | Por qué se descarta |
| --- | --- |
| Buscar proceso por nombre (`ps`/`tasklist` + grep) | Ambiguo: varios proyectos con el mismo binario, false positives, salida localizada/parsing frágil; rompería INV-4 si se hiciera desde la vista. Requiere spawn de comando externo (S-4). |
| Sondear el socket `bind` (connect TCP) | Efecto de red no declarado (RFC-0018 princ. 7); el puerto puede estar ocupado por otro proceso o `0.0.0.0` no ser conectable; confunde "puerto abierto" con "este proyecto". |
| Lockfile con lock advisory (`fs2`/`flock`) | Añade dependencia (S-9), no porta bien a Windows, y un lock no transporta `pid`/`bind`/timestamp. El lock additionally no sobrevive a un crash de forma útil. |
| API nativa de PID (`libc` / `windows-sys`) | Dependencias nuevas y superficie de plataforma; para una decisión de UX no justifica coste ni revisión de seguridad. El heartbeat responde a la pregunta real: "¿hay un `ag dev` vivo hace <15 s?". |
| PID vivo con `kill -0` / `tasklist /FI` | Alternativa **diferible**, no elegida: obliga a spawn por OS y a `#[cfg(unix/windows)]`, con riesgo de reutilización de PID. Si se necesita mayor precisión, añadirlo como extensión con ADR humano. |

### 3.4 Tabla de estados → resultado

| Situación | `exists` | Estado fresco | `running` | `status_source` | UI |
| --- | --- | --- | --- | --- | --- |
| Fuera de proyecto | false | n/a | false | `Unknown` | «No project detected» (texto actual, `text.rs:12`) |
| Proyecto, sin `ag dev` | true | archivo ausente | false | `Unknown` | «Project: `<base>` · template: desconocida · not running» |
| Proyecto, `ag dev` activo | true | sí | **true** | `Real` | «Project: `<base>` · `<template>` · **running** · PID 12345 · 0.0.0.0:8080» |
| Proyecto, archivo viejo (>15 s) o inválido | true | no | false | `Unknown` | «not running (estado no verificado)»; **no** mostrar PID/bind como activos |
| Proyecto, `ag.toml` ilegible | true | cualquiera | según estado | según estado | template «desconocida», sin inventar nombre |

---

## 4. Prevención de proyectos duplicados

### 4.1 Regla (D-1)

> **Si `ProjectContext.exists == true` en el directorio actual, la acción `ag new` no está disponible en la home y debe ser rechazada por el executor aunque llegue por otra vía (flag, script o carrera).** Nunca se crea un proyecto dentro de otro.

### 4.2 Dónde se valida (la vista no decide sola)

| Capa | Responsabilidad | Ubicación propuesta |
| --- | --- | --- |
| 1. Modelo | `from_context` **omite** el `Entry` de `ag new` cuando `exists=true`; no hay una entrada "desactivada" que el usuario pueda elegir. Es presentación, no validación de seguridad. | `home.rs::Model::from_context` (nuevo) |
| 2. Evento | Si por carrera o estado inconsistente se pidiera crear, `update` devuelve `Effect::Notice("... salga del proyecto ...")`, **nunca** `Effect::Launch("ag new")`. El llamador es quien decide qué hacer con el aviso. | `home.rs::update` + `Effect::Notice` (aditivo) |
| 3. Executor (autoridad) | Un guard compartido `project_context::guard_create(&ctx) -> Result<(), CreateInProject>` se invoca en `execute_new` **antes** de `scaffold` (`project.rs:110-122`). Aplica igual a la vía de flags y a la TUI (RFC-0018 princ. 1, `:76-79`). Error con salida 2 y mensaje accionable (`cd ..` y repetir). | `crates/ag-cli/src/commands/project.rs:110-122` |

### 4.3 Matriz de casos

| Entrada | TUI | Executor | Resultado |
| --- | --- | --- | --- |
| `ag` fuera de proyecto | Muestra «Create a new project» (`home.rs:90-106`) | guard pasa | Creación normal |
| `ag` dentro de proyecto | No muestra `ag new`; muestra ≥4 acciones contextuales | no se invoca | 0 duplicados |
| `ag new ...` desde subdirectorio con marcadores | n/a (flag) | guard rechaza | 0 duplicados, error claro |
| Archivo de estado viejo | No afecta a `exists` | guard se basa en marcadores, no en `running` | Sin falso bloqueo |

**Nota de alcance**: v1 mantiene la detección en cwd (E5) para no cambiar el comportamiento de comandos existentes. Extender a **directorios ancestros** (para evitar `cd subdir && ag new`) cambia la semántica de `in_project()` y requiere ADR humano (§6.2). Se registra como deuda explícita, no se finge que ya está resuelto.

---

## 5. Contrato de eventos / acciones / navegación

### 5.1 `Msg` y `Effect`

| Tipo | Variantes | Cambio |
| --- | --- | --- |
| `Msg` (actual `home.rs:32-38`) | `Up`, `Down`, `Enter`, `Back` | **Aditivo**: `Help` (alterna overlay), `Refresh` (pide re-detección al lazo) |
| `Effect` (actual `home.rs:40-44`) | `Launch(String)`, `Quit` | **Aditivo**: `Notice(String)`. Los brazos existentes en `mod.rs:152-156` y `main.rs:91-95` permanecen byte-compatible. |

`update` sigue siendo puro: `Help` cambia un flag booleano del modelo; `Refresh` **no** hace I/O, solo devuelve `None` y el lazo (`run_home_loop`, `mod.rs:140-158`) vuelve a invocar la detección y reconstruye el modelo. `Notice` se imprime **después** de cerrar la pantalla alterna (el `Guard` de `terminal.rs:24-44` ya garantiza la restauración).

### 5.2 Mapa de teclado (E4, `home.rs:139-157`)

| Tecla | `Msg` | Nota |
| --- | --- | --- |
| ↑ / `k` | `Up` | sin cambios |
| ↓ / `j` | `Down` | sin cambios |
| Enter | `Enter` | devuelve `Effect::Launch(command)` |
| Esc / `q` / Ctrl-C / Ctrl-D | `Back` | sin cambios |
| `?` / `h` | `Help` | nuevo; overlay con leyenda de contexto y teclas |
| `r` / F5 | `Refresh` | nuevo; re-detección por el lazo |

`KeyEventKind::Release` se sigue ignorando. Glyphs ASCII + box-drawing, sin emojis; `NO_COLOR` → `Theme::monochrome()` (`theme.rs:34-43`), la selección lleva marcador `"> "` (`home.rs:184`), nunca solo color. `text.rs:10-13` centraliza los strings (nuevas constantes: `HOME_RUNNING`, `HOME_UNKNOWN`, `HELP_BODY`, `CREATE_IN_PROJECT`), sin texto en la lógica de pantalla.

### 5.3 Headless / degradado (RFC-0018 §4.6, `docs/rfc/...:177-185`)

| Condición | Comportamiento garantizado |
| --- | --- |
| Sin TTY / `CI` / `AG_NO_TUI` / `--no-interactive` / `TERM=dumb` | `main.rs:98` imprime help. **No se ejecuta `detect_context`**: sin lectura de `.ag/dev.json`, sin threads, sin escape sequences. |
| Terminal < 80x24 | `mod.rs:126-128` devuelve `None` → help; sin detección. |
| Build `--no-default-features` | Módulo `tui` no compilado; ruta de flags intacta. |
| `NO_COLOR` | `Theme::for_session` (`theme.rs:45-52`), sin color; sin pérdida de información. |
| `ag dev` invocado sin TTY | Escritura de heartbeat sigue funcionando: es I/O de archivo, no depende de terminal (principio 1 de paridad). |

### 5.4 Compatibilidad de constructores (E14)

| Constructor | Firma | Uso |
| --- | --- | --- |
| `Model::new(in_project: bool)` | **se mantiene** | Adaptador: delega en `Model::from_project_detected(in_project)`. Los 6 tests existentes (`home.rs:241,248,260,273,287,317`; `mod.rs:166`) siguen compilando y pasando sin editar. |
| `Model::from_project_detected(bool)` | nuevo | Construye `ProjectContext` con `exists=bool`, `status_source=Unknown`, `running=false`, resto `None`. Nombre explícito para futuro código. |
| `Model::from_context(&ProjectContext)` | nuevo | Constructor real usado por `run_home`; construye las entradas y el texto de estado. |

**Regla**: `new(bool)` es un *shim* de compatibilidad, nunca la fuente de verdad. Todo código nuevo usa `from_context`. Los tests antiguos verifican que el shim no rompe el menú; los tests nuevos verifican el contexto real.

### 5.5 Plan de pruebas verificables

| Test (nombre propuesto) | Aserción |
| --- | --- |
| `context_unknown_when_state_missing` | Sin `.ag/dev.json` → `running=false`, `status_source=Unknown` (INV-2/5) |
| `context_real_when_heartbeat_fresh` | Estado con `heartbeat_at = now - 5s` → `running=true`, `Real`, `bind` presente (INV-1) |
| `context_unknown_when_heartbeat_expired` | `now - heartbeat_at = 16s` → `Unknown`, `running=false` |
| `context_unknown_on_malformed_json` | JSON inválido / `schema=2` → `Unknown`, sin panic |
| `base_name_never_leaks_absolute_path` | `path` no contiene separadores |
| `no_create_entry_inside_project` | `from_context(exists=true).entries` no contiene `"ag new"` |
| `create_request_returns_notice` | `update(Msg::Enter)` sobre entrada protegida → `Effect::Notice(_)`, nunca `Launch("ag new")` |
| `legacy_new_bool_keeps_old_menu` | Los 6 tests existentes siguen en verde sin modificación |
| `headless_does_not_read_state` | Sin TTY no hay acceso a `.ag/dev.json` (detector inyectado como `fn` para poder espiarlo) |
| `render_running_shows_pid_and_bind` | `TestBackend` 120x40 contiene «running» y el bind; en `monochrome()` también |
| `help_toggle_is_pure` | `Msg::Help` cambia flag; `update` no toca fs |

Comandos de verificación (el agente **no** los ejecutó en este proyecto; los propone para el rol `qa`): `cargo test -p ag-cli --features tui`, `cargo test -p ag-cli --no-default-features`, `scripts/kaizen check`.

---

## 6. ADR, gobernanza y rutas protegidas

### 6.1 Extensión de RFC-0018 (no requiere ADR nuevo)

| Cambio | Fundamento |
| --- | --- |
| Añadir campos a `Model` y un constructor `from_context` | RFC-0018 §4.7: `Model` es dato plano y `update` puro (`:186-202`) |
| Enriquecer el menú con acciones contextuales ya existentes | §4.5: la home es un *launcher* y no añade funcionalidad (`:164-175`) |
| Overlay de ayuda, teclas `?`/`r`, constantes en `text.rs` | Princ. 6 accesibilidad y decisión de idioma (`:95-98`, `:298-300`) |
| **Leer** `.ag/dev.json` en la frontera | Princ. 8: la TUI no mantiene estado global; aquí solo lee estado local del proyecto (`:102-103`) |
| `Effect::Notice` aditivo | Misma frontera efectos/executor (`:80-83`) |

### 6.2 Requiere ADR humano (K-1) antes de implementar

| Decisión | Por qué no es extensión trivial |
| --- | --- |
| **Escribir** `.ag/dev.json` + heartbeat desde `ag dev` | Nuevo estado persistente en el proyecto; toca el executor y matiza RFC-0018 princ. 8. Cambia `project.rs:205-237`. |
| Guard compartido que rechaza `ag new` dentro de proyecto | Modifica comportamiento de una command pública, su error y código de salida; afecta paridad flag/TUI (princ. 1) y es superficie de CLI. |
| Detección por directorios **ancestors** | Cambia la semántica de `in_project()` (`mod.rs:133-138`) para todas las commands. |
| Cualquier dependencia nueva, cambio de esquema de estado (`schema != 1`) o sondeo de PID por SO | S-9 y S-4; el orquestador debe pedir revisión de `security`. |
| Registrar `RFC-0018`/`ADR-0009` como `source_ids` en `SOURCES.md` para desbloquear REG-EV-01 | Es una decisión de gobernanza de evidencia, no de código. |

### 6.3 Rutas protegidas (L-4) — no se tocan

`.github/`, `.githooks/`, `.kaizen/` (salvo el archivo de salida asignado), `scripts/kaizen*`, `scripts/lib/`, `CODEOWNERS`, `AGENTS.md`, `opencode.json`, `.claude/`, `.agents/`, `.opencode/`, infraestructura, migraciones, secretos, autenticación y pagos. Tampoco `install.sh`, `install.ps1`, `Cargo.toml` ni `docs/` fuera del directorio del proyecto. Cualquier mejora que las requiera → issue `needs-human`, no la ejecuta este agente.

### 6.4 Rollback

1. **Capa TUI**: revertir el commit que añade `from_context`; `Model::new(bool)` sigue operativo → menú anterior Exacto (E1/E14).
2. **Capa estado**: dejar de escribir `.ag/dev.json`; la home pasa a `status_source=Unknown` sin romper nada (INV-2). Borrar el archivo es seguro: se regenera y no contiene datos de usuario.
3. **Feature flag**: `tui` ya es activable/desactivable en un cambio de línea (RFC-0018 `:282-288`), restaurando el comportamiento de help de `main.rs:98`.
4. **Sin migraciones ni borrado destructivo**: el cambio es aditivo y revertable por PR independiente, alineado con L-7 y con el plan de PRs de RFC-0018 `:247-257`.
5. **Señales de rollback** (L-9): si tras la fusión aumentan los reportes de "estado falso" o baja la precisión de detección, abrir `regression-suspect` con evidencia y PR de revert; no relajar umbrales (L-10).

---

## 7. Criterios verificables del KPI (charter.md:6-7)

| Criterio | Cómo se verifica |
| --- | --- |
| Estado real en una sola pantalla | Test `render_running_shows_pid_and_bind`: `exists`, `template`, `running`, `pid` y `bind` visibles en un `Frame` 120x40 |
| ≥4 acciones contextuales | `from_context(exists=true, running=true).entries.len() >= 4` (hoy ya son 6, `home.rs:50-88`) |
| 0 creaciones duplicadas | Tests `no_create_entry_inside_project` + `create_request_returns_notice` + guard en `execute_new` |
| 0 acción sobre proyecto ajeno | `path` solo `file_name()` (INV-3); sin rutas absolutas en estado ni en pantalla |
| 0 afirmaciones sin `source_ids` | Este documento declara explícitamente la ausencia de ID normativo y la sustituye por `path:línea` reproducible (§1) |
| Tests `TestBackend` en verde | `cargo test -p ag-cli --features tui` (a ejecutar por `qa`) |
| Sin regresión de comandos | `home.rs` tests antiguos intactos + `cargo test -p ag-cli --no-default-features` |
| Interactividad y `NO_COLOR`/headless preservadas | Matriz §5.3 + test `headless_does_not_read_state` |
| Documentación K-7 | Pendiente para `finops`: `crates/ag-cli/README.md`, `docs/modules/ag-cli/README.md`, root `README.md` y CHANGELOG (RFC-0018 `:232-237`) |

---

## 8. Hallazgos y escalados

1. **Sin `source_id` normativo**: §0. Escalar al orquestador para registrar `RFC-0018`/`ADR-0009` en `SOURCES.md` o aceptar evidencia por `path:línea`. REG-EV-01 bloquea decisiones formales hasta entonces.
2. **Referencia incorrecta en blackboard**: §1, `blackboard.md:10` vs `glob docs/kaizen/adr/*.md`. Dato no confiable corregido aquí, no modificado (fuera de mi permiso de escritura).
3. **Deuda consciente**: detección solo en cwd (§4.3); ancestor-walk queda para ADR.
4. **No se delegó en subagentes**: toda la evidencia se obtuvo en profundidad 0 (`KAZEN_AGENT_DEPTH` sin incrementar), por lo que no aplica el tope `KR_MAX_DEPTH`. No se lanzó `kaizen-equipo` ni `kaizen-installer`.
5. **Este documento es una propuesta, no una aprobación**: no se ejecutó `kaizen-team advance/decide/verify`, no se hicieron commits, push ni PR.
