# Dictamen de seguridad — instalacion-rapida-releases

- Rol: `security` (Seguridad y Auditoría) · Fecha/corte: 2026-11-23.
- Alcance: canal rápido (alpha/prebuilt) + TUI animada, Linux y Windows.
- Salida única: este archivo. Sin push/PR/commit; sin `kaizen-team advance/decide/verify`.
- Base de evidencia: documentos de primera parte del repo + comandos reproducibles. No hay entrada `PA-xxx` en `.kaizen/org/knowledge/SOURCES.md` para estos documentos internos: **no invento `source_id`** (K-10 / REG-EV-07). `verificador-evidencias` debe registrarlos antes de `verify` (hallazgo H2).

## 1. Invariantes que se preservan SIEMPRE (veto vinculante)

| # | Invariante | Evidencia verificada | Si se rompe |
|---|---|---|---|
| I1 | Orden fail-closed: self-integrity → manifest → binario → SHA-256 → copia | `install.sh:237` (`verify_self_integrity`) precede a `:239-240` (`install_alpha`); dentro: manifest `:177-192`, binario `:193-197`, entrada `:199-203`, hash `:204-207`, compare `:208-211`, copia `:214-217`. `install.ps1:172` → `:174-176` → `:126-141` → `:143-157` → `:160-162`. Contrato: `docs/security/INSTALLATION_INTEGRITY.md:62-75` | VETO V1 |
| I2 | `AG_INSTALLER_CHECKSUMS` (self-check del script, local, fail-closed) | `install.sh:73-103`; `install.ps1:49-79`. Sin variable = warning, no control (`:78-80` / `:54-57`): está bien así, documentado en `INSTALLATION_INTEGRITY.md:169-180` | VETO V2 |
| I3 | `AG_INSTALLER_BINARY_CHECKSUMS` (manifest local out-of-band) | `install.sh:158,177-183`; `install.ps1:126-133`; `INSTALLATION_INTEGRITY.md:46-49,85-90`. Si se setea, **no se descarga** `checksums.txt` | VETO V2 |
| I4 | `checksums.txt` con LF y nombres base (sin prefijo de ruta) | `.gitattributes` (3 líneas LF); `.github/workflows/release.yml:129-131` (`LC_ALL=C sha256sum *`, `sha256sum --check`); `RFC-0032 §4.3:112-130`. Parsers `$2 == name` (`install.sh:88,199`; `install.ps1:65,145`): un prefijo de ruta deja la entrada sin encontrar y aborta = fail-closed correcto | VETO V1 |
| I5 | Sin elevación de privilegios | Sin `sudo`/`RunAs`/admin check en `install.sh`/`install.ps1` (grep). `install.sh:216` solo `chmod 0755`; `INSTALLATION_INTEGRITY.md:13-14,74-75` | VETO V4 |
| I6 | Nunca ejecutar el binario descargado durante la instalación | `install.sh:214-217` = `mkdir`/`cp`/`chmod`; `install.ps1:160-162` = `Copy-Item`. El `ag --version` de `:219`/`:164` es instrucción al usuario, no ejecución. `INSTALLATION_INTEGRITY.md:67-68` | VETO V3 |

Estado: **los seis invariantes se cumplen en el código actual**. La TUI debe envolverlos, nunca reordenarlos ni reemplazarlos.

## 2. Sustitución en el mismo canal (binary y checksum del mismo BASE_URL)

