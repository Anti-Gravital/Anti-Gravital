# Diseño de la TUI del instalador — `instalacion-rapida-releases`

Estado: propuesta de diseño (no decisión ejecutada). Rol: `dx`. Fecha: 2026-11-23.
Alcance: `install.sh` (Linux/macOS) e `install.ps1` (PowerShell 5.1/7 en Windows).
Este documento NO modifica instaladores, CI, llaves ni dependencias. Es la entrada
para revisión de `architect`, `security` y `qa`.

## 0. Evidencia y trazabilidad (K-10 / EVIDENCIA.md)

Todo lo afirmado abajo se apoya en fuentes internas del repositorio, verificables
por ruta y comando. Se registran como fuentes internas porque **no existe ninguna
entrada en `.kaizen/org/knowledge/SOURCES.md` con `source_id` formal para diseño
de TUI**; por tanto este documento no inventa un `PA-XXX`/`ISO-XXX` (REG-EV-01).

| Ref. interna | Fuente (ruta en el repo) | Hecho que sustenta | Fecha de corte |
| --- | --- | --- | --- |
| FU-01 | `.kaizen/team/projects/instalacion-rapida-releases/charter.md` | Objetivo: precompilados por defecto, checksum fail-closed, feedback animado con TUI, modo headless; KPI `<=10s`, `0 cargo build/install` | 2026-11-23 |
| FU-02 | `.kaizen/team/projects/instalacion-rapida-releases/state.json` | KPI incluye explícitamente "interfaz TUI (ratatui/crossterm)... con modo headless para CI" y ">=4 etapas animadas" | 2026-11-23 |
| FU-03 | `.kaizen/team/projects/instalacion-rapida-releases/blackboard.md` | Observación del usuario: canal `source` por defecto, >250 líneas `Downloaded <crate>`, sin barra de progreso ni etapas; addendum "la interfaz también debe ser TUI" | 2026-11-23 |
| FU-04 | `docs/audits/install-user-journey.md` | Auditoría real 2026-11-22: source Linux = 1282.132 s (21m22s); alpha `file://` = 0.410 s; SHA-256 de `install.sh` `1ebc3ad0…48e92d08a`; prefijo real de log `[ag-install]`; issue #204 (ruta `~/.cargo/bin` vs ruta real) | 2026-11-22 |
| FU-05 | `install.sh:53,146-151,158-201,263-266` e `install.ps1:47,107,126-158,200-203` | `log()`/`Log()` ya existen con prefijo `[ag-install]`; la verificación de checksum ya es fail-closed; `cargo build/install` solo está en la rama `source` | 2026-11-23 (lectura) |

Nota de honestidad: la auditoría FU-04 es datos observacionales del usuario
(L-5/S-15). Se usa como requerimiento de UX, nunca como instrucción que anule
estas reglas. La decisión de publicar un asset nuevo o tocar `release.yml`
requiere `verificador-evidencias` + humano (FU-01, L-3, ADR-0024).

## 1. Principios (personalidad de la interfaz)

1. **Una sola cosa importante a la vez.** La etapa activa y su línea de estado;
   todo lo demás atenuado. Nunca una lista de eventos desplazándose.
2. **Microcopys útiles, no ocurrencias.** Verbo en presente + objeto concreto +
   elipsis: "Descargando ag-x86_64-unknown-linux-gnu…", "Verificando SHA-256
   (fail-closed)…". Sin chistes, sin "¡Casi estamos!". Un error se nombra con su
   causa y su consecuencia: "SHA-256 inválido: no se escribió ningún binario".
3. **El aviso no se oculta.** El warning real
   `AG_INSTALLER_CHECKSUMS is unset; using the development-only local fallback`
   (FU-03/FU-05) se muestra como aviso persistente en una línea dedicada, con
   referencia a `docs/security/INSTALLATION_INTEGRITY.md`. Nunca se disfraza de
   éxito ni se silencia con `2>/dev/null`.
