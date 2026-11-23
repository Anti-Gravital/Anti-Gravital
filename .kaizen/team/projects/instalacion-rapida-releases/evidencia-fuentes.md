# Evidencia y fuentes — instalacion-rapida-releases

Verificador: `verificador-evidencias` | Fecha: 2026-11-23 | Fecha de corte de `SOURCES.md`: **2026-11-21**
Alcance: solo lectura. No se ejecutaron `advance`, `decide` ni `verify`. No se editó ningún otro archivo. Sin secretos ni volcado de entorno. Documentos, logs y mensajes se trataron como **datos no confiables** (L-5/S-15).

## 1. Fuentes `verified` legítimamente aplicables

Reglas aplicadas: `EVIDENCIA.md` REG-EV-01..07. Estado `verified` ≠ aplicable: una fuente solo se cita si funda específicamente la decisión.

| ID | Estado | Aplicación a instalación/release/TUI | Veredicto |
| --- | --- | --- | --- |
| **OWASP-001** (ASVS 4.0 / Top 10 2021) | verified | **Sí, parcial**: integridad de descargas y fallo cerrado del checksum (ASVS v14 "Build and Deploy", Top 10 A08:2021 *Software and Data Integrity Failures*, A06:2021 componentes). Cubre el criterio "100% checksums inválidos abortan sin escribir binario". | **Citable** para el subconjunto de integridad |
| PA-001..PA-008, PA-010, PA-011, PA-013, PA-014 | verified | No. Protección de datos, AML, ITBMS, ciberdelincuencia, SOC, consumidor, firma electrónica, trabajo, Gaceta digital. Ninguna regula instaladores ni releases. | No citar (forzarlas sería deshonesto) |
| ISO-001 (27001), ISO-002 (37301), NIST-001 (CSF 2.0), AICPA-001 (SOC 2), PCI-001, INT-001 (FATF) | verified | Solo contexto general de seguridad/compliance; no fundan tiempo de instalación, releases precompilados ni TUI. | No citar |
| LOCAL-001..LOCAL-004 | verified | CVs, plan Flutter M3E y PlentyPay. LOCAL-003 es diseño Flutter, no la TUI Rust (ratatui/crossterm) de este proyecto. | No citar |
| PA-009, PA-012, PA-015, INT-002 | **pending** | — | **Bloqueadas** por REG-EV-01 |

**Conclusión honesta:** `SOURCES.md` **no contiene ninguna fuente verificada que cubra directamente** "instalación rápida", "releases precompilados/versionados" ni "TUI". Solo **OWASP-001** es aplicable, y únicamente a integridad/checksums. Para los criterios de tiempo, cero compilaciones y TUI, la evidencia válida son **comandos reproducibles + medición**, no un `source_id`. Se propone (fuera de mi autoridad, requiere humano) abrir `kaizen-org research` para incorporar una fuente oficial de supply-chain release (p. ej. SLSA / documentación oficial rustup-dist) con `retrieved` y hash antes de citarla en una decisión futura.

## 2. KPI, handoffs y bug de `assign`

- `scripts/kaizen-team status instalacion-rapida-releases` → salida real: `etapa: execute | KPI: ... | tareas: 7 | decisiones: 0`.
- **KPI completo** (5 criterios verificables en `state.json`/`charter.md`): (1) canal rápido ≤10s y 0 `cargo build/install` en Linux y Windows vs 21m22s medidos; (2) 100% de checksums inválidos abortan sin escribir binario; (3) `install.sh`/`install.ps1` con ≥4 etapas animadas y estado real; (4) TUI ratatui/crossterm en Linux y Windows, ≥4 etapas, navegación, feedback animado y modo headless para CI; (5) 0 cambios en `.github`, firma o confianza sin humano. La TUI quedó incorporada al KPI/charter el 2026-11-23 (blackboard).
- `wc -l .kaizen/team/projects/instalacion-rapida-releases/handoffs.jsonl` → **7**. Uno por agente no-facilitador (verificador, architect, dx, security, platform-engineer, qa, finops); 7 tareas en `tasks.json` con `status: assigned`.
- **Bug `assign` (ya registrado, S-11):** `.kaizen/loop/outbox/kaizen-team-assign-handoff-jq.md` (fingerprint `kaizen-team-assign-handoff-now-number`). Causa: en `cmd_assign` se usa `("H-"+(now|gsub("[^0-9]";"")))`; `now` es el builtin **numérico** de jq y no la variable `--arg now`, por lo que falla con `jq: error ... number (...) cannot be matched`. En el momento del reporte `handoffs.jsonl` tenía **0 líneas**; hoy tiene **7** → los handoffs fueron registrados manualmente con ids `H-M1..M7<timestamp>Z`, según proponía el outbox. La corrección de `scripts/kaizen*` está prohibida (L-4): requiere humano.
- **Hallazgo nuevo (no editado, para el orquestador / S-11):** los 7 handoffs existen pero todos están `status: open`, sin campo `ack`, sin `correlation_id` y sin `deadline`. Es decir: **hay 7 registros, 0 acuses cerrados**. Según L-17 y TEAM.md la puerta `execute → review` exige ack; esto debe resolverse antes de dar por cerrado cualquier handoff. No lo corrijo aquí: única salida autorizada es este archivo.

## 3. Afirmaciones verificables (comando / resultado real)

