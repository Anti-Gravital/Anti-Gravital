# Fase 5 - ag-cloud despliegue simplificado

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-04-modulos-estandar.md](./fase-04-modulos-estandar.md)
> Siguiente: [fase-06-ag-ai-knowledge-graph.md](./fase-06-ag-ai-knowledge-graph.md)

## Fase 5 — `ag-cloud` despliegue simplificado

**Objetivo.** Construir el subsistema de despliegue al estilo Railway/Fly.io. Soporte para los cuatro targets: docker-compose, fly, railway, k8s. Este es el hito de **versión beta pública (0.5)**.

### 5.1 Criterios de entrada

- [ ] Fase 4 completada.
- [ ] Decisión RFC sobre los targets de despliegue soportados en la 1.0.

### 5.2 Entregables

- [ ] Crate `ag-cloud` con módulos para cada target.
- [ ] Especificación del archivo `deploy.ag`.
- [ ] Generador de Dockerfile multi-stage optimizado para imagen mínima.
- [ ] Target docker-compose: generación completa de stack con Caddy como reverse proxy y TLS automático.
- [ ] Target fly: integración con flyctl.
- [ ] Target railway: integración con su API.
- [ ] Target k8s: generación de manifests estándar.
- [ ] Comando `ag deploy`.
- [ ] Comando `ag rollback`.
- [ ] Pipeline de migraciones de base de datos integrado al despliegue.
- [ ] Documentación: "Desde cero a producción en 15 minutos" con cada target.

### 5.3 Criterios de salida (puerta antes de Fase 6 y versión 0.5)

- [ ] El example `todo-api` se despliega exitosamente a Fly.io con `ag deploy`.
- [ ] El example `ecommerce-api` se despliega exitosamente con docker-compose a un VPS y se accede vía dominio con TLS.
- [ ] El example `realtime-chat` se despliega exitosamente a Railway.
- [ ] Versión 0.5 (beta pública) liberada en GitHub Releases.
- [ ] Anuncio público en Hacker News, Reddit `/r/rust`, Twitter/X, Bluesky, LinkedIn.
- [ ] Al menos diez proyectos externos reportan que han desplegado Anti-Gravital en producción o staging.
- [ ] Al menos 1 500 stars en el repositorio.

### 5.4 Riesgos de la fase

El riesgo principal es la dependencia de APIs externas (Fly, Railway) que pueden cambiar. La mitigación es estructurar cada target como un módulo desacoplado con tests de contrato.

---
