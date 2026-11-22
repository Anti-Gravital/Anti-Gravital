# docs: add local Compose stacks and a stack decision table

**Resumen:** Add six opt-in local service stacks with honest config-only verification.

Los stacks son complementarios de las plantillas y no cambian los defaults
nativos ni los documentos maestros.

## Fase afectada

Ninguna fase bloqueante. Artefactos de despliegue opcional de la ola 7 sobre
el alpha `v0.1.0-alpha.1`. No toca `docs/master/`, `docs/roadmap/STATUS.md`
ni archivos compartidos de la CLI.

## Tipo de cambio

- [ ] Codigo
- [x] Documentacion
- [ ] CI / tooling
- [ ] RFC / ADR

## Documentos relacionados

- `deploy/stacks/README.md` (nuevo)
- `deploy/stacks/VERIFICATION.md` (nuevo)
- `deploy/stacks/<stack>/compose.yml`, `.env.example`, `README.md` (nuevos)
- `deploy/stacks/monitoring/prometheus.yml` (nuevo)
- `docs/templates/STACKS.md` (nuevo)
- `docs/INDEX.md`
- `CHANGELOG.md`
- Regla 12 y ADR-0009 (integracion, defaults nativos)

## Plan de prueba

- Parseo real con PyYAML 6.0.3 de los siete archivos YAML.
- `docker compose -f compose.yml config` en los seis stacks y ademas
  `docker compose --profile app -f compose.yml config` en `postgres-api`.
- Comprobar que cada config resuelve volumenes con nombre, puertos y
  healthchecks, sin stderr.
- Comprobar que las etiquetas de imagen existen en la API de tags de Docker Hub.
- Comprobar ausencia de secretos reales, emojis y evidencia de herramientas
  externas; `git diff --check`.
- Registrar como SKIP el inicio de servicios, la descarga de imagenes y las
  pruebas HTTP: el daemon de Docker no estaba disponible.

## Criterios de salida que avanza

Ninguna casilla de `docs/roadmap/STATUS.md` se cierra. Entrega aditiva de
empaquetado y documentacion para la ola 7.

## Cierre de issues

Closes: none

## Checklist

- [x] Pertenece a la ola 7 y no adelanta fases.
- [x] No modifica maestros ni archivos compartidos de la CLI.
- [x] Integra herramientas dominantes sin reemplazarlas (regla 12).
- [x] Defaults nativos y memoria intactos (ADR-0009).
- [x] Sin secretos: `.env.example` con valores locales y `.env` ignorado.
- [x] Verificacion real con evidencia y SKIP honestos.
- [x] Documentacion tecnica en ingles canonico (ADR-0008).
- [x] Sin emojis ni evidencia de herramientas externas.
- [x] `git diff --check` limpio.
