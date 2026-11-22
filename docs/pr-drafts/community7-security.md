# Descriptor de PR - community7/security

## Resumen

security(deploy): read-only audit of wave 7 deployment artifacts and merge gate

## Fase afectada

Gobernanza / seguridad de los artefactos de despliegue de la ola 7. No marca
casillas de la hoja de ruta, no toca los maestros de `docs/master/` ni sus
hashes, y no modifica archivos de otras ramas de la ola 7.

## Tipo de cambio

- [ ] feat
- [ ] fix
- [x] docs
- [ ] refactor
- [ ] test
- [ ] chore
- [ ] ci
- [ ] build
- [ ] perf

## Documentos relacionados

- `docs/audits/deployment-security-review.md` (entregable principal).
- `docs/INDEX.md` y `CHANGELOG.md` (entradas de la auditoria).
- RFC-0021 seccion 4.3 (plan/apply), ADR-0011 (marcas comerciales),
  ADR-0008 (ingles canonico), regla 12 (integracion, no reinvencion).
- Issues abiertos a partir de defectos reales (se listan al cierre).

## Plan de prueba

1. `git diff --check` sobre todo el diff de la rama.
2. Escaneos de contenido: emojis, evidencia de herramientas IA, URLs de
   sesiones, marcas retiradas y prefijos de rama prohibidos.
3. Comprobar la seccion de cierre de issues con el patron de
   `.github/workflows/docs.yml`.
4. Verificar que los unicos archivos cambiados son el informe de auditoria,
   `docs/INDEX.md`, `CHANGELOG.md` y este descriptor.
5. Confirmar que no se edita ningun archivo de las ramas docker, k8s, stacks,
   deploy, deployqa ni design.

## Criterios de salida que avanza

- Existe un informe de auditoria con hallazgos por familia, severidad y
  recomendacion de merge para los artefactos de despliegue.
- Cada defecto real tiene un issue enfocado con reproduccion, criterios de
  aceptacion y comandos de verificacion.
- Las observaciones por debajo del umbral quedan solo en la nota de auditoria.
- La recomendacion de go/no-go es honesta y verificable.

## Cierre de issues

Closes: none
Refs #193
Refs #194
Refs #195

## Checklist

- [x] El cambio pertenece a la fase y a la gobernanza correcta.
- [x] Respeta la documentacion y no altera los maestros.
- [x] No anade dependencias ni complejidad innecesaria.
- [x] La evidencia de verificacion es real y reproducible.
- [x] Documentacion actualizada en la misma PR.
