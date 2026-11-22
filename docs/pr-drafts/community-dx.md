# Descriptor de PR

## Resumen

docs(ag-mail): diseno DX de la API REST gestionada y objetos de marketing, con la condicion exacta de la puerta (Refs #152)

## Fase afectada

Fase 4.6, pista de correo, etapas 4.6-C/4.6-D (RFC-0009). Solo documentacion:
no se escribe codigo ni se abre la superficie `api` mas alla del webhook ya
implementado. No avanza ninguna casilla de `docs/roadmap/STATUS.md`.

## Tipo de cambio

- [ ] Correccion de bug
- [x] Documentacion (ficha de modulo ag-mail, cambio en CHANGELOG)
- [ ] Gobernanza (no crea ni modifica RFC/ADR)
- [ ] Nueva feature
- [ ] Mantenimiento
- [ ] Cambio que rompe compatibilidad

## Contexto

El issue #152 pide decidir si la API REST gestionada y los objetos de marketing
son implementables ahora sin romper la puerta de release. La respuesta es
diseno unicamente, por tres razones documentadas en el propio documento:

1. La puerta de Correo A de RFC-0009 (entrega real pass SPF/DKIM/DMARC con
   evidencia) no tiene prueba: el puerto 25 esta bloqueado (DEBT-022 / #153).
   RFC-0009 seccion 5 ordena A antes que C/D; adelantarse viola la regla 8.
2. RFC-0009 secciones 4.5/4.6 solo esbozan la superficie: no hay contrato de
   peticiones/respuestas, errores ni idempotencia. Codificar seria a ciegas.
3. RFC-0024 (acceso por capacidades a agentes) se acepto el 2026-11-21, despues
   de RFC-0009, y no esta reflejado en el. Disenar la API sin el generaria
   retrabajo. Cualquier comando `ag mail api ...` ademas requiere RFC (regla 28).

El entregable faltante es el diseno, no una implementacion especulativa.

## Documentos relacionados

- `docs/modules/ag-mail/managed-api-dx-design.md` (nuevo)
- `docs/modules/ag-mail/README.md` (indice)
- `CHANGELOG.md`
- `docs/rfc/RFC-0009-ag-mail-native-mta.md` (sec. 4.5, 4.6, 5)
- `docs/rfc/RFC-0024-mcp-shared-machine-interface.md`
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` (sec. 4.6.3)
- `CLAUDE.md` reglas 0, 1, 7, 8, 22, 26, 28, 32; `ADR-0008`, `ADR-0009`,
  `ADR-0011`

## Plan de prueba

Documentacion pura; no cambia codigo. Verificacion local:

```sh
git diff --check
grep -R "managed-api-dx-design" docs/modules/ag-mail/README.md CHANGELOG.md
```

El documento especifica el plan de prueba de la futura implementacion:
`cargo test -p ag-mail --features api` y los jobs `mail-*` en verde, con
PostgreSQL 16 local o efimero, sin terceros.

## Criterios de salida que avanza

Ninguno. La etapa 4.6-C sigue parcial (webhook firmado) y la 4.6-D sin iniciar
en `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` seccion 4.6.1. La puerta se
abre solo con la condicion descrita en la seccion 8 del diseno.

## Cierre de issues

Closes: none

- Refs #152 (avanza con el contrato disenado; no cierra: la implementacion
  sigue pendiente de la puerta de Correo A y de la enmienda de RFC-0024)

## Checklist

- [x] Pertenece a la fase correcta (diseno documental antes que codigo).
- [x] Respeta la documentacion y no introduce capacidades no documentadas.
- [x] No anade dependencias ni codigo; no aplica compilacion ni tests nuevos.
- [x] CHANGELOG actualizado en el mismo cambio.
- [x] Sin emojis, sin atribucion de herramientas, mensaje corto.
