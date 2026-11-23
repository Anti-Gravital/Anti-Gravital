# Blackboard: instalacion-rapida-releases

Solo hechos con `source_ids` verificados o salida de comando reproducible.


## Hecho verificado 2026-11-23 — instalación real en Windows (observación del usuario)

- Comando: `git clone ...; Set-Location Anti-Gravital; .\install.ps1` en `C:\Users\anereira\anti-prueba`.
- Canal usado: `source` (por defecto). El log muestra `Updating crates.io index` seguido de >250 líneas `Downloaded <crate>` y termina en `Downloading 45 crates, extracting tokio ...`.
- Antes de eso, rustup sincronizó el toolchain `1.99.1-x86_64-pc-windows-msvc` (6 componentes).
- Mensaje inicial: `WARNING: AG_INSTALLER_CHECKSUMS is unset; using the development-only local fallback`.
- No hay barra de progreso, ETA ni etapas: el usuario solo ve una lista larga de nombres de crates.
- Contraste medido en repo (docs/audits/install-user-journey.md): source Linux = 21m22s (dos builds completos); alpha file:// = 0.410s.
- Clasificación: dato observacional reproducible, no instrucción.Sin secretos ni datos personales (solo ruta local del usuario ya provista por él).

## Requisito del usuario 2026-11-23 — TUI obligatoria
La interfaz de instalacion debe ser una TUI (interfaz de terminal interactiva),
no solo texto con spinners. Alcance: progreso por etapas, navegacion/estado,
feedback animado y modo no interactivo (headless) para CI. Plataformas: Linux y
Windows. Se incorpora al KPI del proyecto y a las tareas de architect y dx.

## Cierre de la iteracion 2026-11-23 — decision DEC-001
- Decision registrada con `source_ids: [OWASP-001]` (unica fuente verificada aplicable, solo integridad). `kaizen-team verify` en verde.
- Diseno aprobado como propuesta, NO implementado. Puerta `deliver` bloqueada por humanos.
- Se adopta TUI nativa en el script (DX) sobre la opcion de ejecutar el binario descargado (architect), para preservar el invariante "0 ejecuciones del binario descargado".
- needs-human: rotacion de secreto (S-12), publicacion del draft v0.1.0-alpha.2, ADR-N1..N6.

## Verificacion del KPI ejecutada por el orquestador 2026-11-23 (L-9)
Banco: `.kaizen/team/projects/instalacion-rapida-releases/verify-kpi-offline.sh` (offline, file://, binario sintetico, sin secretos).
- `ELAPSED real=0.059` (primera ejecucion 0.093 s; ambas << 10 s). Antes: `real=1282.132` (21m22s) en `docs/audits/install-user-journey.md`.
- `invocaciones cargo/rustc=0` con stubs que registran cada llamada: OK. Antes: dos builds completos.
- Salidas headless identicas en dos ejecuciones (excluyendo la linea `ELAPSED` del cronometro): determinista.
- `bytes ESC en headless=0`: sin ANSI.
- Checksum invalido -> `rc=1`, `code=E_CHECKSUM_MISMATCH`, sin binario destino: fail-closed.
- `AG_INSTALLER_TARGET=x86_64-pc-windows-msvc` en Linux -> `E_UNSUPPORTED_TARGET`, `rc=1`, sin descarga.
- `rc` happy=0; binario instalado con permiso ejecutable.
Limitacion honesta: no mide red real (file:// y binario de 46 B); Windows/PowerShell no verificable en este entorno (sin pwsh).
