# Descriptor de PR - community5/docs

## Resumen

docs(alpha): public alpha status, try-it flow, release notes and install guide EN+ES

## Fase afectada

Gobernanza / documentacion de release. Ninguna casilla de la hoja de ruta se
marca como avanzada. No se tocan los maestros de `docs/master/` ni sus hashes.

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

- ADR-0008 (ingles canonico, vitrina bilingue con ingles primero).
- RFC-0024 seccion 9 (prerequisitos MCP sin cumplir) y RFC-0028 (evidencia CI
  del feature `otlp`).
- `docs/audits/PRE_FASE5_RELEASE_GATE.md` (puerta de release abierta).
- Version de release `0.1.0-alpha.1` acordada por el coordinador de la ola 5.
- Entregables: `README.md`, `docs/releases/0.1.0-alpha.1.md`,
  `docs/INDEX.md`, `docs/manual/04-instalacion-y-onboarding.md`.

## Plan de prueba

1. `git diff --check` sobre todo el diff de la rama.
2. Comprobar que todos los enlaces relativos nuevos resuelven archivos
   existentes: `docs/releases/0.1.0-alpha.1.md`, `docs/manual/02-primera-api.md`,
   `docs/manual/04-instalacion-y-onboarding.md`, `examples/`,
   `docs/security/INSTALLATION_INTEGRITY.md`, `docs/audits/PRE_FASE5_RELEASE_GATE.md`.
3. Verificar paridad EN/ES de las secciones nueva: estado alpha, Try it /
   Probarlo, Alpha what works / Alpha que funciona, tabla de vistazo y mapa de
   documentacion.
4. Escanear emojis, evidencia de herramientas IA y prefijos prohibidos en
   nombres de archivo con los patrones de `.github/workflows/docs.yml`.
5. Confirmar que no se edita `docs/master/`, `CHANGELOG.md` ni archivos de
   otras ramas de la ola 5.
6. Reconciliacion con hallazgos de seguridad (#191 y O3): no se afirma que
   `ag dev` enlaza solo loopback ni que no existen binarios precompilados; el
   canal opt-in se describe como solo-checksums y pendiente de verificacion.

## Criterios de salida que avanza

- La cara publica declara honestamente alpha `v0.1.0-alpha.1` disponible para
  pruebas y no listo para produccion, en ingles y espanol.
- Existe una ruta corta documentada: instalar desde el fuente, crear proyecto,
  ejecutar `ag dev` y generar artefactos con `ag generate`.
- Existen notas de release con alcance, limitaciones, evidencia pendiente de QA
  y ruta de actualizacion, y estan indexadas en `docs/INDEX.md`.
- El manual de instalacion indica que el alpha se instala desde el fuente y
  como verificar la version.

## Cierre de issues

Closes: none
Refs #191

## Checklist

- [x] El cambio pertenece a la fase y a la gobernanza correcta.
- [x] Respeta la documentacion y no altera los maestros.
- [x] No anade dependencias ni complejidad innecesaria.
- [x] La evidencia de verificacion es real y reproducible.
- [x] Documentacion actualizada en la misma PR (README y manual incluidos).
