# ADR-0010: versionado, catálogo de capacidades y propagación de instalaciones

Estado: aceptado
Fecha: 2026-11-22
Aprobado por: Angel Nereira

## Contexto
Las iteraciones del equipo producen lecciones, evaluaciones, definiciones mejoradas y nuevas capacidades (REA con 263 herramientas). Sin una versión y un catálogo agregado, cada usuario tendría que reconstruir manualmente qué existe y cómo instalarlo en cada proyecto.

## Decisión
1. `VERSION` semver como fuente única; `scripts/kaizen-release` con `status|notes|check|cut|install`.
2. Catálogo generado desde capacidades REA reales (`.kaizen/catalog/rea-capabilities.json`) más skills del framework: `.kaizen/catalog/catalog.json` y `docs/kaizen/org/CATALOGO-TECNOLOGIAS.md`.
3. `cut <version>` regenera catálogo, gobernanza, agentes y notas; valida eval/gov; commitea, taggea `vX.Y.Z`, pushea y crea GitHub Release.
4. `install --global --project <dir>|--projects-all` propaga los agentes a OpenCode global y a cada proyecto con `.kaizen`, sin sobrescribir su configuración existente.
5. El catálogo distingue capacidades con efectos (`launches_process`, `may_write_filesystem`) y exige el proceso `reverse_engineering` para usarlas.

## Alternativas consideradas
- Versionar solo el código: descartado; los agentes y skills son el producto.
- Instalar sin versión: descartado; no se puede reproducir ni actualizar.

## Límites
`install --project` solo escribe `.opencode/agents/*.md` generados; no modifica `opencode.json` ni archivos propios del proyecto. La instalación de REA requirió aprobación humana (ADR-0009).

## Cómo se verificará
- `kaizen-release status` muestra versión, catálogo y agentes.
- `bash tests/run.sh` valida catálogo y comandos de release en modo no destructivo.
