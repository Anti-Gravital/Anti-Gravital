# Calendario publico de releases

> Fuente verbatim: `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`, tabla
> de resumen de fases.

Las fechas son estimaciones, no compromisos. La regla 4 de las reglas
de oro dice: "El proyecto se libera cuando esta listo, no cuando lo
exige una fecha externa."

| Fase | Hito | Duracion estimada | Estado |
| --- | --- | --- | --- |
| 0 | Fundaciones y gobernanza | 1-2 meses | En curso (criterios externos pendientes; implementacion bajo RFC-0001) |
| 1 | The Shield MVP | 2-3 meses | Implementacion tecnica completa |
| 2 | The Core MVP | 2 meses | Implementacion tecnica completa |
| 3 | Anti-DSL alpha (v0.1 a v0.8) | 3 meses | Implementacion tecnica disponible |
| 4 | Modulos estandar | 3 meses | Implementacion tecnica disponible |
| 4.5 | ag-mail + ag-domains: comunicacion y dominios | 1-2 meses | Implementacion tecnica disponible |
| 4.6 | MTA nativo de ag-mail (A/B/C) y motor ag-workers (D) | Aditiva | ag-workers S1-S5 disponible y verificado en CI; paridad PostgreSQL viva en Issues #108/#109/#103 |
| 5 | ag-cloud y version 0.5 beta publica | 2 meses | Pendiente |
| 6 | ag-ai y Knowledge Graph | 2 meses | Pendiente |
| 7 | ag-migrate importadores | 2 meses | Pendiente |
| 8 | ag-mobile Flutter bridge | 2 meses | Pendiente |
| 9 | Sistema de plugins WASI | 2 meses | Pendiente |
| 10 | Endurecimiento y version 1.0 | 3 meses | Pendiente |

Duracion total estimada: 25 a 30 meses.
Hito de version beta publica (0.5): final de Fase 5, aproximadamente
mes 15. Las Fases 4.5 (ADR-0007) y 4.6 (ADR-0010 para el MTA nativo de
ag-mail; RFC-0012/ADR-0013 para ag-workers) son aditivas y no adelantan
este hito.
Hito de version 1.0 estable: final de Fase 10, aproximadamente mes 30.

## Cadencia post-1.0

A partir de la 1.0 el proyecto entra en modo de mantenimiento estable
con releases minor cada 3 meses, releases patch segun necesidad de
seguridad y una linea de soporte LTS anunciada por el comite tecnico.
