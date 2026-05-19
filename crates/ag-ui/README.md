# ag-ui

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 4 o posterior.
> Criticidad: Opcional.
> Capitulo de arquitectura: docs/architecture/05-ecosistema-modulos.md

## Dominio

Renderizado del lado del servidor con askama, hidratacion selectiva e integracion HTMX. No compite con frameworks SPA: cubre el rango donde un stack JS completo es excesivo. Para SPA o SSR ricas, el patron recomendado es Anti-Gravital como backend mas Next.js como frontend.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/05-ecosistema-modulos.md`.
- Hoja de ruta del crate: `docs/modules/ag-ui/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
