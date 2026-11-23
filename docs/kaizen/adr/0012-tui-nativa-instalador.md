# ADR-0012: TUI nativa en el instalador con modo headless determinista

Estado: aceptado
Fecha: 2026-11-23
Decisor: Angel Nereira (BDFL)

Aprobación humana explícita del BDFL otorgada el 2026-11-23 para esta decisión y para su registro con estado `aceptado`. El agente documenta la decisión; no la aprueba por sí mismo (K-1, REG-EV-01).

## Contexto

La observación que originó el proyecto es una instalación real en Windows con cientos de líneas `Downloaded <crate>` y sin barra de progreso ni ETA (`blackboard.md`, dato no confiable L-5/S-15; `DEC-001` §Evidencia). El canal `source` tarda 21 m 22 s y hace dos builds (`docs/audits/install-user-journey.md:71,94`); el canal precompilado tardó 0.410 s sobre `file://` (`:135,148`, sin medir red). El KPI del proyecto exige feedback animado con al menos 4 etapas y un modo headless para CI (`charter.md`, `state.json`; `diseno-tui.md` FU-01/FU-02).

Existe una TUI en `crates/ag-cli/src/tui/` (`ratatui` 0.30 + `crossterm` reexportado, `PROP-07` en `diseno-arquitectura.md`), cuyo propio módulo declara que "una pantalla nunca ejecuta trabajo; el llamador ejecuta la petición" (`crates/ag-cli/src/tui/mod.rs:1-5`). El arquitecto propuso reutilizarla con replay de eventos desde un `EventLog`; `security` dictaminó que esa opción **ejecuta el binario descargado antes de copiarlo**, rompiendo el invariante I6 / veto V3, y además la TUI no aparece hasta después de la descarga (`revision-seguridad.md §1, §4`; `DEC-001` ítem 3). El prefijo de log real es `[ag-install]`, ya usado por `log()`/`Log()` y visible en la auditoría (`install-user-journey.md:141-148`; `diseno-tui.md` FU-05).

### Evidencia

Etiquetas: `verified` = comprobado en esta iteración con comando de solo lectura; `source-backed` = consta en fuente interna citada; `inference` = deducción; `unknown` = sin verificar. No se inventa ningún `source_id` (REG-EV-01/K-10).

| Hecho | Etiqueta | Fuente verificable |
| --- | --- | --- |
| Invariante: nunca ejecutar el binario descargado | source-backed | `revision-seguridad.md §1 I6 / V3`; `INSTALLATION_INTEGRITY.md:67-68` |
| Prefijo real `[ag-install]` | source-backed | `docs/audits/install-user-journey.md:141-148`; `diseno-tui.md` FU-05 |
| `ratatui`/`crossterm` ya presentes en `ag-cli` | source-backed | `diseno-arquitectura.md` PROP-07 (`crates/ag-cli/Cargo.toml`, `Cargo.lock`) |
| Conflicto de propuestas resuelto a favor de DX | source-backed | `DEC-001:23-26` |
| Cero dependencias y cero assets nuevos | inference | Derivado de la decisión: la TUI vive en los scripts, sin manifiestos ni `release.yml` |

## Decisión

