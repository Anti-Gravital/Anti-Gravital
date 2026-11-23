# ADR-0011: instalación rápida por defecto mediante el canal `release` precompilado

Estado: aceptado
Fecha: 2026-11-23
Decisor: Angel Nereira (BDFL)

Aprobación humana explícita del BDFL otorgada el 2026-11-23 para esta decisión y para su registro con estado `aceptado`. El agente documenta la decisión; no la emite ni la aprueba por sí mismo (K-1, REG-EV-01).

## Contexto

El canal por defecto del instalador era `source`: toolchain + compilación del workspace (`cargo build --locked --workspace --release` y `cargo install --path crates/ag-cli --locked`). La auditoría reproducible `docs/audits/install-user-journey.md` midió `real=1282.132` s (**21 m 22 s**, líneas 71 y 94) con **dos** builds completos, frente a `real=0.410` s del canal precompilado sobre `file://` con cero acciones de Cargo (líneas 135 y 148). Comandos de reproducción documentados en `docs/audits/install-user-journey.md:55-95,114-160`: `time bash install.sh` y `AG_INSTALLER_CHANNEL=alpha AG_INSTALLER_BASE_URL=file://<release> time bash install.sh`. La cifra de 0.410 s usa un binario sintético de 62 B sobre `file://`: **no mide red** (`DEC-001` §Evidencia; `diseno-tui.md` FU-04).

El release `v0.1.0-alpha.2` ya produce 8 binarios (`ag` y `ag-lsp` × 4 targets) y `checksums.txt` (`.github/workflows/release.yml:31-43,54-72,123-131`). El propio workflow valida que el tag coincida con `[workspace.package].version` de `Cargo.toml` (`.github/workflows/release.yml:105-116`) y crea el release como `draft` + `prerelease` (`:184-197`); la publicación es una acción humana (RFC-0032 §4.4:135-148).

Conviven tres valores de versión: `Cargo.toml` = `0.1.0-alpha.2` (canónico), `VERSION` = `1.0.0` (etiqueta de intención del roadmap / versión del framework Kaizen, ADR-0010) y ejemplos `v0.1.0-alpha.1` en `docs/security/INSTALLATION_INTEGRITY.md:26,30,96`. Derivar el tag de `VERSION` produciría `v1.0.0`, inexistente (`analisis-releases.md §2, §6 E3/E7`). Además, `resolve_base_url()` abortaba sin `AG_INSTALLER_BASE_URL`/`AG_INSTALLER_RELEASE_TAG` y los instaladores detectaban triples no publicados (`x86_64-apple-darwin`, `aarch64-pc-windows-msvc`) que terminan en 404 (`analisis-releases.md §3.1, §7 hallazgo 3`; `diseno-arquitectura.md §2.2`).

Seguridad emitió `APPROVE` condicionado y el veto V9 a cambiar el default sin decisión humana (`revision-seguridad.md §5-6`). `DEC-001` recomienda al humano aprobar esta ADR antes de implementar.

### Evidencia

Etiquetas: `verified` = comprobado en esta iteración con comando de solo lectura; `source-backed` = consta en fuente interna citada; `inference` = deducción; `unknown` = sin verificar. No se inventa ningún `source_id` (REG-EV-01/K-10).

| Hecho | Etiqueta | Fuente verificable |
| --- | --- | --- |
| Versión canónica `0.1.0-alpha.2` | verified | `grep -n -A3 '^\[workspace.package\]' Cargo.toml` → `Cargo.toml:37`; validada en `.github/workflows/release.yml:105-116` |
| `VERSION` = `1.0.0`, no leído por el pipeline | verified | `cat VERSION` → `1.0.0`; `analisis-releases.md §6 E3` |
| 21 m 22 s (1282.132 s), dos builds | source-backed | `docs/audits/install-user-journey.md:71,94` |
| 0.410 s sobre `file://`, 0 acciones de Cargo | source-backed | `docs/audits/install-user-journey.md:135,148`; alcance declarado en `DEC-001` §Evidencia |
| 4 targets y 8 assets publicados | verified | `sed -n '31,43p' .github/workflows/release.yml` |
| Recomendación humana y veto previo | source-backed | `DEC-001:21-22`; `revision-seguridad.md §5-6` (V9) |

