# Fase 4 - Modulos estandar

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-03-anti-dsl-alpha.md](./fase-03-anti-dsl-alpha.md)
> Siguiente: [fase-05-ag-cloud.md](./fase-05-ag-cloud.md)

## Fase 4 — Módulos estándar

**Objetivo.** Completar los módulos batteries-included: auth, realtime, cache, storage, observe. Cada uno como crate independiente, con tests, documentación y ejemplos.

### 4.1 Criterios de entrada

- [ ] Fase 3 completada.
- [ ] DSL versión 0.5 (auth y políticas) iniciada.

### 4.2 Entregables

- [ ] DSL versión 0.5: declaración de auth y políticas RBAC.
- [ ] DSL versión 0.6: declaración de eventos.
- [ ] Crate `ag-auth` completo: WebAuthn, JWT Ed25519, OAuth2 (Google, GitHub), API keys, refresh tokens con rotación.
- [ ] Crate `ag-realtime` completo: WebSocket binario, SSE fallback, NATS embebido para casos pequeños, cliente NATS externo para producción.
- [ ] Crate `ag-cache` completo: moka L1 + Redis L2 con fred, invalidación por evento.
- [ ] Crate `ag-storage` completo: adaptadores S3, MinIO, filesystem local. URLs firmadas. Procesamiento de imágenes.
- [ ] Crate `ag-observe` completo: tracing, OpenTelemetry exporter, métricas Prometheus, dashboards Grafana JSON incluidos.
- [ ] Integración de tokio-console en modo dev.
- [ ] Example `realtime-chat` en `examples/`.
- [ ] Example `ai-backend` en `examples/` que demuestra streaming SSE.
- [ ] Tests de integración cross-module.

### 4.3 Criterios de salida (puerta antes de Fase 5)

- [ ] Los cinco módulos publicados en crates.io con sus respectivos releases independientes.
- [ ] Cobertura de tests ≥ 80% en cada módulo.
- [ ] Documentación cada módulo: README, guía de uso, referencia de API.
- [ ] Performance: el módulo `ag-realtime` sostiene 50 K conexiones WebSocket en una instancia 2 vCPU sin degradación.
- [ ] Performance: el módulo `ag-cache` muestra ≥ 1 M ops/segundo en L1.
- [ ] Al menos cinco issues bug reports cerrados por la comunidad.
- [ ] Al menos 500 stars en el repositorio.

### 4.4 Riesgos de la fase

El riesgo principal es la fragmentación del esfuerzo entre cinco módulos paralelos. La mitigación es secuenciar la implementación: primero auth (bloquea muchos casos de uso), luego data avanzado, luego realtime, luego cache, luego storage, luego observe.

---