- Riesgo real y documentado: `install.sh:186,194` e `install.ps1:136-141` descargan manifest y binario del mismo `BASE_URL`. Quien controla ese release puede sustituir ambos. `INSTALLATION_INTEGRITY.md:77-90` y `RFC-0032 §4.3:127-130` lo admiten expresamente.
- Mitigación **sin inventar firma**: la interfaz ya existe — `AG_INSTALLER_BINARY_CHECKSUMS` (I3). Recomendación: el canal rápido debe recordar/ofrecer esa variable y explicar que el checksum del mismo canal **no es un ancla**; el manifest local solo mueve el origen del checksum fuera del canal, no autentica (`INSTALLATION_INTEGRITY.md:88-90`).
- **Prohibido**: crear `.minisig`, generar/incrustar clave, publicar huella, tocar `SECURITY.md`, ni llamar "verificado" al canal por defecto. `RFC-0032 §4.5:150-166` y `§9:226-238`; `RFC-0030 §4.10:247-253, §9:313-316` (draft, sin key); issue #141 sigue abierto.
- **VETO V5**: cualquier cambio que convierta el canal rápido en "autenticado" por defecto, o que trate el `checksums.txt` del propio release como trust anchor.

## 3. Redacción de la TUI (nunca decir "verificado/firmado/auténtico")

- Redacción ya correcta que la TUI debe **reutilizar**: `install.sh:176` "checksum verification only (no signing key, no signature)", `:212` "Checksum verified for…"; `install.ps1:125,158`; `release.yml:153-167` "Integrity, not authenticity … NOT a signature and NOT a trust anchor".
- Permitido: "checksum verificado", "integridad verificada", "coincide con `checksums.txt`", "sin firma / no es autenticidad".
- Prohibido: "verificado", "firmado", "auténtico", "seguro", "de confianza", "trust anchor"; iconos de candado/check verde que impliquen autenticidad; ocultar el warning de `AG_INSTALLER_CHECKSUMS` unset (`install.sh:79`, `install.ps1:55`).
- Modo headless (CI) debe conservar los mismos mensajes y el fail-closed: un mismatch sigue siendo salida no cero y sin copia (`:208-211` / `:154-157`). La animación no debe ocultar ni retrasar errores (S-7). **VETO V6** si la TUI "salta" etapas fallidas.

## 4. Reglas S-8 / S-10 / S-11 / S-15 / S-16 aplicadas al feedback animado

- **S-8**: los logs actuales no tienen secretos, pero sí imprimen rutas (`install.sh:217`, `install.ps1:162`) y URLs de descarga (`install.sh:187,193`, `install.ps1:137,140`). Regla para `dx`: no imprimir URLs con query/token ni rutas de usuario en artefactos versionados; en headless, redactar query strings. La ruta local del usuario presente en `blackboard.md:8` **no debe copiarse** a artefactos versionados (ver H1; no la reproduzco aquí).
- **S-10**: grep en `install.sh`, `install.ps1` y `.github/workflows/release.yml` → **0 supresiones** (`nosec`, `|| true`, `continue-on-error`, `--no-verify`, skip). La TUI no debe Introducirlas ni usar `try/catch` que trague excepciones (S-7). **VETO V6**.
- **S-11**: hallazgos H1–H4 listados en §5. Este archivo es el registro para el orquestador (tengo ámbito de salida única y no tengo token): debe sanearse con `kaizen-security redact` antes de publicarse. No hay fallos omitidos.
- **S-15**: no instalar skills/MCP nuevos. La TUI pedida (`blackboard.md:16-20`: ratatui/crossterm, Linux/Windows) implica dependencias nuevas → **S-9**: verificar nombre exacto, mantenimiento y lockfile fijado, con revisión humana. Si se añade un binario helper TUI, es un artefacto nuevo de la cadena de confianza: checksum propio y revisión humana; **nunca ejecutar el `ag` descargado para dibujar la TUI** (I6/V3). La dependencia TUI tampoco debe romper el KPI "0 compilaciones" del charter:8 — eso es decisión de arquitectura/scope, no la apruebo aquí.
- **S-16**: sin contenedores/root/`latest` nuevos. CI ya usa actions pinneadas a SHA (`release.yml:45,50,69,83`) y permisos mínimos (`contents: read` en `:18-19`; `contents: write`/`actions: read` solo en el job de release `:79-80`). Sin cambios.

## 5. Vetos, rutas protegidas (L-4) y hallazgos