## Decisión

1. **Canal por defecto.** `AG_INSTALLER_CHANNEL=release` es el valor por defecto: descarga del binario versionado `ag-<target>[.exe]`, verificación SHA-256 contra `checksums.txt` y copia atómica. `alpha` se acepta como alias con aviso de migración; `source` solo se activa de forma explícita (`AG_INSTALLER_CHANNEL=source`).
2. **Sin fallback automático.** Si falla la descarga o la verificación, el instalador NO compila: emite un evento `failed` con código estable (`E_DOWNLOAD_FAILED`, `E_MANIFEST_MISSING`, `E_CHECKSUM_MISMATCH`) y un mensaje accionable con la orden exacta para compilar, dejando `BIN_DIR` intacto. La fricción es intencionada: un fallback automático convertiría 0.410 s en 21 m 22 s sin que el usuario lo pidiera (`diseno-arquitectura.md §2.4`).
3. **Fuente de versión canónica.** `[workspace.package].version` de `Cargo.toml` (`0.1.0-alpha.2` en la fecha de esta decisión), leído con el mismo `awk` de `.github/workflows/release.yml:105-116` y validado con el formato estricto `^[0-9]+\.[0-9]+\.[0-9]+-alpha\.[0-9]+$`. `VERSION=1.0.0` queda como **etiqueta informativa**: una divergencia se reporta como `version_source_mismatch=true` sin fallar y sin alterar el tag. No se edita `VERSION` en esta decisión; sincronizarlo o eliminarlo requiere ADR humano.
4. **Guard de targets no publicados.** Tabla cerrada de los cuatro triples de `.github/workflows/release.yml:31-43` (`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`). Cualquier otro triple, incluidos `x86_64-apple-darwin` y `aarch64-pc-windows-msvc`, se rechaza con `E_UNSUPPORTED_TARGET` **antes** de cualquier descarga. Ampliar la matriz exige tocar `.github/workflows/release.yml` (L-4) y otro ADR humano.
5. **Orden fail-closed intacto.** self-integrity → manifest → binario → SHA-256 → copia atómica, sin ejecutar el binario descargado. El checksum sigue siendo integridad, no autenticidad (ADR-0013).
6. **Alcance de archivos.** La implementación vive en `install.sh`, `install.ps1` y documentación K-7; **no** se modifica `.github/`, `scripts/`, `Cargo.toml`, `VERSION` ni `SECURITY.md`.
7. **Publicación del draft.** El release `v0.1.0-alpha.2` ya fue **publicado por un humano** conforme a RFC-0032 §4.4. La publicación permanece como acción humana fuera de la automatización y no se deshace por esta ADR.

## Alternativas consideradas

| Alternativa | Motivo de rechazo |
| --- | --- |
| Mantener `source` como default | Coste medido de 21 m 22 s y dos builds; oculta el progreso al usuario (`install-user-journey.md:71,94`; `DEC-001` §Evidencia) |
| Fallback automático a `source` ante un fallo del canal rápido | Convierte 0.410 s en ~21 min sin consentimiento; rompe el criterio del charter (`diseno-arquitectura.md §2.4`, PROP-09) |
| Derivar el tag de `VERSION` | `v1.0.0` no existe → 404 o error de formato (`analisis-releases.md §2, §6 E3/E7`) |
| Añadir targets (Mac Intel / Win ARM64) o metadata `latest.json` | Exige `.github/workflows/release.yml` (L-4), humano y ADR; diferido (`analisis-releases.md §3.3, §5`) |
| Publicar los binarios ya firmados | No existe clave, #141 sigue abierto y RFC-0030 es borrador (ADR-0013) |

