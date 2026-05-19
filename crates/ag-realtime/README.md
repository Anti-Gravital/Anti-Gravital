# ag-realtime

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 4.
> Criticidad: Estandar.
> Capitulo de arquitectura: docs/architecture/08-modulos-batteries-included.md

## Dominio

Tiempo real: WebSocket binario, SSE como fallback, NATS embebido para casos pequenos y cliente NATS externo para produccion. Soporta pub/sub, suscripciones por canal, presence, broadcasting selectivo y reconexion con replay.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/08-modulos-batteries-included.md`.
- Hoja de ruta del crate: `docs/modules/ag-realtime/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