**Vetos vinculantes de security** (con evidencia; 2 turnos sin acuerdo → `needs-human`):
V1 romper el orden fail-closed o el formato LF/nombres base · V2 eliminar o debilitar `AG_INSTALLER_CHECKSUMS` o `AG_INSTALLER_BINARY_CHECKSUMS` · V3 ejecutar el binario descargado · V4 introducir elevación · V5 lenguaje de autenticidad/firma o make-default sin RFC-0030 · V6 suppressions S-10, errores tragados o redacción engañosa · V7 secretos/rutas de usuario en logs o artefactos · V8 nueva dependencia/skill/MCP sin aprobación humana (S-9/S-15) · **V9 cambiar el canal por defecto de `source` a prebuilt sin decisión humana** (ver H3).

**Cambios que requieren humano (L-4; este equipo no los ejecuta):**
`.github/` (incluye `release.yml` y `SECURITY.md`), `.githooks/`, `.kaizen/` salvo este archivo, `scripts/kaizen*`, `scripts/lib/`, `CODEOWNERS`, `AGENTS.md`/`CLAUDE.md`/`GEMINI.md`/`opencode.json`/`.claude/`/`.agents/`/`.opencode/`, infraestructura, migraciones, secretos, autenticación y pagos. Además: firma/trust anchor (`RFC-0030 §9`) y la publicación del draft (`RFC-0032 §4.4:146-148` es acción humana). Esto coincide con el KPI del `charter.md:7` ("0 cambios en .github, firma o confianza sin humano").

**Hallazgos (S-11):**
- **H1 (media · privacidad/S-8)**: `blackboard.md:8` contiene una ruta local de Windows con nombre de usuario en un artefacto versionado. No la reproduzco. Propuesta: el orquestador aplica `kaizen-security redact` y la sustituye por una descripción genérica. No la corrijo yo: único archivo de salida.
- **H2 (proceso · K-10/REG-EV-07)**: documentos internos citados aquí no tienen `source_id` registrado en `.kaizen/org/knowledge/SOURCES.md`. No invento IDs. `verificador-evidencias` debe registralos antes de `verify`.
- **H3 (media · diseño)**: `charter.md:4` pide "precompilados versionados **por defecto**", pero hoy el default es `source` (`install.sh:45`, `install.ps1:42`, `INSTALLATION_INTEGRITY.md:13-18`) y `RFC-0032 §4.4` define el alpha como **opt-in** con draft publicado por humano. Hacer prebuilt el default convertiría la verificación del mismo canal en la única por defecto y elevaría el riesgo de sustitución. Requiere ADR/decisión humana; mientras tanto, V9.
- **H4 (baja · S-8)**: la TUI no debe ecoar URLs completas ni rutas de usuario; ver §3 y §4.

**Verificación ejecutada (K-10, sin fabricar resultados):** `scripts/kaizen check --only policy|descriptors|ratchet --quiet` → `rc=0` en los tres; `git status --porcelain` sin cambios nuevos antes/después. No ejecuté `check` completo porque `KZ_LINT_CMD` (`config.env:3`) corre `py_compile`, que escribe en `scripts/` (ruta prohibida en esta tarea) y `KZ_TEST_CMD` (`config.env:2`) corre la suite: eso excede la salida única. Tampoco ejecuté `kaizen-team advance/decide/verify` (prohibido).

## 6. Dictamen final

**APPROVE condicionado.**

Apruebo la dirección de seguridad del canal rápido tal como está hoy documentada e implementada (opt-in alpha, checksum-only, fail-closed, sin firma, sin elevación, sin ejecutar el binario descargado), **condicionada a**:

1. Preservar I1–I6 y la redacción de §3 en la TUI y en el modo headless.
2. No activar el prebuilt como canal por defecto sin decisión humana/ADR (H3/V9).
3. Toda dependencia de la TUI bajo S-9 y ninguna instalación de skills/MCP (S-15).
4. Derivar H1–H4 al orquestador para saneamiento/registro, sin editar rutas protegidas.

Cualquier infracción de V1–V9 convierte este dictamen en **BLOCK** con evidencia, sin auto-aprobación ni publicación.
