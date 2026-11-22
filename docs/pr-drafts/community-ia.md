# Descriptor de PR

## Resumen

Add an MCP readiness review mapping RFC-0024 tools to current executors, capability schema, security gates and test cases (design only, no code)

## Fase afectada

Fase 4.8, etapa F (aditiva). Solo analisis previo: no Implementa ni avanza casillas del tablero.

## Tipo de cambio

- [ ] Correccion de bug
- [x] Documentacion / gobernanza (`docs/audits/`, `docs/INDEX.md`, `CHANGELOG.md`)
- [ ] CI
- [ ] Nueva feature
- [ ] Cambio que rompe compatibilidad

## Contexto

RFC-0024 y ADR-0017 definen una interfaz de maquina por capacidades, pero el
repositorio no contiene el sustrato que suponen: no existen `--output json`,
`ag describe`, plan/apply, registro de auditoria, `Secret<T>` ni `ag deploy`.
Este trabajo documenta la brecha y propone un contrato verificable antes de
escribir codigo, para no crear un segundo camino de ejecucion.

## Cambios

- `docs/audits/mcp-readiness-review.md`: mapeo herramienta por herramienta,
  propuesta de esquema del manifiesto de capacidades, comprobaciones de
  seguridad por tool mutante, limites de autodepuracion, checklist Priorizado y
  casos de prueba (incluido rechazo de agente sin capacidad).
- `docs/INDEX.md`: enlace a la nueva auditoria.
- `CHANGELOG.md`: entrada de documentacion, sin codigo.

## Documentos relacionados

- `docs/rfc/RFC-0024-mcp-shared-machine-interface.md`
- `docs/rfc/RFC-0021-server-install-remote-console-and-machine-interface.md` (etapa E)
- `docs/rfc/RFC-0020-environment-and-secrets.md` (tipo `Secret`)
- `docs/adr/0016-open-interfaces-between-ag-and-fabric.md`
- `docs/adr/0017-mcp-capability-scoped-machine-interface.md`
- `docs/diagrams/mcp-layer-flow.md`

## Plan de prueba

```sh
# Solo documentacion: no hay codigo que compilar ni tests que ejecutar.
test -f docs/audits/mcp-readiness-review.md
grep -q 'mcp-readiness-review' docs/INDEX.md
grep -q 'MCP readiness review' CHANGELOG.md

# El job "prohibited content scan" de CI verifica atribucion, nombres de
# herramientas, emojis y marcas retiradas en todos los archivos versionados.
# Los archivos de este cambio solo usan terminos de producto ("connected
# agent", "MCP"), sin atribucion de herramientas ni emojis.
```

## Criterios de salida que avanza

- Ninguno: no se cierra ninguna casilla de `docs/roadmap/STATUS.md`. La etapa F
  sigue sin iniciar a la espera de la enmienda RFC y del sustrato de la etapa E.

## Cierre de issues

Closes: none

## Checklist final

- [x] Pertenece a la fase correcta y respeta la documentacion.
- [x] No rompe arquitectura ni anade dependencias.
- [x] No requiere compilacion: el cambio es solo documentacion.
- [x] Documentacion actualizada junto al cambio.
- [x] Sin evidencia de herramientas IA; atribuido a personas y a la organizacion.
