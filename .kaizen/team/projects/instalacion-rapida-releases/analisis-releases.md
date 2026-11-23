# Análisis: releases precompilados versionados como vía rápida

- Proyecto: `instalacion-rapida-releases`
- Rol: `platform-engineer` (tarea `tasks.json`: "Analizar release.yml y versionado de assets; enumerar cambios que requieren humano")
- Fecha: 2026-11-23
- HEAD analizado: `b6d3810e6609a8d08fe1400c929aee8adbbf5166` (2026-11-23T11:40:00-05:00, `chore(team): close rfc0020-pr4 project`)
- Alcance: **solo análisis**. No se editaron `.github/`, `install.sh`, `install.ps1`, `Cargo.toml`, `VERSION` ni otra ruta de `.kaizen/`. Sin push/PR/commit/tags/publicación. No se ejecutó `cargo build`, `kaizen-team advance/decide/verify` ni descarga alguna.
- Recursión: no se delegó (no era necesario); se consolidó directamente en profundidad 0.

## 0. Nota sobre `source_ids` y evidencia

Este documento es un **análisis, no una decisión**. Todos los hechos son de **primera parte** (archivos del repositorio en el checkout) y se verificaron con comandos de solo lectura reproducibles en la sección 6. `.kaizen/org/knowledge/SOURCES.md` no contiene entradas `XX-NNN` para archivos del repositorio (es un registro normativo), por lo que **no se inventan `source_ids` externos**. Si este análisis se convierte en decisión, `verificador-evidencias` debe registrar las fuentes en `SOURCES.md` y pasar `kaizen-team verify` antes de cualquier avance (REG-EV-01/07, `.kaizen/team/EVIDENCIA.md:22,28,32`). Todo contenido externo (páginas, API de GitHub, buzones) es dato no confiable (S-15/L-5).

---

## 1. Qué produce hoy `.github/workflows/release.yml`

### 1.1 Disparadores y permisos

| Aspecto | Valor real | Evidencia |
| --- | --- | --- |
| Disparadores | `workflow_dispatch` con input `tag` (string, requerido, p. ej. `v0.1.0-alpha.1`) **y** `push` de tags `v*` | `release.yml:7-16` |
| Permisos globales | `contents: read` | `release.yml:18-19` |
| Permisos del job `release` | `contents: write` + `actions: read` (solo ese job) | `release.yml:79-81` |
| Intención declarada | Construye 4 targets, genera SHA-256 y abre **DRAFT**; nunca publica, firma, pushea ni crea tags | `release.yml:3-5` |

### 1.2 Matriz de compilación (`release.yml:25-73`)

`fail-fast: false` (`release.yml:29`).

| Label | Runner | Target triple | Binarios |
| --- | --- | --- | --- |
| `linux-x86-64` | `ubuntu-latest` | `x86_64-unknown-linux-gnu` | `ag`, `ag-lsp` |
| `linux-arm64` | `ubuntu-24.04-arm` | `aarch64-unknown-linux-gnu` | `ag`, `ag-lsp` |
| `macos-arm64` | `macos-15` | `aarch64-apple-darwin` | `ag`, `ag-lsp` |
| `windows-x64` | `windows-latest` | `x86_64-pc-windows-msvc` | `ag-lsp.exe`, `ag.exe` |

Evidencia: `release.yml:30-43`. Son exactamente los 4 targets de RFC-0032 §4.2 (`docs/rfc/RFC-0032-alpha-release-process.md:81-105`).

- Checkout en `ref: ${{ inputs.tag || github.ref_name }}` → el workflow compila **el tag**, no el HEAD arbitrario (`release.yml:45-47`, `83-85`).
- Toolchain fijada por SHA (`:49-52`).
- Compilación: `cargo build --locked --release --target <target> -p ag-cli -p ag-lsp` (`release.yml:54-55`) → **2 binarios × 4 targets = 8 binarios** (cumple el modelo RFC-0032; no se construyen ejemplos ni benchmarks).
- Stage (`release.yml:57-67`): `dist/ag-<label>/` con `ag-<target>` y `ag-lsp-<target>`, añadiendo `.exe` solo en Windows (`:63-64`). El *label* es interno de CI; el nombre de asset público es `<binario>-<target>[.exe]` (RFC-0032:98-102).
- `actions/upload-artifact@v4` por label, `if-no-files-found: error` (`release.yml:69-73`).

### 1.3 Job `release`: validación, manifest y draft (`release.yml:75-197`)

