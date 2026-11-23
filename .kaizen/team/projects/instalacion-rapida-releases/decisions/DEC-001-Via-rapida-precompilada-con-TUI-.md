decision_id: DEC-001
topic: Via rapida precompilada con TUI nativa y fail-closed
source_ids: ["OWASP-001"]
retrieved: 2026-11-23T17:35:04Z
status: verified
human_owner: BDFL
---

# Decisión: Vía rápida precompilada con TUI nativa y fail-closed

## Evidencia

- **Fuente verificada citable:** `OWASP-001` (ASVS 4.0 / Top 10 2021; `retrieved` 2026-11-21, estado `verified` en `SOURCES.md`). Aplica **solo** a integridad de descargas y fallo cerrado del checksum (ASVS v14, A08:2021). No se cita para rendimiento, TUI ni versionado (REG-EV-02).
- **Hash de corte de esta decisión (2026-11-23):** `sha256sum install.sh` = `74411c2c8828b0bc796841e7ac8594a2e3f6c88b3601fda185357bba88434624`; `sha256sum install.ps1` = `9f6022f7186bbed86b93ef4942edb170510ad709d5a42b348793401c3fc9274c`. `sha256sum -c checksums/installers.sha256` → OK (el hash `1ebc3ad0…` del audit ya no corresponde al fichero actual; ver outbox `audit-hash-y-log-windows`).
- **Mediciones reales previas** (`docs/audits/install-user-journey.md`): canal `source` = 21 m 22 s con **dos** builds completos; canal precompilado sobre `file://` = 0.410 s con 0 acciones de Cargo; checksum inválido = exit 1 y ningún binario instalado. La medición de 0.410 s usa un binario sintético de 62 B sobre `file://`: **no mide red** (finops lo declara).
- **Observación del usuario (dato no confiable, L-5):** instalación real en Windows con cientos de líneas `Downloaded <crate>`, sin progreso ni ETA. Solo existe como transcripción en el blackboard, sin log crudo versionado.
- **Artefactos de la escuadra:** `evidencia-fuentes.md`, `diseno-arquitectura.md`, `diseno-tui.md`, `revision-seguridad.md`, `analisis-releases.md`, `plan-verificacion.md`, `impacto-documentacion.md` (todos en este proyecto). 7 handoffs con ack, `kaizen-team sync` → 0 acuses pendientes.

## Opciones y decisión

1. **Default del instalador.** El arquitecto propone invertir el default a un canal `release` con `source` solo explícito y sin fallback automático; plataforma y seguridad coinciden en que invertir la decisión documentada "never guesses a version" (RFC-0032 / `INSTALLATION_INTEGRITY.md`) **requiere ADR humano**. → **Decisión de este equipo: NO cambiar el default por iniciativa propia.** Se recomienda al humano aprobar ADR-N1 y, solo entonces, implementar.
2. **Fuente de versión.** Canónica = `[workspace.package].version` de `Cargo.toml` (`0.1.0-alpha.2`), validada con el mismo `awk` que `release.yml:105-116`. `VERSION=1.0.0` es etiqueta de intención del framework, **no** fuente del tag. → acordado como recomendación técnica; su regularización requiere ADR-N2 humano.
3. **TUI.** Dos propuestas en conflicto:
   - Arquitecto: reutilizar `ratatui`/`crossterm` ya presentes en `ag-cli`, con replay de eventos desde un `EventLog`. **Inconveniente:** ejecuta el binario descargado antes de copiarlo al `BIN_DIR`, rompiendo el invariante verificado "el binario descargado no se ejecuta durante la instalación" (seguridad); además la TUI no aparece hasta después de la descarga.
   - DX: TUI nativa en el propio script (ANSI/VT100 en bash; `System.Console` en PowerShell), 6 etapas reales, modo headless determinista, sin dependencias nuevas y sin ejecutar el binario descargado.
   → **Se adopta la propuesta de DX para esta decisión.** La opción del arquitecto queda diferida a ADR-N3/N5 con humano. El KPI se ajustó en consecuencia (TUI nativa, 0 ejecuciones del binario descargado).
