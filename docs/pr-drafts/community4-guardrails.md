# Descriptor de PR - community4/guardrails

## Resumen

ci(guardrails): weekly audit issue, stale PRs, derived-doc and file-size checks

## Fase afectada

Gobernanza / CI. Sin cambio de fase ni de hoja de ruta. Los maestros de
`docs/master/` no se tocan, asi que los hashes de `docs.yml` siguen validos.

## Tipo de cambio

- [ ] feat
- [ ] fix
- [ ] docs
- [ ] refactor
- [ ] test
- [x] chore
- [x] ci
- [ ] build
- [ ] perf
- [ ] security

## Documentos relacionados

- RFC-0022, secciones 4.2, 4.3, 4.6 y 4.8; plan de implementacion, PR 2, 3 y 5.
  Es una RFC aceptada (2026-11-10), asi que autoriza este cambio de CI sin RFC
  nueva (regla 28).
- `CLAUDE.md`, reglas 22, 26, 28 y 29.
- `CONTRIBUTING.md` y `tools/README.md` se actualizan en esta misma PR.
- Issue #168 como contexto: el gate `--locked` que introdujo ya esta cerrado;
  esta PR no lo cierra ni lo reabre.

## Plan de prueba

1. `bash -n tools/*.sh` (sintaxis de todos los scripts).
2. `bash tools/test-guardrails.sh` (43 aserciones con `gh` falso y temporales).
3. `bash tools/check-file-size.sh` sobre el workspace real.
4. `bash tools/check-derived-docs.sh` sobre el workspace real.
5. `git diff --check` sobre el diff completo.
6. Verificacion de YAML de los tres workflows nuevos con un parser local.
7. Escaneo de emojis y de evidencia de herramientas IA en los archivos nuevos.

## Criterios de salida que avanza

- RFC-0022 4.2: job semanal de `cargo audit` y `cargo deny` con issue
  `security` unico e idempotente.
- RFC-0022 4.3: limite de 600 lineas por archivo con allowlist que solo
  puede reducirse (umbral de funcion de 80 lineas configurado en clippy.toml).
- RFC-0022 4.6: comprobacion de derivados regenerados, con diff como fallo.
- RFC-0022 4.8: cierre de pull requests inactivos durante 30 dias.

No se implementa la comprobacion de tablas de comandos de la seccion 4.6: una
comparacion literal contra la ayuda de clap es fragile y se dejaria romper con
cambios de texto. Requiere diseno propio antes de anadirla.

## Cierre de issues

Closes: none
Refs #168 (contexto de `--locked`; el issue ya esta cerrado y verificado)

## Checklist

- [x] El cambio pertenece a la fase y a la RFC correcta.
- [x] Respeta la documentacion y no altera los maestros.
- [x] No anade dependencias ni complejidad innecesaria.
- [x] Tiene tests automaticos y mensajes de falla accionables.
- [x] Documentacion y CHANGELOG actualizados en la misma PR.