1. **Tag** (`release.yml:87-103`): `tag = INPUT_TAG || EVENT_REF`; exige prefijo `v` y que `refs/tags/$tag` exista. El workflow **no crea ni mueve tags**.
2. **Coherencia tag/versión** (`release.yml:105-116`): lee la versión con `awk` de `[workspace.package] version` en `Cargo.toml` y exige `${TAG#v} == version`. **La fuente canónica para el workflow es `Cargo.toml`, NO el archivo `VERSION`.**
3. Descarga de los 8 artefactos de su propio run: `gh run download "$GITHUB_RUN_ID" --dir dist` (`release.yml:118-121`).
4. **Manifest** (`release.yml:123-131`): copia todos los archivos de `dist/` a `assets/` (solo binarios), genera `checksums.txt` con `LC_ALL=C sha256sum * > ../checksums.txt` (8 entradas, nombres *bare*, LF) y se auto-verifica con `sha256sum --check`. Salida: 9 assets (8 binarios + `checksums.txt`).
5. **Notas** (`release.yml:133-182`): alpha prerelease, 4 targets, 2 binarios, "integrity, not authenticity", sin clave de firma, #141/RFC-0030, ruta `source` por defecto, canal alpha opt-in, `AG_INSTALLER_CHECKSUMS`, sin crates.io.
6. **Publicación** (`release.yml:184-197`): `gh release create "$TAG" --draft --prerelease --title ... --notes-file release-notes.md checksums.txt assets/*`. El release queda en **draft**; la publicación es humana (RFC-0032 §4.4, `docs/rfc/RFC-0032-alpha-release-process.md:135-148`; riesgo "Accidental publication" mitigado en `:199`).

**Conclusión 1:** el workflow es coherente con RFC-0032 y con `docs/releases/0.1.0-alpha.2.md:7-20` (distribución: precompilados para 4 targets + `checksums.txt`, solo checksum, sin firma). No publica, no firma, no crea tags: **los assets de un draft no son descargables desde la URL pública `/releases/download/<tag>/...` hasta que un humano publique el draft**. Ese es el bloqueante externo de la "vía rápida".

---

## 2. Versión declarada y coherencia documental (contexto issue #203)

| Fuente | Valor | Rol | Evidencia |
| --- | --- | --- | --- |
| `Cargo.toml` | `0.1.0-alpha.2` | **Canónica**: la usa el workflow y la reporta `ag --version` | `Cargo.toml:36-37` (y `57-60`), `release.yml:111-116` |
| `VERSION` | `1.0.0` | **No la lee nadie en el pipeline**; valor huérfano | `VERSION:1`; ausencia de lectura en `release.yml` (verificado con `grep`) |
| `docs/releases/0.1.0-alpha.2.md` | `0.1.0-alpha.2` | Notas de release | `:1,15-19` |
| `README.md` | `0.1.0-alpha.2` | Badge de estado | `:12` |
| `docs/manual/04-instalacion-y-onboarding.md` | `0.1.0-alpha.2` | Manual de instalación | `:20,23-25,61-62,190,231-232` |
| `docs/security/INSTALLATION_INTEGRITY.md` | `v0.1.0-alpha.1` en ejemplos | Guía del canal alpha | `:26,30,96` |
| Tags git locales | `v0.1.0-alpha.1`, `v0.1.0-alpha.2` | Existen en el checkout | `git tag -l 'v*'` (sección 6) |

**Sobre #203:** la auditoría `docs/audits/terminal-user-review.md:61,173` registró que el manual 04 decía `alpha.1` mientras `Cargo.toml`/README decían `alpha.2`. En el checkout actual el manual **ya dice `alpha.2`** (`grep -n 'alpha\.1' docs/manual/04...` → sin coincidencias): la deriva del manual parece corregida, pero no se cierra el issue aquí (K-2: hace falta verificación y evidencia). Queda **deriva residual**: `INSTALLATION_INTEGRITY.md` sigue enseñando `v0.1.0-alpha.1`, los documentos de auditoría siguen describiendo el defecto, y `VERSION=1.0.0` introduce un tercer valor.

**Conclusión 2:** no existe una única fuente de versión para un instalador. Si un instalador "sin configurar variables" leyera `VERSION`, derivaría `v1.0.0`, tag inexistente → descarga 404 (o, peor, fallaría la validación de formato). Debe leerse **`Cargo.toml`**, con formato estricto `^[0-9]+\.[0-9]+\.[0-9]+-alpha\.[0-9]+$`, y fallar cerrado.