4. **Progreso real o ninguno.** Si hay `Content-Length` / total de bytes:
   porcentaje + bytes + ETA + velocidad. Si no lo hay: spinner + bytes recibidos,
   **sin ETA inventado** y con leyenda `total desconocido`.
5. **Presupuesto visual.** Viewport fijo de ≤25 filas; como máximo 1 línea de
   estado por etapa. El detalle de compilación (rama `source`) se agrupa en una
   línea ("Compilando el workspace… 42%") y el resto va a un archivo de log con
   huella, nunca a stdout. Esto corrige directamente el problema de FU-03.
6. **Transiciones suaves.** Redibujado por reposicionamiento de cursor, no
   borrado de pantalla completo; spinner a ~10 fps; barra de progreso con
   *throttle* de 100 ms; sin parpadeo y sin una línea nueva por evento.

## 2. Estructura de pantalla

### 2.1 Etapas (6, cumple >=4)

| # | Etapa | Estado de éxito (texto literal en la línea) |
| --- | --- | --- |
| 1 | Resolver versión | `v0.1.0-alpha.2 desde releases (tag ...)` |
| 2 | Detectar plataforma | `linux-x86_64 · glibc` / `windows-x86_64 · msvc` |
| 3 | Descargar | `ag-<triple> (9.3 MiB) · 100%` |
| 4 | Verificar SHA-256 | `OK (digest coincide, fail-closed)` |
| 5 | Instalar | ruta **real** devuelta por la herramienta (no literal `~/.cargo/bin`; ver issue #204 en FU-04) |
| 6 | Listo | `ag 0.1.0-alpha.2 · 6.8s · 0 compilaciones` |

En rama `source`, la etapa 5 se rotula `Instalar (compilando desde fuente — puede
tardar)` y el KPI de tiempo no se aplica; se muestra honestamente.

### 2.2 Indicadores (nunca solo color)

| Estado | TUI (Unicode) | TUI sin Unicode/ANSI | Headless (texto) |
| --- | --- | --- | --- |
| pendiente | `○` | `[ ]` | `PENDIENTE` |
| activo | `◉` + spinner `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` | `[>]` + `|/-\` | `INICIO` |
| ok | `OK` | `[x]` | `OK` |
| error | `ERR` | `[!]` | `FALLO` |
| omitido | `–` | `[-]` | `OMITIDO` |

Cada símbolo va acompañado de la palabra (`OK`, `FALLO`, `PENDIENTE`). En
`NO_COLOR` se caen los atributos, no los símbolos.

### 2.3 Componentes

- **Cabecera**: producto + versión resuelta. Una fila.
- **Barra de etapas**: 6 filas, una por etapa, con indicador, nombre y detalle.
- **Barra de progreso** (solo etapa 3 o 5 con total conocido):
  `[████████████░░░░░░░░] 58%  5.4/9.3 MiB  ETA 2s  2.1 MiB/s`.
  Sin total: `[··· spinner ···] 5.4 MiB recibidos (total desconocido)`.
- **Línea de estado**: una única fila con la acción real en curso + spinner.
- **Aviso**: una fila para el warning de checksums (sección 1.3).
- **Pie**: `q cancelar   ? ayuda` (en error: `Enter reintentar   q cancelar   ? ayuda`).

### 2.4 Frames ASCII de ejemplo

**Frame A — inicio (etapa 2 activa):**

```
+----------------------------------------------------------------------+
| Anti-Gravital · instalación rápida                    v0.1.0-alpha.2  |
+----------------------------------------------------------------------+
| Etapas                                                               |
|  1 OK Resolver versión       OK (v0.1.0-alpha.2 desde releases)       |
|  2 ◉ Detectar plataforma    linux-x86_64 · glibc            ⠙         |
|  3 ○ Descargar              PENDIENTE                                 |
|  4 ○ Verificar SHA-256      PENDIENTE                                 |
|  5 ○ Instalar               PENDIENTE                                 |
|  6 ○ Listo                  PENDIENTE                                 |
+----------------------------------------------------------------------+
| Estado   Detectando plataforma y libc…                               |
| Aviso    AG_INSTALLER_CHECKSUMS no definido: fallback local de        |
|          desarrollo. Ver docs/security/INSTALLATION_INTEGRITY.md     |
+----------------------------------------------------------------------+
| q cancelar   ? ayuda                                                 |
+----------------------------------------------------------------------+
```

**Frame B — descarga (progreso con total conocido):**

```
|  2 OK Detectar plataforma    OK (linux-x86_64 · glibc)                 |
|  3 ◉ Descargar       [████████████░░░░░░░░] 58% 5.4/9.3 MiB ETA 2s 2.1 MiB/s |
|  4 ○ Verificar SHA-256      PENDIENTE                                 |
+----------------------------------------------------------------------+
| Estado   Descargando ag-x86_64-unknown-linux-gnu…                    |
```

**Frame C — verificación y éxito (estado final):**

```
|  3 OK Descargar              OK (9.3/9.3 MiB)                          |
|  4 ◉ Verificar SHA-256      comparando 64 hex…               ⠹         |
...
|  4 OK Verificar SHA-256      OK (digest coincide, fail-closed)         |
|  5 OK Instalar               OK (/home/.../.cargo/bin/ag)              |
|  6 OK Listo                  ag 0.1.0-alpha.2 · 6.8s · 0 compilaciones |
+----------------------------------------------------------------------+
| Estado   Listo. Ejecuta: ag --version                                 |
+----------------------------------------------------------------------+
```

**Frame D — error fail-closed (checksum inválido):**

```
+----------------------------------------------------------------------+
|  3 OK Descargar              OK (9.3/9.3 MiB)                          |
|  4 ERR Verificar SHA-256      FALLO (digest no coincide)                |
|  5 – Instalar               OMITIDO (verificación fallida)            |
|  6 – Listo                  OMITIDO                                   |
+----------------------------------------------------------------------+
| ERROR    SHA-256 inválido. No se escribió ningún binario.             |
|          Esperado: 7f3a…c91d   Obtenido: b204…5ee8                    |
|          Reintenta (Enter), cancela (q) o reporta con DIG-0007.        |
+----------------------------------------------------------------------+
| Enter reintentar   q cancelar   ? ayuda                               |
+----------------------------------------------------------------------+
```

El digest es un valor público del asset, no un secreto (S-1). Nunca se muestra
un token, una URL con credenciales ni el contenido de una variable de entorno.

## 3. Interacción

| Entrada | Comportamiento |
| --- | --- |
| `q` / `Esc` (TUI) | Overlay `¿Cancelar la instalación? [s/N]`. `s`/`Enter` cancela y limpia el parcial; `n`/`Esc` vuelve. Sin confirmación incidental. |
| `?` | Overlay de ayuda con la tabla de teclas y el significado de los indicadores. Se cierra con `?`/`Esc`. |
| `Enter` (en error) | Reintenta **solo** la etapa fallida (descarga o verificación), máx. 3 intentos con backoff 1/2/4 s. **Nunca** reintenta relajando la verificación ni aceptando un digest distinto. Agotados los intentos: error terminal con código de salida. |
| `Ctrl-C` (SIGINT) | Cancela de inmediato (convención Unix; sin confirmación), limpia el archivo parcial, sale con código `130`. En PowerShell, `ConsoleCancelEventArgs.Cancel = $true` + limpieza en `finally`. |
| `SIGTERM` | Misma limpieza, salida `143`. No se ignoran señales. |
| `AG_INSTALLER_UI` | `auto` (default), `tui`, `headless`. Permite forzar modo para CI/pruebas (L-7, reversible). |
| `AG_INSTALLER_REDUCE_MOTION=1` | Sin spinner: indicador estático `[>]`; se respeta también si el usuario pide accesibilidad de movimiento reducido. |

### 3.1 Matriz de modo (detección en tiempo de arranque)

| Condición | Modo |
| --- | --- |
| stdout es TTY (`[ -t 1 ]` / `! [Console]::IsOutputRedirected`) y `NO_COLOR` ausente y `TERM != dumb` y `CI` no truthy | TUI con atributos |
| TTY pero `NO_COLOR` definido | TUI sin color (símbolos + palabras) |
| `CI` truthy, o no TTY (redirección/tubería), o `TERM=dumb`, o `TERM` vacío | **headless** |
| `AG_INSTALLER_UI=headless` | headless (extrae, gana) |
| Windows conhost sin soporte VT | TUI vías APIs de consola de .NET, sin códigos ANSI |

### 3.2 Headless: líneas estables, sin color, sin redibujado

Formato canónico, prefijo `[ag-install]` (YA usado por `log()`/`Log()`, FU-05):

```
[ag-install] 1/6 Resolver versión: OK (v0.1.0-alpha.2)
[ag-install] 2/6 Detectar plataforma: OK (linux-x86_64)
[ag-install] 3/6 Descargar: INICIO (ag-x86_64-unknown-linux-gnu, 9.3 MiB)
[ag-install] 3/6 Descargar: 100% (9.3/9.3 MiB)
[ag-install] 4/6 Verificar SHA-256: OK
[ag-install] 5/6 Instalar: OK (/home/.../.cargo/bin/ag)
[ag-install] 6/6 Listo (6.8s, 0 compilaciones)
```

En error, la última línea es
`[ag-install] 4/6 Verificar SHA-256: FALLO (digest no coincide); binario no escrito; exit=3`.
Sin secuencias de escape, sin `\r`, sin caracteres de control distintos de `\n`.
Este modo es la superficie de verificación automática del KPI de `qa` (sección 6).

## 4. Plataformas y decisión de implementación

### 4.1 Decisión recomendada (honesta y reversible): TUI nativa en el propio instalador

Implementar la capa TUI **dentro de `install.sh` e `install.ps1`**, sin crates,
sin binarios nuevos y sin tocar `release.yml`:

- **bash (Linux/macOS, bash 3.2+ por compatibilidad con macOS):** `printf` con
  secuencias ANSI VT100 básicas (16 colores + atributos), `stty -echo -icanon`
  + lectura de 1 byte para teclas, `trap` para `INT TERM`, `tput cols/lines`
  cuando esté disponible con default 80x24. Sin `mapfile`/arrays asociativos
  (bash 3.2 no los tiene).
- **PowerShell 5.1/7 (Windows):**
  - Windows Terminal / PowerShell 7: ANSI nativo.
  - `conhost` clásico: usar APIs de `System.Console` (`CursorLeft`,
    `CursorTop`, `CursorVisible`, `Write` con `ForegroundColor`,
    `ReadKey($true)`), que funcionan sin ANSI. **No usar `Add-Type`**: compila
    C# en memoria y añadiría una etapa de compilación que queremos evitar.
  - PowerShell 7 en Linux reutiliza el mismo script; la detección de
    sección 3.1 decide ANSI vs APIs de consola.

Por qué esta opción y no ratatui/crossterm **ahora**:

| Opción | 0 compilaciones | ¿Toca release.yml? | Cobertura del ciclo | Veredicto |
| --- | --- | --- | --- | --- |
| TUI nativa en el script (recomendada) | Sí | No | Etapas 1-6 | **Adoptar** |
| TUI en el binario `ag` | Sí (ya precompilado) | No | Solo etapas 4-6: problema de *bootstrap*, no existe `ag` instalado durante 1-3 | Insuficiente sola; véase 4.3 |
| ratatui/crossterm como asset precompilado nuevo | Sí, si se publica precompilado | **Sí** (nuevo asset + checksum en el manifiesto) | Requiere un bootstrap en texto para descargar el propio binario TUI | Diferir a ADR con humano |

Esta decisión es reversible: la TUI es una capa de presentación detrás de
`AG_INSTALLER_UI`; desactivarla no cambia la lógica de descarga/verificación.

### 4.2 Si se adopta ratatui/crossterm en el futuro (condiciones, no autorización)

`ratatui` y `crossterm` son crates reales y mantenidos, pero **su sola mención no
cumple S-9**: antes de cualquier adopción hay que comprobar nombre exacto,
mantenimiento y versión con fecha de corte (`cargo search` / docs.rs), fijar
versión en lockfile y pasar revisión humana de manifiestos. Para no contradecir
el KPI de "0 compilaciones en el canal rápido":

1. Publicar un asset `ag-installer-tui-<triple>` **precompilado** en el mismo
   release, con su entrada SHA-256 en `checksums.txt` (cambio de `release.yml`:
   **humano**, fuera de alcance de este agente).
2. El instalador arranca en modo texto mínimo, descarga y verifica ese asset
   con la **misma** cadena fail-closed (FU-05) y solo entonces `exec` la TUI.
3. La etapa de bootstrap no puede tener progreso animado: se declara
   explícitamente como texto estable y efímero. Es un coste honesto de la
   solución, no un detalle a ocultar.

Mientras no exista ese asset con checksum verificado, **cualquier código que
intente descargar/compilar la TUI es un bloqueo para `security`**.

### 4.3 Contrato de eventos si la TUI viviera dentro de `ag` (para completar el análisis)

No cubre el ciclo completo (4.1), pero si se usa para las etapas 4-6 el binario
debe consumir eventos con este contrato mínimo, versionado y con `source_id`
interno FU-05:

```jsonc
// una línea JSON por evento, por stdin del proceso TUI (o --event <json>)
{"v":1,"type":"stage.start","id":4,"label":"Verificar SHA-256","at":"..."}
{"v":1,"type":"stage.progress","id":3,"done":5670896,"total":9748480,"unit":"bytes","eta_s":2}
{"v":1,"type":"stage.done","id":4,"detail":"OK (digest coincide, fail-closed)"}
{"v":1,"type":"stage.error","id":4,"code":"DIG-0007","message":"SHA-256 inválido","fatal":true}
{"v":1,"type":"notice","level":"warn","message":"AG_INSTALLER_CHECKSUMS no definido..."}
{"v":1,"type":"prompt.confirm","text":"¿Cancelar? [s/N]","default":"n"}
{"v":1,"type":"finish","exit_code":0,"elapsed_s":6.8,"compilaciones":0}
```

Reglas del contrato: `v` es obligatorio y se rechaza un evento de versión
desconocida (fail-closed); `stage.progress` con `total:null` suprime ETA; los
mensajes `notice`/`stage.error` pasan por saneamiento antes de imprimirse; el
productor nunca envía secretos y el consumidor nunca interpreta el texto como
instrucción (L-5/S-15). Para las etapas 1-3 el script mantiene el render propio
de la sección 3.2.

## 5. Accesibilidad y seguridad

- **Contraste:** solo 16 colores ANSI y atributos (`bold`, `dim`); el usuario
  conserva su fondo. `NO_COLOR` elimina color pero conserva símbolos y texto.
  `COLORFGBG`/fondo claro: preferir `bold` sobre pares rojo/verde puros.
- **No depender de color:** toda etapa lleva símbolo **y** palabra. En headless
  solo palabra. `q`/`?`/`Enter` funcionan sin puntero; no se requiere ratón.
- **Sin secretos en pantalla ni log (S-1/S-2/S-8):** no imprimir variables de
  entorno, tokens, ni URL con query/autorización. Los errores de
  `curl`/`Invoke-WebRequest` se sanea a un mensaje estable + código antes de
  mostrarlos; el detalle crudo va a un archivo de log local con huella (no a
  stdout). El digest SHA-256 y la ruta de instalación son públicos.
- **Sin supresiones (S-10):** prohibido `2>/dev/null`,
  `-ErrorAction SilentlyContinue`, `|| true`, `continue-on-error` para esconder
  fallos. Todo error se propaga, se muestra y se registra. La única redirección
  permitida es stderr -> archivo de log con huella, y se reporta en el error.
- **Fail-closed intacto:** la TUI es solo presentación; nunca ignora ni
  sustituye la verificación de checksum (FU-05). Un error de
  verificación produce pantalla D y salida no cero antes de escribir el binario.
- **Headless para KPI:** las líneas de la sección 3.2 son la evidencia
  reproducible que `qa` puede comparar con `grep`, sin depender de terminal.

## 6. Criterios de aceptación comprobables

Cada criterio indica qué aparece y qué NO aparece en stdout.

| # | Comando / escenario | Debe aparecer en stdout | No debe aparecer |
| --- | --- | --- | --- |
| CA-1 | `script -qec 'bash install.sh' /dev/null` con TTY | Las 6 etiquetas de etapa, un indicador de etapa activa, 1 línea de estado, `Listo`, `0 compilaciones` | Más de 25 filas visibles a la vez; una línea nueva por evento; chistes |
| CA-2 | `NO_COLOR=1 TERM=xterm-256color bash install.sh </dev/null >out.txt` | Símbolos `OK/ERR/○` + palabras `OK/FALLO/PENDIENTE` | Secuencias `\033[` (comprobar `grep -q $'\033' out.txt` -> falso) |
| CA-3 | `CI=true bash install.sh >out.txt 2>&1` (y equivalente PS) | Exactamente el formato `[ag-install] N/6 ...` de 3.2; `6/6 Listo` | Spinner, `\r`, redibujado, color |
| CA-4 | Asset con SHA-256 modificado en entorno sintético | `4/6 Verificar SHA-256: FALLO`, `no se escribió ningún binario`, `exit=3`, "OMITIDO" en etapas 5-6 | `Listo`, ruta de binario instalado |
| CA-5 | Enviar `q` en etapa 3 | Overlay `¿Cancelar la instalación? [s/N]`; con `n` continúa | Cancelación sin confirmación |
| CA-6 | Enviar `?` | Overlay con `q`, `?`, `Enter`, `Ctrl-C` | Salida del programa |
| CA-7 | Enviar `Ctrl-C` | Limpieza del parcial y salida `130` | Archivo parcial persistente; salida `0` |
| CA-8 | `AG_INSTALLER_UI=headless ...` en cualquier terminal | Modo headless | Cualquier ANSI |
| CA-9 | Captura del canal rápido | `0 compilaciones`, `6.xs` | `cargo build`, `cargo install`, `Updating crates.io index`, `Downloaded <crate>` |
| CA-10 | Cualquier captura | Ruta real devuelta por la herramienta en etapa 5 | Literal fijo `~/.cargo/bin` cuando la ruta real difiera (issue #204, FU-04) |
| CA-11 | Cualquier captura | Warning de `AG_INSTALLER_CHECKSUMS` en línea de aviso | Warning omitido o transformado en éxito |

**Códigos de salida:** `0` ok · `1` cancelado por usuario · `2` error de red ·
`3` checksum inválido · `4` permisos/ruta · `130` SIGINT · `143` SIGTERM.

## 7. Riesgos, límites y siguiente paso

- La TUI nativa depende de que las secuencias/APIs de consola estén disponibles;
  la matriz de 3.1 garantiza degradación a headless sin pérdida de información.
- La opción ratatui/crossterm queda **diferida** y ligada a un ADR con humano:
  no se descarga ni se compila nada en el canal rápido sin ese acuerdo (FU-01).
- `security` debe confirmar el saneamiento de errores de red y la ausencia de
  supresiones; `architect` debe confirmar que la capa de presentación no acopla
  la lógica fail-closed; `qa` debe ejecutar CA-1..CA-11 en Linux y Windows.
- Si `verificador-evidencias` decide que hace falta un `source_id` formal para
  esta especificación, se escala: no se fabrica uno aquí (REG-EV-01, K-10).