1. **TUI nativa en el propio instalador**, sin crates, sin binarios nuevos y sin tocar `.github/workflows/release.yml`:
   - **bash (Linux/macOS, bash 3.2+ por compatibilidad):** `printf` con secuencias ANSI/VT100 básicas, `stty -echo -icanon` y lectura de un byte para teclas, `trap` para `INT`/`TERM`, `tput cols/lines` con default 80×24; sin `mapfile` ni arrays asociativos.
   - **PowerShell 5.1/7 (Windows):** API `System.Console` (`CursorLeft`, `CursorTop`, `CursorVisible`, `Write` con `ForegroundColor`, `ReadKey($true)`) para `conhost` sin VT; ANSI nativo en Windows Terminal / PowerShell 7. **Sin `Add-Type`** (compila C# en memoria y añadiría una etapa de compilación) y sin módulos NuGet.
2. **Seis etapas reales**, cubiertas desde el primer byte: `resolve_version` → `detect_platform` → `download` → `verify` → `install` → `ready`. Cada etapa muestra símbolo **y** palabra (no solo color); los estados `started`/`progress`/`succeeded`/`failed` y `bytes`/`total` solo se emiten cuando una acción real los observa (regla anti-ficción, K-10). Nunca se inventa ETA si `total` es desconocido.
3. **Modo headless determinista con formato estable.** Formato público de una línea por evento, prefijo `[ag-install]`, ASCII, sin secuencias ANSI (`\x1b`), sin timestamps, sin redibujado, con `seq` monótono:

   ```
   [ag-install] N/6 <etapa>: <estado> (<detalle>)
   ```

   `N` es el índice de etapa 1..6; la línea pública es la proyección estable del esquema interno de eventos (`seq`, `stage`, `state`, `bytes`, `total`, `code`, `source`) descrito en `diseno-arquitectura.md §3.3, §4.3`. Se adopta la forma de `diseno-tui.md §3.2` porque es la que coincide con el prefijo ya observado en los logs reales; el formato es un contrato público: cambiarlo rompe CI. En `failed`, la salida termina con exit ≠ 0, el código estable y sin datos personales ni trazas internas (S-7/S-8).
4. **Cero ejecución del binario descargado.** La TUI se dibuja en el script; el binario `ag` no se ejecuta antes de la copia atómica. `ag --version` es una instrucción para el usuario, no una acción del instalador (I6/V3).
5. **Selección y degradación.** `AG_INSTALLER_UI=auto|tui|headless` decide en arranque: headless si `CI`, salida no TTY, `TERM=dumb` o `TERM` vacío; TUI en caso contrario. Se respetan `NO_COLOR` (caen atributos, no símbolos) y `AG_INSTALLER_REDUCE_MOTION=1` (sin spinner). Códigos de salida: `0` ok · `1` cancelado · `2` red · `3` checksum · `4` permisos/ruta · `130` SIGINT · `143` SIGTERM.
6. **Redacción obligatoria** de la TUI y del modo headless: "checksum verified / integrity, not authenticity"; el warning de `AG_INSTALLER_CHECKSUMS` no definido permanece visible en una línea de aviso (S-10). Detalle en ADR-0013.

## Alternativas consideradas

| Alternativa | Motivo de rechazo |
| --- | --- |
| Reutilizar `ratatui`/`crossterm` dentro de `ag` con replay de eventos (propuesta del arquitecto) | Ejecuta el binario descargado antes de copiarlo, rompiendo I6/V3, y la TUI no aparece hasta después de la descarga (bootstrap). `DEC-001:23-26`, `revision-seguridad.md §1, §4` |
| Binario auxiliar `ag-setup-<target>` para TUI desde el primer byte | Diferido: exige 4 assets nuevos, entrada propia en `checksums.txt` y tocar `.github/workflows/release.yml` (L-4), ampliando la superficie de checksum (`diseno-arquitectura.md §3.4 opción C`; `diseno-tui.md §4.2`) |
| Curseo ANSI en `install.ps1` (`[Console]::SetCursorPosition`) como presentador principal | Duplica la lógica, es frágil en `conhost`, deja secuencias ESC en logs (S-8) y no da navegación real; admitido solo como micro-fallback de línea única (`diseno-arquitectura.md §3.4 opción B`) |
| `Add-Type`, módulos de terceros o paquetes NuGet | Añade compilación en memoria y dependencias nuevas sin aprobación humana (S-9/S-15); `DEC-001:25` |

## Consecuencias (positivas y negativas)

**Positivas**

- Las seis etapas son visibles desde el primer byte, sin ejecutar código descargado.
- Cero dependencias nuevas, cero assets nuevos y cero cambios en `.github/`.
- Una sola semántica de eventos en Linux, macOS y Windows; headless comparable con `diff` en CI.
- Capa de presentación detrás de una bandera: desactivarla no altera descarga ni verificación (L-7).

**Negativas**

- La lógica de presentación se duplica en bash y PowerShell; requiere pruebas de paridad.
- Depende de TTY/VT: en `conhost` clásico o sin TTY degrada a headless; se pierde animación, no información.
- El formato headless es un contrato público: cualquier cambio exige actualizar CI y documentación (K-7).
- bash 3.2 limita las estructuras disponibles; el spinner debe respetar `REDUCE_MOTION`/`NO_COLOR` por accesibilidad.

## Rollback

La TUI es una capa de presentación detrás de `AG_INSTALLER_UI`/`AG_INSTALLER_PROGRESS`: forzar `AG_INSTALLER_UI=headless` la desactiva sin tocar la lógica de descarga/verificación (L-7; `diseno-tui.md §4.1`). Reversión completa: revertir la capa TUI en `install.sh` y `install.ps1` y su documentación K-7; sin tocar `.github/`, sin publicar assets nuevos y sin ejecutar el binario descargado. No hay estado externo que deshacer. Verificar con `git diff --name-only` y `git diff Cargo.toml Cargo.lock` (cero líneas nuevas en manifiestos).

## Cómo se verificará

- `CI=true bash install.sh >out.txt 2>&1` (y equivalente en PS): aparece exactamente `[ag-install] N/6 ...`, sin spinner, sin `\r`, sin color; `grep -q $'\033' out.txt` → falso; dos ejecuciones `diff out1.txt out2.txt` → idénticas (CA-3 de `diseno-tui.md §6`, CA-6 de `diseno-arquitectura.md §7`).
- Con TTY: las 6 etiquetas de etapa, un indicador de etapa activa y `0 compilaciones`; como máximo 25 filas visibles (CA-1).
- Con `NO_COLOR=1`: símbolos y palabras `OK/FALLO/PENDIENTE`, sin `\033[` (CA-2).
- Con checksum modificado: `4/6 Verificar SHA-256: FALLO`, `no se escribió ningún binario`, `exit=3`, etapas 5-6 `OMITIDO` (CA-4); la animación nunca salta ni retrasa un error (V6).
- Sin red y sin ejecución del binario: inspección de la ruta de instalación (solo `mkdir`/`cp`/`chmod` o `Copy-Item`), sin `exec` del asset (I6).
- Honestidad: los criterios TUI y headless están **pendientes de ejecución** sobre el instalador modificado (`DEC-001` §Incertidumbre); esta ADR no afirma que ya pasaron.

## Referencias

- `.kaizen/team/projects/instalacion-rapida-releases/diseno-tui.md` §1-§6 (FU-01..FU-05, CA-1..CA-11).
- `.kaizen/team/projects/instalacion-rapida-releases/diseno-arquitectura.md` §3, §4.3, §4.4, §7 (CA-5..CA-8).
- `.kaizen/team/projects/instalacion-rapida-releases/revision-seguridad.md` §1 (I6), §3, §5 (V3/V6).
- `.kaizen/team/projects/instalacion-rapida-releases/decisions/DEC-001-Via-rapida-precompilada-con-TUI-.md` ítem 3.
- `docs/security/INSTALLATION_INTEGRITY.md:67-68`; `docs/audits/install-user-journey.md:141-148`; issue #204; ADR-0011, ADR-0013.