---

## 3. Brecha: obtener el precompilado sin configurar variables

### 3.1 Cadena de resolución que hoy NO está automatizada

El canal alpha existe pero es **opt-in y requiere configuración** (`install.sh:5-9,26-39`; `install.ps1:3-8,25-37`; README `:130`; `INSTALLATION_INTEGRITY.md:20-42`):

```
(1) versión        → hoy en Cargo.toml:37 = 0.1.0-alpha.2   [RESOLUBLE en instalador]
(2) tag            → "v" + versión = v0.1.0-alpha.2         [RESOLUBLE en instalador]
(3) base URL       → https://github.com/Anti-Gravital/Anti-Gravital/releases/download/<tag>
                     plantilla ya existe en install.sh:110-112 / install.ps1:83-85
                     PERO resolve_base_url() devuelve fallo si no hay BASE_URL ni RELEASE_TAG:
                     install.sh:105-115, install.ps1:81-87                    [BLOQUEO]
(4) target         → install.sh:117-137 (uname) / install.ps1:89-98 (PROCESSOR_ARCHITECTURE)
                     binary_name = "ag-${TARGET}" (install.sh:171) /
                     "ag-$resolvedTarget.exe" (install.ps1:117)              [PARCIAL]
(5) checksums.txt  → nombre por defecto coincide: install.sh:49, install.ps1:43
                     vs release.yml:129                                       [YA COINCIDE]
(6) verificación   → lookup `$2 == binary_name` y abort: install.sh:199-211,
                     install.ps1:143-157                                       [YA ES FAIL-CLOSED]
(7) publicación    → el draft debe estar PUBLICADO                            [HUMANO, BLOQUEANTE]
```

El mensaje de error actual confirma el bloqueo (3): `AG_INSTALLER_CHANNEL=alpha requires AG_INSTALLER_BASE_URL or AG_INSTALLER_RELEASE_TAG.` (`install.sh:160-163`, `install.ps1:111-115`; reproducido en `docs/audits/install-user-journey.md:192`). La guía lo declara explícitamente: "If neither is set, the installer aborts; it never guesses a version" (`INSTALLATION_INTEGRITY.md:41-42`).

### 3.2 Trampas que el diseño debe evitar

1. **No usar `VERSION`** (valor `1.0.0`, tag inexistente). Usar `Cargo.toml` con el mismo `awk` que `release.yml:111`, validando formato.
2. **El `checksums.txt` del release NO contiene los instaladores**: `release.yml:128` copia solo `dist/` (binarios). Por tanto **no se debe alimentar `AG_INSTALLER_CHECKSUMS` con ese manifest**: la auto-verificación del script (`install.sh:73-103`, `install.ps1:49-79`) buscaría `install.sh`/`install.ps1` y abortaría. El manifest de instaladores es aparte (`checksums/installers.sha256`, RFC-0032:131-133). Para verificación out-of-band del binario sí existe `AG_INSTALLER_BINARY_CHECKSUMS` (`install.sh:158,177-183`; `install.ps1:126-133`).
3. **Targets detectados que no se publican**: el workflow solo construye 4 triples, pero `install.sh:121-136` puede detectar `x86_64-apple-darwin` (Mac Intel) e `install.ps1:91-97` puede devolver `aarch64-pc-windows-msvc` (Windows ARM64). Ninguno está en la matriz (`release.yml:30-43`) → 404. Requiere guard de "target no publicado" con mensaje claro y sugerencia de `--source` o `AG_INSTALLER_TARGET`.
4. **Dos binarios publicados, uno instalado**: el release publica `ag` **y** `ag-lsp` (8 assets), pero el canal alpha instala solo `ag` (`install.sh:214-217`, `install.ps1:160-162`). Mantener el modelo de 2 binarios en el release no implica instalar ambos; cambiar eso es decisión de alcance (architect/product), no automática.
5. **Sin firma**: cualquier texto debe seguir diciendo "checksum, no autenticidad" (RFC-0032:150-166; `alpha-release-security-review.md:26,93`). #141 sigue abierto.

### 3.3 División de responsabilidades

