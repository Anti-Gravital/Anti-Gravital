# Capitulo 13. Puente de aplicaciones nativas (ag-mobile)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 13
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [12-migracion-ag-migrate.md](./12-migracion-ag-migrate.md)
> Siguiente: [14-observabilidad-ag-observe.md](./14-observabilidad-ag-observe.md)

## 13. Puente de aplicaciones nativas (`ag-mobile`): Flutter y clientes generados

El reposicionamiento más importante derivado del análisis crítico es que Anti-Gravital no compite con Flutter. Se posiciona como **el backend nativo ideal para aplicaciones Flutter**. Esto multiplica el valor estratégico del proyecto: en lugar de competir con un framework de UI multiplataforma maduro y muy bien diseñado, Anti-Gravital se convierte en su compañero natural.

### 13.1 Generación de SDK Dart

`ag-mobile` genera un paquete Dart completo a partir del `schema.ag`. El paquete incluye tipos generados con freezed para inmutabilidad, cliente HTTP basado en dio con interceptores para autenticación, cliente WebSocket para realtime, soporte de Server-Sent Events, y mocks para tests.

### 13.2 Autenticación nativa Flutter

El módulo incluye widgets y servicios listos para los flujos de autenticación. La integración con WebAuthn aprovecha las plataformas nativas (Android Credential Manager API, iOS Passkeys vía AuthenticationServices). El flujo OAuth2 usa `flutter_appauth` con configuración auto-generada.

### 13.3 Offline-first y sincronización

`ag-mobile` ofrece un layer de sincronización offline opcional. Las operaciones se encolan localmente en una base SQLite, se replican al servidor cuando hay conectividad, y los conflictos se resuelven con políticas declarativas en el schema (last-write-wins, custom merge, server-wins). Es una funcionalidad ambiciosa que se implementa en una fase tardía del roadmap.

### 13.4 Otros clientes generados

Aunque Flutter es el target prioritario para móvil, el sistema de codegen es extensible. La versión 1.0 incluye generadores para Dart (Flutter), TypeScript (React, Vue, Svelte, Next.js), y Kotlin (Android nativo, opcionalmente Kotlin Multiplatform). Una versión posterior puede incluir Swift y Python.

---

