# Descriptor de PR - community4/review

## Resumen

docs(audit): read-only wave-4 security and low-level review with three findings

## Fase afectada

Gobernanza / auditoria. Ninguna fase de la hoja de ruta se marca como avanzada.
Los maestros de `docs/master/` y los hashes de `docs.yml` no se tocan.

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
- [x] security

## Documentos relacionados

- RFC-0018, RFC-0019, RFC-0020, RFC-0021 y RFC-0022 (alcance de las siete ramas).
- ADR-0008 (ingles canonico en documentacion tecnica) y ADR-0011 (marcas).
- Reglas de `CLAUDE.md` 16, 18, 22, 26, 28, 29 y 35.
- `docs/audits/wave4-review.md` (entregable principal), `docs/INDEX.md` y
  `CHANGELOG.md`, actualizados en esta misma PR.

## Plan de prueba

1. `git diff --check` sobre todo el diff de esta rama.
2. Escaneo de evidencia de herramientas IA (patrones de `docs.yml`).
3. Escaneo de emojis y de marcas comerciales retiradas (patrones de `docs.yml`).
4. Comprobacion de que el descriptor declara cierre de issues.
5. Revision de que ningun archivo de otra rama de la wave 4 fue modificado.

## Criterios de salida que avanza

- Revision de seguridad de `Secret<T>`, salida JSON, hooks, permisos de CI y
  ausencia de descargas de red nuevas.
- Revision de bajo nivel de validacion antes de abrir listener, aciclicidad de
  dependencias de plantillas, idempotencia de scripts y ausencia de panics por
  entrada CLI malformada.
- Hallazgos registrados con severidad, rama afectada y recomendacion.
- Issues #187, #188 y #190 abiertos con pasos reproducibles y criterios de
  verificacion; ningun issue cerrado falsamente.

## Cierre de issues

Closes: none
Refs #187
Refs #188
Refs #190

## Checklist

- [x] El cambio pertenece a la fase y a la gobernanza correcta.
- [x] Respeta la documentacion y no altera los maestros.
- [x] No anade dependencias ni complejidad innecesaria.
- [x] La evidencia de verificacion es real y reproducible.
- [x] Documentacion y CHANGELOG actualizados en la misma PR.
