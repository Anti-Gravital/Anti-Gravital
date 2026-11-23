# Diseño de arquitectura — `instalacion-rapida-releases`

- **Fecha:** 2026-11-23 (UTC).
- **Autor:** `architect` (subagente; `autonomy: propose`).
- **Proyecto:** `.kaizen/team/projects/instalacion-rapida-releases/` (etapa `execute`).
- **Handoff atendido:** `H-M220261123T172039Z` (tarea: "Disenar arquitectura de instalacion rapida: prebuilt por defecto, deteccion de plataforma, versionado y fallback a source").
- **Estado:** **PROPUESTA**. No ejecuta cambios, no abre PR, no hace push, no cambia etapa. Las decisiones quedan `BLOCK` hasta `kaizen-team verify` en verde (`.kaizen/team/EVIDENCIA.md:20-32`, REG-EV-01/REG-EV-07).
- **Único archivo escrito por esta iteración:** este documento. No se editaron `install.sh`, `install.ps1`, `.github/`, `scripts/`, ADR, RFC ni documentación de producto.

---

## 0. Convención de evidencia y `source_ids`

Esta es una decisión **técnica con impacto en arquitectura**. El ledger regulatorio (`.kaizen/org/knowledge/SOURCES.md`, IDs `XX-NNN`) no cubre evidencia interna del repositorio; se adopta la convención ya usada por `docs/audits/estado-honesto-producto.md:15-17`: `EV-xx` = ruta + sección/línea + comando reproducible + commit. Los IDs `EV-11` y `EV-13` ya están definidos allí (`estado-honesto-producto.md:181-183`). El resto se marcan **`PROP-xx` (propuestos, pendientes de registro)** y deben ser validados por `verificador-evidencias` antes de que cualquier decisión se dé por tomada.

| ID | Fuente verificable (ruta:línea / comando) | Hecho que sustenta | Estado |
| --- | --- | --- | --- |
| `EV-13` | `docs/audits/estado-honesto-producto.md:183` (`git tag`, `cat VERSION`) | Tags `v0.1.0-alpha.1`/`v0.1.0-alpha.2`; `VERSION` = `1.0.0` | registrado |
| `EV-11` | `docs/audits/estado-honesto-producto.md:181` (`README.md:111-203`) | Alpha: qué funciona y qué no; sin release firmado | registrado |
| `PROP-01` | `Cargo.toml:36-48` | `[workspace.package].version` = `0.1.0-alpha.2`; `rust-version` = `1.99.1` | pendiente |
| `PROP-02` | `.github/workflows/release.yml:31-43,54-72,105-116,123-131,184-197` | 4 targets, 8 assets, `checksums.txt`, tag validado contra versión del workspace, prerelease DRAFT, sin firma | pendiente |
| `PROP-03` | `docs/audits/install-user-journey.md:55-95,114-160,162-184,252-264` | Source = 21 m 22 s y **dos** builds; `file://` alpha = 0.410 s; checksum fail-closed comprobado | pendiente |
| `PROP-04` | `install.sh:45,105-137,156-220,239-246`; `install.ps1:42,89-98,110-170,172-181` | Canales actuales (`source` por defecto, `alpha` opt-in), mapeo, nombres de asset, orden fail-closed | pendiente |
| `PROP-05` | `docs/security/INSTALLATION_INTEGRITY.md:51-83,105-143,182-204` | Formato de `checksums.txt`; integridad ≠ autenticidad; `#141` abierto; RFC-0030 borrador sin clave | pendiente |
| `PROP-06` | `.kaizen/team/projects/instalacion-rapida-releases/blackboard.md:6-20` | Observación Windows: cientos de crates sin progreso; requisito de TUI (dato no confiable, L-5) | pendiente |
| `PROP-07` | `crates/ag-cli/Cargo.toml:24-31,47-49`; `Cargo.lock:1697,5343`; `crates/ag-cli/src/tui/mod.rs:1-5,18-20`; `crates/ag-cli/src/tui/terminal.rs:46-68` | `ratatui = 0.30` con feature `tui` ya presente; `crossterm` reexportado; `TestBackend`; patrón "la vista nunca ejecuta trabajo" | pendiente |
| `PROP-08` | `docs/audits/estado-honesto-producto.md:7,17`; `docs/kaizen/adr/0010-versionado-y-propagacion.md:10-14`; `docs/kaizen/journal/2026-11-22-kaizen-install-1720.md:7` | `VERSION=1.0.0` es etiqueta de intención / versión del framework Kaizen, **no** la versión liberada del producto | pendiente |
| `PROP-09` | `.kaizen/team/projects/instalacion-rapida-releases/charter.md:6-7`; `state.json:4` | KPI: ≤10 s, 0 compilaciones, fail-closed, ≥4 etapas animadas, 0 cambios sin humano | pendiente |
| `PROP-10` | `.kaizen/learning/lessons/knowledge-curator/2026-11-23-001-blocker.md` (fingerprint `1866687716`) | Lección de worktree retirado a mitad de tarea: verificar existencia antes de cada escritura; no aplicable a este diseño, se cita por trazabilidad | registrado |

