# ag-observe

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 4.
> Criticidad: Estandar.
> Capitulo de arquitectura: docs/architecture/14-observabilidad-ag-observe.md

## Dominio

Observabilidad nativa: tracing estructurado, exporter OpenTelemetry (OTLP), metricas Prometheus, integracion con tokio-console en modo dev y dashboards Grafana incluidos como JSON. Cero configuracion para casos comunes.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/14-observabilidad-ag-observe.md`.
- Hoja de ruta del crate: `docs/modules/ag-observe/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