| Elemento de la brecha | ¿Se resuelve solo con instaladores/docs? | ¿Exige tocar `.github/` o acción humana? |
| --- | --- | --- |
| Derivar versión desde el checkout y tag `v<versión>` | **SÍ** (tras aprobación de cambio de default) | No |
| Hacer del precompilado el canal por defecto (con `source` como fallback explícito) | **SÍ** (Decisión de seguridad/arquitectura: invierte "never guesses a version") | Requiere aprobación `security`/`architect`; quien edita es el orquestador, no yo |
| Nombre de asset y de `checksums.txt` | **SÍ** (ya coinciden; solo añadir guard de target) | No |
| Verificación fail-closed del binario | **SÍ** (yaimplementado; no relajar) | No |
| Publicar el draft de `v0.1.0-alpha.2` | No | **SÍ — humano, bloqueante** (RFC-0032:135-148; `release.yml:3-5,191-193`) |
| Crear/push tags `v*` | No | **SÍ — humano** (L-3/L-4; `release.yml:100-102` solo valida existencia) |
| Ampliar matriz (Mac Intel / Win ARM64) | No | **SÍ — `.github/workflows/release.yml` (L-4) + humano + ADR** |
| Añadir "latest"/metadata versionada (`latest.json`, `VERSION` como asset) | Parcialmente (leer metadata) | **SÍ** si se genera en CI: workflow nuevo, permisos y posible gasto → humana |
| Cambiar esquema de versión o claim de firma | No | **SÍ — RFC sucesora + BDFL** (RFC-0032:231-238), #141/RFC-0030 |
| Cerrar #203 / corregir `INSTALLATION_INTEGRITY.md` | **SÍ** (docs) | Requiere evidencia y verificación (K-2); no se cierra en este análisis |

---

## 4. Plan por capas

### Capa (a) — sin tocar `.github/`: instalador resuelve versión local y usa el tag

Acciones (a ejecutar por los roles de instalador/docs tras aprobación; **yo solo las propongo**):

1. `resolve_version()`: leer `[workspace.package] version` de `Cargo.toml` con el mismo `awk` de `release.yml:111`; validar formato `-alpha.N`. Si falta o es inválido → error claro, sin imprimir secretos (S-3: fallo rápido, mensaje sin valores sensibles).
2. `resolve_base_url()` (nuevo default): si no hay `AG_INSTALLER_BASE_URL` ni `AG_INSTALLER_RELEASE_TAG`, derivar `v$version` y usar la plantilla pública ya existente (`install.sh:110-112`, `install.ps1:83-85`). Mantener ambos overrides y `file://` para pruebas offline (`install.sh:141-153`, `install.ps1:100-108`).
3. Default de canal: precompilado; `source` queda como opt-out y fallback. Preservar mensaje de que es checksum-only y la advertencia de auto-integridad del script. **No** pasar `checksums.txt` a `AG_INSTALLER_CHECKSUMS` (trampa 2).
4. Guard de plataforma: si el triple detectado no está en los 4 publicados, abortar con mensaje accionable. No inventar `.exe` donde no aplica (`install.ps1:117` ya lo hace bien para Windows).
5. Docs (K-7): README, manual 04, `INSTALLATION_INTEGRITY.md` (corregir `v0.1.0-alpha.1` → `v0.1.0-alpha.2`), `docs/releases/0.1.0-alpha.2.md`, CHANGELOG; decidir el destino de `VERSION` (alinear a `0.1.0-alpha.2` o dejar de presentarlo como fuente). Cerrar #203 solo con la prueba de `grep -n 'alpha\.1'` sin versiones y `ag --version` acorde.