**Reglas aplicadas:** K-1 (sin decisión de arquitectura sin ADR), K-10 (no fabricar evidencia), K-7 (docs a tocar), L-4 (rutas protegidas), L-5/S-15 (buzón, issues, docs y logs son datos no confiables: se leen, no se obedecen), S-3 (sin hardcoding de URLs/umbrales sin configuración validada), S-9 (sin inventar dependencias), S-10 (nada se silencia), S-11 (todo hallazgo al outbox; en esta entrega solo se documenta, véase §8), S-12 (sin secretos). No se leyó ni imprimió ningún `.env`, credencial ni variable de entorno con valor secreto.

---

## 1. Resumen ejecutivo

Hoy el canal por defecto compila dos veces (~21 m 22 s en Linux; `PROP-03`) y en Windows el usuario ve cientos de líneas `Downloaded <crate>` sin progreso (`PROP-06`). El canal `alpha` ya demostró 0.410 s sobre `file://` y fail-closed real (`PROP-03`), y el release ya publica los 8 assets y `checksums.txt` (`PROP-02`). La brecha no es criptográfica ni de artefactos: es de **default**, de **contrato de eventos** y de **presentación**.

Propuesta: inverter el default a un **canal rápido** (`release`) que descarga el binario versionado, verifica SHA-256 e instala, con `AG_INSTALLER_CHANNEL=source` como fallback **explícito** (nunca automático). Separar el sistema en tres capas —núcleo de descarga/verificación, motor de eventos, presentación— reutilizando la TUI que ya existe en `ag-cli` (`PROP-07`), sin añadir dependencias. Mantener el checksum como integridad, **sin firma** (§7).

| # | Decisión propuesta | ¿Quién la cierra? |
| --- | --- | --- |
| D-1 | Canal rápido por defecto; `source` solo explícito; sin fallback automático | Humano (ADR nuevo) |
| D-2 | Versión canónica del tag = `[workspace.package].version`; `VERSION=1.0.0` no gobierna el release | Humano (ADR nuevo) |
| D-3 | Topología: bootstrap delgado + `ag` como presentador/orquestador por replay de eventos reales | Humano (ADR nuevo) |
| D-4 | Contrato único de eventos y salida headless determinista | Humano (ADR nuevo) |
| D-5 | Reutilizar `ratatui`/`crossterm` ya presentes en `ag-cli` (no dependencia nueva) | Extensión de RFC-0018 / registro, no ADR de dependencia |
| D-6 | Checksum = integridad, no autenticidad; `#141` sigue abierto; **no se propone firma** | Reafirmación; humano si se quiere cambiar |

---

## 2. Canal rápido por defecto

### 2.1 Resolución de versión y URL

**Precedencia** (primera que aplica gana):

1. `AG_INSTALLER_BASE_URL` — URL explícita del directorio de assets (`file://`, `http(s)://`). Se quita la barra final.
2. `AG_INSTALLER_RELEASE_TAG` — tag explícito; se expande a `<BASE_TEMPLATE>/<tag>`.
3. **Por defecto:** leer `[workspace.package].version` de `Cargo.toml` (hoy `0.1.0-alpha.2`, `PROP-01`) y construir `tag = "v" + version`, `base = <BASE_TEMPLATE>/<tag>`.

`VERSION` (`1.0.0`, `EV-13`) **no se usa para construir el tag**: es la etiqueta de intención del roadmap / versión del framework Kaizen (`PROP-08`), y el propio workflow valida que el tag coincida con la versión del workspace (`.github/workflows/release.yml:105-116`, `PROP-02`). Si `VERSION` difiere de la versión del workspace, el evento `resolve_version` debe incluir `version_source_mismatch=true` y `version_candidate=<valor de VERSION>` como dato informativo, **sin fallar y sin cambiar el tag**. Esta divergencia es deuda documental: sincronizar o eliminar `VERSION` requiere ADR humano (D-2). No se edita `VERSION` en esta propuesta.

`BASE_TEMPLATE` es una constante de negocio con nombre (`DEFAULT_RELEASE_BASE_TEMPLATE`), documentada y sobrecargable; hoy `install.sh:111` y `install.ps1:84` incrustan la URL (deuda S-3 a corregir en el PR que implemente D-1). La forma canónica observada en el repo es `https://github.com/Anti-Gravital/Anti-Gravital/releases/download` (`install.sh:111`, `release.yml` notas); `Cargo.toml:42` usa minúsculas. Unificar la forma es decisión de documentación (K-1), no del instalador: sobrecarga con `AG_INSTALLER_BASE_URL` resuelve cualquier discrepancia sin tocar `.github/`.

