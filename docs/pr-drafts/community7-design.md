# docs: define canonical template design and deployment contract

**Resumen:** Add one English canonical reference that makes the 20+ project
templates consistent, composable and safe to deploy, without changing code.

La galeria de la ola 6 explica como elegir una plantilla, pero no existia un
contrato comun de esqueleto, salud, configuracion ni limites plantilla/adaptador.
Este documento fija ese contrato y separa lo que genera el framework, lo que
aporta el operador y lo que sigue siendo nativo por defecto. Senala ademas la
deriva real frente a las plantillas actuales como seguimiento, sin corregirla
aqui ni afirmar capacidades inexistentes.

## Fase afectada

Ninguna fase bloqueante. Documentacion de apoyo a las olas 6 y 7, sobre el alpha
`v0.1.0-alpha.1`. No toca `docs/master/`, no adelanta la Fase 5 y no cierra la
puerta de release.

## Tipo de cambio

- [ ] Codigo
- [x] Documentacion
- [ ] CI / tooling
- [ ] RFC / ADR

## Documentos relacionados

- `docs/templates/TEMPLATE_DESIGN.md` (nuevo)
- `docs/INDEX.md`
- `CHANGELOG.md`
- `docs/templates/GALLERY.md` y `docs/templates/README.md` (ola 6, rama gallery)
- `templates/README.md`
- `docs/rfc/RFC-0019-project-layered-architecture.md`
- `docs/rfc/RFC-0020-environment-and-secrets.md`
- `docs/rfc/RFC-0021-server-install-remote-console-and-machine-interface.md`
- `docs/adr/0009-gobernanza-correctiva.md`

## Plan de prueba

- `git diff --check` limpio.
- Comprobar que todos los enlaces relativos nuevos resuelven dentro de esta rama
  (solo se enlaza `docs/templates/TEMPLATE_DESIGN.md`, creado aqui; GALLERY se
  cita como texto por pertenecer a otra rama).
- Comprobar que el documento cubre los doce apartados exigidos: esqueleto,
  nombres y variables, salud y readiness, logging, precedencia, secretos,
  plantilla frente a adaptador, composicion, actualizacion, checklist y
  matriz Docker/Kubernetes.
- Comprobar que la matriz distingue generate/operador/nativo y que no afirma
  artefactos de despliegue ya entregados.
- Comprobar que la seccion de conflictos cita evidencia real de las plantillas
  actuales (`BIND`, `/health` parcial, `[env]` pendiente).
- Comprobar ausencia de emojis, de nombres de herramientas IA y de prefijos
  prohibidos en rutas y texto.
- No se edita `cli.rs`, `commands/`, `templates/` ni directorios de otros roles.

## Criterios de salida que avanza

Ninguna casilla de `docs/roadmap/STATUS.md` se cierra. Es documentacion de
apoyo que desbloquea la redaccion coherente de los artefactos de las ramas
docker, k8s, deploy y stacks.

## Cierre de issues

Closes: none

## Checklist

- [x] Pertenece a la ola 7 y no adelanta fases.
- [x] No modifica maestros ni el texto de estado alpha del README.
- [x] No anade dependencias ni complejidad de codigo.
- [x] Referencia tecnica en ingles canonico (ADR-0008).
- [x] Estado honesto: costumbre normativa frente a adoption real.
- [x] Enlaces relativos verificados en esta rama.
- [x] Sin emojis ni evidencia de herramientas externas.
- [x] `git diff --check` limpio.
