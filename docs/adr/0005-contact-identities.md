# ADR-0005 - Identidades de contacto oficiales del proyecto

- Estado: aceptado
- Fecha: 2026-05-19
- RFC origen: ninguna; autorizacion verbal del BDFL inicial registrada
  en la conversacion de setup de Fase 0. Esta ADR sirve como registro
  formal del cambio.

## Contexto

Los maestros instalados en `docs/master/` usaban placeholders para los
correos institucionales del proyecto: `security@gravital.io` y
`hello@antigravital.dev`. Estos correos no existen ni estaban
asignados. La regla del repositorio dice que los maestros no se editan
sin RFC, pero tambien dice que los placeholders se reconcilian contra
la realidad cuando la realidad esta disponible.

Durante el setup de Fase 0, el BDFL inicial (Angel Nereira) declaro
los correos oficiales y autorizo el reemplazo de los placeholders en
los maestros y en los archivos operativos.

## Decision

Las identidades de contacto oficiales del proyecto Anti-Gravital son:

- Correo raiz del proyecto: `anti@gravitalcloud.com`.
- Mantenedor inicial (BDFL): Angel Nereira, primario
  `angelnereira@gravitalcloud.com`, alternativo
  `contact@angelnereira.com`.

Aplicacion concreta:

1. Contacto general del proyecto y reportes de conducta:
   `anti@gravitalcloud.com`.
2. Reportes de seguridad: GitHub Security Advisories primero;
   `anti@gravitalcloud.com` como primario por correo;
   `angelnereira@gravitalcloud.com` como respaldo.
3. Escalado al BDFL: `angelnereira@gravitalcloud.com`.

Se actualizan:

- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 15.3.
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` entregables de Fase 0.
- `docs/master/VERSION.md` con nuevos hashes SHA-256 y entrada de
  historial.
- `SECURITY.md`, `CODE_OF_CONDUCT.md`, `docs/governance/external-deliverables.md`,
  `docs/roadmap/STATUS.md`.
- Workflow `.github/workflows/docs.yml` con los nuevos hashes
  esperados.
- Derivados verbatim regenerados via `tools/split-masters.sh` y
  `tools/scaffold-docs.sh`.

## Consecuencias

Positivas:

- Los canales de contacto del proyecto pasan de aspiracionales a
  reales. Cualquier persona puede reportar seguridad o conducta de
  inmediato.
- Las casillas correspondientes de `docs/roadmap/STATUS.md` se
  marcan como completadas.
- Reduce ambiguedad sobre quien recibe los reportes.

Negativas:

- Los dominios `gravital.io` y `antigravital.dev` que aparecian en los
  placeholders quedan sin uso. Se mantiene la opcion de registrarlos
  como redirecciones a `gravitalcloud.com` si el proyecto los
  considera estrategicos en el futuro.
- El historial git refleja el cambio sobre los maestros; eso es
  deseable para auditabilidad.

## Alternativas consideradas

- Mantener los placeholders en los maestros y reflejar la realidad
  solo en los archivos operativos: rechazado porque crearia
  divergencia entre la fuente de verdad y la operacion real.
- Abrir RFC formal completa: rechazado por desproporcion; se trata de
  un reemplazo factual de placeholders por valores concretos, sin
  cambio arquitectonico.

## Notas

- Cualquier futuro cambio en estas identidades debera repetir el
  procedimiento: actualizar maestros, recomputar hashes en
  `docs/master/VERSION.md`, regenerar derivados y dejar registro en un
  nuevo ADR.
