# Record committee decisions for RFC-0025, RFC-0027 and issue #152

## Resumen

Record three BDFL decisions of 2026-11-22: RFC-0025 accepted as design with implementation deferred, RFC-0027 deferral accepted (keep chumsky 0.9.3), Mail C/D deferred per Hoja de Ruta 4.6.3.

## Fase afectada

Gobernanza transversal. RFC-0025 targets Fase 5+ (no phase opened); RFC-0027 amends the Fase 3 compiler stack (no code change); issue #152 belongs to Fase 4.6 mail track (C/D). No phase state changes and no master document is touched.

## Tipo de cambio

- [x] Decision record (ADR-0020, ADR-0021)
- [x] Cambio de estado de RFC (RFC-0025, RFC-0027) con fecha y decisor, regla 28
- [x] Documentacion de decision para la issue #152
- [x] Indices (`docs/rfc/README.md`, `docs/adr/README.md`) y CHANGELOG
- [ ] RFC nuevo (no requerido: las decisiones se toman sobre RFC existentes)
- [ ] Codigo, dependencias o CI (ninguno)

## Documentos relacionados

- RFC-0025, ADR-0020, issue #154, `docs/audits/PRE_FASE5_RELEASE_GATE.md`
- RFC-0027, ADR-0021, RFC-0003, issues #180 y #156
- issue #152, `docs/modules/ag-mail/managed-api-dx-design.md`, RFC-0009, Hoja de Ruta 4.6.3
- `CLAUDE.md` reglas 5, 8, 22, 26, 28; `docs/rfc/README.md`

## Cambios concretos

- `docs/adr/0020-ag-data-orm-rls-decision.md` y `docs/adr/0021-chumsky-upgrade-deferral.md`: nuevos, con fecha 2026-11-22 y decisor BDFL.
- `docs/rfc/RFC-0025-ag-data-orm-rls.md`: estado "accepted as design, implementation deferred"; seccion 10 con fecha y decisor.
- `docs/rfc/RFC-0027-chumsky-parser-upgrade.md`: estado "accepted, defer"; seccion 9 con fecha y decisor.
- `docs/modules/ag-mail/managed-api-dx-design.md`: seccion "Committee decision (2026-11-22)" con el diferimiento de Mail C/D.
- Indices de RFC y ADR y CHANGELOG actualizados. `docs/roadmap/STATUS.md` no cambia: ninguna casilla cambia de estado.

## Plan de prueba

```
git diff --check
sha256sum docs/master/*.md   # comparar con la linea base; sin cambios
grep -n '^## Cierre de issues' -A3 docs/pr-drafts/community2-decisions.md
```

No hay codigo: no aplican cargo fmt/clippy/test/audit/deny. Las comprobaciones son de documentacion y trazabilidad.

## Criterios de salida que avanza

- RFC-0025: de `proposed` a `accepted as design`, con bloqueo de implementacion explicito.
- RFC-0027: de `proposed` a `accepted` con recomendacion de diferir adoptada.
- Issue #152: decision formal registrada; sigue abierta, ninguna puerta se cierra.
- Issue #180: decision registrada; sigue abierta hasta la reconciliacion del tablero.

## Cierre de issues

Refs #180
Refs #152
Closes: none

## Checklist

- [x] Decisiones dentro del alcance: no se inventa codigo ni fase nueva
- [x] Fecha y decisor registrados en cada RFC y ADR (regla 28)
- [x] Sin cambios en `docs/master/` (hash verificado antes y despues)
- [x] Issues #152 y #180 permanecen abiertas: puertas pendientes
- [x] Sin emojis, sin evidencia de herramientas IA, mensajes <=256 caracteres
