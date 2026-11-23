# ADR-0013: el checksum es integridad, no autenticidad

Estado: aceptado
Fecha: 2026-11-23
Decisor: Angel Nereira (BDFL)

Aprobación humana explícita del BDFL otorgada el 2026-11-23 para esta decisión y para su registro con estado `aceptado`. El agente documenta la decisión; no la aprueba por sí mismo (K-1, REG-EV-01).

## Contexto

RFC-0032 definió un alpha sin firma: el manifiesto `checksums.txt` se genera en el mismo dominio de confianza que los binarios que atestigua (`.github/workflows/release.yml:123-131,153-167`) y no constituye un ancla de confianza. No existe clave de firma, no se ha hecho la ceremonia de RFC-0030, el issue #141 sigue abierto y RFC-0030 continúa en borrador (`docs/security/INSTALLATION_INTEGRITY.md:182-204`; `DEC-001` ítem 4). `security` elevó a veto V5 cualquier cambio que llame "autenticado" al canal por defecto o que trate el `checksums.txt` del propio release como trust anchor (`revision-seguridad.md §2, §5`).

La TUI y la documentación deben reutilizar la redacción honesta ya presente: `install.sh` ("Checksum verification only (no signing key, no signature)" / "Checksum verified for …") e `install.ps1` ("checksum verification only (integrity, not authenticity)"). `revision-seguridad.md §3` registró esas líneas como `install.sh:176,212` e `install.ps1:125,158` en el HEAD analizado por la escuadra; una lectura actual con `grep -n 'checksum verification only\|Checksum verified\|no signing key' install.sh install.ps1` da `install.sh:428,514` e `install.ps1:531`, porque el worktree está en cambio concurrente: el texto se mantiene, los números de línea pueden derivar. Las notas del release conservan "Integrity, not authenticity … NOT a signature and NOT a trust anchor" (`.github/workflows/release.yml:153-167`).

### Evidencia

Etiquetas: `source-backed` = fuente verificada registrada; `verified` = comprobado en esta iteración con comando de solo lectura; `inference` = deducción; `unknown` = sin verificar.

| Hecho | Etiqueta | Fuente verificable |
| --- | --- | --- |
| Integridad de descargas y fallo cerrado del checksum | source-backed | `OWASP-001` — OWASP ASVS 4.0 / Top 10 2021, A08:2021 (ASVS v14), registrada como `verified` en `.kaizen/org/knowledge/SOURCES.md:31` (`retrieved` 2026-11-21). Aplica **solo** a integridad/fail-closed; no a rendimiento, TUI ni versionado (REG-EV-02, `DEC-001` §Evidencia) |
| Sin clave, sin `.minisig`, sin `SECURITY.md`, #141 abierto | source-backed | `docs/rfc/RFC-0032-alpha-release-process.md §4.5, §9`; `DEC-001:27`; `revision-seguridad.md §2` |
| Redacción actual del release | verified | `grep -n 'integrity\|authenticity\|trust anchor\|signature' .github/workflows/release.yml` → `:153,161,165,175` |
| Estado real de #141 en el tracker | unknown | No consultado en esta iteración (contenido externo, S-15); se reporta según RFC-0032/DEC-001, sin verificación independiente |

No se inventa ningún otro `source_id`: los documentos internos se citan por ruta y comando reproducible (REG-EV-01/K-10).

## Decisión

1. **El checksum SHA-256 es integridad, NO autenticidad.** Demuestra que el binario descargado coincide byte a byte con el publicado en el release; **no** demuestra quién lo publicó ni que el canal sea de confianza. Quien controla el `BASE_URL` puede sustituir binario y checksum a la vez (`INSTALLATION_INTEGRITY.md:77-90`; RFC-0032 §4.3:127-130).
2. **Sin nueva ancla de confianza.** No se crea clave de firma, no se genera `.minisig` ni firma GPG, no se publica huella, no se modifica `SECURITY.md`, no se cierra #141 y RFC-0030 sigue borrador. Está prohibido afirmar "verificado" en sentido absoluto, "firmado", "auténtico", "seguro", "de confianza" o "trust anchor", y usar iconos de candado/check verde que impliquen autenticidad (V5/V6).
3. **Redacción obligatoria** en TUI, headless y documentación: **"checksum verified / integrity, not authenticity"** (en español: "checksum verificado / integridad, no autenticidad"). Se reutilizan los mensajes reales citados en Contexto. El aviso de `AG_INSTALLER_CHECKSUMS` sin definir se muestra como aviso persistente, nunca se disfraza de éxito ni se silencia (S-10).
4. **Mitigación out-of-band, no firma.** `AG_INSTALLER_BINARY_CHECKSUMS` acepta un manifiesto local obtenido por un canal revisado independientemente y **suprime la descarga** de `checksums.txt`; mueve el origen del checksum fuera del release, pero **no autentica** el binario (`INSTALLATION_INTEGRITY.md:85-90`). `AG_INSTALLER_CHECKSUMS` conserva su semántica de autoverificación del script. El orden fail-closed (self-integrity → manifest → binario → SHA-256 → copia atómica) no se reordena ni se relaja (I1/V1/V2).
5. **Supersición.** Esta ADR se da por superada solo mediante una **nueva RFC** (sucesora de RFC-0032 / aceptación de RFC-0030) y una decisión **BDFL** documentada, con ceremonia de clave, publicación out-of-band y la evidencia exigida para cerrar #141. Nunca por un cambio unilateral de redacción, de canal o de librería.

