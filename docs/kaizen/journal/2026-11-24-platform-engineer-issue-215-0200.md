# 2026-11-24 02:00 UTC — platform-engineer / issue #215

## Tarea
Corregir el bug determinista de CI #215: `cargo deny check --all-features` es un
flag invalido en cargo-deny 0.20.2 y hace que el issue semanal de seguridad
nunca se cierre. Trabajo en el worktree `denyflag`, rama
`community30/denyflag`, con autorizacion humana para editar `.github/`.

## Decision
Fix minimal: quitar `--all-features` del workflow (linea 44) y de la etiqueta
del cuerpo del issue en `tools/security-report.sh` (linea 67). No se toca
`deny.toml` (`all-features = false` se mantiene: es el mecanismo soportado).
Se documenta la alternativa (`all-features = true` o el flag global
`cargo deny --all-features check`) sin aplicarla, para no cambiar un
comportamiento que el proyecto no pidio. Se alinean ademas las dos referencias
de `CONTRIBUTING.md` (EN/ES) que documentaban el comando roto (K-1/K-7).
Sin supresiones: se preserva la integridad de la auditoria (S-10).

## Evidencia (comandos y resultados reales)
- `cargo deny --version` -> `cargo-deny 0.20.2`; `cargo --version` -> 1.99.1.
- `cargo deny check --all-features` -> exit 2, `error: unexpected argument '--all-features' found`.
- `cargo deny check` -> exit 0, `advisories ok, bans ok, licenses ok, sources ok`.
- `cargo deny --offline check` -> exit 0 (mismo resultado sin red).
- `cargo deny --all-features check` -> exit 0 (flag en posicion global; opcion no aplicada).
- `python3 -c "import yaml; ..."` -> YAML OK (exit 0).
- `bash -n tools/security-report.sh` -> exit 0, sin salida.
- `bash tools/test-guardrails.sh` -> `OK: all guardrail script tests passed.` (exit 0).
- `git diff --check` -> exit 0, sin salida.
- Content scan `git grep -- 'cargo deny check --all-features' -- .github tools scripts CONTRIBUTING.md deny.toml` -> sin coincidencias.
- `bash scripts/ag-check` -> `Summary: PASS=8 FAIL=0 SKIP=0` / `ag-check: all selected checks passed.` (exit 0).
- `scripts/kaizen check` -> `ok: kaizen check: todo en verde` (exit 0).

## Resultado
Fix + evidencia preparados en la rama `community30/denyflag`: workflow, script,
CONTRIBUTING.md, descriptor (ambas rutas), CHANGELOG, outbox S-11 y este
journal. Sin push/PR/merge. El cierre efectivo de #215 depende del run real de
GitHub Actions tras la fusion (no ejecutable localmente).

## Aprendizaje
Un flag de CLI invalido puede disfrazarse de fallo de seguridad recurrente y
mantener abierto un issue para siempre. cargo-deny 0.20.2 acepta
`--all-features` solo como flag global (antes del subcomando); no se deben
asumir las opciones de `cargo test`/`clippy` en `cargo deny`. Verificar siempre
"herramienta mal invocada" frente a "advisory encontrado" antes de escalar.
