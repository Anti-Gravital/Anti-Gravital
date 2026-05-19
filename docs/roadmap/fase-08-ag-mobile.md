# Fase 8 - ag-mobile Flutter bridge

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-07-ag-migrate.md](./fase-07-ag-migrate.md)
> Siguiente: [fase-09-plugins-wasi.md](./fase-09-plugins-wasi.md)

## Fase 8 — `ag-mobile` Flutter bridge

**Objetivo.** Construir la integración con Flutter como objetivo prioritario móvil. Generación de SDK Dart completo, auth nativo, realtime.

### 8.1 Criterios de entrada

- [ ] Fase 7 completada.
- [ ] Al menos un colaborador con experiencia significativa en Flutter se ha unido al proyecto.

### 8.2 Entregables

- [ ] Crate `ag-mobile` con generador Dart.
- [ ] Paquete pub `anti_gravital` publicado en pub.dev:
  - [ ] Tipos generados con freezed.
  - [ ] Cliente HTTP con dio + interceptores.
  - [ ] Cliente WebSocket.
  - [ ] Cliente SSE.
  - [ ] Mocks para tests.
- [ ] Widgets de autenticación: registro y login con WebAuthn nativo (Android Credential Manager, iOS Passkeys), OAuth2.
- [ ] Example `flutter-fullstack` en `examples/`: app Flutter completa con backend Anti-Gravital.
- [ ] Documentación: guía de usuario Flutter.

### 8.3 Criterios de salida (puerta antes de Fase 9)

- [ ] El paquete `anti_gravital` en pub.dev tiene al menos 50 likes.
- [ ] El example `flutter-fullstack` corre en Android, iOS y web.
- [ ] Al menos una aplicación Flutter externa usa Anti-Gravital en staging o producción.
- [ ] Al menos 4 500 stars en el repositorio.

### 8.4 Riesgos de la fase

El riesgo principal es que el cambio de contexto Rust → Dart tenga fricciones imprevistas. La mitigación es comenzar con el caso más simple (CRUD) y construir incrementalmente.

---