## Consecuencias (positivas y negativas)

**Positivas**

- Arranque rápido y cero compilaciones en el canal rápido, con la medición previa de 0.410 s como referencia (no como garantía de red).
- Versión reproducible directamente desde el checkout, alineada con la validación del workflow.
- Fallo explícito y fail-closed, con la orden exacta para la ruta `source` cuando el usuario la quiera.
- Sin dependencias nuevas y sin tocar rutas protegidas; reversible mediante override.

**Negativas**

- El canal rápido depende de que el release esté publicado; si el draft no estuviera publicado, la descarga daría 404 (el bloqueante ya resuelto por el humano, §Decisión 7).
- `VERSION` sigue divergente: deuda documental que solo un humano puede regularizar.
- Un checkout desactualizado puede derivar un tag viejo o inexistente; mitigado por validación de formato y fail-closed, nunca por "mejor esfuerzo".
- El binario y su checksum provienen del mismo release: riesgo de sustitución en el mismo canal, mitigado —no eliminado— por `AG_INSTALLER_BINARY_CHECKSUMS` (ADR-0013).

## Rollback

Mitigación inmediata sin revertir nada: `AG_INSTALLER_CHANNEL=source bash install.sh` (o `.\install.ps1` en Windows) fuerza la ruta original. Reversión completa: revertir los commits que implementaron el canal `release` en `install.sh`, `install.ps1` y la documentación inventariada en `diseno-arquitectura.md §5` e `impacto-documentacion.md`, **sin tocar `.github/`** y **sin deshacer el release publicado**: no se eliminan ni reescriben tags ni assets (`analisis-releases.md §4 capa (b), rollback`). Volver a poner el release en draft sería una acción humana separada, no prevista aquí. Verificar con `git diff --name-only` y reejecutando los criterios CA-1..CA-4 de `diseno-arquitectura.md §7`.

## Cómo se verificará

- `env -u AG_INSTALLER_CHANNEL AG_INSTALLER_BASE_URL=file://<release> bash install.sh` sobre un release sintético: `grep -E 'cargo build|Updating crates.io index|Downloaded '` sin coincidencias, `ag` instalado (CA-1).
- `VERSION=1.0.0` con workspace `0.1.0-alpha.2`: el evento `resolve_version` reporta `tag="v0.1.0-alpha.2"` y `version_source_mismatch=true`; la URL pedida contiene `v0.1.0-alpha.2`, nunca `v1.0.0` (CA-3).
- `AG_INSTALLER_TARGET=x86_64-apple-darwin` y `aarch64-pc-windows-msvc`: rechazo con `E_UNSUPPORTED_TARGET` antes de descargar (CA-4).
- Checksum inválido, entrada ausente, manifest ausente y binario ausente: exit ≠ 0 y `BIN_DIR` sin `ag` (CA-2).
- Honestidad de medición: 21 m 22 s y 0.410 s son cifras **previas** del audit; los valores posteriores al cambio están pendientes de ejecución (`DEC-001` §Incertidumbre, `plan-verificacion.md`). Esta ADR no afirma que el KPI ≤10 s esté verificado en red.

## Referencias

- `docs/rfc/RFC-0032-alpha-release-process.md` §4.1, §4.3, §4.4, §9.
- `docs/security/INSTALLATION_INTEGRITY.md` §Alpha channel, §Known limitations.
- `.kaizen/team/projects/instalacion-rapida-releases/decisions/DEC-001-Via-rapida-precompilada-con-TUI-.md` (ítems 1, 2 y 5).
- `.kaizen/team/projects/instalacion-rapida-releases/diseno-arquitectura.md` §2, §6, §7; `analisis-releases.md` §2-§4, §6; `revision-seguridad.md` §5-§6.
- `docs/audits/install-user-journey.md:71,94,135,148`; `docs/kaizen/adr/0010-versionado-y-propagacion.md`; issues #141, #203; ADR-0012, ADR-0013.