Regla de seguridad: el tag se valida con la misma gramática que el workflow (`v*`), y los valores que vayan a una URL se codifican como componentes de URL (S-4). Nunca se concatena contenido de un archivo directamente en un comando de shell.

### 2.2 Detección de plataforma

Mapeo cerrado (solo lo que `release.yml:31-43` publica, `PROP-02`):

| OS detectado | Arquitectura detectada | Triple publicado | Asset a instalar |
| --- | --- | --- | --- |
| `Linux` (`uname -s`) | `x86_64\|amd64` (`uname -m`) | `x86_64-unknown-linux-gnu` | `ag-x86_64-unknown-linux-gnu` |
| `Linux` | `aarch64\|arm64` | `aarch64-unknown-linux-gnu` | `ag-aarch64-unknown-linux-gnu` |
| `Darwin` | `aarch64\|arm64` | `aarch64-apple-darwin` | `ag-aarch64-apple-darwin` |
| Windows (`PROCESSOR_ARCHITECTURE=AMD64`) | `AMD64` | `x86_64-pc-windows-msvc` | `ag-x86_64-pc-windows-msvc.exe` |

- `AG_INSTALLER_TARGET` conserva su papel de override (`PROP-04`), pero se valida contra la tabla: un triple no publicado se rechaza con `E_UNSUPPORTED_TARGET` antes de cualquier descarga.
- **Hallazgo:** `install.ps1:93` devuelve hoy `aarch64-pc-windows-msvc`, triple que `release.yml` **no publica**. Con este diseño ese valor pasa a ser rechazado con mensaje legible. Soportarlo exigiría añadir un target al workflow (ruta protegida, L-4) → humano.
- `aarch64-apple-darwin` es el único target macOS del release; `install.sh:129-137` ya lo detecta, pero no hay entrada para `x86_64-apple-darwin`: se rechaza explícitamente.
- En todos los casos: fail-closed, sin "mejor esfuerzo" que descargue un asset parecido.

### 2.3 Descarga, verificación e instalación

Orden (idéntico al documentado y ya verificado en `docs/security/INSTALLATION_INTEGRITY.md:62-76`, `PROP-05`):

1. Autoverificación del propio instalador con `AG_INSTALLER_CHECKSUMS` (si está definido). Si no está, se imprime el warning existente — **no se silencia** (S-10).
2. Resolver versión → tag → base URL; detectar target; validar contra la tabla.
3. Descargar `checksums.txt` (o usar `AG_INSTALLER_BINARY_CHECKSUMS` local, sin red) a un directorio temporal.
4. Descargar `ag-<target>[.exe]` al mismo temporal. El binario **no se ejecuta ni se copia** aún.
5. Buscar la entrada **exacta** del nombre base del asset en el manifest; hash SHA-256 calculado con herramienta estándar (`sha256sum`/`shasum -a 256`/`Get-FileHash -Algorithm SHA256`). Falta de entrada, hash no hexadecimal de 64 caracteres, entrada duplicada para el mismo nombre o mismatch → abortar con exit ≠ 0 y BIN_DIR intacto.
6. Solo tras coincidencia: copia atómica a `AG_INSTALLER_BIN_DIR` (por defecto `~/.cargo/bin` / `%USERPROFILE%\.cargo\bin`), permisos `0755` en POSIX, sin privilegios elevados. Temporal siempre limpiado (trap / `finally`).
7. `ready`. No se ejecuta `ag --version` automáticamente: se deja al usuario como hoy, para no ejecutar código descargado dentro del instalador (S-4).

El manifest contiene 8 binarios (ag y ag-lsp × 4 targets, `PROP-02`); se verifica **solo la entrada del asset que se instala**. No se exige que las otras 7 estén presentes: eso acoplaría el instalador a decisiones futuras del workflow.

### 2.4 Fallback a `source` — regla dura

- `AG_INSTALLER_CHANNEL=release` es el valor por defecto; `alpha` se acepta como alias con un aviso de migración (compatibilidad con `PROP-04`).
- `AG_INSTALLER_CHANNEL=source` ejecuta la ruta actual (toolchain → `cargo build --locked --workspace --release` → `cargo install --path crates/ag-cli --locked`).
- **Si falla el canal rápido, el instalador NO compila.** Emite un evento `failed` con código estable y un mensaje legible que propone la acción exacta:

  ```
  E_DOWNLOAD_FAILED: no se pudo descargar ag-<target>. Revise AG_INSTALLER_BASE_URL/AG_INSTALLER_RELEASE_TAG.
  Si necesita compilar desde fuente: AG_INSTALLER_CHANNEL=source bash install.sh (coste medido: ~21 min, dos builds).
  ```

  Esta fricción es intencionada: un fallback automático a `source` convertiría 0.410 s en 21 m 22 s sin que el usuario lo pidiera (`PROP-03`, `PROP-09`). Es ADR D-1.