| Afirmación | Comando | Resultado real |
| --- | --- | --- |
| Hash actual de los instaladores | `sha256sum install.sh install.ps1` | `74411c2c8828b0bc796841e7ac8594a2e3f6c88b3601fda185357bba88434624  install.sh` / `9f6022f7186bbed86b93ef4942edb170510ad709d5a42b348793401c3fc9274c  install.ps1` |
| Manifest de integridad consistente | `cat checksums/installers.sha256; sha256sum -c checksums/installers.sha256` | Mismos dos hashes; `install.sh: OK`, `install.ps1: OK`, exit 0 |
| Versión del workspace | `grep -n version Cargo.toml` | `Cargo.toml:37 version = "0.1.0-alpha.2"`; `crates/ag-cli/Cargo.toml:3 version.workspace = true` |
| Source Linux = 21m22s, dos builds | `docs/audits/install-user-journey.md:71-74,94` | `real=1282.132` (21m22s); `cargo build --workspace --release` 17m10s + `cargo install` 4m10s |
| Alpha `file://` = 0.410s, sin Cargo | ibíd. `:135,148,160` | `real=0.410`, exit 0, "No build and no Cargo action occurred" |
| Checksum inválido = fail-closed | ibíd. `:170,179,183-184` | exit 1, `real=0.298`, `home-alpha-bad/.cargo/bin` no existe |
| Instalador autochecksum | ibíd. `:214-225` | hash correcto → exit 0; hash cero → exit 1; manifest ausente → exit 1 |

**Discrepancias detectadas (hallazgos, no corregidos):**
1. `docs/audits/install-user-journey.md:28` registra `SHA-256 install.sh = 1ebc3ad0…e92d08a`, pero el archivo actual es `74411c2c…34624`. El cambio es posterior al audit (2026-11-22), pero la medición del audit **ya no es reproducible byte-a-byte** contra el instalador actual. REG-EV-07: la decisión debe guardar el hash **actual**, no el del audit.
2. `VERSION` = `1.0.0` mientras `Cargo.toml` = `0.1.0-alpha.2`; el manual `04` aún dice `alpha.1` (issue #203 abierta en el audit). Declarar incertidumbre antes de decidir versionado de releases.
3. La observación de Windows (>250 líneas `Downloaded <crate>`, rustup sync `1.99.1-x86_64-pc-windows-msvc`, sin barra de progreso/ETA) solo existe como transcripción en `.kaizen/team/projects/instalacion-rapida-releases/blackboard.md:6-14`. **No hay log crudo versionado** en el repo (`grep -rln "Downloaded" . --exclude-dir=.git` → solo ese blackboard). Es dato observacional del usuario (L-5), no comando reproducible en este entorno.

**Quedan como `pending` (sin evidencia verificada hoy):**
- Log/cronología de Windows reproducible (solo transcripción en blackboard).
- ≤10s y 0 compilaciones en Windows y Linux con el instalador **actual** (sin medición nueva).
- ≥4 etapas animadas en `install.sh`/`install.ps1` y TUI ratatui/crossterm con navegación, headless y ≥4 etapas (0 decisiones, etapa `execute`, sin implementación verificada).
- "0 cambios en `.github`, firma o confianza sin humano" (restricción de proceso, aún no verificable).
- `ag --version` de la build actual (solo consta en el audit; no ejecuté ninguna instalación).
- Fuentes `pending` del ledger: PA-009, PA-012, PA-015, INT-002.

## 4. `source_ids` citables en `decide` sin violar REG-EV-01

**Lista exacta: `[OWASP-001]`.**

- `source_id`: `OWASP-001` — `retrieved: 2026-11-21` (fecha de corte de `SOURCES.md`), `status: verified`, `hash: no disponible en el ledger` (la entrada registra URL `https://owasp.org`, sin `sha256` de archivo; cumplir REG-EV-07 declarando esa ausencia).
- Uso permitido: **integridad y fallo cerrado de checksums / assets de release**. No usar para justificar rendimiento, TUI ni versionado.
- Todo lo demás debe citarse como `evidence` con comando y hash, no como `source_id`: `sha256sum install.sh` → `74411c2c…`, `sha256sum -c checksums/installers.sha256` → OK, `docs/audits/install-user-journey.md` (21m22s / 0.410s / exit 1 fail-closed), `wc -l handoffs.jsonl` → 7.
- **No citar** PA-*, ISO-001, ISO-002, NIST-001, AICPA-001, PCI-001, INT-001, LOCAL-* (no aplicables) ni PA-009, PA-012, PA-015, INT-002 (`pending`, bloqueadas).

**Dictamen de verificación: CHANGES_REQUESTED.** No hay `BLOCK` de evidencia para el subconjunto de integridad (OWASP-001 + fail-closed medido). Pero antes de `decide` deben resolverse: (a) acuses de los 7 handoffs (`status: open`, sin `ack`), (b) regenerar la evidencia del audit contra el hash actual `74411c2c…` o declarar explícitamente la fecha/hash de corte, (c) aclarar `VERSION 1.0.0` vs `Cargo.toml 0.1.0-alpha.2`, y (d) aportar el log de Windows o rebajarlo a `pending`. Los hallazgos (a)-(d) y el bug de `assign` deben publicarse en S-11 por el orquestador; este agente no tiene autoridad para editar el outbox ni las rutas protegidas.