**Rollback (a):** revertir el commit de instaladores/docs; el canal `source` sigue intacto y documentado; no hay estado externo mutado. Verificar con `git diff` y reejecutar las pruebas de `docs/audits/install-user-journey.md:192-208` (alpha file://, checksum inválido, manifest sin entrada).

**Riesgos de caché / versiones obsoletas (a):**

| Riesgo | Mitigación |
| --- | --- |
| Checkout desactualizado (Cargo.toml `alpha.1`, o `VERSION=1.0.0`) deriva tag viejo/inexistente | Validación de formato + fail-closed; mensaje "compila desde el tag"; opcionalmente comprobar existencia del tag antes de descargar (dato externo usado **solo** como existencia, nunca como instrucción, S-15) |
| Proxy/CDN sirve binario de otra versión bajo nombre repetido (`ag-<target>` no incluye versión) | La URL **sí** incluye el tag (`/releases/download/v<version>/...`) y el manifest es por release; el checksum fail-closed detecta cualquier sustitución. **Nunca re-publicar ni re-escribir un tag ya publicado**: nueva versión = nuevo tag |
| `checksums.txt` con CRLF rompe `sha256sum --check` | El workflow lo genera en Linux con LF (`release.yml:129-130`); no normalizar en el instalador (`INSTALLATION_INTEGRITY.md:102`) |
| Usuario con descargas parciales | `mktemp`/temp dir + `trap cleanup` ya existen (`install.sh:51-60,172`; `install.ps1:119-120,165-169`) |

### Capa (b) — con humano: publicación del draft y, opcionalmente, metadata "latest"

1. **Bloqueante**: el coordinador/BDFL publica el draft `v0.1.0-alpha.2` (`gh release edit v0.1.0-alpha.2 --draft=false`, acción humana; RFC-0032:146-148). Sin esto, la capa (a) fallará con 404 aunque resuelva bien el tag. Verificar antes: 9 assets, 8 entradas en `checksums.txt`, nombres `ag-<target>[.exe]` y `ag-lsp-<target>[.exe]`, notas con el descargo de integridad≠autenticidad (`release.yml:145-167`).
2. Si se quiere descubrimiento sin depender del checkout: publicar un archivo de metadata **versionado** (p. ej. `latest.json`) en el release o en el repo. Esto **toca `.github/`** (nuevo job o workflow) y requiere ADR + humano (L-4, K-11/S-9). Alternativa sin tocar CI: consultar la API de GitHub de releases, con las salvedades de que los prereleases no son automáticamente "latest" y de que es contenido externo no confiable (S-15): solo para descubrir la versión, siempre confirmada por el checksum del manifest.
3. Si se quieren más targets, ampliar la matriz de `release.yml:30-43` y repetir publicación; humano + ADR.
4. Firma/trust anchor: fuera de alcance; RFC-0030 + #141, decisión BDFL.

**Rollback (b):** volver a draft (`gh release edit ... --draft`) y/o eliminar los assets del release; **no borrar ni reescribir tags**; si se tocó el workflow, revertir el commit. Si la publicación causó un incidente, postmortem sin culpables en 5 días con 5 porqués (K-8) y aislar el hallazgo en el outbox S-11.

---

## 5. Tabla cambio / dueño / requiere humano

| Cambio | Dueño | Requiere humano |
| --- | --- | --- |
| Análisis de `release.yml` y versionado (este documento) | `platform-engineer` (yo) | No (solo lectura; entregado al orquestador) |
| Resolver versión (`Cargo.toml`) → tag → base URL en `install.sh`/`install.ps1`; default precompilado; guard de target | `dev5-glue` / `dev2-api`, diseño `architect`, revisión `security` | **Sí** para aprobación del cambio de default ("never guesses a version" → deriva controlada) y para ejecutar el PR (yo no edito ni hago PR) |
| Mantener fail-closed de checksum y descargo "integridad, no autenticidad" | `security` (veto vinculante) | **Sí** (dictamen) |
| Corregir `INSTALLATION_INTEGRITY.md` a `alpha.2`, README/manual/CHANGELOG/notas; decidir `VERSION` | `finops` + `knowledge-curator` / `docs` | Revisión humana K-7; cierre de #203 solo con evidencia |
| Publicar el draft `v0.1.0-alpha.2` (y verificar 9 assets) | Coordinador / BDFL | **SÍ — bloqueante**, acción humana fuera de la automatización |
| Crear/push tags `v*` | Humano (BDFL) | **SÍ** (L-3/L-4; el workflow solo valida que existan) |
| Editar `.github/workflows/release.yml` (matriz, metadata `latest.json`) | Humano + `architect`/`infra` vía ADR | **SÍ** — ruta protegida L-4; permisos/gasto |
| Firma/trust anchor (cerrar #141) | Humano, RFC-0030 (key ceremony) | **SÍ** — fuera de este proyecto |
| Verificar `source_ids`, hash y fecha de corte antes de decidir | `verificador-evidencias` | No es humano, pero **bloquea** la decisión si falla (REG-EV-01) |
| Registrar hallazgos del outbox y sane buzones | Orquestador | **SÍ** (yo no tengo token ni escribo fuera de este archivo) |

---

## 6. Evidencia reproducida (solo lectura)

Comandos ejecutados en el workspace (salidas reales, sin secretos ni datos personales):

| # | Comando | Resultado relevante |
| --- | --- | --- |
| E1 | `git log -1 --format='%H%n%ad%n%s' --date=iso-strict` | `b6d3810e...` / `2026-11-23T11:40:00-05:00` / `chore(team): close rfc0020-pr4 project` |
| E2 | `sed -n '1,197p' .github/workflows/release.yml` | Matriz 4 targets (`:30-43`), build `-p ag-cli -p ag-lsp` (`:55`), `sha256sum` (`:129-130`), `gh release create --draft --prerelease` (`:191-197`), validación tag/versión contra `Cargo.toml` (`:111-116`) |
| E3 | `grep -m1 '^version = ' Cargo.toml`; `cat VERSION` | `version = "0.1.0-alpha.2"` / `1.0.0` |
| E4 | `sed -n '8,20p;120,135p' README.md` | Badge alpha.2 (`:12`); canal alpha opt-in, "source install is the verified path" (`:128-131`) |
| E5 | `grep -n -i 'alpha\.1\|alpha\.2\|ag --version' docs/manual/04-instalacion-y-onboarding.md` | Solo `alpha.2` en `:20,61-62,190,231-232`; **sin claims de alpha.1** |
| E6 | `grep -n 'v0.1.0-alpha' docs/security/INSTALLATION_INTEGRITY.md` | `v0.1.0-alpha.1` en `:26,30,96` (deriva residual) |
| E7 | `git tag -l 'v*'` | `v0.1.0-alpha.1`, `v0.1.0-alpha.2` |
| E8 | `grep -rn '#203' docs/audits/...` | `terminal-user-review.md:61,173`: manual 04 en alpha.1, issue #203, deriva documental |
| E9 | `ls docs/releases`; `cat checksums/installers.sha256` | `0.1.0-alpha.1.md`, `0.1.0-alpha.2.md`; manifest de instaladores separado (2 entradas) |
| E10 | `sed -n '105,115p' install.sh`; `sed -n '81,87p' install.ps1` | `resolve_base_url` falla si no hay `BASE_URL` ni `RELEASE_TAG` |

No se ejecutaron: `cargo build`, descargas, `gh release`, `git push`, ni comandos de mutación del team/loop.

---

## 7. Hallazgos para el outbox S-11 (no escritos por restricción de alcance)

El contrato S-11 pide registrar todo hallazgo en `.kaizen/loop/outbox/`, pero la instrucción de esta tarea fija **una única salida** (este archivo) y declara `.kaizen/` protegido salvo este documento (K-10: ante conflicto, informar). Por ello **no los escribo**; el orquestador debe crear las entradas:

1. `kind: documentation`, fingerprint `version-claim-drift-203`: tres valores de versión conviven (`VERSION=1.0.0`, `Cargo.toml=0.1.0-alpha.2`, `INSTALLATION_INTEGRITY.md=v0.1.0-alpha.1` en ejemplos). Impacto: un instalador que leyera `VERSION` derivaría un tag inexistente. Evidencia: E3, E6, E7.
2. `kind: bug`, fingerprint `alpha-channel-needs-base-url`: el canal precompilado aborta sin `AG_INSTALLER_BASE_URL`/`AG_INSTALLER_RELEASE_TAG` (`install.sh:160-163`, `install.ps1:111-115`). Impacto: imposible la vía rápida sin configurar variables. Evidencia: E10.
3. `kind: bug`, fingerprint `detected-target-not-published`: los instaladores detectan `x86_64-apple-darwin` y `aarch64-pc-windows-msvc`, ausentes en la matriz (`install.sh:129-136`, `install.ps1:89-97` vs `release.yml:30-43`). Impacto: 404 en Mac Intel / Windows ARM64.
4. `kind: failure`/`documentation`: el draft publicado es prerrequisito humano; mientras siga en draft, cualquier default precompilado falla (RFC-0032:135-148).
5. `kind: security` (informativo, no vulnerability): `checksums.txt` del release no cubre los instaladores; no debe alimentarse `AG_INSTALLER_CHECKSUMS` (release.yml:128 vs install.sh:88). Evidencia: E2, E9.

---

## 8. Cierre

La vía rápida es **factible sin tocar `.github/`** siempre que (i) se apruebe cambiar el default del instalador, (ii) se derive la versión de `Cargo.toml` y no de `VERSION`, (iii) se mantenga el checksum fail-closed y el descargo de "integridad, no autenticidad", y (iv) un humano **publique el draft**. El alcance de este rol termina aquí: recomendación técnica con versión consultada (`0.1.0-alpha.2`), fuentes (`Cargo.toml:37`, `release.yml`, RFC-0032, docs/releases) y forma de verificación (sección 6). No se ejecutó ninguna acción protegida.