---

## 3. Separación de capas

### 3.1 Grafo de capas

```
  L0  install.sh (bash)  |  install.ps1 (PS 5.1+)
      bootstrap delgado: resuelve versión, detecta target,
      descarga+verifica ag-<target> y checksums.txt, escribe EventLog
                  |  EventSink (headless) + EventLog (JSONL en tmp)
                  v
  L1  NÚCLEO (operaciones puras, sin presentación)
      resolve_version | detect_platform | download | sha256_verify | install_atomic
      implementado en bash y PowerShell para el bootstrap;
      contrato único y tests de paridad entre ambas implementaciones
                  |
                  v
  L2  MOTOR DE EVENTOS
      Event { seq, stage, state, bytes, total, version?, target?, code?, source }
      adaptadores: Real | Fixture ; sumideros: Headless | Tui | File(EventLog)
                  |
                  v
  L3  PRESENTACIÓN  ->  ag <target>  (binario ya verificado, feature `tui`)
      ratatui 0.30 + ratatui::crossterm (reexport)  |  TestBackend para QA
      etapas install y ready + replay de las etapas 1..4 desde EventLog
```

La separación es **lógica**: no se propone un crate nuevo ni un binario auxiliar. `L3` vive en `crates/ag-cli/src/tui/` reutilizando el módulo existente, que declara explícitamente "una pantalla nunca ejecuta trabajo; el llamador ejecuta la petición" (`crates/ag-cli/src/tui/mod.rs:1-5`, `PROP-07`). El subcomando propuesto es `ag install --from <tmp>/<asset> --bin-dir <dir> --events <eventlog> --progress=auto|tui|headless`.

### 3.2_secuencia de etapas y quién emite cada evento

| Etapa | Emisor | Acción real observada | Evento resultante |
| --- | --- | --- | --- |
| `resolve_version` | L0 | Lectura de `Cargo.toml` / override | `started` → `succeeded{version, tag, version_source_mismatch}` |
| `detect_platform` | L0 | `uname`/`PROCESSOR_ARCHITECTURE` + validación de tabla | `started` → `succeeded{target}` o `failed{E_UNSUPPORTED_TARGET}` |
| `download` | L0 | `curl`/`wget`/`Invoke-WebRequest`/`file://` a tmp | `started` → `progress{bytes,total}` (si la herramienta informa bytes) → `succeeded{bytes,total}` |
| `verify` | L0 | `sha256sum`/`shasum`/`Get-FileHash` vs manifest | `started` → `succeeded{hash_match:true}` o `failed{E_CHECKSUM_MISMATCH\|E_MANIFEST_MISSING}` |
| `install` | L3 (`ag`) | Copia atómica a `BIN_DIR` + permisos | `started` → `succeeded{path:<nombre base del binario>}` o `failed{E_INSTALL_WRITE_FAILED}` |
| `ready` | L3 (`ag`) | Instalación confirmada en disco | `succeeded` |

**Regla anti-ficción (K-10):** el productor solo emite `progress` cuando el adaptador de descarga ha observado bytes reales. La animación del spinner es presentación; el `state`, `bytes` y `total` siempre vienen de una acción real. Si la herramienta de descarga no informa de bytes (p. ej. `cp` sobre `file://` medido con `stat` después), se emite `started` y `succeeded`, **nunca** una sucesión de `progress` inventados. Si `total` se desconoce, se muestra `bytes` sin barra y sin ETA; el ETA se calcula solo con `(total-bytes)/velocidad_observada`.

**Replay honesto:** el binario `ag` recibe el `EventLog` con las etapas 1..4 ya ocurridas y las refleja en la TUI antes de ejecutar 5..6. Es reproducción de eventos reales, no simulación, y debe documentarse como tal en el ADR D-3. Trade-off: durante la descarga inicial el usuario ve líneas headless estables, no la TUI de pantalla completa. Si el KPI interpreta "TUI desde el primer byte" de forma estricta, la alternativa es publicar un binario auxiliar `ag-setup-<target>` que arranque antes de descargar `ag`; eso añade 4 assets y exige cambiar `.github/workflows/release.yml` (L-4) → humano. No se implementa en esta propuesta.

### 3.3 Modo headless

