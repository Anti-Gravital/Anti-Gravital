# Iteracion 2026-11-24 02:08 UTC — community30/metricsrace

## Tarea

Endurecer el job `metrics` de `.github/workflows/kaizen-gates.yml` contra la
carrera de publicacion detectada como seguimiento de #208 (push simple sin
rebase ni reintento, que producia `failed to push some refs` bajo pushes
concurrentes a main).

## Decision

Aplicar defensa en profundidad en un unico cambio coherente:
`concurrency` de workflow (`group: kaizen-metrics`, `cancel-in-progress: false`)
para serializar ejecuciones y encolar sin cancelar, mas un bucle de 5 intentos con
`git pull --rebase origin main` para cubrir pushes externos que avancen origin
entre checkout y push. Fallo en abierto (S-10): rebase fallido o reintentos
agotados => exit 1. Sin supresiones. Compuertas framework/ratchet/content-policy
y `needs` sin cambios.

## Evidencia (comandos y resultados reales)

- YAML parse con PyYAML 6.0.3: OK, `concurrency` presente, `needs` intacto.
- Extraccion del bloque `run` + `bash -n`: OK (RC=0).
- Simulacion con `git` falso: rechazo transitorio => exit 0 en intento 3; rechazo
  persistente => exit 1 tras 5 intentos; rebase fallido => exit 1 inmediato.
- `git diff --cached --check`: sin salida, RC=0.
- `bash scripts/ag-check`: PASS=8 FAIL=0 SKIP=0, RC=0.
- `bash scripts/kaizen check`: todo en verde, RC=0.
- `bash scripts/kaizen-security check`: no leaks found, RC=0.
- Escaneo de contenido: sin IA/brand/emoji/supresiones reales/secretos.
- Detalle completo en `docs/audits/community30-metricsrace-evidence.md`.

Limitacion: GitHub Actions no se ejecuta localmente; la concurrencia solo se
valida por parseo YAML y su efecto real se observara en GitHub. Sin push/PR/merge.

## Resultado

Cambio listo para commit en `community30/metricsrace`. Descriptor en
`docs/kaizen/pr-drafts/community30-metricsrace.md` (`Refs #208`, `Closes: none`)
y copia en `docs/pr-drafts/` para el tooling; CHANGELOG actualizado. Ningun
issue se cierra (208 ya esta cerrado).

## Aprendizaje

Una correccion pura de `pull --rebase` puede generar conflictos al anadir lineas
a un JSONL por dos jobs a la vez: serializar primero con `concurrency` elimina
la causa, y el reintento queda como red de seguridad para push externos. Anadir
siempre la salida de verificacion al commit para que el revisor no tenga que
reconstruir la evidencia.
