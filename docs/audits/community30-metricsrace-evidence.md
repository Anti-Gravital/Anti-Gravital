# Evidencia de endurecimiento del job `metrics` (community30/metricsrace)

Fecha: 2026-11-24 02:08 UTC
Rama: `community30/metricsrace` (worktree aislado)
Archivo tocado: `.github/workflows/kaizen-gates.yml`
Seguimiento de: #208 (ya cerrado; ver
`docs/audits/issue-208-ci-failure-kaizen-gates-verification.md`).

## Cambio aplicado

1. `concurrency` a nivel de workflow: `group: kaizen-metrics`,
   `cancel-in-progress: false`. Serializa las ejecuciones; encola en vez de
   cancelar, de modo que no se pierde ninguna linea de metricas.
2. El `git push` simple se sustituye por un bucle de 5 intentos: si el push es
   rechazado, hace `git pull --rebase origin main` y reintenta. Un rebase fallido
   o el agotamiento de reintentos devuelve exit 1 (fail-closed, S-10). No se
   anadio `|| true`, `skip`, `continue-on-error` ni supresion alguna.
3. Las compuertas `framework`, `ratchet`, `content-policy` y el `needs` del job
   `metrics` quedan intactos.

## Verificaciones (salida real)

### YAML parse

```
$ python3 -c "import yaml; d=yaml.safe_load(open('.github/workflows/kaizen-gates.yml')); print('YAML OK; top keys:', list(d.keys())); print('concurrency:', d['concurrency']); print('metrics needs:', d['jobs']['metrics']['needs'])"
YAML OK; top keys: ['name', True, 'permissions', 'concurrency', 'jobs']
concurrency: {'group': 'kaizen-metrics', 'cancel-in-progress': False}
metrics needs: ['framework', 'content-policy', 'ratchet']
RC=0
```

Nota: PyYAML 6.0.3 interpreta `on:` como booleano `True` (YAML 1.1); es una
particularidad del parser, no un error del archivo. GitHub Actions usa YAML 1.2.

### Sintaxis del script inline

El bloque `run` del step `Registrar metricas` se extrajo via PyYAML a
`/tmp/opencode/metrics_inline.sh` y se verifico:

```
$ bash -n /tmp/opencode/metrics_inline.sh
bash -n: OK (RC=0)
```

### Comportamiento del bucle de reintento (simulacion con `git` falso)

Tres escenarios en un sandbox aislado (`/tmp/opencode/metricsrace-sim*`), con un
`git` de pruebas que controla los resultados de `push`/`pull --rebase`:

```
[3a] rechazo transitorio (push falla 2 veces, luego exitosa):
  ... fake-git: push attempt 1 -> error: failed to push some refs
  ... fake-git: pull --rebase origin main
  ... fake-git: push attempt 2 -> error: failed to push some refs
  ... fake-git: pull --rebase origin main
  ... fake-git: push attempt 3 -> OK
  EXIT=0

[3b] rechazo persistente:
  ... push attempt 1..5 -> todos rechazados, rebase entre cada uno
  metrics: no se pudo publicar tras 5 intentos
  EXIT=1   (push attempts: 5)

[3c] rebase fallido:
  ... fake-git: push attempt 1 -> rechazado
  ... error: conflict (rebase)
  metrics: rebase fallo; error real, no se silencia
  EXIT=1
```

### git diff --check

```
$ git add -A && git diff --cached --check
(sin salida)   RC=0
```

### scripts/ag-check

```
$ bash scripts/ag-check
PASS: descriptors
PASS: fmt
PASS: clippy
PASS: test
PASS: audit
PASS: deny
PASS: masters
PASS: prohibited
Summary: PASS=8 FAIL=0 SKIP=0
ag-check: all selected checks passed.
AG_CHECK_RC=0
```

Re-ejecutado sobre el arbol final (tras anadir este archivo y la entrada de
diario): identico resultado, `PASS=8 FAIL=0 SKIP=0`, `AG_CHECK_RC=0`.

### scripts/kaizen check

```
$ bash scripts/kaizen check
ok: descriptores validados
ok: politicas de contenido
ok: formato
ok: lint
ok: tests
ok: deuda 0 <= 0
ok: kaizen check: todo en verde
KAIZEN_CHECK_RC=0
```

### scripts/kaizen-security check

```
$ bash scripts/kaizen-security check
inf 0 commits scanned.
inf scanned ~5979 bytes (5.98 KB) in 38.3ms
inf no leaks found
kaizen-security: ok
SEC_CHECK_RC=0
```

### Escaneo de contenido

Sobre `.github/workflows/kaizen-gates.yml`, `CHANGELOG.md` y ambos descriptores:

- Atribucion de IA / URLs de chat de IA: ninguna.
- Marcas retiradas (ADR-0011): ninguna.
- Emojis: ninguno.
- `|| true` / `noqa` / `nosec` / `--no-verify` / `continue-on-error`: ninguna
  supresion real. Las coincidencias de `grep` son menciones descriptivas en
  prosa ("no se anadio `|| true`"), no directivas activas.
- Patrones de secretos: sin valores. Las coincidencias son nombres de variables
  de entorno y dependencias ya documentadas en `CHANGELOG.md`
  (`AG_MODEL_API_KEY`, `jsonwebtoken`), no credenciales.

## Limitacion de honestidad

GitHub Actions no se puede ejecutar localmente: la simulacion valida la logica
del bucle con un `git` de pruebas, no el entorno real de runners. La concurrencia
declarativa se verifico por parseo YAML; su efecto en la programacion de runs
solo se observara en GitHub. No se hicieron push, PR ni merge.