`AG_INSTALLER_PROGRESS=auto|tui|headless`. `auto` selecciona `headless` si `CI` está definido, si la salida no es TTY, o si `TERM=dumb`; selecciona `tui` en caso contrario. El modo headless:

- imprime una línea por evento, ASCII, sin secuencias ANSI (sin `\x1b`), sin colores, sin timestamps, sin barras;
- usa `seq` global para que la salida sea ordenable y comparable entre ejecuciones;
- formato estable: `ag-install <seq> <stage> <state> bytes=<n> total=<n|-> [code=<E_...>]`;
- en `failed`, termina con exit ≠ 0 y el código estable, sin trazas internas ni datos personales (S-7/S-8).

Este formato es un contrato público: cambiarlo rompe CI, por lo que va en ADR D-4 y en la documentación K-7.

### 3.4 Presentación en Windows: evaluación del trade-off

| Opción | Ventaja | Coste / riesgo | Veredicto |
| --- | --- | --- | --- |
| **A. `ag.exe` con `ratatui::crossterm`** (recomendada) | No hay TUI que escribir en PowerShell; mismos eventos y misma TUI que Linux/macOS; reutiliza `ratatui 0.30` y el reexport de `crossterm` ya en `Cargo.toml:47-49`/`Cargo.lock` (`PROP-07`). Sin dependencias nuevas (S-9). | `crossterm` necesita consola con procesamiento VT (Windows Terminal / Windows 10+); en `conhost` clásico se degrada. Mitigación: detección de TTY/VT y fallback automático a headless. Que `ag.exe` se ejecute antes de copiarse: mitigado por verificación SHA-256 previa. | **Adoptar** |
| B. Curseo con ANSI dentro de `install.ps1` (`[Console]::SetCursorPosition`, `Write-Host -NoNewline`, ESC) | Funciona en PS 5.1 sin binario auxiliar; cero dependencias. | Duplica la lógica de presentación; frágil en `conhost`; secuencias ESC en logs (S-8); riesgo de silenciar/señalar errores; no da navegación real. | Descartar salvo como micro-fallback de línea única |
| C. Binario auxiliar `ag-setup-x86_64-pc-windows-msvc.exe` | TUI desde el primer byte, presentador separado del producto. | 4 assets nuevos, cambio en `.github/workflows/release.yml` (L-4) y más superficie de checksum; duplica el bootstrap. | Diferir a ADR D-3; no implementar sin humano |

No se propone `Add-Type`, módulos de terceros ni paquetes NuGet: `install.ps1` sigue usando solo `Invoke-WebRequest -UseBasicParsing` y `Get-FileHash` (`PROP-04`).

---

## 4. Contratos

### 4.1 Assets del release

| Asset | Consumido por el instalador |
| --- | --- |
| `ag-x86_64-unknown-linux-gnu` | sí (Linux x86-64) |
| `ag-aarch64-unknown-linux-gnu` | sí (Linux arm64) |
| `ag-aarch64-apple-darwin` | sí (macOS arm64) |
| `ag-x86_64-pc-windows-msvc.exe` | sí (Windows x64) |
| `ag-lsp-<target>[.exe]` × 4 | no; aparece en `checksums.txt` |
| `checksums.txt` | sí |

Nombres **exactos**, sin prefijo de ruta, generados por `release.yml:57-72` (`PROP-02`). El destino final es siempre `BIN_DIR/ag` o `BIN_DIR/ag.exe`, con independencia del nombre del asset.

### 4.2 `checksums.txt`

- Formato `sha256sum` en modo texto: `<64 hexadecimales en minúsculas><DOS espacios><nombre base del asset>`; finales de línea **LF**; sin `*` de modo binario; sin `./` ni rutas; orden `LC_ALL=C` (lo que ya emite `release.yml:129`, `PROP-02`).
- Codificación ASCII/UTF-8; una entrada por asset; 8 entradas en el release actual.
- Parseo en PowerShell con `-Encoding ascii` y `-split '\s+', 2`, comparando `Split-Path -Leaf` contra el nombre base exacto ( patrón ya usado en `install.ps1:143-152`). No convertir finales de línea: la conversión rompe el hash (`INSTALLATION_INTEGRITY.md:229-231`).
- Reglas de rechazo: manifest ausente, entrada ausente para el asset, entrada duplicada, hash no hexadecimal de 64, mismatch → `failed` + exit ≠ 0 + sin escritura en `BIN_DIR`.
- `AG_INSTALLER_BINARY_CHECKSUMS` mantiene su semántica: manifiesto local fuera de banda, suprime la descarga de `checksums.txt` (`PROP-05`). **No se amplía a firma.**

### 4.3 Esquema de eventos