4. **Integridad.** Se mantiene el orden fail-closed existente: self-integrity → manifest → binario → SHA-256 → copia atómica. `AG_INSTALLER_CHECKSUMS` y `AG_INSTALLER_BINARY_CHECKSUMS` conservan su semántica. El texto de la TUI debe decir "checksum verified / integrity, not authenticity"; prohibido "firmado", "auténtico" o "trust anchor". **No se crea clave, no se toca `SECURITY.md`, #141 sigue abierto, RFC-0030 sigue borrador.**
5. **Ruta de entrega.** Sin tocar `.github/`: resolver versión desde `Cargo.toml`, derivar tag, descargar, verificar, instalar; documentación K-7 (README, manual 04, `INSTALLATION_INTEGRITY.md` → alpha.2, CHANGELOG, descriptor). Con humano: publicar el draft `v0.1.0-alpha.2` (bloqueante: sin publicación no hay assets descargables), crear/push tags, y cualquier cambio de matriz o metadata `latest.json`.

**Decisión consolidada:** el diseño queda **aprobado como propuesta** (APPROVE condicionado de `security` y `finops`; `verificador-evidencias` emitió CHANGES_REQUESTED por acuses/hash, ya resueltos salvo la reproducción de Windows). **No se implementa ni se entrega**: la puerta `deliver` está bloqueada por humanos.

## Incertidumbre y escalado

- **needs-human (S-12, bloqueante):** `qa` reportó un bloque `env` con credenciales en texto plano en `docs/audits/tui-transcripts/config-noninteractive-flag.json`. No se abrió ni se copió ningún valor; outbox `secret-en-audit-tui-transcript.md`. Requiere inspección y **rotación por humano** antes de publicar cualquier PR; ver `docs/kaizen/runbooks/secret-leak.md`.
- **needs-human (publicación):** el release `v0.1.0-alpha.2` sigue en draft; publicarlo es acción humana (RFC-0032 §4.4). Sin esto, el KPI de ≤10 s no es alcanzable en el mundo real.
- **needs-human (ADR-N1..N6):** cambio de default, fuente de versión, topología de TUI, contrato de eventos, extensión de la TUI y reafirmación de "sin firma". K-1: sin ADR no hay decisión de arquitectura.
- **Pendiente de verificación:** ≤10 s y 0 compilaciones en Linux y Windows con el instalador **modificado** (el plan de QA existe; la columna "después" está vacía). Los criterios TUI y headless tampoco están implementados. No se fabrica evidencia (K-10).
- **Reglas duras preservadas:** sin firma, sin gasto, sin pagos, sin datos personales, sin `.github/`, `.kaizen/` de reglas ni `scripts/kaizen*` editados por este equipo; ningún push, PR ni publicación por agentes.

## Pos-verificacion (2026-11-23, orquestador)
`verify-kpi-offline.sh`: `real=0.059` / `0.093` s vs 1282.132 s antes; 0 invocaciones cargo/rustc; salida headless determinista y sin ANSI; checksum invalido y target no publicado -> exit 1 sin binario. Windows y descarga por red quedan pendientes de entorno real (K-10).

## Regeneracion del manifest de instaladores (2026-11-23)
Tras implementar los instaladores, `checksums/installers.sha256` se regenero:
`install.sh` = `8a08f28fc10523344126f0d2836952dd5262924e68fb7b5923c51c8f21a4356e`,
`install.ps1` = `b77ae838a43f4553237ff14d1219fc062c4c89b44c6fd033404aa5da1bbd9e6f`.
`sha256sum -c checksums/installers.sha256` → ambos OK (antes FAILED: los hashes del audit ya no coincidian). El manifest sigue siendo desarrollo, no trust anchor (S-11/INSTALLATION_INTEGRITY).
