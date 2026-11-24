## Resumen

Correcciones deterministas de CI y hardening del framework, autorizadas por humano
para editar rutas protegidas (.github/, scripts/). Fase actual; sin alcance nuevo.

## Fase afectada

Gobernanza / CI (fase 4.8, sin salto de fase).

## Tipo de cambio

fix + hardening.

## Documentos relacionados

CHANGELOG.md, CONTRIBUTING.md (EN/ES), docs/audits/chrono-edge-monitor.md,
docs/audits/community30-metricsrace-evidence.md, journals L-13.

## Plan de prueba

ag-check 8/8; YAML parse de 3 workflows; chrono monitor PASS;
scripts/kaizen check verde; CI en main tras merge.

## Criterios de salida

#214 y #215 cerrados con run verde; #185 sigue abierto (bloqueado por upstream);
ningun gate debilitado, ninguna supresion nueva (S-10).

## Checklist final

- [x] Cambios minimos y autorizados en rutas protegidas
- [x] Integridad del checksum de gitleaks intacta
- [x] Fail-closed preservado en monitor y metrics
- [x] Docs EN/ES alineadas

## Cierre de issues

Closes #214
Closes #215
Refs #185
Refs #208