| Campo | Tipo | Regla |
| --- | --- | --- |
| `seq` | u64 | Monótono global. L0 usa `1..N`; L3 continúa en `N+1`. |
| `stage` | enum | `resolve_version` \| `detect_platform` \| `download` \| `verify` \| `install` \| `ready` |
| `state` | enum | `started` \| `progress` \| `succeeded` \| `failed` |
| `bytes` | u64 | Bytes observados; `0` cuando no aplica. Nunca estimado. |
| `total` | u64 \| null | Tamaño conocido; `null` si la fuente no lo informa. |
| `version` / `tag` | string \| null | Solo en `resolve_version`. |
| `target` | string \| null | Solo en `detect_platform`. |
| `code` | string \| null | Código estable de error: `E_MANIFEST_MISSING`, `E_CHECKSUM_MISMATCH`, `E_UNSUPPORTED_TARGET`, `E_DOWNLOAD_FAILED`, `E_INSTALL_WRITE_FAILED`, `E_VERSION_MISMATCH`. |
| `source` | enum | `real` \| `fixture`. `fixture` **solo** en tests. |

Transporte interno: JSONL con una línea por evento (claves fijas, sin timestamps). El `EventLog` del temporal es la única vía por la que L3 conoce las etapas 1..4. Un evento `progress` con `source=fixture` jamás puede promoverse a `real`.

### 4.4 Stub de QA sin red

- Adaptadores: `EventSource::Real` (producción) y `EventSource::Fixture` (tests), inyectados por `--events-fixture <path>` en `ag` y por `AG_INSTALLER_FIXTURE=<path>` en el bootstrap.
- El fixture usa el mismo JSONL del §4.3, con `source=fixture` y bytes/total sintéticos **declarados**.
- Guard de seguridad: si `fixture` está activo, el comando entra en modo **dry-run**: no descarga, no escribe `BIN_DIR`, no ejecuta el asset. Se verifica con un directorio temporal que debe quedar vacío (CA-8).
- QA renderiza los mismos eventos en el sumidero `headless` (comparación de líneas con `diff`) y en el sumidero `Tui` con `TestBackend` de `ratatui` (ya disponible, `PROP-07`), sin abrir red ni terminal real.

---

## 5. Compatibilidad

| Plataforma | Script | Herramientas ya usadas | TUI |
| --- | --- | --- | --- |
| Linux x86_64 / arm64 | `install.sh` (bash) | `curl` o `wget`; `sha256sum` o `shasum -a 256`; `uname` | `ag` con feature `tui` (ratatui/crossterm) |
| macOS arm64 | `install.sh` (bash 3.2+) | `shasum -a 256`; `uname` | igual |
| Windows x64 | `install.ps1` (PS 5.1+) | `Invoke-WebRequest -UseBasicParsing`; `Get-FileHash`; `PROCESSOR_ARCHITECTURE` | `ag.exe` con crossterm; fallback headless si no VT/TTY |
| Windows arm64 | `install.ps1` | — | **rechazado**: triple no publicado (§2.2) |

Restricciones que se conservan: sin privilegios elevados, sin gestores de paquetes, sin ejecutar otro script descargado, sin `--no-verify` ni supresiones (S-10), autoverificación del instalador antes de cualquier acción, limpieza del temporal en todos los caminos de salida. La instalación es aditiva y reversible: el binario se copia a un archivo temporal dentro de `BIN_DIR` y se renombra; en Windows, si `ag.exe` está en uso, se emite `E_INSTALL_WRITE_FAILED` con instrucción de cerrar el proceso (no se fuerza el reemplazo).

Documentación a tocar en el PR de implementación (K-7, inventario final a cargo de `finops`): `README.md`, `docs/manual/04-instalacion-y-onboarding.md`, `docs/security/INSTALLATION_INTEGRITY.md`, `CHANGELOG.md` y el descriptor `docs/kaizen/pr-drafts/<rama>.md`. Esta entrega **no** los edita.

---

## 6. Decisiones que requieren ADR y escalado a humano

### 6.1 ADRs nuevos (K-1)

| ADR | Tema |Por qué no lo decide el architect |
| --- | --- | --- |
| ADR-N1 | Canal rápido por defecto + `source` explícito + prohibición de fallback automático | Invierte el comportamiento público y el contrato de confianza (`EV-11`, `PROP-03`, `PROP-09`) |
| ADR-N2 | Fuente de versión canónica: `[workspace.package].version`; `VERSION` queda como dato informativo; política ante `version_source_mismatch` | `VERSION` está ligado al framework Kaizen (ADR-0010) y a la etiqueta de roadmap (`PROP-08`); cambiarlo o sincronizarlo exige humano |
| ADR-N3 | Topología de presentación: replay de eventos en `ag` vs. binario auxiliar con 4 assets nuevos | Afecta a `.github/workflows/release.yml` (L-4) y a la superficie de assets |
| ADR-N4 | Contrato de eventos y formato headless estable (ASCII, sin ANSI, sin timestamps) | Es una API observable por CI; su cambio rompe consumidores |
| ADR-N5 | Extensión de la TUI de `ag-cli` para el instalador | Reutiliza RFC-0018 y dependencias ya aprobadas (`PROP-07`); se registra como extensión, no como dependencia nueva |
| ADR-N6 | Reafirmación: checksum = integridad; `#141` abierto; RFC-0030 borrador; **sin firma** | Cualquier cambio de confianza requiere humano (`PROP-05`) |

