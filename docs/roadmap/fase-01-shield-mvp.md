# Fase 1 - The Shield MVP

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-00-fundaciones-y-gobernanza.md](./fase-00-fundaciones-y-gobernanza.md)
> Siguiente: [fase-02-core-mvp.md](./fase-02-core-mvp.md)

## Fase 1 — The Shield MVP

**Objetivo.** Implementar la capa Shield del núcleo: una pipeline de middleware Tower que valida, autentica básicamente, aplica rate limiting y entrega requests a un handler placeholder. Sin Core completo todavía. Sin DSL todavía. El producto es un binario que responde HTTP con seguridad básica y benchmark publicable.

### 1.1 Criterios de entrada

- [ ] Fase 0 completada con todos sus criterios de salida marcados.
- [ ] Al menos un contribuidor adicional al mantenedor principal está activo en el repositorio.

### 1.2 Entregables

- [ ] Crate `ag-core` con módulo `shield` operativo.
- [ ] Soporte de HTTP/1.1 y HTTP/2 vía Axum + Tokio.
- [ ] Terminación TLS 1.3 con rustls.
- [ ] Middleware de validación de payload básico (deserialización con serde y restricciones simples).
- [ ] Middleware de autenticación JWT con verificación Ed25519.
- [ ] Middleware de rate limiting con governor (token bucket por IP).
- [ ] Middleware CORS y CSRF con defaults seguros.
- [ ] Middleware de logging estructurado con `tracing`.
- [ ] Configuración mínima desde archivo TOML.
- [ ] Tests unitarios con cobertura ≥ 80% del crate `ag-core`.
- [ ] Tests de integración end-to-end del pipeline Shield.
- [ ] Benchmark Hello World ejecutable: `cargo bench` produce cifras reproducibles.
- [ ] Documentación API del crate generada con `cargo doc`, publicada en `docs.rs`.
- [ ] Capítulo del manual de usuario explicando cómo usar la Shield directamente como librería.

### 1.3 Criterios de salida (puerta antes de Fase 2)

- [ ] Benchmark Hello World alcanza ≥ 300 K req/s en hardware de referencia documentado.
- [ ] Latencia p99 del pipeline Shield ≤ 1 ms a 100 K req/s.
- [ ] Memoria del proceso idle ≤ 15 MB.
- [ ] Tiempo de arranque ≤ 100 ms.
- [ ] CI pasa en las cuatro plataformas objetivo.
- [ ] Análisis estático con `clippy` sin warnings.
- [ ] Análisis de dependencias con `cargo-audit` sin vulnerabilidades conocidas.
- [ ] Cero bloques `unsafe` no documentados.
- [ ] Al menos un blog post técnico publicado sobre la arquitectura de la Shield.
- [ ] Al menos diez stars en el repositorio.

### 1.4 Riesgos de la fase

El riesgo principal es underestimar la complejidad de TLS y rate limiting en producción. La mitigación es usar exclusivamente crates probados (rustls, governor) y no rodar implementaciones propias. El riesgo secundario es que las cifras de benchmark no alcancen el objetivo; la mitigación es publicar lo que se mide con honestidad y documentar el déficit.

---
