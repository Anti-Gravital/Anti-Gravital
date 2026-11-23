# ADR-0014: estado local de `ag dev` y guard de creación dentro de un proyecto

Estado: aceptado
Fecha: 2026-11-23
Decisor: Angel Nereira (BDFL)

Aprobación humana explícita otorgada el 2026-11-23 para el proyecto `tui-proyectos-ux`. Extensión de RFC-0018: la vista sigue siendo un launcher puro; este ADR autoriza únicamente el estado local y el guard compartido.

## Contexto

La home TUI de `ag` solo distingue «dentro/fuera de proyecto» con un bool (`crates/ag-cli/src/tui/home.rs:46-117`, `tui/mod.rs:133-138`). No existe mecanismo para detectar un proyecto **en ejecución** (`grep -rni "pid|is_running|already running|lockfile" crates/ag-cli/src` → sin resultados). `ag dev` lanza `cargo` y espera (`commands/project.rs:205-237`) sin dejar rastro consultable. El usuario pidió una interfaz más intuitiva que trate mejor los proyectos ya creados y en ejecución, con mejor UX/DX.

Ya existe precedente de estado local por proyecto en `.ag/` (`crates/ag-cli/README.md:206,226`). `serde`/`serde_json` ya son dependencias de `ag-cli`; no se añade ninguna.

## Decisión

1. `ag dev` escribe `.ag/dev.json` dentro del directorio del proyecto con `schema=1`, `pid`, `bind`, `started_at` y `heartbeat_at` (segundos Unix). Actualiza el heartbeat cada 5 s mediante un hilo `std` mientras el hijo `cargo` sigue vivo, y elimina el archivo en salida limpia. Escritura atómica con temporal + `rename`. Sin elevación de privilegios y sin rutas absolutas ni secretos en el estado.
2. La home TUI **lee** ese archivo en la frontera (`tui::run_home`) y construye un `ProjectContext` tipado. `running=true` solo si `schema=1`, el JSON parsea y `heartbeat_at` está dentro de un TTL de 15 s; en cualquier otro caso el estado es `Unknown` y la vista no afirma «running», no muestra PID ni bind como activos (anti-ficción, K-10).
3. Regla D-1: si el proyecto existe en el directorio actual, la home no ofrece `ag new` y un guard compartido en `execute_new` rechaza la creación también por flags, con error accionable y salida 2. Cero proyectos duplicados.
4. Se preservan los 9 tests existentes de `home.rs`/`mod.rs` mediante el adaptador `Model::new(bool)`; se mantiene el modo headless, `--no-default-features`, `NO_COLOR` y el fallback <80x24.

## Alternativas rechazadas

- Buscar el proceso por nombre (`ps`/`tasklist`): ambiguo y frágil; spawn de comando externo (S-4).
- Sondear el socket: confunde «puerto abierto» con «este proyecto» y añade red no declarada.
- Lockfile con `fs2`/`flock` o PID vivo con `kill -0`/P-Invoke: dependencias nuevas o plataforma específica, sin beneficio para la pregunta real.
- Mostrar `running` por la mera existencia del archivo: falso estado (K-10).

## Consecuencias

- Positivas: el usuario ve en una pantalla nombre base, template, si el proyecto corre y en qué bind; acciones contextuales; cero duplicados.
- Negativas: un archivo de estado más por proyecto; si el proceso muere sin limpieza, el archivo queda pero expira a los 15 s. La detección sigue limitada al directorio actual (ancestor-walk queda para otro ADR).
- Gobernanza: `RFC-0018`/`ADR-0009` no están registrados como `source_ids` en `SOURCES.md`; la evidencia de este ADR son rutas:línea reproducibles, no un ID normativo inventado (REG-EV-01).

## Rollback

1. Revertir la capa TUI: `Model::new(bool)` queda operativo y el menú vuelve al estado anterior.
2. Dejar de escribir `.ag/dev.json`; la home pasa a `Unknown` sin romper comandos. Borrar el archivo es seguro.
3. El feature `tui` sigue activable/desactivable en una línea, restaurando el help de `main.rs`.

Sin migraciones, sin infraestructura y sin borrado destructivo (L-7). Si tras la fusión aparecen reportes de «estado falso», abrir `regression-suspect` con evidencia y PR de revert (L-9); no relajar umbrales (L-10).

## Referencias

- `docs/rfc/RFC-0018-interactive-terminal-experience.md` §4.5-4.8
- `.kaizen/team/projects/tui-proyectos-ux/diseno-contexto-proyecto.md` §2-5
- `.kaizen/team/projects/tui-proyectos-ux/plan-verificacion.md` §1-3