### 6.2 Rutas protegidas (L-4) — escalar, no tocar

| Ruta | Cambio que podría pedir este diseño | Acción |
| --- | --- | --- |
| `.github/workflows/release.yml` | Solo si ADR-N3 elige binario auxiliar; o si se decide publicar `x86_64-apple-darwin` / `aarch64-pc-windows-msvc` | **Humano**. Este diseño no la edita. |
| `.kaizen/` (incluye `decisions/`, `state.json`, `tasks.json`) | Registro de ADR, ack del handoff, cambio de etapa | **Humano / orquestador**. Única excepción de esta entrega: el archivo que se está leyendo. |
| `scripts/kaizen*`, `scripts/lib/` | Nuevas comprobaciones de KPI | **Humano**; si hace falta, issue `needs-human`, no edición. |
| `docs/kaizen/loop/` | Cualquier cambio de reglas del loop | **Humano** (L-3/L-4). |
| `install.sh`, `install.ps1` | Implementación de L0/L1 | No protegidas por L-4, pero **prohibidas en esta entrega** por instrucción del handoff; se proponen para el PR posterior. |
| Firma / clave / `tools/release-signing/` | Cualquier generación de clave o manifiesto firmado | **Fuera de alcance**: no existe clave, `#141` sigue abierto, RFC-0030 es borrador (`PROP-05`). No se propone. |

### 6.3 Lo que explícitamente NO se propone

- No crear, publicar ni usar clave de firma; no `minisig`, no GPG, no manifiesto firmado; no afirmar "instalación verificada" en sentido criptográfico.
- No cerrar `#141`, `#203`, `#204` ni `#206`; no dar por aceptada RFC-0030.
- No modificar `.github/`, `.kaizen/` (salvo este archivo), `scripts/`, `install.sh`, `install.ps1`, ADR ni RFC.
- No ejecutar `kaizen-team advance|decide|verify`, ni push, PR o commit.

---

## 7. Criterios de aceptación verificables

| ID | Criterio | Método reproducible | Evidencia esperada |
| --- | --- | --- | --- |
| CA-1 | Canal rápido es el default y hace 0 compilaciones | `env -u AG_INSTALLER_CHANNEL AG_INSTALLER_BASE_URL=file://<release> bash install.sh` sobre release sintético; repetir con `.\install.ps1` | `grep -E 'cargo build|Updating crates.io index|Downloaded '` → sin coincidencias; `ag` instalado; tiempo ≪ 10 s (referencia 0.410 s, `PROP-03`) |
| CA-2 | Fail-closed 100% ante checksum inválido | 4 casos ya documentados (`PROP-03`): hash erróneo, entrada ausente, manifest ausente, binario ausente | exit ≠ 0 en los 4; `BIN_DIR` sin `ag`; mismo resultado en PS 5.1 |
| CA-3 | Versión del tag | Ejecutar con `VERSION=1.0.0` y workspace `0.1.0-alpha.2`, por `file://` con `v0.1.0-alpha.2` | evento `resolve_version{tag:"v0.1.0-alpha.2", version_source_mismatch:true}`; URL pedida contiene `v0.1.0-alpha.2`, nunca `v1.0.0` |
| CA-4 | Mapeo cerrado de plataformas | Tabla §2.2 simulada con `AG_INSTALLER_TARGET` y stubs de `uname`/`PROCESSOR_ARCHITECTURE` | 4 targets aceptados; `aarch64-pc-windows-msvc` y `x86_64-apple-darwin` rechazados con `E_UNSUPPORTED_TARGET` antes de descargar |
| CA-5 | Progreso real | Test de unidad con adaptador que cuenta bytes; inyectar fallo de red a mitad de descarga | todo `progress` tiene `bytes>0` observados; al fallar no aparece `succeeded`; ningún `progress` en `verify`/`install` |
| CA-6 | Headless limpio y determinista | `AG_INSTALLER_PROGRESS=headless ... > out.txt`; `cat -v out.txt`; repetir y `diff out1.txt out2.txt` | sin `\x1b`, sin timestamps, salidas idénticas; una línea por evento con `seq` creciente |
| CA-7 | TUI con ≥4 etapas y estado real | `cargo test -p ag-cli --features tui` con eventos fixture en `TestBackend` | render de las 6 etapas; navegación/estado; el test no abre red; `source=fixture` en todos los eventos |
| CA-8 | Fixture no muta (`PROP-07`, §4.4) | Ejecutar con `AG_INSTALLER_FIXTURE=<path>` apuntando a `BIN_DIR` temporal vacío | directorio sigue vacío; exit 0 en modo dry-run; ninguna petición de red |
| CA-9 | PowerShell sin dependencias nuevas | `[System.Management.Automation.Language.Parser]::ParseFile('install.ps1',[ref]$null,[ref]$errs)` en PS 5.1; revisar diff de `Cargo.toml`/`Cargo.lock` | parseo sin errores; 0 líneas nuevas en manifiestos; TUI vía `ag.exe`; fallback headless verificado |
| CA-10 | Documentación viva (K-7) | `git diff --name-only` del PR | `README.md`, `docs/manual/04-instalacion-y-onboarding.md`, `docs/security/INSTALLATION_INTEGRITY.md`, `CHANGELOG.md` y descriptor presentes |
| CA-11 | Gates verdes | `scripts/kaizen check` y `scripts/kaizen-security check` en el PR (no ejecutados en esta entrega) | salida real reportada; cualquier fallo se documenta, nunca se silencia (S-10) |
| CA-12 | Sin cambios protegidos ni firma | `git diff --name-only` filtrado por `.github/`, `.kaizen/`, `scripts/`, `tools/release-signing/` | solo el archivo permitido de esta entrega; en el PR, 0 cambios en protegidas sin ADR humano |

