# community29-i214 — verificacion de fallo CI `kaizen-security`

## Resumen
Issue #214 (auto-detectado; etiquetas `ci-failure`, `auto-detected`) cita el run 45199162295 de `kaizen-security` en el commit 4d8e377 con resultado `failure`. Verificado con comandos reales: el fallo es real y recurrente, NO transitorio y NO resuelto en main@97b9402.

Causa raiz: `.github/workflows/kaizen-security.yml:15` define `GITLEAKS_VERSION: ""` (vacio). Los steps de instalacion de gitleaks (`:33` y `:65`) salen con exit 1 al ejecutar `[ -n "$GITLEAKS_VERSION" ] || { echo "Define GITLEAKS_VERSION en el workflow"; exit 1; }`. El run 45199162295 (evento `schedule`, job `full-history` 137777660985) murio en "Instalar gitleaks con checksum verificado"; el step "Escaneo de arbol e historial completos" nunca corrio. El defecto existe desde baf18fd (2026-11-22) y no se ha corregido.

Los runs verdes recientes (45278983700 y anteriores) son todos de evento `push`: en push se omiten `full-history` (solo `schedule`/`workflow_dispatch`, `:53`) y `secrets-and-silence` (solo `pull_request`, `:19`); solo corre `audit`, que pasa. Por tanto NO son evidencia de que el escaneo de secretos este sano.

Hallazgo reportado en `.kaizen/loop/outbox/kaizen-security-gitleaks-version-empty.md` (S-11). La correccion toca `.github/` (ruta protegida, L-4): corresponde a un humano.

## Plan de prueba
Comandos ejecutados en `community29/i214@97b9402` (worktree de QA), salidas reales:

1. Lectura del issue:
   - `gh issue view 214 --json number,title,body,state,labels,url` -> `OPEN`, titulo `[ci-failure] kaizen-security`, cuerpo con run 45199162295 y commit 4d8e377.
2. Estado del workflow:
   - `gh run list --workflow kaizen-security --limit 60 --json event,conclusion,databaseId,createdAt` -> 1 unico run `schedule` reciente (45199162295) con `failure`; el resto son `push` con `success`. Ultimo run: 45278983700, push, success, 2026-11-23T22:22:04Z.
   - `gh run view 45199162295` -> `X full-history in 5s (ID 137777660985)`; step "Instalar gitleaks con checksum verificado" en X; "Escaneo de arbol e historial completos" sin ejecutar.
   - `gh api repos/Anti-Gravital/Anti-Gravital/actions/jobs/137777660985/logs` -> `GITLEAKS_VERSION:` (vacio), `Define GITLEAKS_VERSION en el workflow`, `##[error]Process completed with exit code 1.`
3. Codigo y procedencia del defecto:
   - `grep -n GITLEAKS_VERSION .github/workflows/kaizen-security.yml` -> linea 15 `GITLEAKS_VERSION: ""`; lineas 33 y 65 con la guarda que falla.
   - `git blame -L 13,15 -- .github/workflows/kaizen-security.yml` -> `baf18fd8 (2026-11-22) GITLEAKS_VERSION: ""`.
   - `grep -rn GITLEAKS .kaizen/config.env .kaizen/security/security.env` -> solo `KS_REQUIRE_GITLEAKS=0` (config local; no repara el workflow).
   - Referencia de version (2026-11-23): `gh api repos/gitleaks/gitleaks/releases/latest --jq .tag_name` -> `v8.30.1`.
4. Verificacion local:
   - `scripts/kaizen-security check` -> `no leaks found` / `kaizen-security: ok` (RC=0). No replica CI: gitleaks esta instalado aqui y `KS_REQUIRE_GITLEAKS=0`.
   - `git diff --check` -> RC=0, sin salida.
   - `bash scripts/ag-check` (primera pasada, antes del descriptor) -> PASS fmt/clippy/test/masters/prohibited; FAIL descriptors (faltaba este archivo); FAIL audit (no pudo descargar RustSec advisory-db: error de red) y FAIL deny (lock de advisory-db). Summary PASS=5 FAIL=3.
   - Reintento: `bash scripts/ag-check --only audit,deny` -> PASS audit; `bash scripts/ag-check --only deny` -> PASS deny (RC=0). Ambos fallos iniciales fueron ambientales/transitorios (red y lock), no causados por este cambio; se registran en el outbox `kaizen-ag-check-audit-deny-transitorio.md` (S-11) y ya estan tracking en el issue #215.
   - Crear el descriptor y convertir a ASCII los simbolos del log de CI copiado en el outbox: pasada intermedia de `bash scripts/ag-check` -> FAIL `prohibited` por `checkmark`/`punto medio` en el outbox; corregido. Pasada final: PASS: descriptors, fmt, clippy, test, audit, deny, masters, prohibited; Summary PASS=8 FAIL=0 SKIP=0; `ag-check: all selected checks passed` (RC=0).
   - `scripts/kaizen check` (full): todas las puertas en verde (RC=0). `scripts/kaizen-security check`: `no leaks found` / `kaizen-security: ok` (RC=0). `git diff --cached --check`: RC=0. Escaneo de contenido de los tres archivos nuevos: sin tokens de credencial, sin supresiones sin referencia.

Nota de frontmatter de evidencia: ningun comando imprimio secretos ni datos personales; la salida del log de CI se cito solo en las lineas de error y de variable vacia.

## Cierre de issues
Closes: none

#214 NO se cierra: el fallo es real y persistente. Cerrarlo como "transitorio" seria un cierre falso (K-2/K-10).

Condicion exacta de cierre para el coordinador (requiere humano, L-4):
1. Fijar `GITLEAKS_VERSION` en `.github/workflows/kaizen-security.yml:15` a una version real con checksum verificado (referencia 2026-11-23: `v8.30.1`, la decide el responsable).
2. Ejecutar `workflow_dispatch` de `kaizen-security` y verificar que el job `full-history` termina en verde con el step "Escaneo de arbol e historial completos" ejecutado (no omitido).
3. Verificar un run de `pull_request` con el job `secrets-and-silence` en verde.
4. Solo entonces cerrar #214 citando los IDs de esos runs.

Comentario propuesto para el coordinador (no publicado por este agente):
"Verificado: el fallo no es transitorio. Run 45199162295 (schedule, job full-history) murio en la instalacion de gitleaks porque GITLEAKS_VERSION esta vacio en .github/workflows/kaizen-security.yml:15; el escaneo nunca corrio. Los runs verdes son push y omiten ese job. Requiere fix humano en ruta protegida; #214 sigue abierto. Condicion de cierre: fijar version + dispatch y PR en verde."
