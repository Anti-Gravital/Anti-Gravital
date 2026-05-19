# ag-cloud

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 5.
> Criticidad: Opcional.
> Capitulo de arquitectura: docs/architecture/10-despliegue-ag-cloud.md

## Dominio

Despliegue simplificado al estilo Railway/Fly.io. Soporta cuatro targets: docker-compose con Caddy como reverse proxy y TLS automatico, Fly.io via flyctl, Railway via su API, y Kubernetes con generacion de manifests estandar. Genera Dockerfiles multi-stage optimizados.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/10-despliegue-ag-cloud.md`.
- Hoja de ruta del crate: `docs/modules/ag-cloud/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