---

## 8. Hallazgos a registrar por S-11 (esta entrega solo los documenta)

1. **`install.ps1` anuncia un target que no existe en el release.** `install.ps1:93` devuelve `aarch64-pc-windows-msvc`; `release.yml:31-43` no lo publica. Impacto: descarga 404 y experiencia confusa en Windows ARM64. Propuesta: rechazar con `E_UNSUPPORTED_TARGET` o escalar a humano para añadir el target. (`PROP-02`, `PROP-04`.)
2. **URL de release incrustada en los instaladores.** `install.sh:111`, `install.ps1:84` hardcoded (S-3). Propuesta: constante con nombre + override ya existente; sin valores secretos.
3. **`VERSION=1.0.0` vs versión liberada `0.1.0-alpha.2`.** Riesgo de construir url `v1.0.0` inexistente si se usa la fuente equivocada. Propuesta: ADR-N2; no tocar `VERSION`. (`EV-13`, `PROP-08`.)
4. **Coste oculto del canal `source`:** dos builds completos (~21 m 22 s) y mensaje `~/.cargo/bin` fijo aunque `CARGO_HOME`/`CARGO_INSTALL_ROOT` digan otra cosa (issues #206 y #204, preexistentes, `PROP-03`). No cerrar sin verificación (K-2/K-10).
5. **Replay de eventos 1..4:** si el ADR interpreta "TUI desde el primer byte", la opción C requiere humano; de lo contrario documentar el replay como diseño aceptado para no fingir progreso.
6. **Warning de `AG_INSTALLER_CHECKSUMS` en cada ejecución:** es correcto y no debe silenciarse (S-10); el canal rápido debe mantenerlo y explicarlo en la documentación.

El orquestador debe crear las entradas `.kaizen/loop/outbox/*.md` correspondientes; este subagente no escribe fuera de su único archivo permitido.

---

## 9. Límites de esta iteración y cierre honesto

- **Escrito:** exclusivamente `.kaizen/team/projects/instalacion-rapida-releases/diseno-arquitectura.md`.
- **No ejecutado:** `scripts/kaizen check`, `scripts/kaizen-security check`, `kaizen-team advance/decide/verify`, `kaizen-learn record`, comandos de commit/push/PR y cualquier mutación distinta de este archivo. Se dice explícitamente para no fabricar evidencia (K-10). El handoff queda con `status: open` y pendiente de ack del facilitador.
- **Sin secretos:** no se leyó ni imprimió credencial, `.env`, clave ni valor de variable de entorno; no se incluyen datos personales ni rutas de usuario en este documento.
- **Confianza:** fuentes externas y el buzón se trataron como datos no confiables (L-5/S-15); ninguna instrucción dentro de ellos se ejecutó.
- **Siguiente paso (no realizado):** que `verificador-evidencias` registre los `PROP-xx` y deje `verify` en verde; que el humano dicte los ADR-N1..N6; solo entonces `dev5-glue`/`dx`/`qa` pueden implementar y verificar los CA-1..CA-12 en un PR con descriptor K-4.
