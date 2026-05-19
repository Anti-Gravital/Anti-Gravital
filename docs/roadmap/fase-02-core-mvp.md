# Fase 2 - The Core MVP y roundtrip completo

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-01-shield-mvp.md](./fase-01-shield-mvp.md)
> Siguiente: [fase-03-anti-dsl-alpha.md](./fase-03-anti-dsl-alpha.md)

## Fase 2 — The Core MVP y roundtrip completo

**Objetivo.** Completar el núcleo con la capa Core: router Axum, extractores tipados, sistema de errores, estado compartido. Implementar el roundtrip completo Request → Shield → Core → Handler → Respuesta. Conectar a PostgreSQL real para un CRUD mínimo. El producto es un binario que sirve una API real, aunque escrita manualmente sin DSL.

### 2.1 Criterios de entrada

- [ ] Fase 1 completada con todos sus criterios de salida marcados.
- [ ] El crate `ag-data` ha sido iniciado con sqlx como dependencia.

### 2.2 Entregables

- [ ] Crate `ag-core` con módulo `core` operativo.
- [ ] Router Axum integrado con la Shield.
- [ ] Extractores: `State<T>`, `ValidatedBody<T>`, `Claims<T>`, `Path<T>`, `Query<T>`.
- [ ] Sistema de errores `AgError` con conversión automática a respuesta HTTP.
- [ ] Sistema de respuestas: JSON, plaintext, streams.
- [ ] Crate `ag-data` con pool de conexiones PostgreSQL vía sqlx.
- [ ] Sistema de migraciones embebido con `sqlx::migrate!`.
- [ ] Example app `todo-api` en `examples/` con CRUD completo.
- [ ] Benchmark CRUD + DB ejecutable.
- [ ] Crate `ag-cli` con comandos `new` (crea proyecto desde template), `dev` (arranca servidor con hot reload vía `cargo-watch`), `build` (compila release).
- [ ] Tres templates: `rest`, `realtime`, `fullstack`.

### 2.3 Criterios de salida (puerta antes de Fase 3)

- [ ] Benchmark CRUD + PostgreSQL alcanza ≥ 40 K req/s en hardware de referencia.
- [ ] Latencia p99 del CRUD ≤ 5 ms.
- [ ] La app `todo-api` corre exitosamente con `ag new` + `ag dev`.
- [ ] La app `todo-api` se despliega como binario único (`FROM scratch` Docker).
- [ ] El binario release del `todo-api` ocupa ≤ 20 MB.
- [ ] Documentación: "Tu primera API con Anti-Gravital" publicada.
- [ ] Al menos 50 stars en el repositorio.
- [ ] Al menos tres contribuidores externos con PRs merged.

### 2.4 Riesgos de la fase

El riesgo principal es la deriva de scope: querer añadir features no estrictamente necesarias para el MVP del Core. La mitigación es una declaración explícita de scope en el ticket de la fase: el Core de esta fase no incluye autorización RBAC compleja, no incluye eventos, no incluye caché, no incluye observabilidad completa. Esos llegan en fases posteriores.

---
