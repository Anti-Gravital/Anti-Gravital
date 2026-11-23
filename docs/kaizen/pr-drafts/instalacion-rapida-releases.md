## Resumen

Documentar la vía rápida precompilada por defecto, la TUI nativa de seis etapas con modo headless y corregir la deriva documental del instalador.

## Componente afectado

Documentación de instalación, integridad y release del proyecto `instalacion-rapida-releases`. El comportamiento del instalador (`install.sh`, `install.ps1`) lo implementan otros roles en paralelo: **este cambio no toca esos archivos**.

## Tipo de cambio

- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [x] documentation
- [ ] refactor

## Estado y procedencia

Descriptor creado el 2026-11-23 por `finops` bajo K-4/K-7. Sin commit, push ni PR (los ejecuta el orquestador, K-12/L-3). No se fabrica evidencia: la verificación final de los criterios CA es responsabilidad de QA sobre el instalador modificado; este descriptor recoge el plan, no un resultado (K-10).

## Alcance

- Canal por defecto `release`: versión resuelta desde `[workspace.package].version` de `Cargo.toml` (`0.1.0-alpha.2`, **no** `VERSION`), binario precompilado versionado descargado y verificado por SHA-256 antes de instalar, sin compilación ni descarga de toolchain.
- `AG_INSTALLER_CHANNEL=source` permanece como ruta explícita (unos 21 minutos y dos builds en el host medido). **No hay fallback automático** de `release` a `source`.
- TUI nativa de seis etapas (resolver versión, detectar plataforma, descargar, verificar SHA-256, instalar, listo) con modo headless para CI (`AG_INSTALLER_PROGRESS=headless`; también `CI`, `TERM=dumb`, `NO_COLOR` o stdout no TTY), una línea `[ag-install] N/6 ...` por etapa y orden fail-closed intacto.
- Corrección documental: ejemplos `v0.1.0-alpha.1` → `v0.1.0-alpha.2`, alcance de los manifests de checksum y reiteración de "integrity, not authenticity" (#141 abierto, RFC-0030 borrador).
- Hecho de release: `v0.1.0-alpha.2` está **publicado** (no draft, 9 assets); la publicación es acción humana (RFC-0032 §4.4) y no la realizó este rol.

## Documentos tocados (K-7)

1. `README.md` (secciones EN y ES: tabla de versión, *Try it / Probarlo*, *Works / Funciona*, *Does not yet work / Aun no funciona*, *Quick start / Inicio rapido*, *No package-manager install*).
2. `docs/manual/04-instalacion-y-onboarding.md` (EN y ES: prerequisitos, canales, vista de progreso/headless, verificación de versión `0.1.0-alpha.2`).
3. `docs/security/INSTALLATION_INTEGRITY.md` (default `release`, versión desde `Cargo.toml` no `VERSION`, `checksums.txt` no cubre instaladores, `AG_INSTALLER_CHECKSUMS` / `AG_INSTALLER_BINARY_CHECKSUMS`, "integrity, not authenticity", ejemplos alpha.2).
4. `docs/releases/0.1.0-alpha.2.md` (**addendum aditivo fechado 2026-11-23**; evidencia histórica intacta).
5. `CHANGELOG.md` (entrada `Unreleased`; ningún issue cerrado).
6. `docs/kaizen/pr-drafts/instalacion-rapida-releases.md` (este descriptor).

No tocados: `install.sh`, `install.ps1`, `.github/`, `.kaizen/`, `scripts/`, `checksums/`, `VERSION`, `Cargo.toml`, ADR/RFC ni `docs/audits/tui-transcripts/config-noninteractive-flag.json` (S-12).

## Criterios de aceptación resumidos (CA-1..CA-12)

Basados en `diseno-tui.md` §6 y, para CA-12, derivados de `DEC-001` / `impacto-documentacion.md`.

| # | Criterio (resumen) |
| --- | --- |
| CA-1 | TTY: 6 etiquetas de etapa, indicador activo, 1 línea de estado, `Listo`, `0 compilaciones`; ≤25 filas, sin línea por evento ni chistes. |
| CA-2 | `NO_COLOR`: símbolos y palabras sin secuencias `\033[`. |
| CA-3 | `CI=true`: formato exacto `[ag-install] N/6 ...`, `6/6 Listo`, sin spinner/`\r`/color. |
| CA-4 | SHA-256 modificado: `4/6 ... FALLO`, sin binario escrito, `exit=3`, etapas 5-6 `OMITIDO`. |
| CA-5 | `q` en etapa 3: overlay de confirmación; `n` continúa. |
| CA-6 | `?`: overlay de ayuda con `q`, `?`, `Enter`, `Ctrl-C`. |
| CA-7 | `Ctrl-C`: limpieza del parcial y salida `130`. |
| CA-8 | `AG_INSTALLER_PROGRESS=headless` fuerza modo headless sin ANSI. |
| CA-9 | Captura del canal rápido: `0 compilaciones`, sin `cargo build`, `cargo install`, `Updating crates.io index` ni `Downloaded <crate>`. |
| CA-10 | Etapa 5 muestra la ruta real, no el literal `~/.cargo/bin` (#204). |
| CA-11 | Warning de `AG_INSTALLER_CHECKSUMS` visible, no omitido ni convertido en éxito. |
| CA-12 | Versión resuelta desde `Cargo.toml` (`0.1.0-alpha.2`), nunca desde `VERSION`; `source` solo explícito y sin fallback automático (derivado de DEC-001). |

Códigos de salida: `0` ok · `1` cancelado · `2` red · `3` checksum inválido · `4` ruta/permisos · `130` SIGINT · `143` SIGTERM.

## Plan de prueba (a ejecutar por QA/orquestador antes del PR)

- `bash -n install.sh` y análisis de `install.ps1` (lectura; sin supresiones S-10).
- `script -qec 'AG_INSTALLER_CHANNEL=release bash install.sh' /dev/null` (CA-1); `NO_COLOR=1 ... >out.txt` y `grep -q $'\033' out.txt` → falso (CA-2).
- `CI=true AG_INSTALLER_PROGRESS=headless bash install.sh >out.txt 2>&1` y comparación con el formato de 3.2 de `diseno-tui.md` (CA-3/CA-8).
- Casos fail-closed con `file://` y checksum manipulado/entrada ausente/binario ausente: `rc=1` o `exit=3`, sin binario copiado (CA-4; ver `plan-verificacion.md` §4.2).
- `AG_INSTALLER_CHANNEL=source` en entorno aislado: medir los dos builds (~21 min de referencia) y confirmar que no hay fallback (CA-9/CA-12).
- `git diff --check` sobre los seis archivos tocados; `scripts/kaizen check` lo ejecuta el orquestador al integrar (no simulado aquí, K-10).
- `grep -n 'alpha\.1' docs/security/INSTALLATION_INTEGRITY.md docs/manual/04-instalacion-y-onboarding.md` → sin salida.

## Impacto en calidad (ratchet)

- Cobertura: igual (solo documentación).
- Advertencias: igual.
- Deuda (TODO/FIXME): igual.
- Coste: cero gasto nuevo decidido aquí; CDN/mirror/tarifa son `needs-human` con `source_ids` de proveedor (`impacto-documentacion.md` §2).

## Rollback (L-7)

1. Revertir los commits de documentación en una rama `kaizen/*` junto con los cambios del instalador que describen.
2. Restaurar el texto anterior de los seis documentos en el mismo cambio para que doc y código no discrepen (K-1), sin borrar el addendum de release si el revert es solo parcial: en ese caso marcar el addendum como `superseded` con fecha.
3. No borrar assets ni tocar `.github/`, workflow de release, firma ni rangos de versión; publicar/despublicar es humano (RFC-0032 §4.4/§8).
4. Si el canal rápido deja de estar disponible: volver a `source` como default y restaurar mensajes; la interfaz ya existe, es un revert de comportamiento.
5. Si dos enfoques no revierten limpiamente: issue `needs-human` con lo intentado, observado e hipótesis (L-11) y hallazgo S-11.

## Cierre de issues

- **Relacionado con #206, pendiente de verificación.** No se escribe `Closes #206`: el descriptor solo puede cerrarlo cuando la evidencia final de QA verifique la eliminación de los dos builds y el tiempo del canal rápido en Linux y Windows (K-2/K-10).
- #203 y #204: referenciados, **no cerrados** aquí; sus correcciones deben verificarse antes de cerrarlos.
- #141: **no se cierra**; sin firma ni ancla de confianza (RFC-0030 borrador).

## Bloqueante humano antes del PR (S-12)

`qa` reportó credenciales en texto plano en `docs/audits/tui-transcripts/config-noninteractive-flag.json`. **No se abrió ni se copió ningún valor** (S-2/S-12). Requiere inspección y **rotación por un humano** antes de publicar cualquier PR, conforme a `docs/kaizen/runbooks/secret-leak.md` y al outbox `secret-en-audit-tui-transcript.md`. Además de este bloqueante, el cambio de default, la fuente de versión y la reafirmación de "sin firma" requieren la aprobación humana de ADR-N1/N2/N6 (DEC-001, K-1).

## Checklist

- [ ] Compila / pasa tests (lo verifica QA sobre el instalador modificado; no ejecutado aquí).
- [ ] `scripts/kaizen check` en verde (orquestador al integrar).
- [x] Documentación actualizada (los seis archivos de este cambio).
- [x] Línea base no empeora (documentación; sin cambios de código).
- [ ] Rotación S-12 completada por humano.
- [ ] Evidencia final de CA-1..CA-12 adjunta antes de escribir `Closes #206`.
