# community9/dnsacme

## Resumen

Offline ACME mock harness and evidence for #87/#179; staging split and migration-point correction, no production code.

## Fase afectada

Fase 5 (RFC-0011, extension of `ag-domains`). Documentation and local tooling only; no roadmap phase change.

## Tipo de cambio

- [x] Documentacion (guia, referencia, evidencia, CHANGELOG)
- [x] Herramienta local de verificacion (scripts/)
- [ ] Codigo de produccion
- [ ] Cambio de dependencias
- [ ] Cambio de arquitectura o RFC

## Documentos relacionados

- `docs/ag-domains/how-to/acme-staging-test.md` (guia de staging; separacion mock local vs real)
- `docs/ag-domains/acme-mock-evidence.md` (evidencia real del mock, sin CA)
- `docs/ag-domains/reference/instant-acme-0.8-migration.md` (correccion de numeros de linea, #179)
- `docs/ag-domains/README.md` (indice)
- RFC-0011 fases E-F; ADR-0008 (ingles canonico); ADR-0009 (modo nativo por defecto)
- Issues #87 y #179: permanecen abiertos con `blocked`

## Plan de prueba

```
bash -n scripts/acme-mock-harness.sh
python3 -m py_compile scripts/acme-mock-harness.py
scripts/acme-mock-harness.sh /tmp/acme-mock-run.log
git diff --check
```

Ademas se ejecutaron los escaneos de contenido del workflow `docs.yml` (sin
atribucion de herramientas IA, sin emojis, sin marcas retiradas) y el job de
palabras clave de cierre del descriptor. Tres ejecuciones consecutivas
terminaron en `result: PASS` con codigo 0 y la misma secuencia de ocho estados.
Sin ejecucion de staging.

## Criterios de salida que avanza

- Evidencia reproducible offline para la maquina de estados de orden ACME (#87: avanza, no cierra).
- Verificacion de que los nueve puntos de migracion siguen aplicando al codigo actual (#179: avanza, no cierra).
- Guia de staging con separacion explicita entre mock local y staging real con credenciales.

## Cierre de issues

Closes: none
Refs #87
Refs #179

## Checklist

- [x] Pertenece a la fase correcta
- [x] Respeta la documentacion y RFC-0011
- [x] No rompe arquitectura ni anade dependencias
- [x] No modifica codigo de produccion ACME
- [x] Sin evidencia de herramientas IA, sin emojis, sin marcas prohibidas
- [x] Verificacion real registrada, sin claims de certificado emitido por CA