## Alternativas consideradas

| Alternativa | Motivo de rechazo |
| --- | --- |
| Firmar los assets ahora | Requiere la ceremonia de clave de RFC-0030, dos canales out-of-band y demostración integral; no existe clave y #141 está `blocked` (RFC-0032 §3, §4.5) |
| Tratar `checksums.txt` del release como trust anchor | Viaja en el mismo dominio de confianza que los binarios; VETO V5 (`.github/workflows/release.yml:161`; `revision-seguridad.md §2`) |
| Llamar "verified install" al canal por defecto | Induce falsa confianza; V5/V6 y RFC-0032 §6 ("integrity only, not authenticity") |
| Ocultar el warning de `AG_INSTALLER_CHECKSUMS` unset | Supresión prohibida por S-10; es una ayuda al desarrollo, no un control (`INSTALLATION_INTEGRITY.md:169-180`) |

## Consecuencias (positivas y negativas)

**Positivas**

- Honestidad verificable: la cadena de confianza se describe como es, sin inventar un ancla.
- Fail-closed preservado y superficie criptográfica nueva igual a cero.
- `AG_INSTALLER_BINARY_CHECKSUMS` ofrece una mitigación real y reversible contra la sustitución en el mismo canal.
- Una sola redacción para TUI, headless, release y documentación.

**Negativas**

- No existe autenticidad criptográfica: el usuario debe obtener el manifiesto por un canal que él mismo proteja para mitigar la sustitución.
- #141 permanece abierto y cualquier futuro cambio de confianza exige RFC + BDFL.
- La redacción queda congelada como contrato: cambios unsafe deben revisarse y no deben debilitar el descargo.

## Rollback

No aplica una reversión: esta ADR es una **reafirmación de no-hacer**. Si en el futuro se decide introducir firma, no se "revierte" esta decisión: se emite una RFC sucesora y el BDFL marca esta ADR como `reemplazada por ADR-XXXX`. Hasta entonces, cualquier cambio que contradiga los puntos 2-4 se revierte y se registra como hallazgo S-11; `security` mantiene veto vinculante (V5/V6).

## Cómo se verificará

- Revisar la salida de la TUI y del headless: aparece "checksum verified / integrity, not authenticity" (o su equivalente en español) y **no** aparecen "firmado", "auténtico", "trust anchor" ni un check verde que implique autenticidad (CA-3/CA-4 de `revision-seguridad.md §3`).
- `git diff --name-only`: cero cambios en `SECURITY.md`, `.github/` ni `tools/release-signing/`; `find . -name '*.minisig'` → sin resultados.
- Checksum inválido: exit ≠ 0 y ningún binario escrito (`docs/audits/install-user-journey.md:162-184`); con `AG_INSTALLER_BINARY_CHECKSUMS` apuntando a un manifiesto local, no se descarga `checksums.txt` (`:194-208`; `revision-seguridad.md §1 I3`).
- #141 y RFC-0030: se reportan según fuente interna; su estado en el tracker externo queda como `unknown` hasta verificación humana (no se fabrica evidencia, K-10).

## Referencias

- `OWASP-001` (`.kaizen/org/knowledge/SOURCES.md:31`) — íntegramente limitado a integridad y fallo cerrado.
- `docs/rfc/RFC-0032-alpha-release-process.md` §4.3, §4.5, §6, §9; RFC-0030 (borrador, sin clave).
- `docs/security/INSTALLATION_INTEGRITY.md:46-49,77-90,182-204`; `.github/workflows/release.yml:123-131,153-167`.
- `.kaizen/team/projects/instalacion-rapida-releases/revision-seguridad.md` §1-§5 (I1/I3, V1/V2/V5/V6); `DEC-001:27`.
- Issue #141 (abierto, `blocked`); ADR-0011, ADR-0012.
